---
name: manufacturing-retail-construction-data-task
description: Curate data for manufacturing/industrial IoT, retail/e-commerce, hospitality/travel, and construction/engineering — sensor fusion validation, demand forecasting, inventory optimization, fraud detection, and structural integrity monitoring.
recommended_skills: [time-series-data-task, embedding-analysis, data-pipeline-monitoring-task, anomaly-detection]
recommended_guidelines: [industry-verticals-data-task, streaming-edge-mesh-data-task, evaluation-dataset-design-task]
---

## Manufacturing / Industrial IoT

```python
def validate_sensor_fusion(sensor_readings, ground_truth, fusion_method):
    """Multiple sensors observing same phenomenon — do they agree?"""
    n_sensors = len(sensor_readings)
    fusion_output = fusion_method(sensor_readings)
    fusion_error = np.mean(np.abs(fusion_output - ground_truth))
    
    individual_errors = [np.mean(np.abs(s - ground_truth)) for s in sensor_readings]
    fusion_improvement = np.mean(individual_errors) - fusion_error
    
    return {"fusion_error": float(fusion_error), "individual_errors": [float(e) for e in individual_errors],
            "fusion_improvement": float(fusion_improvement),
            "fusion_helps": fusion_improvement > 0,
            "recommendation": "USE_FUSION" if fusion_improvement > 0 else "BEST_SINGLE_SENSOR"}

def calibrate_predictive_maintenance(model_predictions, actual_failures, lead_time_days):
    """Do predictions arrive before failures, with enough time to act?"""
    results = []
    for failure_date, predictions in zip(actual_failures, model_predictions):
        first_warning = next((p for p in reversed(predictions) if p["probability"] > 0.7), None)
        if first_warning:
            warning_days_before = (failure_date - first_warning["date"]).days
            results.append({"detected": True, "lead_time_days": warning_days_before, 
                            "sufficient": warning_days_before >= lead_time_days})
        else:
            results.append({"detected": False, "lead_time_days": 0})
    
    return {"detection_rate": np.mean([r["detected"] for r in results]),
            "mean_lead_time_days": np.mean([r["lead_time_days"] for r in results if r["detected"]]),
            "sufficient_lead_time": np.mean([r["sufficient"] for r in results if r["detected"]])}

def trace_defect_root_cause(defect_instances, process_parameters, process_steps):
    """Which process step caused the defect?"""
    step_correlations = {}
    for step in process_steps:
        param_values = [d[step] for d in defect_instances]
        step_correlations[step] = {"variance": float(np.var(param_values)),
                                    "suspicious": np.var(param_values) > np.mean([np.var([d[s] for d in defect_instances]) for s in process_steps]) * 2}
    return {"likely_root_cause": max(step_correlations, key=lambda k: step_correlations[k]["variance"]),
            "step_analysis": step_correlations}
```

## Retail / E-Commerce

```python
def validate_demand_forecast(forecast, actual_sales, product_categories):
    """Predictions vs what actually sold — by category."""
    results = {}
    for cat in set(product_categories):
        mask = np.array(product_categories) == cat
        cat_mape = np.mean(np.abs(forecast[mask] - actual_sales[mask]) / np.maximum(actual_sales[mask], 1)) * 100
        results[cat] = float(cat_mape)
    overall_mape = np.mean(np.abs(forecast - actual_sales) / np.maximum(actual_sales, 1)) * 100
    return {"overall_mape": float(overall_mape), "per_category": results,
            "acceptable": overall_mape < 20, "worst_category": max(results, key=results.get)}

def audit_inventory_optimization(predicted_stockouts, actual_stockouts, product_value):
    """Did optimization prevent stockouts without overstocking?"""
    tp = sum(1 for p, a in zip(predicted_stockouts, actual_stockouts) if p and a)
    fp = sum(1 for p, a in zip(predicted_stockouts, actual_stockouts) if p and not a)
    fn = sum(1 for p, a in zip(predicted_stockouts, actual_stockouts) if not p and a)
    return {"stockout_precision": tp / max(tp + fp, 1), "stockout_recall": tp / max(tp + fn, 1),
            "missed_stockout_cost": sum(product_value[i] for i in range(len(actual_stockouts)) if not predicted_stockouts[i] and actual_stockouts[i])}

def detect_retail_fraud(flagged_transactions, confirmed_fraud, transaction_values):
    """Detected vs actual fraud — are expensive frauds being caught?"""
    tp = sum(1 for f, c in zip(flagged_transactions, confirmed_fraud) if f and c)
    fp = sum(1 for f, c in zip(flagged_transactions, confirmed_fraud) if f and not c)
    fn = sum(1 for f, c in zip(flagged_transactions, confirmed_fraud) if not f and c)
    missed_fraud_value = sum(v for f, c, v in zip(flagged_transactions, confirmed_fraud, transaction_values) if not f and c)
    return {"precision": tp / max(tp + fp, 1), "recall": tp / max(tp + fn, 1),
            "missed_fraud_value": float(missed_fraud_value),
            "high_risk": missed_fraud_value / max(sum(transaction_values), 1) > 0.1}
```

## Construction / Engineering

```python
def validate_project_progress(reported_completion_pct, actual_completion_pct, milestones):
    """Reported vs actual — are projects on track?"""
    discrepancies = [r - a for r, a in zip(reported_completion_pct, actual_completion_pct)]
    milestone_accuracy = {}
    for ms in milestones:
        ms_preds = [p for p, m in zip(reported_completion_pct, milestones) if m == ms["name"]]
        ms_actuals = [a for a, m in zip(actual_completion_pct, milestones) if m == ms["name"]]
        if ms_preds: milestone_accuracy[ms["name"]] = float(np.mean(np.abs(np.array(ms_preds) - np.array(ms_actuals))))
    return {"mean_overstatement": float(np.mean(discrepancies)),
            "projects_over_reported": float(np.mean([d > 0.05 for d in discrepancies])),
            "milestone_accuracy": milestone_accuracy}

def monitor_structural_integrity(sensor_readings, safety_thresholds, structure_components):
    """Are sensor readings within safety limits for each component?"""
    alerts = []
    for component in structure_components:
        readings = sensor_readings.get(component["id"], [])
        threshold = safety_thresholds.get(component["type"], float("inf"))
        exceeding = [r for r in readings if r > threshold]
        if exceeding:
            trend = np.polyfit(range(len(readings[-30:])), readings[-30:], 1)[0] if len(readings) >= 30 else 0
            alerts.append({"component": component["id"], "type": component["type"],
                           "max_exceedance": max(exceeding), "trend": "DETERIORATING" if trend > 0 else "STABLE",
                           "action": "INSPECT" if max(exceeding) > threshold * 1.1 else "MONITOR"})
    return {"alerts": alerts, "structures_at_risk": len(alerts),
            "critical": [a for a in alerts if a["action"] == "INSPECT"]}
```

## Quality Gate

- Manufacturing: sensor fusion improvement > 0; predictive maintenance detection > 80%.
- Retail: demand forecast MAPE < 20%; fraud recall > 70%.
- Construction: project over-reporting < 10%; zero critical structural alerts.
