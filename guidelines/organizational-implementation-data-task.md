---
name: organizational-implementation-data-task
description: Implement Data Lattice across an organization — maturity model assessment, adoption roadmap, team training curriculum, pipeline integration checklists, and ROI measurement for data quality initiatives.
recommended_skills: [cost-model-task, team-operations-data-task, annotation-economics-task]
recommended_guidelines: [dataset-certification-task, data-lifecycle-governance-task, data-strategy-foundation-models-task]
---

## Data Maturity Model

```python
DATA_MATURITY_LEVELS = {
    1: {"name": "Ad Hoc", "description": "No standard practices, individual heroics",
        "checklist": ["data_quality_not_measured", "no_versioning", "manual_cleaning", "no_documentation"]},
    2: {"name": "Repeatable", "description": "Some documented practices, inconsistent execution",
        "checklist": ["basic_dedup", "simple_splits", "informal_versioning", "minimal_docs"]},
    3: {"name": "Defined", "description": "Standardized processes, documented, trained",
        "checklist": ["full_curation_pipeline", "contamination_scans", "data_cards", "versioned_datasets"]},
    4: {"name": "Managed", "description": "Measured, monitored, continuously improving",
        "checklist": ["automated_monitoring", "drift_detection", "certification", "feedback_loops"]},
    5: {"name": "Optimizing", "description": "Data as strategic asset, proactive quality",
        "checklist": ["attribution_pipeline", "portfolio_theory", "ecosystem_certification", "auto_curation"]},
}

def assess_maturity(team_practices, desired_level=3):
    current = 1
    for level in range(1, 6):
        checklist = DATA_MATURITY_LEVELS[level]["checklist"]
        implemented = sum(1 for check in checklist if team_practices.get(check, False))
        if implemented == len(checklist): current = level
        else: break
    return {"current_level": current, "desired_level": desired_level,
            "gap": desired_level - current, "recommendation": _maturity_roadmap(current, desired_level)}

def _maturity_roadmap(current, desired):
    steps = []
    for level in range(current + 1, desired + 1):
        missing = [c for c in DATA_MATURITY_LEVELS[level]["checklist"]]
        steps.append({"level": level, "name": DATA_MATURITY_LEVELS[level]["name"], 
                       "practices_to_implement": missing})
    return steps
```

## Adoption Roadmap

| Phase | Month | What | Success Metric |
|-------|-------|------|----------------|
| **Pilot** | 1-2 | One team, one dataset, full pipeline | Gold-certified dataset shipped |
| **Validate** | 3-4 | Compare model trained on curated vs raw data | >3% improvement on key metric |
| **Expand** | 5-8 | Roll out to all ML teams, train champions | 80% of datasets Bronze+ certified |
| **Institutionalize** | 9-12 | Automated monitoring, certification program | Zero contamination incidents, all datasets versioned |
| **Optimize** | 12+ | Attribution, portfolio theory, ecosystem | ROI > 5x on data quality investment |

## Team Training

```python
TRAINING_CURRICULUM = {
    "foundations": {"duration_days": 2,
        "modules": ["universal_principles", "cleaning_basics", "splitting_correctly", "versioning_101"]},
    "practitioner": {"duration_days": 3,
        "modules": ["contamination_detection", "embedding_analysis", "label_quality", "bias_audit"]},
    "advanced": {"duration_days": 2,
        "modules": ["data_attribution", "feedback_loops", "mixture_optimization", "governance"]},
}

def assign_training(team_members, maturity_assessment):
    assignments = {}
    for member in team_members:
        if member["experience_years"] < 2: assignments[member["id"]] = "foundations"
        elif member.get("certified_curator"): assignments[member["id"]] = "advanced"
        else: assignments[member["id"]] = "practitioner"
    return assignments
```

## Pipeline Integration

```python
PIPELINE_INTEGRATION_POINTS = {
    "data_ingestion": [benchmark_contamination_scan, schema_validation],
    "preprocessing": [dataset_cleaning, exact_dedup, semantic_dedup],
    "splitting": [dataset_splitting, leakage_audit],
    "quality_audit": [label_quality_audit, bias_audit, embedding_analysis],
    "release": [data_card_writer, dataset_versioning, certification],
}

def check_pipeline_integration(existing_pipeline, required_checks):
    integrated = []
    missing = []
    for stage, checks in required_checks.items():
        for check in checks:
            if check in existing_pipeline.get(stage, []): integrated.append(f"{stage}:{check}")
            else: missing.append(f"{stage}:{check}")
    return {"integration_score": len(integrated)/max(len(integrated+missing), 1),
            "missing_integrations": missing, "ready": len(missing) == 0}
```

## ROI Measurement

```python
def measure_data_quality_roi(pre_quality_metrics, post_quality_metrics, model_impact, business_value_per_point):
    quality_improvement = post_quality_metrics["overall_score"] - pre_quality_metrics["overall_score"]
    model_improvement = model_impact.get("accuracy_gain", 0)
    direct_value = model_improvement * business_value_per_point
    
    cost_of_quality = sum([
        post_quality_metrics.get("pipeline_cost", 0),
        post_quality_metrics.get("training_cost", 0),
        post_quality_metrics.get("certification_cost", 0),
    ])
    
    prevented_cost = pre_quality_metrics.get("contamination_risk_cost", 0) + \
                      pre_quality_metrics.get("label_error_cost", 0)
    
    roi = (direct_value + prevented_cost) / max(cost_of_quality, 1)
    return {"direct_value": direct_value, "prevented_cost": prevented_cost,
            "cost_of_quality": cost_of_quality, "roi": roi,
            "positive_roi": roi > 1.0,
            "payback_months": cost_of_quality / max(direct_value / 12, 1)}
```

## Quality Gate

- Maturity assessment completed; gap to desired level documented.
- Adoption roadmap approved with executive sponsor.
- 80% of ML team trained at appropriate curriculum level.
- Pipeline integration score > 0.8.
- Data quality ROI > 2x within 12 months.
