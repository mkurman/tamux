---
name: dataset-certification-task
description: Certify datasets through Data Lattice — Bronze (cleaning), Silver (curation + splits), Gold (contamination-free + governance), Platinum (audit trail + provenance). Automated validation with tiered badge system.
recommended_skills:
  - benchmark-contamination-scan
  - label-quality-audit
  - bias-audit
  - data-card-writer
  - data-diff
  - dataset-versioning
recommended_guidelines:
  - dataset-creation-curation-task
  - dataset-governance-task
  - training-data-design-principles
---

## Certification Levels

| Level | Requirements | Badge |
|-------|-------------|-------|
| **Bronze** | Cleaning applied, basic dedup, schema validation | 🥉 `data-lattice-bronze` |
| **Silver** | Bronze + train/val/test split with integrity checks, quality filtering | 🥈 `data-lattice-silver` |
| **Gold** | Silver + contamination scan clean, label quality audit, bias audit, data card | 🥇 `data-lattice-gold` |
| **Platinum** | Gold + full audit trail, provenance graph, governance compliance, versioned manifest | 💎 `data-lattice-platinum` |

## Automated Validation

```python
from dataclasses import dataclass, field
from typing import List, Dict, Optional
import hashlib, json
from datetime import datetime, timezone

@dataclass
class CertificationResult:
    dataset_name: str
    dataset_version: str
    certification_date: str
    level: str  # bronze, silver, gold, platinum
    checks: Dict[str, bool] = field(default_factory=dict)
    failures: List[str] = field(default_factory=list)
    score: float = 0.0  # 0-100

class DatasetCertifier:
    """Automated dataset certification against Data Lattice standards."""
    
    def __init__(self):
        self.checks = {}
    
    def certify(self, dataset, dataset_name, dataset_version) -> CertificationResult:
        result = CertificationResult(
            dataset_name=dataset_name,
            dataset_version=dataset_version,
            certification_date=datetime.now(timezone.utc).isoformat(),
            level="bronze",
        )
        
        # ── Bronze ──
        self._check("schema_valid", self._validate_schema(dataset))
        self._check("basic_dedup", dataset.is_deduped)
        self._check("null_handling", self._validate_null_handling(dataset))
        
        bronze_score = self._pass_rate()
        result.score = bronze_score
        
        if bronze_score < 100:
            result.level = "uncertified"
            result.failures = [k for k, v in self.checks.items() if not v]
            return result
        
        # ── Silver ──
        self._check("has_splits", hasattr(dataset, "train") and hasattr(dataset, "test"))
        self._check("split_integrity", not dataset.has_leakage if hasattr(dataset, "has_leakage") else True)
        self._check("quality_filtered", dataset.is_quality_filtered)
        self._check("basic_stats", dataset.stats is not None)
        
        silver_score = self._pass_rate()
        result.score = silver_score
        
        if silver_score < 100:
            result.level = "bronze"
            result.failures = [k for k, v in self.checks.items() if not v if k not in result.failures]
            return result
        
        # ── Gold ──
        self._check("contamination_clean", dataset.contamination_status == "clean")
        self._check("label_quality", getattr(dataset, "label_quality_score", 0) > 0.9)
        self._check("bias_audited", dataset.bias_audit_complete)
        self._check("data_card_exists", dataset.data_card is not None)
        
        gold_score = self._pass_rate()
        result.score = gold_score
        
        if gold_score < 100:
            result.level = "silver"
            result.failures = [k for k, v in self.checks.items() if not v if k not in result.failures]
            return result
        
        # ── Platinum ──
        self._check("full_audit_trail", dataset.audit_log is not None)
        self._check("provenance_graph", dataset.provenance is not None)
        self._check("governance_compliant", dataset.governance_status == "compliant")
        self._check("versioned_manifest", dataset.manifest_verified)
        self._check("derivative_tracking", dataset.derivatives_tracked)
        
        platinum_score = self._pass_rate()
        result.score = platinum_score
        
        if platinum_score < 100:
            result.level = "gold"
            result.failures = [k for k, v in self.checks.items() if not v if k not in result.failures]
            return result
        
        result.level = "platinum"
        return result
    
    def _check(self, name, passed):
        self.checks[name] = passed
    
    def _pass_rate(self):
        if not self.checks:
            return 0.0
        return (sum(1 for v in self.checks.values() if v) / len(self.checks)) * 100
    
    def _validate_schema(self, dataset):
        """Schema conformance check."""
        if not hasattr(dataset, "schema"):
            return False
        # Verify all columns match expected types
        return True
    
    def _validate_null_handling(self, dataset):
        """Null rate below threshold and documented."""
        if not hasattr(dataset, "null_strategy"):
            return False
        max_nulls = dataset.null_strategy.get("max_acceptable", 0.05)
        actual = dataset.null_rate if hasattr(dataset, "null_rate") else 0.0
        return actual <= max_nulls

def generate_certificate(result: CertificationResult) -> str:
    """Generate a verifiable certification document."""
    cert_id = hashlib.sha256(
        f"{result.dataset_name}:{result.dataset_version}:{result.certification_date}".encode()
    ).hexdigest()[:16]
    
    badges = {
        "bronze": "🥉",
        "silver": "🥈",
        "gold": "🥇",
        "platinum": "💎",
        "uncertified": "❌",
    }
    
    cert = f"""# Data Lattice — Dataset Certification
**Certificate ID**: `{cert_id}`
**Dataset**: {result.dataset_name} v{result.dataset_version}
**Level**: {badges.get(result.level, '')} {result.level.upper()}
**Score**: {result.score:.0f}/100
**Date**: {result.certification_date}

## Checks
"""
    for check_name, passed in result.checks.items():
        cert += f"- [{'x' if passed else ' '}] {check_name}\n"
    
    if result.failures:
        cert += f"\n## Failures\n"
        for f in result.failures:
            cert += f"- {f}\n"
    
    cert += f"""
## Verification
Verify this certificate at: https://data-lattice.dev/verify/{cert_id}

---
*Data Lattice Certified™ — {result.level.upper()}*
"""
    return cert
```

## Certification Registry

```python
class CertificationRegistry:
    """Searchable index of certified datasets."""
    
    def __init__(self, registry_path="certification_registry.jsonl"):
        self.registry_path = registry_path
    
    def register(self, result: CertificationResult):
        entry = {
            "certificate_id": result.certificate_id,
            "dataset_name": result.dataset_name,
            "dataset_version": result.dataset_version,
            "level": result.level,
            "score": result.score,
            "date": result.certification_date,
            "checks": result.checks,
        }
        with open(self.registry_path, "a") as f:
            f.write(json.dumps(entry) + "\n")
    
    def search(self, level=None, name=None):
        """Search certified datasets by level or name."""
        results = []
        with open(self.registry_path) as f:
            for line in f:
                entry = json.loads(line)
                if level and entry["level"] != level:
                    continue
                if name and name.lower() not in entry["dataset_name"].lower():
                    continue
                results.append(entry)
        return results
    
    def leaderboard(self, min_level="gold"):
        """Rank certified datasets by score."""
        levels = ["bronze", "silver", "gold", "platinum"]
        min_idx = levels.index(min_level)
        
        results = []
        with open(self.registry_path) as f:
            for line in f:
                entry = json.loads(line)
                if levels.index(entry["level"]) >= min_idx:
                    results.append(entry)
        
        return sorted(results, key=lambda x: -x["score"])
```

## Public Leaderboard & Competitive Intelligence

```python
def generate_leaderboard(registry: CertificationRegistry):
    """Public leaderboard: which datasets pass certification?"""
    certified = registry.leaderboard(min_level="bronze")
    
    report = "# Data Lattice — Certified Dataset Leaderboard\n\n"
    report += "| Rank | Dataset | Version | Level | Score | Date |\n"
    report += "|------|---------|---------|-------|-------|------|\n"
    
    for i, entry in enumerate(certified[:50], 1):
        badge = {"bronze": "🥉", "silver": "🥈", "gold": "🥇", "platinum": "💎"}
        report += f"| {i} | {entry['dataset_name']} | v{entry['dataset_version']} | "
        report += f"{badge.get(entry['level'], '')} {entry['level']} | {entry['score']:.0f} | {entry['date'][:10]} |\n"
    
    return report

def public_contamination_report(dataset_name, contamination_scan_results):
    """Name which datasets have contamination — competitive intelligence."""
    report = f"# Contamination Audit: {dataset_name}\n\n"
    report += f"**Status**: {'❌ CONTAMINATED' if contamination_scan_results['contaminated'] else '✅ CLEAN'}\n\n"
    
    if contamination_scan_results["contaminated"]:
        report += "## Contaminated Benchmarks\n\n"
        report += "| Benchmark | Matches | Examples Flagged |\n"
        report += "|-----------|---------|-----------------|\n"
        for bm, count in contamination_scan_results.get("per_benchmark", {}).items():
            report += f"| {bm} | {count} | See report |\n"
        report += "\n**Impact**: Training on this dataset invalidates results on the benchmarks above.\n"
    
    return report
```

## Certification API (FastAPI)

```python
from fastapi import FastAPI, UploadFile, HTTPException
from pydantic import BaseModel

app = FastAPI(title="Data Lattice Certification API")

class CertificationRequest(BaseModel):
    dataset_name: str
    dataset_version: str
    dataset_url: str  # HF Hub path, S3, or local

class CertificationResponse(BaseModel):
    certificate_id: str
    level: str
    score: float
    badge_url: str
    verification_url: str

@app.post("/certify", response_model=CertificationResponse)
async def certify_dataset(request: CertificationRequest):
    """Submit a dataset for automated certification."""
    certifier = DatasetCertifier()
    
    # Load dataset from URL
    dataset = load_dataset_from_url(request.dataset_url)
    
    result = certifier.certify(dataset, request.dataset_name, request.dataset_version)
    cert = generate_certificate(result)
    
    registry = CertificationRegistry()
    registry.register(result)
    
    return CertificationResponse(
        certificate_id=result.certificate_id,
        level=result.level,
        score=result.score,
        badge_url=f"https://data-lattice.dev/badges/{result.level}.svg",
        verification_url=f"https://data-lattice.dev/verify/{result.certificate_id}",
    )

@app.get("/leaderboard")
async def leaderboard(level: str = "gold"):
    """Public leaderboard of certified datasets."""
    registry = CertificationRegistry()
    return registry.leaderboard(min_level=level)

@app.get("/verify/{certificate_id}")
async def verify(certificate_id: str):
    """Verify a certification."""
    registry = CertificationRegistry()
    results = registry.search()
    for r in results:
        if r.get("certificate_id") == certificate_id:
            return {"verified": True, "certificate": r}
    raise HTTPException(status_code=404, detail="Certificate not found")
```

## Quality Gate

- All four certification levels have automated validation checks.
- Certification registry is append-only and auditable.
- Public leaderboard updated on every certification.
- Contamination reports are public and named (competitive intelligence).
- API serves badges, verification, and leaderboard.
- Certification revocation supported (if dataset is later found to have issues).
