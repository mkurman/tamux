---
name: evaluation-infrastructure-data-task
description: Build evaluation infrastructure datasets — ablation study design, test-time augmentation validation, ensemble member selection, post-publication benchmark leakage monitoring, and leaderboard gaming detection.
recommended_skills: [benchmark-contamination-scan, robustness-engineering-task, evaluation-dataset-design-task]
recommended_guidelines: [data-contamination-task, cross-validation-strategy-task, reproducibility-science-task]
---

## Ablation Study Design

```python
def design_ablation_study(model_components, test_tasks, budget_hours=48):
    """What to ablate, in what order?"""
    n_components = len(model_components)
    n_tasks = len(test_tasks)
    
    # Priority: ablate components with highest compute cost first
    priorities = sorted(model_components, key=lambda c: c["training_cost_hours"], reverse=True)
    
    # Budget: single ablation = retrain without component
    ablations_possible = budget_hours // sum(c["training_cost_hours"] for c in model_components)
    
    return {"planned_ablations": min(n_components, ablations_possible),
            "order": [c["name"] for c in priorities],
            "tasks": [t["name"] for t in test_tasks],
            "interpretation": "Ablate from most to least expensive. Each removal reveals necessity."}

def validate_ablation_completeness(ablation_results, full_model_performance):
    """Did ablations cover enough to explain model performance?"""
    component_contributions = sum(ablation_results.values())
    explained = component_contributions / full_model_performance
    return {"explained_fraction": float(explained),
            "complete": explained > 0.8,
            "gaps": "Unexplained performance suggests missing components or interactions"}
```

## Test-Time Augmentation

```python
def validate_tta_effectiveness(model, test_data, augmentations, n_augmentations=10):
    """Does TTA actually help?"""
    single_pred = model.predict(test_data)
    
    tta_preds = []
    for aug_fn in augmentations:
        aug_preds = []
        for ex in test_data:
            augmented = [aug_fn(ex) for _ in range(n_augmentations)]
            preds = [model.predict(a) for a in augmented]
            aug_preds.append(np.mean(preds, axis=0))
        tta_preds.append(aug_preds)
    
    single_acc = accuracy_score(test_data.labels, single_pred)
    tta_acc = accuracy_score(test_data.labels, np.mean(tta_preds, axis=0).argmax(axis=1))
    
    return {"single_accuracy": single_acc, "tta_accuracy": tta_acc,
            "improvement": tta_acc - single_acc, "tta_helps": tta_acc > single_acc + 0.01,
            "recommendation": "USE_TTA" if tta_acc > single_acc + 0.01 else "SKIP_TTA"}
```

## Ensemble Selection

```python
def select_ensemble_members(candidate_models, validation_data, target_size=5):
    """Which models to ensemble?"""
    predictions = np.array([m.predict(validation_data) for m in candidate_models])
    diversity = 1 - np.mean([np.mean(predictions[i] == predictions[j]) for i in range(len(candidate_models)) for j in range(i+1, len(candidate_models))])
    
    # Greedy: add model that most improves ensemble accuracy
    selected = [0]
    for _ in range(target_size - 1):
        candidates = [i for i in range(len(candidate_models)) if i not in selected]
        best, best_acc = None, -float("inf")
        for c in candidates:
            ensemble_pred = np.mean(predictions[selected + [c]], axis=0).argmax(axis=1)
            acc = accuracy_score(validation_data.labels, ensemble_pred)
            if acc > best_acc: best, best_acc = c, acc
        if best is not None: selected.append(best)
    
    return {"selected_indices": selected, "ensemble_size": len(selected),
            "diversity_score": float(diversity), "predicted_accuracy_gain": "~2-5%"}
```

## Post-Publication Leakage Monitoring

```python
class ContinuousContaminationMonitor:
    def __init__(self, benchmark_registry, training_dataset_id):
        self.registry = benchmark_registry
        self.dataset_id = training_dataset_id
        self.last_scan = None
    
    def check_new_benchmarks(self):
        """New benchmarks released since last scan?"""
        new_bencharks = [b for b in self.registry if b["release_date"] > self.last_scan]
        if new_bencharks:
            return {"action": "RESCAN", "new_benchmarks": len(new_bencharks),
                    "benchmarks": [b["name"] for b in new_bencharks]}
        return {"status": "CLEAN"}
    
    def check_retractions(self):
        """Any datasets pulled from the registry?"""
        retracted = [b for b in self.registry if b["status"] == "retracted"]
        return {"retracted": retracted, "needs_review": len(retracted) > 0}
```

## Leaderboard Gaming Detection

```python
def detect_leaderboard_gaming(submissions, benchmark_metrics, baseline_threshold):
    """When do submissions optimize leaderboard at expense of real capability?"""
    suspicious = []
    for submission in submissions:
        signals = []
        # Signal 1: Leaderboard gap vs random evaluation
        if submission["leaderboard_score"] - submission.get("random_eval_score", 0) > 0.1:
            signals.append("leaderboard_overfit")
        # Signal 2: Many submissions with same score (overfitting to eval set)
        if submission["submission_count"] > 10 and np.std(submission["recent_scores"]) < 0.001:
            signals.append("eval_set_overfit")
        # Signal 3: Submission uses models suspiciously larger than state-of-art
        if submission.get("model_params", 0) > 10 * baseline_threshold:
            signals.append("brute_force_not_generalization")
        
        if len(signals) >= 2:
            suspicious.append({"submission": submission["id"], "signals": signals})
    
    return {"suspicious": suspicious, "gaming_rate": len(suspicious) / max(len(submissions), 1)}
```

## Quality Gate

- Ablations explain > 80% of model performance.
- TTA improves accuracy by > 1pp before being used.
- Ensemble diversity > 0.3.
- Post-publication monitoring active; rescans on new benchmark releases.
- Leaderboard gaming rate < 5% of submissions.
