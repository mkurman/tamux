---
name: intersectional-evaluation-task
description: Design intersectional evaluation datasets — multi-axis fairness (race×gender×age×disability×income), failure detection at intersections, mitigation effectiveness, historical bias quantification, and sample size requirements per intersection.
recommended_skills: [bias-audit, label-quality-audit, dataset-splitting]
recommended_guidelines: [evaluation-dataset-design-task, data-contamination-task]
---

## Overview

Single-axis fairness is insufficient. A model that is "fair" on gender AND "fair" on race can still be deeply unfair to Black women. Intersectional evaluation measures performance at the COMBINATIONS that matter.

## Phase 1: Intersectional Axis Definition

```python
# Core protected attributes (jurisdiction-dependent)
INTERSECTIONAL_AXES = {
    "gender": ["male", "female", "non-binary", "self-described", "unknown"],
    "race_ethnicity": ["white", "black", "hispanic", "asian", "indigenous", "multiracial", "other"],
    "age_group": ["0-17", "18-34", "35-54", "55-74", "75+"],
    "disability_status": ["no_disability", "disability"],
    "income_level": ["low", "middle", "high"],
}

def enumerate_intersections(axes, min_group_size=30):
    from itertools import product
    axis_values = [axes[ax] for ax in axes]
    intersections = list(product(*axis_values))
    return [{"name": "×".join(inter), "values": dict(zip(axes.keys(), inter)),
             "viable": True} for inter in intersections]
```

**Warning**: Full Cartesian product explodes fast. 5 genders × 7 races × 5 ages × 2 disabilities × 3 incomes = 1,050 intersections. Most will be too small. Prioritize by population frequency AND historical disadvantage.

## Phase 2: Intersectional Performance Audit

```python
def audit_intersections(model, test_data, axes):
    results = {}
    
    for intersection_name, intersection_values in enumerate_intersections(axes).items():
        mask = np.ones(len(test_data), dtype=bool)
        for axis, value in intersection_values["values"].items():
            mask &= (test_data[axis] == value)
        
        if mask.sum() < 30:
            results[intersection_name] = {"n": int(mask.sum()), "viable": False, 
                                           "warning": "insufficient_samples"}
            continue
        
        perf = evaluate(model, test_data[mask])
        results[intersection_name] = {"n": int(mask.sum()), "performance": perf, "viable": True}
    
    # Find worst-performing intersections
    worst = sorted([(k, v) for k, v in results.items() if v.get("viable")],
                    key=lambda x: x[1]["performance"])[:10]
    
    # Compound effect: is intersection worse than sum of individual axis penalties?
    overall_perf = evaluate(model, test_data)
    for name, result in results.items():
        if not result.get("viable"): continue
        individual_penalties = sum(
            _axis_penalty(model, test_data, axis, value)
            for axis, value in intersection_values["values"].items()
        )
        intersection_penalty = overall_perf - result["performance"]
        result["compound_effect"] = intersection_penalty - individual_penalties
        result["compounding"] = result["compound_effect"] > 0.02  # more than 2pp extra
    
    return results

def _axis_penalty(model, test_data, axis, value):
    axis_mask = test_data[axis] == value
    if axis_mask.sum() < 30: return 0
    overall_perf = evaluate(model, test_data)
    axis_perf = evaluate(model, test_data[axis_mask])
    return overall_perf - axis_perf
```

## Phase 3: Mitigation Effectiveness Audit

```python
def audit_mitigation_intersectional(model_before, model_after, test_data, axes):
    before_results = audit_intersections(model_before, test_data, axes)
    after_results = audit_intersections(model_after, test_data, axes)
    
    mitigation_impact = {}
    for intersection_name in before_results:
        if not before_results[intersection_name].get("viable") or \
           not after_results[intersection_name].get("viable"):
            continue
        
        delta = after_results[intersection_name]["performance"] - \
                before_results[intersection_name]["performance"]
        mitigation_impact[intersection_name] = {
            "before": before_results[intersection_name]["performance"],
            "after": after_results[intersection_name]["performance"],
            "delta": delta,
            "improved": delta > 0.01,
            "worsened": delta < -0.01,
        }
    
    return {"by_intersection": mitigation_impact,
            "n_improved": sum(1 for v in mitigation_impact.values() if v["improved"]),
            "n_worsened": sum(1 for v in mitigation_impact.values() if v["worsened"]),
            "net_positive": sum(1 for v in mitigation_impact.values() if v["improved"]) > 
                            sum(1 for v in mitigation_impact.values() if v["worsened"])}
```

## Phase 4: Historical Bias Quantification

```python
def measure_historical_bias(dataset, axes, target_col, reference_year=2020):
    """How did label rates differ across intersections historically?"""
    pre_ref = dataset[dataset["year"] < reference_year]
    post_ref = dataset[dataset["year"] >= reference_year]
    
    pre_bias = {}
    for name, values in enumerate_intersections(axes).items():
        mask = np.ones(len(pre_ref), dtype=bool)
        for axis, value in values["values"].items():
            mask &= (pre_ref[axis] == value)
        if mask.sum() < 30: continue
        pre_bias[name] = pre_ref[mask][target_col].mean()
    
    overall_rate = pre_ref[target_col].mean()
    bias_scores = {name: rate - overall_rate for name, rate in pre_bias.items()}
    
    return {"overall_rate": overall_rate, "per_intersection_bias": bias_scores,
            "most_disadvantaged": min(bias_scores, key=bias_scores.get),
            "most_advantaged": max(bias_scores, key=bias_scores.get)}
```

## Phase 5: Sample Size Requirements

```python
def intersectional_sample_sizes(population_distribution, target_confidence=0.05, power=0.8):
    """Minimum viable sample per intersection."""
    from statsmodels.stats.power import TTestIndPower
    
    analysis = TTestIndPower()
    
    requirements = {}
    for intersection, pop_share in population_distribution.items():
        effect_size = 0.02 / pop_share  # smaller share → need larger effect
        n = analysis.solve_power(effect_size=effect_size, alpha=target_confidence, power=power)
        requirements[intersection] = {"population_share": pop_share, 
                                       "min_n": int(np.ceil(n)),
                                       "achievable": pop_share * 100000 >= n}
    
    n_total = sum(r["min_n"] for r in requirements.values() if r["achievable"])
    return {"per_intersection": requirements, "estimated_total_required": n_total}
```

## Quality Gate

- All viable intersections (n ≥ 30) evaluated.
- Compound effect measured — not just sum of single-axis penalties.
- Mitigation audit shows net positive across intersections.
- Historical bias quantified and documented.
- Sample size requirements computed; infeasible intersections documented as limitations.
- Worst-performing intersection identified and targeted for improvement.
