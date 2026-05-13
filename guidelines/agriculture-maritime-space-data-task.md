---
name: agriculture-maritime-space-data-task
description: Curate data for agriculture/food systems, maritime/ocean monitoring, and space/astronomical observation — crop yield validation, marine population estimation, orbital prediction accuracy, and exoplanet detection validation.
recommended_skills: [time-series-data-task, embedding-analysis, evaluation-dataset-design-task]
recommended_guidelines: [satellite-geospatial-sources-task, robustness-engineering-task, data-contamination-task]
---

## Agriculture / Food Systems

### Crop Yield Validation

```python
def validate_yield_prediction(predictions, actual_harvests, regions, crop_types):
    """Do predictions match what actually came out of the ground?"""
    results = {}
    for crop in crop_types:
        crop_preds = [p for p, a, r, c in zip(predictions, actual_harvests, regions, crop_types) if c == crop]
        crop_actuals = [a for p, a, r, c in zip(predictions, actual_harvests, regions, crop_types) if c == crop]
        mape = np.mean(np.abs(np.array(crop_preds) - np.array(crop_actuals)) / np.maximum(np.array(crop_actuals), 1)) * 100
        results[crop] = {"mape_pct": float(mape), "usable": mape < 15}
    
    by_region = {}
    for region in set(regions):
        mask = np.array(regions) == region
        region_mape = np.mean(np.abs(predictions[mask] - actual_harvests[mask]) / np.maximum(actual_harvests[mask], 1)) * 100
        by_region[region] = float(region_mape)
    
    return {"per_crop": results, "per_region": by_region,
            "overall_mape": np.mean(list(r["mape_pct"] for r in results.values())),
            "acceptable": np.mean(list(r["mape_pct"] for r in results.values())) < 20}
```

### Soil Health Assessment

```python
def validate_soil_proxy(proxy_measurements, lab_results, soil_attributes):
    """Do proxy measurements match actual lab-tested soil quality?"""
    correlations = {}
    for attr in soil_attributes:
        if attr in proxy_measurements and attr in lab_results:
            r = np.corrcoef(proxy_measurements[attr], lab_results[attr])[0, 1]
            correlations[attr] = {"r": float(r), "valid_proxy": abs(r) > 0.7}
    return {"correlations": correlations, "valid_proxies": sum(1 for v in correlations.values() if v["valid_proxy"]),
            "recommendation": "USE_PROXIES" if sum(1 for v in correlations.values() if v["valid_proxy"]) > len(soil_attributes)/2 else "NEED_LAB_TESTS"}

PEST_OUTBREAK_SIGNALS = {
    "temperature_anomaly": "Unseasonably warm = earlier pest emergence",
    "humidity_spike": "High humidity = fungal disease risk",
    "wind_pattern_change": "Wind direction shift = pest migration route",
    "crop_stress_ndvi": "Declining NDVI = potential infestation",
}
```

### Food Safety Compliance

```python
def verify_food_safety(inspection_results, actual_safety_outcomes):
    """Inspection vs actual safety — do inspections predict outcomes?"""
    tp = sum(1 for insp, actual in zip(inspection_results, actual_safety_outcomes) if insp == "fail" and actual == "fail")
    fn = sum(1 for insp, actual in zip(inspection_results, actual_safety_outcomes) if insp == "pass" and actual == "fail")
    fp = sum(1 for insp, actual in zip(inspection_results, actual_safety_outcomes) if insp == "fail" and actual == "pass")
    return {"false_negatives": fn, "missed_contamination_events": fn,
            "inspection_sensitivity": tp / max(tp + fn, 1),
            "recommendation": "IMPROVE_INSPECTION" if fn > 0 else "ADEQUATE"}
```

## Maritime / Ocean

### Ocean Current Validation

```python
def validate_current_model(predictions, drifter_trajectories, forecast_hours=[24, 48, 72]):
    """Do predicted currents match actual drifter trajectories?"""
    results = {}
    for hours in forecast_hours:
        separation_km = []
        for pred, actual in zip(predictions, drifter_trajectories):
            pred_pos = pred.get_position_at(hours)
            actual_pos = actual.get_position_at(hours)
            if pred_pos and actual_pos:
                separation_km.append(_haversine_km(pred_pos, actual_pos))
        results[f"{hours}h"] = {"mean_error_km": np.mean(separation_km), 
                                  "max_error_km": np.max(separation_km)}
    return results

def estimate_marine_population(survey_counts, actual_population_estimates, confidence=0.95):
    """Survey vs actual population — the fundamental challenge in fisheries."""
    ratios = [s / max(a, 1) for s, a in zip(survey_counts, actual_population_estimates)]
    return {"mean_survey_ratio": np.mean(ratios), "bias_direction": "OVERCOUNT" if np.mean(ratios) > 1.1 else "UNDERCOUNT" if np.mean(ratios) < 0.9 else "ACCURATE",
            "correction_factor": 1 / max(np.mean(ratios), 0.01)}
```

## Space / Astronomical

### Orbital Prediction Accuracy

```python
def validate_orbit_predictions(predicted_positions, observed_positions, objects):
    """Do predicted orbital paths match observed trajectories?"""
    errors = []
    for obj_id in objects:
        pred = predicted_positions[obj_id]
        obs = observed_positions[obj_id]
        error_km = np.linalg.norm(np.array(pred) - np.array(obs))
        errors.append({"object": obj_id, "error_km": float(error_km),
                        "acceptable": error_km < 10})  # 10km threshold for LEO
    return {"mean_error_km": np.mean([e["error_km"] for e in errors]),
            "max_error_km": np.max([e["error_km"] for e in errors]),
            "acceptable_rate": np.mean([e["acceptable"] for e in errors])}

def validate_exoplanet_detection(candidates, confirmed, detection_method):
    """Discovery vs false positive — how many candidates are real?"""
    confirmed_set = set(c["id"] for c in confirmed)
    results = []
    for candidate in candidates:
        is_confirmed = candidate["id"] in confirmed_set
        results.append({"id": candidate["id"], "signal_to_noise": candidate["snr"],
                         "confirmed": is_confirmed, "method": detection_method})
    
    snr_thresholds = np.arange(3, 20, 1)
    precision_at_threshold = []
    for thr in snr_thresholds:
        above = [r for r in results if r["signal_to_noise"] > thr]
        if not above: continue
        precision_at_threshold.append({"snr_threshold": float(thr), 
                                        "precision": np.mean([r["confirmed"] for r in above]),
                                        "n_candidates": len(above)})
    
    return {"candidates": len(candidates), "confirmed": len(confirmed_set - set(c["id"] for c in candidates)),
            "false_positive_rate": 1 - len(confirmed_set & set(c["id"] for c in candidates)) / max(len(candidates), 1),
            "precision_by_snr": precision_at_threshold}
```

## Quality Gate

- Agriculture: yield MAPE < 20%; soil proxies correlate with lab > 0.7.
- Food safety: zero false negatives (missed contamination).
- Maritime: current model error < 50km at 72h.
- Space: orbit error < 10km for LEO; exoplanet FPR < 50% at SNR > 5.
