---
name: data-innovation-integration-task
description: Integrate innovation management with data architectures — stage-gate data for ideation/validation/scaling, design experiments per innovation type, coordinate multi-innovation portfolios, capture failure patterns as data assets, and synchronize data scale with innovation scale.
recommended_skills: [cost-model-task, data-pipeline-monitoring-task, dataset-versioning]
recommended_guidelines: [data-strategy-foundation-models-task, data-feedback-loop-task, experimental-methodology-data-task]
---

## Overview

Innovation consumes data and produces data. An ideation phase needs broad exploratory datasets. A validation phase needs rigorous experimental designs. A scaling phase needs production-grade data pipelines. Failed innovations are not waste — they are the most valuable data you can collect, if you capture the failure pattern. This guideline maps every innovation stage to its data requirements and treats innovation failure as a data asset.

## Innovation Stage → Data Stage Requirements

```python
INNOVATION_DATA_STAGES = {
    "ideation": {
        "data_volume": "small (100s-1000s examples)",
        "data_diversity": "maximum — explore edge cases, anomalies, counterfactuals",
        "label_quality": "directional (human intuition acceptable)",
        "data_freshness": "one-time snapshot",
        "pipeline_investment": "minimal — notebooks and ad-hoc queries",
        "success_metric": "hypothesis_generated",
        "data_output": "exploration_log, initial_patterns, candidate_hypotheses",
    },
    "validation": {
        "data_volume": "medium (1000s-10000s examples)",
        "data_diversity": "controlled — representative sample, holdout design",
        "label_quality": "rigorous (inter-annotator agreement ≥ 0.8)",
        "data_freshness": "fixed snapshot (version-locked for reproducibility)",
        "pipeline_investment": "moderate — reproducible preprocessing, versioned splits",
        "success_metric": "statistically_significant_improvement",
        "data_output": "experiment_results, effect_size, confidence_intervals, holdout_scores",
    },
    "scaling": {
        "data_volume": "large (100k+ examples)",
        "data_diversity": "production-representative — covers long tail",
        "label_quality": "production-grade (automated with human audit sampling)",
        "data_freshness": "continuous streaming with drift detection",
        "pipeline_investment": "full — automated, monitored, SLA-backed",
        "success_metric": "production_impact",
        "data_output": "production_metrics, drift_reports, feedback_loops",
    },
    "retirement": {
        "data_volume": "archival",
        "data_diversity": "full historical record",
        "label_quality": "original labels preserved with version metadata",
        "data_freshness": "frozen",
        "pipeline_investment": "cold storage with retrieval capability",
        "success_metric": "auditability",
        "data_output": "archived_dataset, failure_postmortem, lessons_learned",
    },
}

def stage_gate(innovation, current_data_capability):
    required_stage = INNOVATION_DATA_STAGES.get(innovation.get("stage", "ideation"), {})
    gates = {
        "label_quality": innovation.get("label_quality_score", 0) >= required_stage.get("label_quality_min", 0),
        "pipeline_ready": current_data_capability.get("automation_level", 0) >= 
                          required_stage.get("pipeline_investment_min", 0),
        "data_volume": innovation.get("available_examples", 0) >= required_stage.get("volume_min", 0),
    }
    all_passed = all(gates.values())
    return {"innovation": innovation["id"], "stage": innovation["stage"],
            "gates": gates, "advance_ready": all_passed,
            "blockers": [k for k, v in gates.items() if not v]}
```

## Innovation Type → Experimentation Design

```python
EXPERIMENT_DESIGN = {
    "product": {
        "design": "a_b_test_or_controlled_rollout",
        "metrics": ["adoption_rate", "retention", "revenue_per_user"],
        "data_requirements": ["user_segments", "event_stream", "control_holdout"],
        "minimum_detectable_effect": 0.02,
        "duration_weeks_min": 2,
        "sample_size_formula": "power_analysis(alpha=0.05, power=0.80, mde=0.02)",
    },
    "process": {
        "design": "before_after_with_control_group",
        "metrics": ["throughput", "error_rate", "cost_per_unit"],
        "data_requirements": ["operational_logs", "time_motion_study", "cost_accounting"],
        "minimum_detectable_effect": 0.05,
        "duration_weeks_min": 4,
        "sample_size_formula": "paired_t_test_power(alpha=0.05, power=0.80, mde=0.05)",
    },
    "business_model": {
        "design": "concierge_or_wizard_of_oz_mvp",
        "metrics": ["willingness_to_pay", "conversion_rate", "unit_economics"],
        "data_requirements": ["customer_interviews", "pricing_survey", "competitor_pricing"],
        "minimum_detectable_effect": 0.10,
        "duration_weeks_min": 8,
        "sample_size_formula": "conjoint_analysis_design(attributes=N, levels=K)",
    },
}

def design_experiment(innovation_type, constraints):
    template = EXPERIMENT_DESIGN.get(innovation_type, EXPERIMENT_DESIGN["product"])
    return {
        "design": template["design"],
        "metrics": template["metrics"],
        "data_plan": template["data_requirements"],
        "mde": template["minimum_detectable_effect"],
        "min_duration_weeks": template["duration_weeks_min"],
        "constraint_check": {
            "budget_feasible": constraints.get("budget", 0) >= template.get("estimated_cost", 0),
            "timeline_feasible": constraints.get("timeline_weeks", 0) >= template["duration_weeks_min"],
        },
    }
```

## Innovation Portfolio → Data Portfolio Coordination

```python
def coordinate_innovation_data_portfolio(innovations, available_data_assets, budget):
    """Schedule data resources across multiple concurrent innovations."""
    allocations = []
    remaining_budget = budget
    
    priority_order = sorted(innovations, 
                           key=lambda i: (i.get("strategic_value", 3), i.get("roi_estimate", 0)),
                           reverse=True)
    
    for innovation in priority_order:
        stage = innovation.get("stage", "ideation")
        data_need = _estimate_data_cost(stage, innovation.get("data_requirements", []))
        if data_need["cost"] <= remaining_budget:
            allocations.append({"innovation": innovation["id"], "allocated_data": data_need,
                                "status": "FUNDED"})
            remaining_budget -= data_need["cost"]
        else:
            allocations.append({"innovation": innovation["id"], "allocated_data": None,
                                "status": "QUEUED", "deficit": data_need["cost"] - remaining_budget})
    
    return {"allocations": allocations, "remaining_budget": remaining_budget,
            "funded_count": sum(1 for a in allocations if a["status"] == "FUNDED"),
            "queued_count": sum(1 for a in allocations if a["status"] == "QUEUED")}
```

## Innovation Failure → Data Pattern Capture

```python
FAILURE_CAPTURE_SCHEMA = {
    "experiment_id": "string",
    "hypothesis": "string",
    "data_used": "versioned_dataset_reference",
    "failure_mode": ["null_result", "negative_result", "data_quality_issue",
                     "pipeline_failure", "concept_drift", "label_noise"],
    "root_cause_category": ["data", "model", "infrastructure", "assumption", "external"],
    "learnings": "structured_text",
    "reusable_artifacts": ["cleaned_dataset", "evaluation_harness", "failure_test_case"],
    "prevention_patch": "test_or_gate_that_would_have_caught_this",
}

def capture_failure(experiment, failure_mode, root_cause):
    """Failed innovations generate the most valuable data. Capture it."""
    failure_record = {
        "experiment_id": experiment["id"],
        "hypothesis": experiment["hypothesis"],
        "data_used": experiment.get("dataset_version"),
        "failure_mode": failure_mode,
        "root_cause_category": root_cause,
        "learnings": _extract_learnings(experiment, failure_mode),
        "reusable_artifacts": experiment.get("artifacts", []),
        "prevention_patch": _generate_prevention(experiment, failure_mode, root_cause),
    }
    
    # Store in failure knowledge base
    _append_failure_registry(failure_record)
    
    # Tag the dataset as "part_of_failed_experiment_X" for future learning
    _tag_dataset(experiment.get("dataset_version"), 
                {"failed_experiment": experiment["id"], "failure_mode": failure_mode})
    
    return {"failure_captured": True, "record_id": _hash_record(failure_record)}
```

## Innovation Scaling → Data Scaling Synchronization

```python
SCALING_SYNCHRONIZATION = {
    "data_volume_check": "Can pipeline handle 10x-100x volume within SLA?",
    "data_quality_check": "Does quality hold at scale or degrade with volume?",
    "data_latency_check": "Does latency stay within SLA at target throughput?",
    "data_drift_check": "Is monitoring in place for distribution shift at scale?",
    "data_cost_check": "Is unit cost sustainable at target volume?",
}

def scaling_readiness(innovation, current_pipeline, target_scale):
    checks = {}
    checks["volume"] = current_pipeline["max_throughput"] >= target_scale["volume"] * 1.5
    checks["quality"] = current_pipeline.get("quality_at_scale_benchmark", 0) >= 0.95
    checks["latency"] = current_pipeline["p99_latency"] <= target_scale["latency_sla"]
    checks["drift"] = current_pipeline.get("drift_monitoring", False)
    checks["cost"] = (current_pipeline["cost_per_unit"] * target_scale["volume"]) <= target_scale["budget"]
    
    all_ready = all(checks.values())
    return {"innovation": innovation["id"], "checks": checks,
            "scaling_ready": all_ready,
            "blockers": [k for k, v in checks.items() if not v]}
```

## Quality Gate

- Every innovation has a stage-gated data plan with explicit advance criteria.
- Experiment designs match innovation type with correct statistical power.
- Multi-innovation portfolios have coordinated data allocation — no double-booking.
- Every failed innovation generates a structured failure record in the registry.
- Scaling-ready innovations pass all five synchronization checks before production launch.
