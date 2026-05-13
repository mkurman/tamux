---
name: legal-environmental-climate-data-task
description: Curate data for legal/justice systems and environmental/climate — case outcome validation, judicial bias detection, climate model ensemble validation, extreme event attribution, and carbon offset verification.
recommended_skills: [bias-audit, embedding-analysis, evaluation-dataset-design-task, data-contamination-task]
recommended_guidelines: [intersectional-evaluation-task, robustness-engineering-task, dataset-governance-task]
---

## Legal / Justice System

### Case Outcome Validation

```python
def validate_case_predictions(predictions, actual_outcomes, protected_attributes):
    """Do predictions match actual verdicts? By protected group?"""
    overall_accuracy = accuracy_score(actual_outcomes, predictions)
    group_metrics = {}
    for attr, groups in protected_attributes.items():
        for group in groups:
            mask = groups[group]
            if mask.sum() < 10: continue
            group_metrics[f"{attr}_{group}"] = accuracy_score(actual_outcomes[mask], predictions[mask])
    
    disparities = {k: v - overall_accuracy for k, v in group_metrics.items()}
    return {"overall_accuracy": overall_accuracy, "group_metrics": group_metrics,
            "max_disparity": max(abs(d) for d in disparities.values()),
            "biased": any(abs(d) > 0.1 for d in disparities.values()),
            "worst_group": min(group_metrics, key=group_metrics.get)}

def detect_judicial_bias(decisions, judge_demographics, defendant_demographics):
    """Systematic patterns in sentencing/decisions by judge-defendant combinations."""
    bias_matrix = {}
    for judge_id, judge_info in judge_demographics.items():
        for defendant_group in defendant_demographics:
            cases = decisions[(decisions["judge_id"]==judge_id) & (decisions["defendant_group"]==defendant_group)]
            if len(cases) < 20: continue
            bias_matrix[f"{judge_id}_{defendant_group}"] = {"rate": cases["harsh_outcome"].mean(), "n": len(cases)}
    return bias_matrix

def audit_legal_precedent_evolution(precedents, temporal_decisions):
    """How do precedents evolve? Track influence chains."""
    citation_graph = {}
    for case_id, case_info in precedents.items():
        later_cites = [d for d in temporal_decisions if case_id in d.get("cites", [])]
        citation_graph[case_id] = {"n_citing_cases": len(later_cites), 
                                    "time_span_years": max(d["year"] for d in later_cites) - case_info["year"] if later_cites else 0}
    return citation_graph
```

### Recidivism Calibration

```python
def validate_recidivism_prediction(scores, outcomes, race_groups, time_window_years=2):
    """Fairness + accuracy tradeoff — does model treat all groups equally?"""
    from sklearn.metrics import roc_auc_score
    overall_auc = roc_auc_score(outcomes, scores)
    group_aucs = {}
    for race, mask in race_groups.items():
        if mask.sum() < 20: continue
        group_aucs[race] = roc_auc_score(outcomes[mask], scores[mask])
    return {"overall_auc": overall_auc, "group_aucs": group_aucs,
            "auc_disparity": max(group_aucs.values()) - min(group_aucs.values()),
            "fair": max(group_aucs.values()) - min(group_aucs.values()) < 0.05}
```

## Environmental / Climate

### Climate Model Ensemble Validation

```python
def validate_climate_ensemble(model_predictions, observations, lead_times=[1, 5, 10]):
    """Do ensemble predictions match observations at various lead times?"""
    results = {}
    for lead in lead_times:
        ensemble_mean = np.mean([m[lead] for m in model_predictions], axis=0)
        rmse = np.sqrt(np.mean((ensemble_mean - observations[lead]) ** 2))
        spread = np.std([m[lead] for m in model_predictions], axis=0).mean()
        results[f"lead_{lead}y"] = {"rmse": float(rmse), "ensemble_spread": float(spread),
                                     "calibrated": abs(rmse - spread) / max(spread, 1e-6) < 0.3}
    return results

def detect_extreme_events(historical_data, current_observation, sigma_threshold=3):
    """Is this event unprecedented?"""
    mean, std = np.mean(historical_data), np.std(historical_data)
    deviation = (current_observation - mean) / std
    return {"sigma_deviation": float(deviation), "extreme": abs(deviation) > sigma_threshold,
            "classification": "UNPRECEDENTED" if abs(deviation) > 4 else "EXTREME" if abs(deviation) > 3 else "NORMAL"}
```

### Carbon Offset Verification

```python
def verify_carbon_offset(claimed_offset_tons, project_data, verification_method):
    """Claimed vs actual reduction — the biggest gap in climate policy."""
    measured = _estimate_actual_reduction(project_data, verification_method)
    gap = claimed_offset_tons - measured
    return {"claimed_tons": claimed_offset_tons, "measured_tons": measured,
            "gap_tons": gap, "verification_ratio": measured / max(claimed_offset_tons, 1),
            "credible": measured / max(claimed_offset_tons, 1) > 0.8,
            "rating": "HIGH_INTEGRITY" if measured/claimed_offset_tons > 0.8 else "OVERSTATED"}
```

### Sensor Network Calibration

```python
def validate_sensor_network(sensors, calibration_events):
    """Distributed monitoring — are all sensors telling the same story?"""
    cross_sensor_consistency = {}
    for event in calibration_events:
        readings = {s["id"]: s["readings"].get(event["timestamp"]) for s in sensors}
        valid = [v for v in readings.values() if v is not None]
        if len(valid) < 3: continue
        mean, std = np.mean(valid), np.std(valid)
        outliers = {sid: val for sid, val in readings.items() if val and abs(val-mean) > 2*std}
        cross_sensor_consistency[event["id"]] = {"n_outliers": len(outliers), 
                                                   "outlier_sensors": list(outliers.keys())}
    return cross_sensor_consistency
```

## Quality Gate

- Legal: max group disparity < 10pp; recidivism AUC disparity < 0.05.
- Climate: ensemble RMSE matches spread (calibrated); extreme events classified with σ deviation.
- Carbon: verification ratio > 80% for credible offsets.
- Sensors: < 5% outlier rate across calibration events.
