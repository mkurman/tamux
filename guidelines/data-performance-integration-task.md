---
name: data-performance-integration-task
description: Integrate performance management with data architecture — map performance dimensions to data dimensions, design measurements from targets, explain variance with data, construct benchmarks, and track improvement history. Every KPI is a data contract.
recommended_skills: [data-pipeline-monitoring-task, dataset-versioning, cost-model-task]
recommended_guidelines: [data-strategy-foundation-models-task, evaluation-dataset-design-task, cross-validation-strategy-task]
---

## Overview

Performance management without data is opinion. Performance data without management is overhead. This guideline closes the loop: every performance dimension maps to a data dimension, every target maps to a measurement design, every variance has a data explanation, every benchmark is data-constructed, and every improvement is data-tracked. The organization that cannot explain performance variance with data is guessing.

## Performance Dimension → Data Dimension Mapping

```python
PERFORMANCE_DATA_MAP = {
    "financial": {
        "dimensions": ["revenue", "margin", "cash_flow", "roi", "customer_acquisition_cost"],
        "data_sources": ["erp_system", "accounting_ledger", "billing_system", "crm"],
        "granularity": "transactional to quarterly",
        "freshness": "daily_close to quarterly_close",
        "validation": "reconciliation_with_financial_systems",
        "stakeholders": ["CFO", "FP&A", "investors", "board"],
    },
    "operational": {
        "dimensions": ["throughput", "cycle_time", "error_rate", "utilization", "waste"],
        "data_sources": ["process_mining", "iot_sensors", "ticketing_system", "inventory"],
        "granularity": "per-event to daily",
        "freshness": "real-time to daily",
        "validation": "time_motion_study_calibration",
        "stakeholders": ["COO", "plant_managers", "process_engineers"],
    },
    "customer": {
        "dimensions": ["nps", "csat", "churn_rate", "ltv", "feature_adoption"],
        "data_sources": ["survey_platform", "product_analytics", "support_tickets", "crm"],
        "granularity": "per-interaction to cohort",
        "freshness": "real-time to weekly",
        "validation": "survey_response_rate_audit, sample_bias_check",
        "stakeholders": ["CCO", "product", "marketing", "customer_success"],
    },
    "employee": {
        "dimensions": ["engagement", "retention", "productivity", "skill_growth", "internal_mobility"],
        "data_sources": ["hrms", "engagement_survey", "performance_reviews", "lms"],
        "granularity": "per-employee to org_unit",
        "freshness": "quarterly to annual",
        "validation": "anonymization_check, response_rate_audit",
        "stakeholders": ["CHRO", "people_managers", "executive_team"],
    },
    "ml_model": {
        "dimensions": ["accuracy", "latency", "fairness", "robustness", "data_efficiency"],
        "data_sources": ["evaluation_harness", "production_logs", "bias_audit_tool", "drift_detector"],
        "granularity": "per-prediction to per-model-version",
        "freshness": "real-time to per-release",
        "validation": "holdout_sets, cross_validation, a_b_tests",
        "stakeholders": ["ML_team", "product", "compliance", "risk"],
    },
}

def map_performance_to_data(org_performance_model):
    mapping = {}
    for dimension_id, dimension in org_performance_model.items():
        template = PERFORMANCE_DATA_MAP.get(dimension.get("category"))
        if template is None:
            mapping[dimension_id] = {"error": "unknown_performance_category",
                                      "category": dimension.get("category")}
            continue
        data_gaps = []
        for metric in dimension.get("metrics", []):
            if metric not in template["dimensions"]:
                data_gaps.append(metric)
        mapping[dimension_id] = {
            "template": template,
            "covered_metrics": [m for m in dimension.get("metrics", []) if m in template["dimensions"]],
            "data_gaps": data_gaps,
            "data_sources": template["data_sources"],
            "freshness": template["freshness"],
            "coverage_score": 1 - len(data_gaps) / max(len(dimension.get("metrics", [1])), 1),
        }
    return mapping
```

## Performance Target → Measurement Design

```python
def design_measurement(performance_target):
    """Given a target, design the measurement system to track it."""
    target_type = performance_target.get("type", "threshold")
    
    designs = {
        "threshold": {
            "method": "binary_pass_fail_with_confidence_interval",
            "sample_size": "power_analysis(alpha=0.05, power=0.90, effect=target_value)",
            "measurement_frequency": "matches_decision_frequency",
            "alert_threshold": performance_target.get("value"),
            "alert_logic": "actual < target → ALERT",
        },
        "improvement": {
            "method": "paired_before_after_comparison",
            "sample_size": "power_analysis(alpha=0.05, power=0.80, effect=improvement_target)",
            "measurement_frequency": "before_and_after_intervention",
            "alert_threshold": performance_target.get("improvement_pct"),
            "alert_logic": "improvement < target → ALERT",
        },
        "range": {
            "method": "control_chart_with_upper_lower_bounds",
            "sample_size": "30_consecutive_periods_for_baseline",
            "measurement_frequency": "continuous",
            "alert_threshold": f"upper={performance_target.get('upper')}, lower={performance_target.get('lower')}",
            "alert_logic": "outside_range → ALERT",
        },
    }
    
    design = designs.get(target_type, designs["threshold"])
    return {
        "target": performance_target["description"],
        "measurement_design": design,
        "data_collection_plan": {
            "source": performance_target.get("data_source"),
            "frequency": design["measurement_frequency"],
            "sample_size": design["sample_size"],
            "pipeline": "automated" if performance_target.get("frequency") != "annual" else "semi_automated",
        },
    }
```

## Performance Variance → Data Explanation

```python
VARIANCE_DECOMPOSITION = {
    "volume_driven": "Performance changed because input volume changed",
    "quality_driven": "Performance changed because data quality changed",
    "distribution_driven": "Performance changed because data distribution shifted",
    "external_driven": "Performance changed because external conditions changed",
    "measurement_driven": "Performance changed because measurement method changed",
    "contamination_driven": "Performance changed because benchmark leakage is present",
}

def explain_variance(actual_performance, expected_performance, data_signals):
    """When performance deviates, explain why with data, not opinion."""
    variance = actual_performance - expected_performance
    explanations = []
    remaining_variance = variance
    
    # Check each possible driver
    for driver, description in VARIANCE_DECOMPOSITION.items():
        driver_signal = data_signals.get(driver, {})
        if driver_signal.get("detected", False):
            contribution = driver_signal.get("estimated_impact", 0)
            explanations.append({
                "driver": driver,
                "description": description,
                "estimated_contribution": contribution,
                "evidence": driver_signal.get("evidence", []),
                "confidence": driver_signal.get("confidence", 0.5),
            })
            remaining_variance -= contribution
    
    unexplained = remaining_variance if abs(remaining_variance) > 0.01 else 0
    
    return {
        "total_variance": variance,
        "variance_direction": "positive" if variance > 0 else "negative",
        "explained_components": explanations,
        "explained_pct": (variance - unexplained) / max(abs(variance), 0.001) * 100,
        "unexplained": unexplained,
        "verdict": "FULLY_EXPLAINED" if abs(unexplained) < 0.01 
                   else "PARTIALLY_EXPLAINED" if abs(unexplained) < abs(variance) * 0.3
                   else "MOSTLY_UNEXPLAINED",
    }
```

## Performance Benchmark → Data Benchmark Construction

```python
def construct_benchmark(internal_data, external_sources, benchmark_dimension):
    """Build a data-grounded benchmark, not a gut-feel comparison."""
    
    benchmark = {
        "dimension": benchmark_dimension,
        "internal_baseline": {
            "mean": internal_data["mean"],
            "std": internal_data["std"],
            "p25": internal_data["percentiles"]["25"],
            "p50": internal_data["percentiles"]["50"],
            "p75": internal_data["percentiles"]["75"],
            "sample_size": internal_data["n"],
            "time_window": internal_data["time_range"],
        },
        "peer_comparison": [],
        "industry_reference": None,
        "methodology": {
            "normalization": "per_unit_or_per_customer_or_per_employee",
            "seasonality_adjustment": True,
            "outlier_treatment": "winsorize_at_1st_99th_percentile",
        },
    }
    
    for source in external_sources:
        if source.get("dimension") == benchmark_dimension:
            benchmark["peer_comparison"].append({
                "source": source["name"],
                "value": source["value"],
                "percentile_rank": source.get("percentile"),
                "sample_size": source.get("n"),
                "comparability_score": _assess_comparability(internal_data, source),
                "caveats": source.get("caveats", []),
            })
    
    # Determine position
    if benchmark["peer_comparison"]:
        peer_values = [p["value"] for p in benchmark["peer_comparison"]]
        internal_value = internal_data["mean"]
        rank = sum(1 for v in peer_values if v < internal_value)
        benchmark["percentile_rank"] = rank / max(len(peer_values), 1) * 100
        benchmark["position"] = "TOP_QUARTILE" if benchmark["percentile_rank"] >= 75 \
                                else "ABOVE_MEDIAN" if benchmark["percentile_rank"] >= 50 \
                                else "BELOW_MEDIAN" if benchmark["percentile_rank"] >= 25 \
                                else "BOTTOM_QUARTILE"
    
    return benchmark
```

## Performance Improvement → Data Improvement Tracking

```python
IMPROVEMENT_TRACKING = {
    "intervention_id": "string",
    "target_metric": "string",
    "baseline_value": "float",
    "target_value": "float",
    "measurement_schedule": ["immediate", "1_week", "1_month", "3_months", "6_months", "12_months"],
    "expected_trajectory": "improvement_curve_params",
    "sustainability_check": "is_improvement_sustained_at_12_months",
}

def track_improvement(intervention, measurement_history):
    """Track whether improvement is real, sustained, and attributable."""
    baseline = intervention["baseline_value"]
    target = intervention["target_value"]
    
    analysis = {
        "intervention": intervention["id"],
        "immediate_effect": _check_measurement(measurement_history, "immediate", baseline, target),
        "sustained_effect": _check_measurement(measurement_history, "3_months", baseline, target),
        "attribution_confidence": _attribution_score(intervention, measurement_history),
        "learning_rate": _improvement_velocity(measurement_history),
    }
    
    # Is the improvement real?
    analysis["real"] = analysis["immediate_effect"]["significant"] and \
                        analysis["attribution_confidence"] > 0.7
    
    # Is it sustained?
    analysis["sustained"] = analysis["sustained_effect"]["significant"] and \
                            analysis["sustained_effect"]["direction"] == "improved"
    
    # Is it complete?
    current = measurement_history.get("latest", baseline)
    analysis["target_met"] = current >= target if target > baseline else current <= target
    analysis["completion_pct"] = min(100, abs(current - baseline) / max(abs(target - baseline), 0.001) * 100)
    
    return analysis
```

## Quality Gate

- Every performance dimension has mapped data sources with defined freshness and stakeholders.
- Every target has an explicit measurement design with sample size justification.
- No performance variance goes unexplained — "we don't know why" triggers a data investigation.
- All benchmarks cite specific data sources, sample sizes, and comparability caveats.
- Every improvement intervention has a 12-month tracking plan with sustainability checks.
