---
name: niche-domain-data-task
description: Curate data for niche domains — chemical engineering, logistics/transportation, music/audio engineering, archaeology/paleontology, and culinary/food science. Process simulation validation, routing optimization, audio mastering quality, fossil dating calibration, and recipe scaling validation.
recommended_skills: [time-series-data-task, embedding-analysis, evaluation-dataset-design-task]
recommended_guidelines: [industry-verticals-data-task, specialized-modality-data-task]
---

## Chemical Engineering

```python
def validate_process_simulation(simulated_yield, actual_yield, operating_conditions):
    """Does simulation predict actual production output?"""
    errors = np.abs(np.array(simulated_yield) - np.array(actual_yield)) / np.maximum(np.array(actual_yield), 1)
    by_condition = {}
    for condition_name in set(operating_conditions):
        mask = np.array(operating_conditions) == condition_name
        by_condition[condition_name] = float(np.mean(errors[mask]))
    return {"overall_mape": float(np.mean(errors)*100), "per_condition": by_condition,
            "acceptable": np.mean(errors) < 0.1}

def detect_reaction_hazards(reaction_parameters, safety_thresholds):
    """Are reaction conditions within safe operating limits?"""
    violations = []
    for param, value in reaction_parameters.items():
        limit = safety_thresholds.get(param)
        if limit and value > limit["max"]:
            violations.append({"parameter": param, "value": value, "limit": limit["max"], 
                               "risk": limit.get("risk", "UNKNOWN")})
    return {"hazardous_conditions": len(violations), "requires_interlock": len(violations) > 0}
```

## Logistics / Transportation

```python
def validate_routing_optimization(predicted_routes, actual_routes, delivery_times, vehicle_capacities):
    """Does optimized routing actually save time and fuel?"""
    predicted_time = np.sum([r["estimated_time"] for r in predicted_routes])
    actual_time = np.sum([r["actual_time"] for r in actual_routes])
    predicted_fuel = np.sum([r["estimated_fuel"] for r in predicted_routes])
    actual_fuel = np.sum([r["actual_fuel"] for r in actual_routes])
    return {"time_savings_pct": float((predicted_time - actual_time) / max(predicted_time, 1) * 100),
            "fuel_savings_pct": float((predicted_fuel - actual_fuel) / max(predicted_fuel, 1) * 100),
            "optimization_works": actual_time < predicted_time}

def validate_fleet_utilization(vehicle_assignments, demand_forecast, actual_demand):
    """Are vehicles assigned where they're actually needed?"""
    utilization = {}
    for vehicle_id, assignments in vehicle_assignments.items():
        predicted = np.sum([demand_forecast.get(a["route"], 0) for a in assignments])
        actual = np.sum([actual_demand.get(a["route"], 0) for a in assignments])
        utilization[vehicle_id] = min(actual / max(predicted, 1), 1.0)
    return {"mean_utilization": float(np.mean(list(utilization.values()))),
            "underutilized": [k for k, v in utilization.items() if v < 0.5],
            "overutilized": [k for k, v in utilization.items() if v > 0.95]}
```

## Music / Audio Engineering

```python
def validate_audio_mastering(original_audio, mastered_audio, quality_dimensions):
    """Does mastering improve without destroying dynamics?"""
    results = {}
    for dim, measure_fn in quality_dimensions.items():
        orig_score = measure_fn(original_audio)
        mastered_score = measure_fn(mastered_audio)
        results[dim] = {"original": float(orig_score), "mastered": float(mastered_score),
                        "improvement": float(mastered_score - orig_score)}
    
    loudness_war = results.get("dynamic_range", {}).get("improvement", 0) < -3
    return {"quality_changes": results, "loudness_war_detected": loudness_war,
            "recommendation": "GOOD_MASTER" if not loudness_war else "OVER_COMPRESSED"}

def validate_music_recommendation(recommendations, user_satisfaction, genre_diversity):
    """Do recommendations satisfy without creating filter bubbles?"""
    satisfaction_rate = np.mean([s > 3 for s in user_satisfaction])
    diversity_score = 1 - _genre_concentration(recommendations)
    return {"satisfaction": float(satisfaction_rate), "diversity": float(diversity_score),
            "filter_bubble_risk": diversity_score < 0.3}
```

## Archaeology / Paleontology

```python
def validate_fossil_dating(layer_positions, radiometric_dates, stratigraphic_constraints):
    """Do dating estimates match stratigraphic ordering?"""
    violations = []
    for i in range(len(layer_positions) - 1):
        if layer_positions[i] < layer_positions[i+1] and radiometric_dates[i] < radiometric_dates[i+1]:
            violations.append({"layers": (i, i+1), "position_order": "deeper_older",
                               "date_order": "older_deeper_inverted"})
    return {"stratigraphic_consistency": len(violations) == 0, "violations": violations}

def cross_validate_dating_methods(method_a_dates, method_b_dates, accepted_uncertainty_years=1000):
    """Do different dating methods agree?"""
    discrepancies = [abs(a - b) for a, b in zip(method_a_dates, method_b_dates)]
    return {"mean_discrepancy_years": float(np.mean(discrepancies)),
            "consistent": np.mean(discrepancies) < accepted_uncertainty_years,
            "recommendation": "ACCEPT_DATES" if np.mean(discrepancies) < accepted_uncertainty_years else "INVESTIGATE_DISCREPANCY"}
```

## Culinary / Food Science

```python
def validate_recipe_scaling(original_recipe, scaled_recipe, tasting_scores, n_tasters=5):
    """Does scaled recipe produce same quality as original?"""
    quality_ratio = np.mean(scaled_recipe["scores"]) / max(np.mean(original_recipe["scores"]), 1)
    consistency = 1 - np.std(scaled_recipe["scores"]) / max(np.std(original_recipe["scores"]), 1e-6)
    return {"quality_retention": float(quality_ratio), "consistency": float(consistency),
            "scaling_works": quality_ratio > 0.85 and consistency > 0.7,
            "recommendation": "SCALE_UP" if quality_ratio > 0.85 else "ADJUST_RECIPE"}

def validate_sensory_panel(panelist_scores, reference_scores, panelist_experience):
    """Do trained panelists agree with reference standards?"""
    accuracy = {}
    for panelist_id, scores in panelist_scores.items():
        accuracy[panelist_id] = float(np.mean([abs(s - r) < 1 for s, r in zip(scores, reference_scores)]))
    return {"panelist_accuracy": accuracy, "panel_calibrated": np.mean(list(accuracy.values())) > 0.8,
            "needs_retraining": [k for k, v in accuracy.items() if v < 0.7]}
```

## Quality Gate

- Chemical: process simulation MAPE < 10%; zero hazardous conditions.
- Logistics: optimization produces time/fuel savings; utilization 50-95%.
- Music: mastering improves quality without loudness war.
- Archaeology: stratigraphic consistency; multi-method agreement.
- Culinary: recipe scaling quality retention > 85%; panel calibrated.
