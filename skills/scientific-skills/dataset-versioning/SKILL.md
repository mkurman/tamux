---
name: dataset-versioning
description: Version datasets with checksums, manifests, and semantic versioning. Covers DVC integration, provenance tracking, release tagging, and reproducible dataset lifecycles.
tags: [data-versioning, dvc, provenance, reproducibility, checksums, dataset-curation, mlops]
---

# Dataset Versioning

## Overview

Datasets are artifacts that evolve. Versioning turns "which version did we use?" from a forensic investigation into a one-line answer. Every released dataset gets a checksum, a manifest, and a semantic version tag.

## When to Use

Use this skill when:
- Creating, cleaning, or updating a dataset that others (or future-you) depend on.
- Tracking provenance: what source data, code, and parameters produced this dataset.
- Collaborating on datasets where multiple versions coexist.
- Reproducing past experiments that depend on a specific dataset version.

Do not use for:
- One-off exploration that won't be reused.
- Interim files that are fully regenerable from versioned sources.
- Model versioning — use `mlflow` or `wandb` skills.

## Versioning Workflow

### 1. Initialize Version Tracking

```bash
# DVC (Data Version Control) — recommended for datasets > 10 MB
pip install dvc
dvc init
git commit -m "Initialize DVC"
```

Or minimal filesystem-based versioning:

```python
import hashlib
import json
from pathlib import Path
from datetime import datetime, timezone

DATASET_ROOT = Path("datasets")
VERSION_REGISTRY = DATASET_ROOT / "versions.jsonl"

def compute_checksum(filepath: Path, algo: str = "sha256") -> str:
    h = hashlib.new(algo)
    with open(filepath, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return h.hexdigest()
```

### 2. Create a Dataset Manifest

Every dataset release needs a `manifest.json`:

```json
{
  "name": "customer-churn-prediction",
  "version": "1.0.0",
  "created_at": "2026-05-11T17:29:51Z",
  "description": "Cleaned customer churn dataset for binary classification",
  "files": {
    "train.parquet": {
      "checksum_sha256": "a1b2c3d4...",
      "rows": 80000,
      "columns": 24,
      "size_bytes": 5242880
    },
    "val.parquet": {
      "checksum_sha256": "e5f6g7h8...",
      "rows": 10000,
      "columns": 24,
      "size_bytes": 655360
    },
    "test.parquet": {
      "checksum_sha256": "i9j0k1l2...",
      "rows": 10000,
      "columns": 24,
      "size_bytes": 655360
    },
    "schema.json": {
      "checksum_sha256": "m3n4o5p6...",
      "size_bytes": 2048
    },
    "cleaning_audit.json": {
      "checksum_sha256": "q7r8s9t0...",
      "size_bytes": 4096
    }
  },
  "provenance": {
    "source": "https://data.example.com/customers/export/2026-Q1",
    "query_timestamp": "2026-05-10T08:00:00Z",
    "cleaning_script": "scripts/clean_churn.py",
    "cleaning_script_sha256": "u1v2w3x4...",
    "split_seed": 42,
    "transformations_applied": [
      "median_imputation_age",
      "dedup_on_user_id",
      "cap_age_0_to_120",
      "one_hot_encode_region"
    ]
  },
  "schema": {
    "target_column": "churned",
    "protected_attributes": ["gender", "age_group"],
    "train_val_test_split": [0.70, 0.15, 0.15]
  },
  "license": "CC-BY-4.0",
  "limitations": [
    "Data from Q1 2026 only; seasonal patterns not captured",
    "Region X undersampled (3% vs 15% in production)"
  ]
}
```

### 3. Version with DVC

```bash
# Track dataset files with DVC
dvc add datasets/customer-churn-v1.0.0/

# Commit the .dvc file to git
git add datasets/customer-churn-v1.0.0.dvc datasets/.gitignore
git commit -m "dataset: customer-churn v1.0.0 — initial release"

# Push data to remote storage
dvc remote add -d myremote s3://my-bucket/datasets
dvc push

# Tag the release
git tag -a "dataset/customer-churn/v1.0.0" -m "Customer churn dataset v1.0.0"
git push --tags
```

### 4. Semantic Versioning for Datasets

| Bump | When |
||--------|
| **Major** (`v2.0.0`) | Schema change, new/dropped columns, target definition changed, new source data |
| **Minor** (`v1.1.0`) | New rows added from same source, additional features derived from existing data, improved cleaning |
| **Patch** (`v1.0.1`) | Bugfix in cleaning without schema changes, metadata/card updates, reprocessing with same logic |

### 5. Retrieve a Specific Version

```bash
# Checkout a tagged dataset version
git checkout dataset/customer-churn/v1.0.0
dvc checkout  # restore data files

# In Python — verify checksums before loading
manifest = json.loads(Path("datasets/customer-churn-v1.0.0/manifest.json").read_text())
for fname, finfo in manifest["files"].items():
    actual = compute_checksum(Path("datasets/customer-churn-v1.0.0") / fname)
    assert actual == finfo["checksum_sha256"], f"Checksum mismatch: {fname}"
```

### 6. Version Registry

Append to a global registry for discoverability:

```python
def register_version(manifest: dict):
    entry = {
        "name": manifest["name"],
        "version": manifest["version"],
        "released_at": manifest["created_at"],
        "checksum": compute_checksum(Path(f"datasets/{manifest['name']}-v{manifest['version']}/manifest.json")),
        "num_files": len(manifest["files"]),
        "predecessor": manifest.get("predecessor_version"),
    }
    with open(VERSION_REGISTRY, "a") as f:
        f.write(json.dumps(entry) + "\n")
```

## Provenance Rules

1. Every dataset release MUST reference the exact source (URL, query, API call).
2. Every transformation MUST be reproducible from source → release via a versioned script.
3. Never overwrite a released version. Always create a new one.
4. Checksums are mandatory for every file in the release.
5. A predecessor version link must exist for versions > 1.0.0.

## Quality Gate

A dataset version is complete when:
- `manifest.json` exists with all file checksums.
- Provenance traces back to source data and cleaning script.
- The version is tagged in git (and pushed to DVC remote if large).
- A new entry exists in the version registry.
- Loading the dataset by version tag produces identical checksums.
