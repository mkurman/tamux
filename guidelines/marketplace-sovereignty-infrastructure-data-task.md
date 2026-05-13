---
name: marketplace-sovereignty-infrastructure-data-task
description: Curate data for data marketplaces, sovereignty/geopolitics, and infrastructure resilience — quality certification, pricing validation, cross-border compliance, disaster recovery testing, and data corruption detection.
recommended_skills: [data-diff, dataset-versioning, dataset-certification-task, benchmark-contamination-scan]
recommended_guidelines: [data-lifecycle-governance-task, dataset-governance-task, privacy-preserving-data-task]
---

## Data Marketplaces

```python
def validate_marketplace_quality(claimed_quality, actual_quality_audit, certification_levels):
    """Does the price justify the quality?"""
    discrepancies = {}
    for dataset_id, claimed in claimed_quality.items():
        actual = actual_quality_audit.get(dataset_id, {})
        for metric, claimed_val in claimed.items():
            actual_val = actual.get(metric)
            if actual_val is not None and claimed_val != actual_val:
                discrepancies[dataset_id] = discrepancies.get(dataset_id, [])
                discrepancies[dataset_id].append({"metric": metric, "claimed": claimed_val, "actual": actual_val})
    return {"misrepresented_datasets": len(discrepancies),
            "marketplace_trust_score": 1 - len(discrepancies) / max(len(claimed_quality), 1)}

def audit_provider_reputation(claimed_capabilities, delivery_history):
    """Does the provider actually deliver what they claim?"""
    claims_vs_reality = {}
    for provider_id, claims in claimed_capabilities.items():
        history = delivery_history.get(provider_id, [])
        on_time = np.mean([d["on_time"] for d in history]) if history else 0
        quality_ok = np.mean([d["quality_met"] for d in history]) if history else 0
        claims_vs_reality[provider_id] = {"on_time_delivery": float(on_time), "quality_met": float(quality_ok),
                                           "reputation_gap": 1 - min(on_time, quality_ok)}
    return claims_vs_reality
```

## Data Sovereignty & Geopolitics

```python
SOVEREIGNTY_CHECKS = {
    "gdpr": {"transfer_mechanism_required": True, "adequacy_decision_needed": True},
    "data_localization": {"russia": "personal_data", "china": "critical_data", "india": "sensitive_personal"},
    "cross_border": {"schrems_ii": "supplementary_measures", "standard_contractual_clauses": True},
}

def validate_data_localization(dataset_locations, policy_requirements):
    """Is data actually stored where policy requires?"""
    violations = []
    for dataset_id, actual_locations in dataset_locations.items():
        required = policy_requirements.get(dataset_id, [])
        for loc in required:
            if loc not in actual_locations:
                violations.append({"dataset": dataset_id, "required": loc, "actual": actual_locations})
    return {"compliant": len(violations) == 0, "violations": violations}

def assess_foreign_data_dependency(critical_datasets, external_sources, dependency_threshold=0.3):
    """What external data dependencies exist? What's the risk?"""
    dependencies = {}
    for dataset_id, source_info in external_sources.items():
        if dataset_id in critical_datasets:
            dependencies[dataset_id] = {"source_country": source_info.get("country"),
                                         "alternative_available": source_info.get("alternative", False),
                                         "risk_level": "HIGH" if not source_info.get("alternative") else "MODERATE"}
    high_risk = [k for k, v in dependencies.items() if v["risk_level"] == "HIGH"]
    return {"critical_dependencies": len(dependencies), "high_risk_dependencies": len(high_risk),
            "dependency_ratio": len(high_risk) / max(len(critical_datasets), 1),
            "excessive_dependency": len(high_risk) / max(len(critical_datasets), 1) > dependency_threshold}
```

## Data Infrastructure Resilience

```python
def test_backup_recovery(backup_configs, recovery_attempts):
    """Do backups actually recover?"""
    results = []
    for config in backup_configs:
        attempt = next((a for a in recovery_attempts if a["config_id"] == config["id"]), None)
        if attempt:
            results.append({"config": config["name"], "recoverable": attempt["success"],
                            "recovery_time_minutes": attempt.get("duration_min", 0),
                            "data_loss_bytes": attempt.get("data_loss_bytes", 0)})
    return {"backups_tested": len(results), "recovery_success_rate": np.mean([r["recoverable"] for r in results]),
            "mean_recovery_time_min": np.mean([r["recovery_time_minutes"] for r in results if r["recoverable"]]),
            "all_recoverable": all(r["recoverable"] for r in results)}

def detect_data_corruption(integrity_checks, historical_checksums):
    """Continuous integrity monitoring — has data been corrupted?"""
    corrupted = []
    for check in integrity_checks:
        current = check["current_checksum"]
        historical = historical_checksums.get(check["dataset_id"])
        if historical and current != historical[-1]["checksum"]:
            corrupted.append({"dataset": check["dataset_id"], "last_good": historical[-1]["timestamp"],
                               "detected_at": check["timestamp"]})
    return {"corrupted_datasets": corrupted, "corruption_rate": len(corrupted) / max(len(integrity_checks), 1)}

def audit_multi_region_consistency(datasets_by_region, tolerance=0.001):
    """Same data across regions — are they identical?"""
    regions = list(datasets_by_region.keys())
    inconsistencies = []
    for i in range(len(regions)):
        for j in range(i+1, len(regions)):
            d1, d2 = datasets_by_region[regions[i]], datasets_by_region[regions[j]]
            if len(d1) != len(d2):
                inconsistencies.append({"regions": (regions[i], regions[j]), "issue": "row_count"})
                continue
            diffs = np.sum(d1 != d2) if hasattr(d1, '__array__') else len([k for k in d1 if d1[k] != d2[k]])
            if diffs > tolerance * len(d1):
                inconsistencies.append({"regions": (regions[i], regions[j]), "issue": "value_mismatch", "n_diffs": int(diffs)})
    return {"consistent": len(inconsistencies) == 0, "inconsistencies": inconsistencies}
```

## Quality Gate

- Marketplace: trust score > 0.9 (minimal misrepresentation).
- Sovereignty: zero localization violations; foreign dependency < 30%.
- Infrastructure: backup recovery rate = 100%; zero undetected corruptions; multi-region consistency.
