---
name: data-risk-integration-task
description: Integrate risk management with data architecture — map risk types to detection capabilities, severity to monitoring intensity, velocity to alert latency, and mitigation to validation data. Every risk has a data shadow; catch it before it catches you.
recommended_skills: [data-pipeline-monitoring-task, dataset-versioning, benchmark-contamination-scan]
recommended_guidelines: [dataset-governance-task, robustness-engineering-task, data-lifecycle-governance-task]
---

## Overview

Risks move at different speeds and strike from different directions. Financial risk demands real-time transaction monitoring. Reputational risk demands social listening and sentiment tracking. Operational risk demands system health telemetry. Treat each risk type as a data detection problem — define the signal, set the monitoring intensity, match the alert latency, and validate that mitigation actually worked with data, not opinion.

## Risk Type → Detection Capability Mapping

```python
RISK_DETECTION_ARCHITECTURE = {
    "financial": {
        "signals": ["transaction_volume_anomaly", "price_volatility", "counterparty_exposure",
                     "liquidity_pressure", "credit_spread_widening"],
        "data_sources": ["ledger_stream", "market_data_feed", "risk_system_api"],
        "detection_method": "time_series_anomaly_detection",
        "baseline_window_days": 90,
        "false_positive_tolerance": 0.05,
    },
    "operational": {
        "signals": ["pipeline_failure_rate", "latency_p99_spike", "schema_drift",
                     "data_freshness_gap", "dependency_health"],
        "data_sources": ["pipeline_metrics", "infrastructure_telemetry", "incident_tracker"],
        "detection_method": "threshold_alerting_with_hysteresis",
        "baseline_window_days": 30,
        "false_positive_tolerance": 0.10,
    },
    "reputational": {
        "signals": ["sentiment_shift", "mention_velocity", "complaint_cluster",
                     "competitor_action", "regulatory_announcement"],
        "data_sources": ["social_feed", "review_platforms", "news_wire", "regulatory_filings"],
        "detection_method": "nlp_sentiment_tracking",
        "baseline_window_days": 60,
        "false_positive_tolerance": 0.15,
    },
    "strategic": {
        "signals": ["market_share_shift", "technology_disruption_signal",
                     "talent_churn_anomaly", "regulatory_change_indicator"],
        "data_sources": ["market_intelligence", "patent_database", "hr_analytics", "policy_tracker"],
        "detection_method": "leading_indicator_composite",
        "baseline_window_days": 180,
        "false_positive_tolerance": 0.20,
    },
    "compliance": {
        "signals": ["consent_gap", "access_anomaly", "retention_violation",
                     "cross_border_transfer_flag", "model_drift_beyond_threshold"],
        "data_sources": ["access_logs", "consent_registry", "data_catalog", "model_registry"],
        "detection_method": "rule_engine_with_ml_anomaly",
        "baseline_window_days": 90,
        "false_positive_tolerance": 0.01,
    },
}

def map_risk_to_detection(risk_register):
    covered = []
    gaps = []
    for risk_id, risk in risk_register.items():
        architecture = RISK_DETECTION_ARCHITECTURE.get(risk.get("type"))
        if architecture is None:
            gaps.append({"risk": risk_id, "type": risk.get("type"),
                         "issue": "unknown_risk_type_no_detection_architecture"})
            continue
        missing_signals = [s for s in architecture["signals"]
                           if s not in risk.get("monitored_signals", [])]
        if missing_signals:
            gaps.append({"risk": risk_id, "type": risk.get("type"),
                         "missing_signals": missing_signals})
        else:
            covered.append(risk_id)
    return {"covered": covered, "gaps": gaps,
            "coverage_ratio": len(covered) / max(len(risk_register), 1)}
```

## Risk Severity → Monitoring Intensity

| Severity | Monitoring | Sample Rate | Escalation | Review Cadence |
|----------|-----------|-------------|------------|----------------|
| **Critical** | Continuous, multi-signal fusion | Per-event | Immediate page + exec alert | Daily |
| **High** | Continuous, single-signal | Per-minute | Page within 5 min | Weekly |
| **Medium** | Batch, hourly | Hourly aggregate | Ticket auto-create | Monthly |
| **Low** | Batch, daily | Daily aggregate | Dashboard highlight | Quarterly |

```python
def severity_band(risk_severity):
    bands = {
        "critical": {"sample_interval": "event", "escalation_sla_minutes": 0,
                     "review_days": 1, "requires_fusion": True},
        "high":     {"sample_interval": "minute", "escalation_sla_minutes": 5,
                     "review_days": 7, "requires_fusion": False},
        "medium":   {"sample_interval": "hour", "escalation_sla_minutes": 60,
                     "review_days": 30, "requires_fusion": False},
        "low":      {"sample_interval": "day", "escalation_sla_minutes": 1440,
                     "review_days": 90, "requires_fusion": False},
    }
    return bands.get(risk_severity, bands["medium"])
```

## Risk Velocity → Alert Latency Matching

```python
VELOCITY_LATENCY_MAP = {
    "instant":    {"alert_latency": "seconds",   "pipeline_mode": "streaming",
                   "example": "fraud_transaction", "missed_detection_cost": "immediate_loss"},
    "fast":       {"alert_latency": "minutes",   "pipeline_mode": "micro_batch",
                   "example": "pipeline_failure", "missed_detection_cost": "cascading_failure"},
    "moderate":   {"alert_latency": "hours",     "pipeline_mode": "batch_hourly",
                   "example": "data_quality_drift", "missed_detection_cost": "retraining_required"},
    "slow":       {"alert_latency": "days",      "pipeline_mode": "batch_daily",
                   "example": "market_share_shift", "missed_detection_cost": "opportunity_cost"},
    "glacial":    {"alert_latency": "weeks",     "pipeline_mode": "batch_weekly",
                   "example": "technology_disruption", "missed_detection_cost": "strategic_miss"},
}

def validate_alert_latency(risk, pipeline_config):
    velocity = risk.get("velocity", "moderate")
    required = VELOCITY_LATENCY_MAP.get(velocity, VELOCITY_LATENCY_MAP["moderate"])
    actual = pipeline_config.get("alert_latency", "hours")
    latency_hierarchy = ["seconds", "minutes", "hours", "days", "weeks"]
    aligned = latency_hierarchy.index(actual) <= latency_hierarchy.index(required["alert_latency"])
    return {"risk": risk["id"], "velocity": velocity, "required_latency": required["alert_latency"],
            "actual_latency": actual, "aligned": aligned,
            "cost_of_misalignment": required["missed_detection_cost"] if not aligned else None}
```

## Risk Correlation → Data Correlation Discovery

```python
def correlate_risk_signals(risk_events, lookback_days=90):
    """Discover which risks co-occur in time — interconnected risks need interconnected data."""
    from collections import defaultdict
    import numpy as np
    
    risk_time_series = defaultdict(list)
    for event in risk_events:
        if event["timestamp"] >= lookback_days:
            risk_time_series[event["risk_type"]].append(event["timestamp"])
    
    correlations = {}
    risk_types = list(risk_time_series.keys())
    for i, r1 in enumerate(risk_types):
        for r2 in risk_types[i+1:]:
            co_occurrence = len(set(risk_time_series[r1]) & set(risk_time_series[r2]))
            jaccard = co_occurrence / max(len(set(risk_time_series[r1]) | set(risk_time_series[r2])), 1)
            if jaccard > 0.2:
                correlations[f"{r1}_{r2}"] = {"jaccard": jaccard,
                    "recommendation": "merge_monitoring_views" if jaccard > 0.4 else "cross_reference_dashboards"}
    return correlations
```

## Risk Mitigation → Data Validation

```python
def validate_mitigation(risk, mitigation_action, before_metrics, after_metrics):
    """Did mitigation actually work? Data must prove it."""
    required_improvement = risk.get("mitigation_target", 0.5)
    
    before_value = before_metrics.get(risk["metric"], 0)
    after_value = after_metrics.get(risk["metric"], before_value)
    
    if before_value == 0:
        improvement = 0
    else:
        improvement = (before_value - after_value) / before_value
    
    return {
        "risk": risk["id"],
        "mitigation": mitigation_action["description"],
        "metric": risk["metric"],
        "before": before_value,
        "after": after_value,
        "improvement_pct": improvement * 100,
        "target_met": improvement >= required_improvement,
        "recommendation": "MONITOR" if improvement >= required_improvement
                          else "ESCALATE" if improvement > 0
                          else "REMITIGATE",
    }
```

## Quality Gate

- Every risk in the register has mapped detection signals and data sources.
- Critical/high risks have continuous monitoring with sub-5-minute escalation.
- Alert latency matches risk velocity — fast risks never polled daily.
- Correlated risks share monitoring views; interconnected risks never tracked in isolation.
- Every mitigation action has before/after data validation — no "we think it worked."
