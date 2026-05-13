---
name: media-hr-public-sector-data-task
description: Curate data for media/entertainment, HR/workforce, and public sector — content recommendation bias, employee turnover prediction, hiring outcome validation, policy impact measurement, and public service forecasting.
recommended_skills: [bias-audit, time-series-data-task, embedding-analysis, evaluation-dataset-design-task]
recommended_guidelines: [industry-verticals-data-task, organizational-implementation-data-task]
---

## Media / Entertainment

```python
def detect_recommendation_bias(recommendations, user_demographics, content_metadata):
    """Systematic patterns in recommendations — who gets shown what?"""
    bias = {}
    for attr, groups in user_demographics.items():
        for group_name, group_mask in groups.items():
            group_recs = [r for r, m in zip(recommendations, group_mask) if m]
            if not group_recs: continue
            content_types = Counter(c["category"] for rec_list in group_recs for c in rec_list)
            bias[f"{attr}_{group_name}"] = dict(content_types.most_common(5))
    return bias

def validate_viewership_measurement(reported_views, actual_views, confidence=0.95):
    """Reported vs actual — the perpetual gap in media measurement."""
    discrepancy = np.mean([abs(r - a) / max(a, 1) for r, a in zip(reported_views, actual_views) if a > 0])
    return {"mean_discrepancy_pct": float(discrepancy * 100),
            "underreporting": discrepancy > 0.1, "acceptable": discrepancy < 0.05}
```

## Human Resources / Workforce

```python
def validate_hiring_predictions(predictions, actual_performance, protected_attrs):
    """Do hiring model predictions match actual job performance?"""
    overall_corr = np.corrcoef(predictions, actual_performance)[0, 1]
    group_corrs = {}
    for attr, groups in protected_attrs.items():
        for group, mask in groups.items():
            if mask.sum() < 20: continue
            group_corrs[f"{attr}_{group}"] = float(np.corrcoef(predictions[mask], actual_performance[mask])[0,1])
    return {"overall_correlation": float(overall_corr), "group_correlations": group_corrs,
            "valid_predictor": overall_corr > 0.3,
            "fair": max(group_corrs.values()) - min(group_corrs.values()) < 0.15}

def audit_turnover_prediction(predictions, actual_turnover, time_horizon_months=6):
    from sklearn.metrics import roc_auc_score
    auc = roc_auc_score(actual_turnover, predictions)
    precision = np.mean([actual_turnover[i] for i, p in enumerate(predictions) if p > 0.5])
    return {"auc": float(auc), "precision_at_50pct": float(precision),
            "useful": auc > 0.65, "actionable": precision > 0.3,
            "recommendation": "DEPLOY" if auc > 0.65 else "IMPROVE_MODEL"}

def audit_compensation_fairness(salaries, demographics, role_levels):
    """Pay equity — do comparable roles have comparable pay?"""
    disparities = {}
    for level in set(role_levels):
        level_mask = np.array(role_levels) == level
        for attr, groups in demographics.items():
            group_means = {g: np.mean(salaries[level_mask & groups[g]]) for g in groups if (level_mask & groups[g]).sum() >= 5}
            if len(group_means) >= 2:
                disparities[f"{level}_{attr}"] = {"max_min_ratio": max(group_means.values()) / max(min(group_means.values()), 1)}
    return {"disparities": disparities,
            "action_required": [k for k, v in disparities.items() if v["max_min_ratio"] > 1.10]}
```

## Public Sector / Government

```python
def validate_policy_impact(policy_metrics_before, policy_metrics_after, control_group=None):
    """Did the policy actually cause the change?"""
    before_mean = np.mean(policy_metrics_before)
    after_mean = np.mean(policy_metrics_after)
    change_pct = (after_mean - before_mean) / max(abs(before_mean), 1e-6)
    
    if control_group:
        control_change = np.mean(control_group["after"]) - np.mean(control_group["before"])
        treatment_change = after_mean - before_mean
        diff_in_diff = treatment_change - control_change
        return {"raw_change_pct": float(change_pct*100), "diff_in_diff": float(diff_in_diff),
                "attributable_to_policy": abs(diff_in_diff) > abs(control_change*0.2),
                "causal_effect": diff_in_diff}
    return {"raw_change_pct": float(change_pct*100), "causal_claim_possible": False}

def validate_census_accuracy(survey_estimates, administrative_counts, demographics):
    """Survey estimates vs actual counts — how accurate are census/survey methods?"""
    errors = {}
    for demo, groups in demographics.items():
        for group in groups:
            est = survey_estimates.get(f"{demo}_{group}", 0)
            actual = administrative_counts.get(f"{demo}_{group}", 0)
            if actual > 0:
                errors[f"{demo}_{group}"] = float((est - actual) / actual)
    return {"mean_error_pct": float(np.mean(list(errors.values()))*100),
            "undercounted_groups": [k for k, v in errors.items() if v < -0.05],
            "overcounted_groups": [k for k, v in errors.items() if v > 0.05]}
```

## Quality Gate

- Media: viewership discrepancy < 10%; recommendation bias documented.
- HR: turnover prediction AUC > 0.65; compensation disparity < 10%.
- Public sector: policy impact quantified with diff-in-diff; census error < 5%.
