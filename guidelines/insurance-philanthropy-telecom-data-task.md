---
name: insurance-philanthropy-telecom-data-task
description: Curate data for insurance/actuarial beyond basics, philanthropy/nonprofit, and telecommunications/network — claim pattern anomaly detection, program effectiveness measurement, network congestion prediction, and signal quality validation.
recommended_skills: [anomaly-detection, time-series-data-task, evaluation-dataset-design-task, embedding-analysis]
recommended_guidelines: [industry-verticals-data-task, experimental-methodology-data-task, robustness-engineering-task]
---

## Insurance / Actuarial Beyond Basics

```python
def detect_claim_anomalies(claims, historical_patterns, anomaly_threshold=3):
    """Fraud vs legitimate unusual claims — don't confuse the two."""
    anomalies = []
    for claim in claims:
        expected = _predict_expected_claim(claim, historical_patterns)
        deviation = abs(claim["amount"] - expected) / max(expected, 1)
        context_score = _assess_claim_context(claim)  # unusual but explainable?
        if deviation > anomaly_threshold and context_score < 0.3:
            anomalies.append({"claim_id": claim["id"], "deviation_sigma": deviation,
                              "classification": "SUSPICIOUS" if deviation > 5 else "UNUSUAL"})
    return {"anomalies": anomalies, "anomaly_rate": len(anomalies) / max(len(claims), 1),
            "fraud_likely": len([a for a in anomalies if a["classification"]=="SUSPICIOUS"])}

def validate_risk_pooling(pool_claims, pool_premiums, confidence_level=0.99):
    """Does the risk pool have adequate reserves?"""
    total_premiums = sum(pool_premiums)
    total_claims = sum(pool_claims)
    loss_ratio = total_claims / max(total_premiums, 1)
    var_99 = np.percentile(pool_claims, confidence_level * 100)
    return {"loss_": float(loss_ratio), "var_99": float(var_99),
            "solvent": total_premiums > var_99,
            "recommendation": "ADEQUATE_RESERVES" if total_premiums > var_99 else "INCREASE_PREMIUMS_OR_REINSURANCE"}

def validate_underwriting_calibration(predicted_risk, actual_claims, risk_deciles=10):
    """Do model predictions match actual claim rates?"""
    decile_boundaries = np.percentile(predicted_risk, np.linspace(0, 100, risk_deciles + 1))
    calibration = []
    for i in range(risk_deciles):
        mask = (predicted_risk >= decile_boundaries[i]) & (predicted_risk < decile_boundaries[i+1])
        if mask.sum() < 10: continue
        calibration.append({"decile": i+1, "predicted_mean_risk": float(np.mean(predicted_risk[mask])),
                            "actual_claim_rate": float(np.mean(actual_claims[mask]))})
    return calibration
```

## Philanthropy / Nonprofit

```python
def measure_program_effectiveness(program_inputs, program_outputs, control_group=None):
    """Does the program actually help?"""
    effect_size = (np.mean(program_outputs) - np.mean(program_inputs)) / max(np.std(program_inputs), 1e-6)
    
    if control_group:
        control_effect = np.mean(control_group["outputs"]) - np.mean(control_group["inputs"])
        treatment_effect = np.mean(program_outputs) - np.mean(program_inputs)
        diff_in_diff = treatment_effect - control_effect
        return {"effect_size": float(effect_size), "diff_in_diff": float(diff_in_diff),
                "program_works": diff_in_diff > 0.2 * abs(control_effect),
                "impact_estimate": diff_in_diff * len(program_inputs)}
    
    return {"effect_size": float(effect_size), "causal_claim_possible": False,
            "needs_control_group": True}

def validate_grant_allocation(grant_allocations, impact_measurements):
    """Does more money → more impact?"""
    from scipy.stats import spearmanr
    amounts = [g["amount"] for g in grant_allocations]
    impacts = [impact_measurements.get(g["id"], 0) for g in grant_allocations]
    rho, p = spearmanr(amounts, impacts)
    return {"allocation_impact_correlation": float(rho), "significant": p < 0.05,
            "efficient_allocation": rho > 0.5}
```

## Telecommunications / Network

```python
def validate_signal_quality(reported_signals, measured_signals, cell_ids):
    """Reported vs measured — what's actual vs what's claimed?"""
    discrepancies = []
    for reported, measured, cell in zip(reported_signals, measured_signals, cell_ids):
        delta = abs(reported - measured)
        if delta / max(measured, 1) > 0.2:
            discrepancies.append({"cell": cell, "reported": reported, "measured": measured, "delta_pct": float(delta/measured*100)})
    return {"discrepancy_rate": len(discrepancies) / max(len(reported_signals), 1),
            "acceptable": len(discrepancies) / max(len(reported_signals), 1) < 0.05}

def validate_congestion_predictions(predictions, actual_latency, time_points):
    """Do predicted congestion levels match actual network behavior?"""
    mape = np.mean(np.abs(predictions - actual_latency) / np.maximum(actual_latency, 1))
    peak_error = np.max(np.abs(predictions - actual_latency))
    return {"mape": float(mape), "peak_error_ms": float(peak_error),
            "usable": mape < 0.15, "recommendation": "USE_FOR_CAPACITY_PLANNING" if mape < 0.15 else "RECALIBRATE"}

def detect_interference_sources(signal_readings, known_interference_patterns, location_data):
    """What's causing the interference?"""
    matches = []
    for pattern in known_interference_patterns:
        for location in location_data:
            correlation = np.corrcoef(signal_readings[location["id"]], pattern["signature"])[0, 1]
            if correlation > 0.7:
                matches.append({"location": location["id"], "source_type": pattern["type"], 
                                "correlation": float(correlation)})
    return {"interference_sources": matches, "n_sources": len(matches),
            "action_required": len(matches) > 0}
```

## Quality Gate

- Insurance: loss ratio within solvent range; underwriting calibration across all deciles.
- Philanthropy: programs measured with control groups; allocation correlated with impact.
- Telecom: signal discrepancy < 5%; congestion MAPE < 15%; interference sources identified.
