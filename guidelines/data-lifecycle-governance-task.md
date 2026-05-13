---
name: data-lifecycle-governance-task
description: Govern the full data lifecycle — Birth (creation validation) → Adolescence (production readiness) → Adulthood (monitoring) → Retirement (deprecation signals) → Death (deletion compliance). Stage transition criteria and deprecation decision trees.
recommended_skills:
  - data-diff
  - data-card-writer
  - dataset-versioning
  - benchmark-contamination-scan
recommended_guidelines:
  - dataset-governance-task
  - data-pipeline-monitoring-task
  - data-contamination-task
  - dataset-release-checklist
---

## Overview

Datasets are born, they live, they age, they die. Most frameworks only cover creation. This guideline covers the full lifecycle with concrete stage transition criteria, automated gate checks, and deprecation decision trees.

## The Five Lifecycle Stages

```
Birth ───→ Adolescence ───→ Adulthood ───→ Retirement ───→ Death
(create)    (validate)      (monitor)      (deprecate)     (delete)
   │             │               │               │              │
   └── valid? ───┘               │               │              │
                   └── ready? ───┘               │              │
                                 └── decaying? ──┘              │
                                                  └── unused? ──┘
```

## Stage 0: Birth

### Creation Validation

```python
def birth_validation(dataset, spec):
    """
    Before a dataset exists, validate the CREATION, not the data.
    """
    checks = {
        "specification_exists": spec is not None,
        "provenance_recorded": all(source.get("origin") for source in dataset.sources),
        "consent_verified": dataset.consent_status in ("obtained", "not_applicable"),
        "license_compatible": _check_license_compatibility(dataset),
        "minimal_viable_size": len(dataset) >= spec.get("min_size", 100),
        "schema_matches_spec": _check_schema(dataset.schema, spec["schema"]),
    }
    
    passed = all(checks.values())
    
    return {
        "stage": "birth",
        "checks": checks,
        "passed": passed,
        "can_advance": passed,
        "failed_checks": [k for k, v in checks.items() if not v],
    }
```

### Birth Gate Checklist

- [ ] Specification document exists and is reviewed.
- [ ] All data sources have provenance recorded.
- [ ] Consent basis is documented or exemption justified.
- [ ] License is compatible with intended use.
- [ ] Dataset meets minimum viable size.
- [ ] Schema matches specification.

**Fail any = do not advance to Adolescence.**

## Stage 1: Adolescence

### Production Readiness

```python
def adolescence_validation(dataset, quality_audit_results):
    checks = {
        "dedup_complete": dataset.is_deduped,
        "contamination_scan_clean": dataset.contamination_scan_result == "clean",
        "split_integrity": not dataset.has_split_leakage,
        "label_quality_acceptable": quality_audit_results.get("issue_fraction", 1.0) < 0.1,
        "bias_audit_complete": dataset.bias_audit is not None,
        "data_card_exists": dataset.data_card is not None,
        "version_assigned": dataset.version is not None,
        "checksums_verified": dataset.manifest_verified,
    }
    
    passed = all(checks.values())
    
    return {
        "stage": "adolescence",
        "checks": checks,
        "passed": passed,
        "can_advance": passed,
        "failed_checks": [k for k, v in checks.items() if not v],
    }
```

### Adolescence Gate Checklist

- [ ] Deduplication applied and validated.
- [ ] Contamination scan clean against all benchmarks.
- [ ] Train/val/test split integrity verified.
- [ ] Label quality audit complete; noise rate < 10%.
- [ ] Bias audit completed.
- [ ] Data card written.
- [ ] Version assigned and manifest checksums verified.

## Stage 2: Adulthood

### Ongoing Monitoring

```python
def adulthood_monitoring(dataset, monitoring_history):
    """
    Continuously running checks on production datasets.
    """
    checks = {
        "freshness_ok": _check_freshness(dataset),
        "no_schema_drift": not monitoring_history.get("schema_drift_detected", False),
        "distribution_stable": not monitoring_history.get("distribution_drift_detected", False),
        "usage_within_bounds": dataset.usage_count > 0 if dataset.expected_usage else True,
        "no_new_contamination": monitoring_history.get("recent_contamination_scan", "clean") == "clean",
        "pipeline_healthy": monitoring_history.get("last_pipeline_status") == "success",
        "sla_compliant": _check_sla(dataset),
    }
    
    issues = [k for k, v in checks.items() if not v]
    
    return {
        "stage": "adulthood",
        "checks": checks,
        "healthy": len(issues) == 0,
        "issues": issues,
        "should_retire": len(issues) >= 3,  # multiple issues = consider retirement
    }
```

### Adult Health Metrics

| Metric | Green | Yellow | Red |
|-------|-------|-------|-------|
| Freshness | < SLA | 1-2x SLA | > 2x SLA |
| Schema changes | None | Backward-compatible | Breaking change |
| Distribution drift | JS < 0.1 | JS 0.1-0.2 | JS > 0.2 |
| Usage | Active | Declining | Zero for 90 days |
| Pipeline failures | 0 in 30 days | 1-2 | 3+ |

## Stage 3: Retirement

### Deprecation Decision Tree

```python
def should_retire(dataset, monitoring_history, usage_stats):
    reasons = []
    
    # Quality decay
    if monitoring_history.get("distribution_drift_js", 0) > 0.2:
        reasons.append({
            "reason": "distribution_drift",
            "severity": "high",
            "detail": f"JS divergence = {monitoring_history['distribution_drift_js']:.3f}",
        })
    
    # Better replacement exists
    if dataset.superseded_by:
        reasons.append({
            "reason": "superseded",
            "severity": "medium",
            "detail": f"Replaced by {dataset.superseded_by}",
        })
    
    # No usage
    if usage_stats.get("days_since_last_use", 0) > 180:
        reasons.append({
            "reason": "unused",
            "severity": "low",
            "detail": f"No usage for {usage_stats['days_since_last_use']} days",
        })
    
    # Regulatory change
    if dataset.regulatory_status == "non_compliant":
        reasons.append({
            "reason": "regulatory",
            "severity": "critical",
            "detail": dataset.regulatory_detail,
        })
    
    # Consent withdrawn
    if dataset.consent_status == "withdrawn":
        reasons.append({
            "reason": "consent_withdrawn",
            "severity": "critical",
            "detail": "Data subjects withdrew consent — must delete",
        })
    
    # New version available
    if dataset.newer_version_available:
        reasons.append({
            "reason": "newer_version",
            "severity": "low",
            "detail": f"v{dataset.version} → v{dataset.newer_version}",
        })
    
    decision = {
        "dataset_id": dataset.id,
        "version": dataset.version,
        "reasons": reasons,
        "retire": any(r["severity"] in ("critical", "high") for r in reasons),
        "retire_soon": len(reasons) >= 2,
        "retirement_timeline": (
            "immediate" if any(r["severity"] == "critical" for r in reasons)
            else "30_days" if any(r["severity"] == "high" for r in reasons)
            else "90_days" if reasons
            else None
        ),
    }
    
    return decision
```

### Retirement Protocol

1. **Announce deprecation**: Notify all consumers, provide migration path.
2. **Freeze writes**: Stop accepting new data, pipeline runs.
3. **Migration window**: 30-90 days for consumers to switch.
4. **Archive**: Move to cold storage for compliance retention.
5. **Redirect**: All queries return "deprecated, see v{N+1}".

## Stage 4: Death

### Deletion Compliance

```python
def death_protocol(dataset):
    """
    Deletion must be:
    - Complete (all copies, all backups)
    - Verifiable (deletion certificate)
    - Compliant (GDPR Art. 17, CCPA, right to erasure)
    """
    deletion_record = {
        "dataset_id": dataset.id,
        "version": dataset.version,
        "death_date": datetime.now(timezone.utc).isoformat(),
        "death_reason": dataset.death_reason,
        "deletion_scope": [],
    }
    
    # 1. Delete primary storage
    for file_path in dataset.manifest["files"]:
        if os.path.exists(file_path):
            os.remove(file_path)
            deletion_record["deletion_scope"].append({"path": file_path, "status": "deleted"})
    
    # 2. Delete DVC cache
    if dataset.dvc_remote:
        _purge_dvc_cache(dataset)
        deletion_record["deletion_scope"].append({"scope": "dvc_cache", "status": "purged"})
    
    # 3. Delete backups (within SLA timeframe)
    _notify_backup_deletion(dataset)
    
    # 4. Delete derivatives that depend on this data
    derivatives = _find_derivative_datasets(dataset)
    for deriv in derivatives:
        if deriv.can_rebuild:
            _flag_for_rebuild(deriv)
        else:
            _flag_for_review(deriv)  # derived dataset may also need deletion
    
    # 5. Issue deletion certificate
    deletion_certificate = {
        **deletion_record,
        "certificate_id": str(uuid.uuid4()),
        "verified_by": "automated_deletion_protocol_v1",
        "verification_method": "file_existence_checks_negative",
    }
    
    return deletion_certificate
```

### Death Gate Checklist

- [ ] All primary copies deleted and verified.
- [ ] All backup copies scheduled for deletion (within retention window).
- [ ] DVC remote purged.
- [ ] Derivative datasets flagged for rebuild or review.
- [ ] Deletion certificate issued.
- [ ] Consumer notification complete.
- [ ] Regulatory retention period observed (if legally required to keep for N years, archive encrypted, don't delete).

## Lifecycle Stage Registry

```python
class DatasetLifecycle:
    stages = ["birth", "adolescence", "adulthood", "retirement", "death"]
    transitions = {
        "birth": {"advance": "adolescence", "retreat": None},
        "adolescence": {"advance": "adulthood", "retreat": "birth"},
        "adulthood": {"advance": None, "retreat": "retirement"},  # only advance via deprecation
        "retirement": {"advance": "death", "retreat": "adulthood"},  # can un-retire
        "death": {"advance": None, "retreat": None},  # terminal
    }
```

## Quality Gate

- Every dataset has a recorded lifecycle stage.
- Stage transitions require all gate checks to pass.
- Retirement decisions are automated (based on metrics) with human override.
- Death includes verifiable deletion certificate.
- Lifecycle events are logged immutably.
