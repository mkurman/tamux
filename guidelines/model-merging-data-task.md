---
name: model-merging-data-task
description: Curate data for model merging — merge candidate selection data, merge compatibility assessment, post-merge validation benchmarks, merge conflict detection data, and ensemble-to-merge distillation corpora. Covers SLERP, TIES, DARES, model soups, and Franken-merging.
recommended_skills: [dataset-splitting, embedding-analysis, benchmark-contamination-scan]
recommended_guidelines: [model-compression-data-task, evaluation-dataset-design-task, data-feedback-loop-task]
---

## Overview

Model merging combines multiple fine-tuned models into one without additional training. SLERP interpolates weights on a sphere. TIES resolves sign conflicts before averaging. DARES randomly drops and rescales. Model soups average checkpoints along a training trajectory. But the data question is: how do you know the merge worked? What data detects merge conflicts? What data validates that capabilities combined rather than canceled? This guideline treats model merging as a data validation problem.

## Merge Candidate Selection

```python
MERGE_CANDIDATE_ASSESSMENT = {
    "architectural_compatibility": {
        "check": "same_base_architecture — must share weight space geometry",
        "data_need": "model_configs and weight_shapes",
        "incompatible": "different_architectures → merge will produce garbage",
    },
    "task_complementarity": {
        "check": "models_specialize_in_different_non_overlapping_tasks",
        "data_need": "evaluation_data_for_each_candidate_across_all_target_tasks",
        "incompatible": "highly_overlapping_capabilities → merge adds no value",
    },
    "fine_tuning_distance": {
        "check": "candidates_shouldn_t_be_too_far_from_base_in_weight_space",
        "data_need": "weight_space_distance_matrix",
        "incompatible": "too_far → merge will produce incoherent model",
    },
    "data_distribution_overlap": {
        "check": "training_data_distributions_are_compatible",
        "data_need": "training_data_embeddings for each candidate",
        "incompatible": "contradictory_data_distributions → merge conflict likely",
    },
}

def select_merge_candidates(candidate_models, target_tasks, base_model):
    """Which models should be merged and why?"""
    
    assessments = []
    for model in candidate_models:
        # Weight space distance
        weight_distance = _weight_space_distance(model, base_model)
        
        # Task complementarity
        model_perf = _evaluate_on_tasks(model, target_tasks)
        complementarity = _compute_complementarity(model_perf, target_tasks)
        
        # Data distribution overlap
        data_overlap = _distribution_overlap(
            model["training_data_embedding"],
            base_model.get("training_data_embedding")
        )
        
        assessments.append({
            "model": model["id"],
            "task": model["specialization"],
            "weight_distance": weight_distance,
            "complementarity_score": complementarity,
            "data_overlap_with_base": data_overlap,
            "merge_candidate": (
                weight_distance < 0.3 and           # not too far
                complementarity > 0.5 and            # adds new capabilities
                data_overlap < 0.8                   # not redundant data
            ),
        })
    
    candidates = [a for a in assessments if a["merge_candidate"]]
    return {
        "candidates": candidates,
        "recommended_merge_method": _recommend_merge_method(candidates),
        "merge_size": len(candidates),
        "expected_synergy": _estimate_synergy(candidates),
    }
```

## Merge Methods and Their Data Requirements

```python
MERGE_METHODS = {
    "SLERP": {
        "description": "Spherical Linear Interpolation — interpolate weights on hypersphere",
        "data_requirement": "none for merge itself; evaluation data for t selection",
        "parameter": "t ∈ [0,1] — interpolation coefficient, tuned on validation data",
        "best_for": "merging_two_models_with_same_architecture",
        "validation_data": "small_held_out_set (500-1000 examples) to sweep t",
    },
    "TIES": {
        "description": "Trim, Elect Sign, and Merge — resolve sign conflicts before averaging",
        "data_requirement": "none for merge itself; evaluation data for sparsity λ selection",
        "parameter": "λ (density) + k (top-k retention) — tuned on validation data",
        "best_for": "merging_models_with_conflicting_task_vectors",
        "validation_data": "multi_task_validation_set to balance trade-offs",
    },
    "DARES": {
        "description": "Drop And REscale — randomly drop delta weights and rescale",
        "data_requirement": "none for merge itself; evaluation data for drop rate p",
        "parameter": "p ∈ [0,1] — drop probability, tuned on validation data",
        "best_for": "merging_many_models_without_memory_overhead",
        "validation_data": "held_out_set_per_task to tune drop rate per task vector",
    },
    "model_soup": {
        "description": "Average weights of checkpoints along same training trajectory",
        "data_requirement": "none for merge itself; evaluation data to select checkpoint window",
        "parameter": "checkpoint_window — which checkpoints to include",
        "best_for": "improving_robustness_without_additional_training",
        "validation_data": "holdout_set — soup performance should exceed any individual checkpoint",
    },
    "frankestein_merge": {
        "description": "Merge different layers from different models (dangerous)",
        "data_requirement": "per_layer_compatibility_validation",
        "parameter": "layer_mapping — which layers from which model",
        "best_for": "experimental — when you know what you're doing",
        "validation_data": "extensive — every layer combination must be validated independently",
    },
}
```

## Post-Merge Validation

```python
POST_MERGE_VALIDATION = {
    "capability_preservation": {
        "check": "does_merged_model_retain_all_individual_capabilities",
        "benchmark": "per_task_benchmarks_for_each_merged_model",
        "threshold": "≥ 95% of individual model performance on each task",
        "failure_mode": "capability_cancellation — tasks interfere destructively",
    },
    "capability_synergy": {
        "check": "does_merged_model_show_new_combined_capabilities",
        "benchmark": "cross_task_benchmarks — tasks requiring combined knowledge",
        "threshold": "merged > max(individual) on cross-task benchmarks",
        "failure_mode": "no_synergy — merge didn't create new capability, just averaged",
    },
    "merge_conflict": {
        "check": "are_there_tasks_where_merged_model_performs_worse_than_any_individual",
        "benchmark": "per_task_benchmarks_for_all_merged_models",
        "threshold": "merged ≥ min(individual) on every task",
        "failure_mode": "negative_transfer — merge actively hurts some capabilities",
    },
    "weight_interference": {
        "check": "do_certain_layers_show_destructive_interference",
        "benchmark": "layer_wise_ablation — probe each merged layer independently",
        "threshold": "no_layer_causes >5% degradation when ablated individually",
        "failure_mode": "layer_conflict — specific layers from different models destroy each other",
    },
}

def validate_merge(merged_model, individual_models, task_suites):
    results = {}
    
    # Per-task preservation
    for task in task_suites:
        merged_score = evaluate(merged_model, task)
        individual_scores = [evaluate(m, task) for m in individual_models]
        best_individual = max(individual_scores)
        min_individual = min(individual_scores)
        
        results[f"preservation_{task}"] = {
            "merged": merged_score,
            "best_individual": best_individual,
            "preservation_ratio": merged_score / max(best_individual, 0.01),
            "conflict": merged_score < min_individual,
            "pass": merged_score / max(best_individual, 0.01) >= 0.95,
        }
    
    # Cross-task synergy
    cross_tasks = _cross_task_benchmarks(task_suites)
    for cross_task in cross_tasks:
        merged_cross = evaluate(merged_model, cross_task)
        individual_cross = max(evaluate(m, cross_task) for m in individual_models)
        results[f"synergy_{cross_task}"] = {
            "merged": merged_cross,
            "best_individual": individual_cross,
            "synergy_gain": merged_cross - individual_cross,
            "synergy_detected": merged_cross > individual_cross,
        }
    
    merge_success = all(
        r.get("pass", False) for r in results.values() 
        if "preservation_" in str(r)
    ) and not any(
        r.get("conflict", False) for r in results.values()
        if "preservation_" in str(r)
    )
    
    return {
        "results": results,
        "merge_successful": merge_success,
        "conflicts_detected": [k for k, v in results.items() 
                               if v.get("conflict") and "preservation_" in k],
        "synergies_detected": [k for k, v in results.items() 
                               if v.get("synergy_detected") and "synergy_" in k],
    }
```

## Merge Conflict Detection

```python
def detect_merge_conflicts(individual_models, merge_candidates, conflict_test_set):
    """Identify specific examples where merged models produce conflicting predictions."""
    
    conflicts = []
    for example in conflict_test_set:
        predictions = [predict(m, example) for m in individual_models]
        
        # Conflict: models disagree on correct answer
        if len(set(p["answer"] for p in predictions)) > 1:
            conflicts.append({
                "example": example["id"],
                "input": example["input"][:200],
                "model_predictions": {
                    m["id"]: p["answer"] 
                    for m, p in zip(individual_models, predictions)
                },
                "conflict_type": "output_contradiction",
                "severity": "HIGH" if all(p["confidence"] > 0.8 for p in predictions) 
                            else "MEDIUM",
            })
        
        # Hidden conflict: models agree on output but disagree on reasoning
        if all(p["answer"] == predictions[0]["answer"] for p in predictions):
            reasonings = [p.get("reasoning", "") for p in predictions]
            if _reasoning_divergence(reasonings) > 0.5:
                conflicts.append({
                    "example": example["id"],
                    "conflict_type": "reasoning_divergence",
                    "severity": "LOW — output consistent, reasoning diverges",
                })
    
    return {
        "total_conflicts": len(conflicts),
        "conflict_examples": conflicts,
        "conflict_rate": len(conflicts) / max(len(conflict_test_set), 1),
        "merge_risk": "HIGH" if len(conflicts) / max(len(conflict_test_set), 1) > 0.1
                      else "MODERATE" if conflicts else "LOW",
    }
```

## Quality Gate

- Merge candidates assessed for architecture, task complementarity, weight distance, and data overlap.
- Merge method selected based on number of candidates and their conflict profile.
- Post-merge validation confirms ≥95% capability preservation on all individual tasks.
- No merge conflicts where merged model underperforms ALL individual models.
- Synergy detected on cross-task benchmarks — merge creates new capability, not just averages.
- Layer-wise ablation confirms no single layer causes destructive interference.
