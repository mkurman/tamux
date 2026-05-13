---
name: data-diff
description: Compare two dataset versions and produce a structured diff — what was added, removed, changed, with row counts and field-level change summaries.
tags: [data-diff, versioning, provenance, dataset-curation, reproducibility]
---

# Data Diff

## Overview

Given two dataset versions, produce a structured, auditable diff: rows added/removed/modified, field-level changes, distribution shifts, and summary statistics.

## When to Use

Before accepting a new dataset version into training. As a gate in your data pipeline. When debugging why a model's behavior changed.

## Core Diff

```python
import pandas as pd
import numpy as np
from dataclasses import dataclass, field
from typing import List, Dict, Any

@dataclass
class DatasetDiff:
    version_from: str
    version_to: str
    n_rows_before: int
    n_rows_after: int
    rows_added: int = 0
    rows_removed: int = 0
    rows_modified: int = 0
    columns_added: List[str] = field(default_factory=list)
    columns_removed: List[str] = field(default_factory=list)
    columns_modified: List[Dict[str, Any]] = field(default_factory=list)
    distribution_shifts: List[Dict[str, Any]] = field(default_factory=list)
    
def diff_datasets(old_df, new_df, key_cols, old_version, new_version):
    diff = DatasetDiff(version_from=old_version, version_to=new_version,
                       n_rows_before=len(old_df), n_rows_after=len(new_df))
    
    old_keys = set(tuple(r) for r in old_df[key_cols].values)
    new_keys = set(tuple(r) for r in new_df[key_cols].values)
    
    diff.rows_added = len(new_keys - old_keys)
    diff.rows_removed = len(old_keys - new_keys)
    
    # Rows modified (same key, different values)
    common = old_keys & new_keys
    if common:
        old_idx = old_df.set_index(key_cols).loc[list(common)]
        new_idx = new_df.set_index(key_cols).loc[list(common)]
        diff.rows_modified = int((old_idx != new_idx).any(axis=1).sum())
    
    # Column changes
    diff.columns_added = list(set(new_df.columns) - set(old_df.columns))
    diff.columns_removed = list(set(old_df.columns) - set(new_df.columns))
    
    # Column-level stats
    for col in set(old_df.columns) & set(new_df.columns):
        if old_df[col].dtype in (np.float64, np.int64):
            old_mean, new_mean = old_df[col].mean(), new_df[col].mean()
            if abs(old_mean - new_mean) > 1e-6:
                diff.columns_modified.append({
                    "column": col,
                    "old_mean": float(old_mean),
                    "new_mean": float(new_mean),
                    "delta": float(new_mean - old_mean),
                    "old_nulls": int(old_df[col].isnull().sum()),
                    "new_nulls": int(new_df[col].isnull().sum()),
                })
    
    # Distribution shifts (KS test for numeric columns)
    from scipy.stats import ks_2samp
    for col in old_df.select_dtypes(include=np.number).columns:
        ks_stat, ks_p = ks_2samp(old_df[col].dropna(), new_df[col].dropna())
        if ks_p < 0.01:
            diff.distribution_shifts.append({
                "column": col,
                "ks_statistic": float(ks_stat),
                "significant": True,
            })
    
    return diff

def diff_report(diff: DatasetDiff) -> str:
    lines = [
        f"# Dataset Diff: {diff.version_from} → {diff.version_to}",
        f"Rows: {diff.n_rows_before} → {diff.n_rows_after}",
        f"  +{diff.rows_added} added, -{diff.rows_removed} removed, ~{diff.rows_modified} modified",
        f"",
    ]
    if diff.columns_added:
        lines.append(f"Columns added: {diff.columns_added}")
    if diff.columns_removed:
        lines.append(f"Columns removed: {diff.columns_removed}")
    if diff.columns_modified:
        lines.append("Column value changes:")
        for c in diff.columns_modified:
            lines.append(f"  {c['column']}: {c['old_mean']:.4f} → {c['new_mean']:.4f} "
                        f"(Δ={c['delta']:.4f}, nulls: {c['old_nulls']}→{c['new_nulls']})")
    if diff.distribution_shifts:
        lines.append(f"Significant distribution shifts: {[d['column'] for d in diff.distribution_shifts]}")
    return "\n".join(lines)
```

## Quality Gate

- Diff is produced for every version transition.
- Rows added/removed/modified are accounted for.
- Distribution shifts in numeric columns are flagged (KS test, p < 0.01).
- Null rate changes > 5pp are flagged.
- New/removed columns are explicitly listed.
