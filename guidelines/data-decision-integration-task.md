---
name: data-decision-integration-task
description: Map decision architectures to data architectures — classify decisions by type/frequency/stakes/reversibility/accountability, then derive data refresh rates, certainty thresholds, retention policies, and provenance requirements.
recommended_skills: [cost-model-task, data-pipeline-monitoring-task, dataset-versioning]
recommended_guidelines: [business-strategy-task, dataset-governance-task, data-lifecycle-governance-task]
---

## Overview

Every decision the organization makes has a data shadow. Strategic decisions need long-range aggregated data. Tactical decisions need fresh operational data. High-stakes decisions need high-certainty data with provenance trails. When decision architecture and data architecture evolve independently, the result is decisions made on wrong data and data collected for no decision. This guideline maps decision properties to data requirements and enforces alignment.

## Decision Type → Data Requirement

```python
DECISION_CLASSIFICATION = {
    "strategic": {
        "horizon": "quarters to years",
        "data_granularity": "aggregated, trend-level",
        "refresh_rate": "weekly to monthly",
        "certainty_threshold": 0.85,
        "reversibility": "low",
        "accountability": "board/executive",
        "retention_years": 10,
        "provenance": "full audit trail",
    },
    "tactical": {
        "horizon": "weeks to quarters",
        "data_granularity": "semi-aggregated",
        "refresh_rate": "daily to weekly",
        "certainty_threshold": 0.75,
        "reversibility": "medium",
        "accountability": "director/VP",
        "retention_years": 3,
        "provenance": "versioned snapshots",
    },
    "operational": {
        "horizon": "hours to days",
        "data_granularity": "raw, event-level",
        "refresh_rate": "real-time to hourly",
        "certainty_threshold": 0.65,
        "reversibility": "high",
        "accountability": "team lead/IC",
        "retention_years": 1,
        "provenance": "run-level metadata",
    },
}

def classify_decision(decision_spec):
    """Map a decision to its data requirements."""
    dec_type = decision_spec.get("type", "operational")
    template = DECISION_CLASSIFICATION.get(dec_type, DECISION_CLASSIFICATION["operational"])
    gaps = []
    required_fields = ["refresh_rate", "certainty_threshold", "retention_years", "provenance"]
    for field in required_fields:
        if decision_spec.get(f"data_{field}") != template[field]:
            gaps.append({"field": field, "required": template[field],
                         "current": decision_spec.get(f"data_{field}")})
    return {"decision_type": dec_type, "template": template,
            "aligned": len(gaps) == 0, "gaps": gaps}
```

## Decision Frequency → Refresh Rate Matching

```python
REFRESH_RATE_MAP = {
    "hourly":       {"pipeline_latency_sla": "seconds", "staleness_tolerance": "minutes"},
    "daily":        {"pipeline_latency_sla": "minutes", "staleness_tolerance": "hours"},
    "weekly":       {"pipeline_latency_sla": "hours",   "staleness_tolerance": "days"},
    "monthly":      {"pipeline_latency_sla": "hours",   "staleness_tolerance": "days"},
    "quarterly":    {"pipeline_latency_sla": "days",    "staleness_tolerance": "weeks"},
    "annual":       {"pipeline_latency_sla": "days",    "staleness_tolerance": "weeks"},
}

def validate_refresh_frequency(decision_registry, pipeline_metadata):
    misaligned = []
    for dec_id, decision in decision_registry.items():
        required = REFRESH_RATE_MAP.get(decision["frequency"])
        if required is None:
            continue
        pipeline = pipeline_metadata.get(decision["data_source"])
        if pipeline is None:
            misaligned.append({"decision": dec_id, "issue": "no_pipeline_assigned"})
            continue
        if pipeline.get("latency_sla") != required["pipeline_latency_sla"]:
            misaligned.append({"decision": dec_id,
                               "issue": "latency_mismatch",
                               "required": required["pipeline_latency_sla"],
                               "actual": pipeline.get("latency_sla")})
    return {"total_decisions": len(decision_registry),
            "misaligned_count": len(misaligned),
            "misaligned": misaligned,
            "alignment_score": 1 - len(misaligned) / max(len(decision_registry), 1)}
```

## Decision Stakes → Certainty Requirements

| Stake Level | Example | Certainty Threshold | Validation Required |
|-------------|---------|---------------------|---------------------|
| **Fatal** | Drug approval, safety-critical release | ≥ 0.95 | Triple-blind, external audit, adversary review |
| **High** | $10M+ investment, hiring executive | ≥ 0.85 | Cross-validation, holdout verification |
| **Medium** | Feature launch, vendor selection | ≥ 0.75 | A/B test, confidence intervals |
| **Low** | UI color, email subject line | ≥ 0.60 | Simple majority, directional signal |

```python
def certainty_audit(decision, data_quality_report):
    threshold = {"fatal": 0.95, "high": 0.85, "medium": 0.75, "low": 0.60}
    required = threshold.get(decision.get("stakes", "medium"), 0.75)
    actual = data_quality_report.get("certainty_score", 0.0)
    passed = actual >= required
    return {"decision": decision["id"], "stakes": decision.get("stakes"),
            "threshold": required, "actual": actual,
            "sufficient": passed,
            "gap": max(0.0, required - actual) if not passed else 0.0}
```

## Decision Reversibility → Retention Requirements

```python
REVERSIBILITY_RETENTION = {
    "irreversible": {"retention_years": 10, "snapshot_interval": "decision_time",
                     "retention_rationale": "Must reproduce decision context for liability/audit"},
    "partially_reversible": {"retention_years": 3, "snapshot_interval": "monthly",
                              "retention_rationale": "Need to reconstruct sequence of decisions"},
    "reversible": {"retention_years": 1, "snapshot_interval": "quarterly",
                   "retention_rationale": "Sufficient to learn from recent decisions"},
}

def retention_policy(decision_registry):
    policies = {}
    for dec_id, decision in decision_registry.items():
        reversibility = decision.get("reversibility", "reversible")
        policy = REVERSIBILITY_RETENTION.get(reversibility, REVERSIBILITY_RETENTION["reversible"])
        policies[dec_id] = {"retention_years": policy["retention_years"],
                            "snapshot_interval": policy["snapshot_interval"],
                            "rationale": policy["retention_rationale"]}
    return policies
```

## Decision Accountability → Provenance

```python
PROVENANCE_REQUIREMENTS = {
    "board":       ["full_lineage_graph", "data_origin_timestamps", "transformation_audit_log",
                     "approver_chain", "external_auditor_access"],
    "director_vp": ["versioned_snapshots", "schema_evolution_log", "owner_attestation",
                     "quality_report_at_decision_time"],
    "team_lead":   ["run_metadata", "data_freshness_at_query_time", "source_identifier"],
    "individual":  ["source_identifier", "query_timestamp"],
}

def provenance_gate(decision, dataset_provenance):
    required = PROVENANCE_REQUIREMENTS.get(decision.get("accountability_level", "team_lead"), [])
    available = dataset_provenance.get("capabilities", [])
    missing = [r for r in required if r not in available]
    return {"sufficient": len(missing) == 0, "missing_capabilities": missing,
            "required": required, "available": available}
```

## Quality Gate

- All decisions above "high" stakes have validated data certainty.
- No decision runs on stale data — refresh rate matches decision frequency.
- Irreversible decisions have 10-year retention with full provenance.
- Board-level decisions have complete lineage graphs with external auditor access.
- Decision registry is version-controlled and audited quarterly.
