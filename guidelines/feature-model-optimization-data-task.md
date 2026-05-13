---
name: feature-model-optimization-data-task
description: Curate data for feature engineering and model optimization — feature selection stability, drift detection, interaction discovery, NAS evaluation, hyperparameter validation, distillation alignment, pruning impact, and quantization error patterns.
recommended_skills: [embedding-analysis, data-diff, label-quality-audit, bias-audit]
recommended_guidelines: [scaling-law-data-task, evaluation-dataset-design-task, curriculum-learning-data-task]
---

## Feature Engineering

```python
def validate_feature_transform(original_data, transformed_data, target_col, tolerance=0.02):
    """Does transformation preserve information?"""
    from sklearn.feature_selection import mutual_info_regression
    mi_original = mutual_info_regression(original_data.drop(columns=[target_col]), original_data[target_col])
    mi_transformed = mutual_info_regression(transformed_data.drop(columns=[target_col]), transformed_data[target_col])
    info_loss = (mi_original - mi_transformed) / mi_original
    return {"info_loss": float(np.mean(info_loss)), "preserved": np.mean(np.abs(info_loss)) < tolerance}

def measure_feature_selection_stability(selection_results_across_splits):
    """Does selection generalize across splits?"""
    sets = [set(r["selected"]) for r in selection_results_across_splits]
    pairwise_jaccard = [len(sets[i] & sets[j]) / len(sets[i] | sets[j]) for i in range(len(sets)) for j in range(i+1, len(sets))]
    return {"mean_jaccard": float(np.mean(pairwise_jaccard)), "stable": np.mean(pairwise_jaccard) > 0.8}

def detect_feature_drift(reference_distributions, current_distributions, threshold=0.1):
    """When do feature distributions shift?"""
    drifts = {}
    for feature in reference_distributions:
        js = jensenshannon(reference_distributions[feature], current_distributions[feature])
        drifts[feature] = {"js_divergence": float(js), "drifted": js > threshold}
    return {"drifted_features": [k for k, v in drifts.items() if v["drifted"]],
            "drift_fraction": np.mean([v["drifted"] for v in drifts.values()])}

def discover_feature_interactions(features, target, model, n_top=20):
    """Which features work together?"""
    from sklearn.inspection import permutation_importance
    importances = permutation_importance(model, features, target, n_repeats=5)
    return {"top_single_features": np.argsort(importances.importances_mean)[-n_top:],
            "interaction_analysis": "Use SHAP dependence plots or Friedman's H-statistic"}
```

## Model Optimization

```python
def design_nas_search_space(model_family, task_constraints):
    """What architectures are viable to search?"""
    constraints = {"max_params": task_constraints.get("max_params_m", 100),
                   "max_latency_ms": task_constraints.get("max_latency_ms", 100),
                   "min_accuracy": task_constraints.get("min_accuracy", 0.8)}
    return {"search_space": f"{model_family} variants within {constraints}",
            "estimated_search_size": _estimate_nas_space_size(model_family, constraints)}

def validate_hyperparameter_importance(tuning_results, n_top=5):
    """Which hyperparameters matter?"""
    from sklearn.ensemble import RandomForestRegressor
    X = tuning_results[["lr", "batch_size", "dropout", "weight_decay", "warmup_steps"]]
    y = tuning_results["final_accuracy"]
    rf = RandomForestRegressor(n_estimators=100).fit(X, y)
    return {"importances": dict(zip(X.columns, rf.feature_importances_)),
            "most_important": X.columns[np.argmax(rf.feature_importances_)],
            "negligible": [c for c, imp in zip(X.columns, rf.feature_importances_) if imp < 0.05]}

def validate_distillation_alignment(teacher, student, test_data):
    """Does student match teacher behavior?"""
    t_preds = teacher.predict(test_data)
    s_preds = student.predict(test_data)
    agreement = (t_preds == s_preds).mean()
    kl = _kl_divergence(teacher.predict_proba(test_data), student.predict_proba(test_data))
    return {"agreement": float(agreement), "kl_divergence": float(kl),
            "aligned": agreement > 0.9 and kl < 0.1}

def audit_pruning_impact(model, pruned_model, test_suite):
    """What capabilities survive pruning?"""
    impact = {}
    for task, data in test_suite.items():
        orig_perf = evaluate(model, data)
        pruned_perf = evaluate(pruned_model, data)
        impact[task] = {"retained": pruned_perf / orig_perf, "lost": orig_perf - pruned_perf,
                         "critical": pruned_perf / orig_perf < 0.8}
    return {"lost_capabilities": [t for t, i in impact.items() if i["critical"]],
            "retention_rate": np.mean([i["retained"] for i in impact.values()])}

def detect_quantization_errors(model_fp32, model_int8, calibration_data):
    """What data patterns reveal quantization errors?"""
    fp32_preds = model_fp32.predict_proba(calibration_data)
    int8_preds = model_int8.predict_proba(calibration_data)
    errors = np.abs(fp32_preds - int8_preds).max(axis=1)
    return {"mean_error": float(np.mean(errors)), "max_error": float(np.max(errors)),
            "error_rate": float(np.mean(errors > 0.05)), "acceptable": np.mean(errors > 0.05) < 0.01}
```

## Quality Gate

- Feature selection stability: mean Jaccard > 0.8 across splits.
- Feature drift: < 10% of features drifted at any time.
- Distillation: teacher-student agreement > 90%, KL < 0.1.
- Pruning: > 80% retention on all critical tasks.
- Quantization: error rate < 1% at INT8.
