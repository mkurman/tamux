---
name: team-operations-data-task
description: Apply data science to data science teams — skill-to-task matching, handoff quality gates, review efficiency metrics, documentation completeness scoring, and training data for data scientists.
recommended_skills: [annotation-management-task, annotation-economics-task, data-pipeline-monitoring-task]
recommended_guidelines: [cost-model-task, dataset-governance-task]
---

## Overview

Data science teams produce data. But who audits the team's process? This guideline applies the same rigor we apply to datasets to the team that produces them — matching skills to tasks, measuring handoff quality, and tracking review efficiency.

## Phase 1: Skill-to-Task Matching

```python
TEAM_SKILLS = {
    "data_cleaning": ["pandas", "SQL", "missing value handling", "outlier detection"],
    "annotation_design": ["task decomposition", "guideline writing", "IAA measurement"],
    "pipeline_engineering": ["Airflow", "dbt", "schema design", "monitoring"],
    "embedding_analysis": ["sentence-transformers", "UMAP", "clustering", "similarity metrics"],
    "governance": ["GDPR", "HIPAA", "licensing", "consent management"],
}

def assign_tasks(team_members, tasks):
    assignments = {}
    for task in tasks:
        scores = {}
        for member in team_members:
            skill_overlap = len(set(task["required_skills"]) & set(member["skills"]))
            experience = member.get("task_history", {}).get(task["type"], 0)
            availability = member.get("availability", 1.0)
            scores[member["id"]] = 0.5 * skill_overlap / len(task["required_skills"]) + \
                                   0.3 * min(experience / 10, 1.0) + \
                                   0.2 * availability
        best = max(scores, key=scores.get)
        assignments[task["id"]] = {"assigned_to": best, "score": scores[best]}
    return assignments
```

## Phase 2: Handoff Quality Gates

```python
HANDOFF_GATES = {
    "raw_to_clean": [
        "Schema validated",
        "Null handling documented",
        "Cleaning audit log attached",
        "Row count reconciled (before/after)",
    ],
    "clean_to_split": [
        "Deduplication applied",
        "Split integrity verified",
        "Stratification validated",
        "Leakage checks passed",
    ],
    "split_to_train": [
        "Contamination scan clean",
        "Label audit passed",
        "Bias audit complete",
        "Data card written",
    ],
}

def validate_handoff(dataset, stage_from, stage_to):
    gates = HANDOFF_GATES.get(f"{stage_from}_to_{stage_to}", [])
    results = {gate: _check_gate(dataset, gate) for gate in gates}
    passed = all(results.values())
    return {"passed": passed, "gates": results, 
            "can_proceed": passed, 
            "failures": [k for k, v in results.items() if not v]}
```

## Phase 3: Review Efficiency

```python
def track_review_efficiency(reviews):
    metrics = {}
    for reviewer_id, reviewer_reviews in reviews.groupby("reviewer_id"):
        metrics[reviewer_id] = {
            "reviews_completed": len(reviewer_reviews),
            "avg_review_time_minutes": reviewer_reviews["duration_minutes"].mean(),
            "issues_found_rate": (reviewer_reviews["issues_found"] > 0).mean(),
            "false_positive_rate": (reviewer_reviews["issues_found"] & 
                                     ~reviewer_reviews["issue_confirmed"]).mean(),
            "review_quality_score": reviewer_reviews["issue_confirmed"].mean() / 
                                     max(reviewer_reviews["issues_found"].mean(), 0.01),
        }
    return metrics

# Benchmark against team averages
# Flag reviewers: > 2σ faster than mean (rushing), > 2σ slower (bottleneck)
# Flag reviewers: false positive rate > 0.3 (too strict)
```

## Phase 4: Documentation Completeness

```python
DOCUMENTATION_REQUIREMENTS = {
    "dataset": ["data_card", "schema", "cleaning_audit", "version_manifest"],
    "pipeline": ["architecture_diagram", "runbook", "failure_modes", "rollback_procedure"],
    "model": ["model_card", "training_config", "evaluation_results", "limitations"],
}

def score_documentation(artifact_type, documents):
    required = DOCUMENTATION_REQUIREMENTS.get(artifact_type, [])
    present = [doc for doc in required if doc in documents]
    missing = [doc for doc in required if doc not in documents]
    score = len(present) / len(required) if required else 1.0
    return {"score": score, "present": present, "missing": missing,
            "complete": len(missing) == 0}
```

## Phase 5: Training Data for Data Scientists

The meta-problem: what data teaches good data practices?

```python
TRAINING_MODULES = {
    "curation_101": {
        "skills": ["missing value handling", "outlier detection", "basic dedup"],
        "dataset": "curated_examples_with_known_issues",
        "format": "guided exercises → independent practice → peer review",
    },
    "contamination_mastery": {
        "skills": ["benchmark identification", "n-gram scanning", "exclusion list maintenance"],
        "dataset": "contaminated_c4_samples_with_known_benchmark_injection",
        "format": "find the contamination → fix it → verify",
    },
    "governance_practice": {
        "skills": ["license checking", "consent validation", "DPA review"],
        "dataset": "real_licensing_scenarios_with_compliance_questions",
        "format": "case study → team discussion → written assessment",
    },
}
```

## Quality Gate

- Every team member mapped to tasks matching ≥ 60% of their skills.
- Handoff gates automated — pipeline blocks on failure.
- Review efficiency tracked per reviewer; outliers investigated.
- Documentation completeness ≥ 80% for all artifacts.
- New team members complete curation_101 within first month.
