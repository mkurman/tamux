---
name: sports-education-urban-data-task
description: Curate data for sports/performance analytics, educational learning analytics, and urban/city planning — biomechanics validation, learning trajectory reconstruction, traffic simulation, housing demand prediction, and emergency response optimization.
recommended_skills: [time-series-data-task, embedding-analysis, evaluation-dataset-design-task, anomaly-detection]
recommended_guidelines: [industry-verticals-data-task, experimental-methodology-data-task, satellite-geospatial-sources-task]
---

## Sports / Performance Analytics

```python
def validate_biomechanics_data(sensor_readings, video_reconstruction, joint_angles):
    """Sensor calibration — do readings match video ground truth?"""
    errors = []
    for joint in joint_angles:
        sensor_angle = sensor_readings.get(joint)
        video_angle = video_reconstruction.get(joint)
        if sensor_angle is not None and video_angle is not None:
            errors.append(abs(sensor_angle - video_angle))
    return {"mean_error_degrees": float(np.mean(errors)) if errors else 0,
            "max_error_degrees": float(np.max(errors)) if errors else 0,
            "acceptable": np.mean(errors) < 5 if errors else False}

def quantify_team_dynamics(player_positions, ball_position, event_outcomes):
    """Communication and coordination — do team patterns predict outcomes?"""
    from sklearn.ensemble import RandomForestClassifier
    features = []
    for positions, ball in zip(player_positions, ball_position):
        features.append([_team_spread(positions), _formation_entropy(positions), _distance_to_ball(positions, ball)])
    model = RandomForestClassifier().fit(features, event_outcomes)
    return {"coordination_predictive_power": float(model.score(features, event_outcomes)),
            "dynamics_matter": model.score(features, event_outcomes) > 0.55}
```

## Educational Learning Analytics

```python
def reconstruct_learning_trajectory(assessment_results, time_points, knowledge_components):
    """Student knowledge state evolution — what did they learn when?"""
    trajectories = {}
    for kc in knowledge_components:
        kc_scores = [a.get(kc) for a in assessment_results]
        learning_rate = np.polyfit(time_points, kc_scores, 1)[0] if len(kc_scores) >= 3 else 0
        trajectories[kc] = {"final_mastery": kc_scores[-1] if kc_scores else 0,
                              "learning_rate": float(learning_rate),
                              "mastered": kc_scores[-1] > 0.8 if kc_scores else False}
    return trajectories

def validate_assessment_validity(assessment_scores, external_validation_scores):
    """Does the test measure what it claims to measure?"""
    correlation = np.corrcoef(assessment_scores, external_validation_scores)[0, 1]
    return {"criterion_validity": float(correlation),
            "valid": correlation > 0.6,
            "recommendation": "USE_ASSESSMENT" if correlation > 0.6 else "REVISE_ASSESSMENT"}

def detect_dropout_risk(student_features, actual_dropouts, time_window_days=30):
    """Early warning — can we identify at-risk students before they leave?"""
    from sklearn.metrics import roc_auc_score
    risk_scores = _compute_risk_scores(student_features)
    auc = roc_auc_score(actual_dropouts, risk_scores)
    top_decile = np.argsort(risk_scores)[-len(risk_scores)//10:]
    precision = np.mean([actual_dropouts[i] for i in top_decile])
    return {"auc": float(auc), "precision_at_top10pct": float(precision),
            "actionable": precision > 0.3,
            "recommendation": "DEPLOY_INTERVENTION" if precision > 0.3 else "IMPROVE_MODEL"}
```

## Urban / City Planning

```python
def validate_traffic_simulation(simulated_traffic, real_traffic_sensors, time_windows):
    """Does simulation match real traffic at key intersections?"""
    intersection_errors = {}
    for intersection_id, sim_data in simulated_traffic.items():
        real_data = real_traffic_sensors.get(intersection_id, [])
        if len(sim_data) != len(real_data): continue
        mape = np.mean(np.abs(np.array(sim_data) - np.array(real_data)) / np.maximum(np.array(real_data), 1)) * 100
        intersection_errors[intersection_id] = float(mape)
    return {"mean_mape": float(np.mean(list(intersection_errors.values()))),
            "acceptable": np.mean(list(intersection_errors.values())) < 20,
            "worst_intersection": max(intersection_errors, key=intersection_errors.get)}

def validate_housing_demand_prediction(predictions, actual_prices, regions, price_tiers):
    """Policy depends on these predictions being right."""
    results = {}
    for region in set(regions):
        mask = np.array(regions) == region
        if mask.sum() < 5: continue
        mape = np.mean(np.abs(predictions[mask] - actual_prices[mask]) / np.maximum(actual_prices[mask], 1)) * 100
        results[region] = float(mape)
    return {"per_region": results, "overall_mape": float(np.mean(list(results.values()))),
            "policy_ready": np.mean(list(results.values())) < 15}

def optimize_emergency_response(predicted_times, actual_times, station_locations, incident_locations):
    """Response time predictions — do they match reality?"""
    errors = np.abs(np.array(predicted_times) - np.array(actual_times))
    return {"mean_error_minutes": float(np.mean(errors)),
            "p95_error_minutes": float(np.percentile(errors, 95)),
            "acceptable": np.mean(errors) < 2,  # within 2 minutes
            "recommendation": "DEPLOY" if np.mean(errors) < 2 else "RECALIBRATE"}
```

## Quality Gate

- Sports: biomechanics error < 5°; team dynamics predictive power > 0.55.
- Education: assessment validity > 0.6; dropout detection precision > 30%.
- Urban: traffic simulation MAPE < 20%; housing demand MAPE < 15%; emergency response error < 2 minutes.
