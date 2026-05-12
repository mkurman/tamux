---
name: production-deployment-data-task
description: Curate datasets for production ML deployment — A/B test construction, canary validation, rollback triggers, shadow mode telemetry, and drift detection baselines.
recommended_skills:
  - dataset-splitting
  - data-pipeline-monitoring-task
  - embedding-analysis
recommended_guidelines:
  - evaluation-dataset-design-task
  - data-contamination-task
  - data-pipeline-monitoring-task
---

## Overview

Production deployment data is the bridge between "model works in the lab" and "model works in the real world." A/B test datasets, canary validation sets, and drift baselines are not afterthoughts — they're the safety net that catches failures before users do.

## Phase 1: A/B Test Dataset Construction

### Requirements

```python
def construct_ab_test_data(production_traffic, model_a, model_b, 
                            traffic_split=0.5, min_sample_size=10000,
                            stratification_cols=["region", "device_type"]):
    """
    Build A/B test dataset that enables valid statistical comparison.
    
    CRITICAL: Random assignment, not self-selection.
    User chooses A/B randomly, not because A looked better.
    """
    rng = np.random.default_rng(42)
    
    # Assign traffic to A or B randomly
    assignments = rng.binomial(1, traffic_split, len(production_traffic))
    
    # Stratified assignment within groups
    if stratification_cols:
        for group_key, group_df in production_traffic.groupby(stratification_cols):
            n = len(group_df)
            # Ensure balance within each stratum
            group_assignments = rng.permutation([0] * (n // 2) + [1] * (n - n // 2))
            # Apply to group
            ...
    
    test_dataset = {
        "model_a_predictions": model_a.predict(production_traffic[assignments == 0]),
        "model_b_predictions": model_b.predict(production_traffic[assignments == 1]),
        "assignments": assignments,
        "metadata": {
            "traffic_split": traffic_split,
            "min_sample_size": min_sample_size,
            "stratification": stratification_cols,
        },
    }
    
    # Power analysis: can we detect meaningful differences?
    n_a = (assignments == 0).sum()
    n_b = (assignments == 1).sum()
    mde = _minimum_detectable_effect(n_a, n_b)
    
    test_dataset["power_analysis"] = {
        "n_a": n_a, "n_b": n_b,
        "min_detectable_effect": mde,
        "adequate_power": mde < 0.02,  # can detect 2pp difference
    }
    
    return test_dataset
```

### A/B Test Validity Checks

| Check | What It Catches | Action if Failed |
|-------|-----------------|-----------------|
| Assignment balance | One model got more traffic | Rebalance; check for systematic bias |
| Pre-experiment equivalence | Groups differed before experiment | Check randomization; may need re-randomization |
| Stratification balance | Demographics skewed in one arm | Adjust assignment within strata |
| Minimum sample size | Underpowered experiment | Extend experiment duration |
| Contamination | Users saw both models | Exclude crossover users |

## Phase 2: Canary Validation

### Canary Set Construction

```python
def build_canary_set(production_traffic, canary_fraction=0.01, 
                     safety_metric_threshold=0.95):
    """
    Small subset of production traffic that tests new model SAFELY.
    If canary fails, model does NOT roll out to full traffic.
    """
    rng = np.random.default_rng(42)
    
    # Stratified sample: represent all production segments
    canary_idx = []
    for segment_key, segment_df in production_traffic.groupby("segment"):
        n_canary = max(10, int(len(segment_df) * canary_fraction))
        canary_idx.extend(rng.choice(segment_df.index, n_canary, replace=False))
    
    canary_set = production_traffic.loc[canary_idx]
    
    return {
        "data": canary_set,
        "canary_fraction": canary_fraction,
        "safety_threshold": safety_metric_threshold,
        "segments_covered": canary_set["segment"].nunique(),
        "n_canary": len(canary_set),
    }

def evaluate_canary(canary_set, baseline_model, candidate_model, 
                     metrics, safety_threshold=0.95):
    """
    Safety check: does the new model meet minimum performance?
    """
    baseline_scores = {m: evaluate(baseline_model, canary_set, m) for m in metrics}
    candidate_scores = {m: evaluate(candidate_model, canary_set, m) for m in metrics}
    
    failures = []
    for metric in metrics:
        ratio = candidate_scores[metric] / baseline_scores[metric]
        if ratio < safety_threshold:
            failures.append({
                "metric": metric,
                "baseline": baseline_scores[metric],
                "candidate": candidate_scores[metric],
                "ratio": ratio,
                "threshold": safety_threshold,
            })
    
    return {
        "baseline": baseline_scores,
        "candidate": candidate_scores,
        "failures": failures,
        "pass": len(failures) == 0,
        "can_rollout": len(failures) == 0,
    }
```

### Canary Failure Protocol

| Failure Severity | What Happened | Action |
|-----------------|---------------|--------|
| Any safety metric < threshold | New model is worse | BLOCK rollout, investigate |
| All metrics within 2% of baseline | Marginal degradation | Warn; manual review required |
| New model significantly better | Improvement confirmed | Proceed to full rollout |

## Phase 3: Rollback Triggers

### Performance Thresholds

```python
ROLLBACK_TRIGGERS = {
    "accuracy_drop": {
        "condition": "candidate_accuracy < baseline_accuracy * 0.98",
        "action": "AUTOMATIC_ROLLBACK",
        "cooldown": "30 minutes",
    },
    "latency_spike": {
        "condition": "p95_latency > baseline_p95 * 1.5",
        "action": "AUTOMATIC_ROLLBACK",
        "cooldown": "5 minutes",
    },
    "error_rate": {
        "condition": "error_rate > baseline_error_rate * 2.0",
        "action": "AUTOMATIC_ROLLBACK",
        "cooldown": "1 minute",
    },
    "prediction_anomaly": {
        "condition": "prediction_distribution_js_divergence > 0.2",
        "action": "ALERT_ONCALL_ROLLBACK",
        "cooldown": "15 minutes",
    },
    "customer_complaints": {
        "condition": "complaint_rate > baseline * 3.0",
        "action": "ALERT_ONCALL_ROLLBACK",
        "cooldown": "10 minutes",
    },
}
```

## Phase 4: Shadow Mode Telemetry

```python
def collect_shadow_telemetry(production_model, shadow_model, traffic, 
                              sample_rate=0.1):
    """
    Run shadow model on a SAMPLE of production traffic.
    Compare predictions without affecting users.
    """
    rng = np.random.default_rng(42)
    shadow_idx = rng.choice(len(traffic), int(len(traffic) * sample_rate), replace=False)
    
    telemetry = []
    for i in shadow_idx:
        instance = traffic[i]
        
        production_result = production_model.predict(instance)
        shadow_result = shadow_model.predict(instance)
        
        # If both agree and user took the action → strong signal
        # If shadow disagrees and user took production's action → shadow might be worse
        # If shadow disagrees and user didn't take action → shadow might be better
        actual_outcome = instance.get("user_action")  # if available
        
        telemetry.append({
            "instance_id": instance["id"],
            "production_prediction": production_result,
            "shadow_prediction": shadow_result,
            "agreement": production_result == shadow_result,
            "actual_outcome": actual_outcome,
        })
    
    return telemetry
```

## Phase 5: Production Drift Baselines

```python
def establish_drift_baseline(production_data, window_days=30):
    """
    What's "normal" for this model in production?
    Establishes the baseline against which future drift is measured.
    """
    baseline = production_data.tail(window_days)
    
    return {
        "established_date": datetime.now(timezone.utc).isoformat(),
        "window_days": window_days,
        "n_predictions": len(baseline),
        "metrics": {
            "mean_confidence": float(baseline["confidence"].mean()),
            "std_confidence": float(baseline["confidence"].std()),
            "prediction_distribution": baseline["prediction"].value_counts(normalize=True).to_dict(),
            "latency_p50": float(baseline["latency_ms"].median()),
            "latency_p95": float(baseline["latency_ms"].quantile(0.95)),
            "latency_p99": float(baseline["latency_ms"].quantile(0.99)),
            "error_rate": float((baseline["error"].notna()).mean()),
        },
    }
```

## Quality Gate

- A/B test dataset has adequate power (MDE < 2pp).
- Canary set covers ALL production segments.
- Rollback triggers defined with specific thresholds (not "if model is bad").
- Shadow mode telemetry running on ≥ 10% of traffic.
- Drift baseline established and version-controlled.
