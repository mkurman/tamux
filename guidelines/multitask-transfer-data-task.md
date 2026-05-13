---
name: multitask-transfer-data-task
description: Curate datasets for multi-task and transfer learning — task interference detection, synergy identification, priority weighting, transfer success validation, negative transfer detection, and fine-tuning stability.
recommended_skills: [embedding-analysis, dataset-splitting, label-quality-audit]
recommended_guidelines: [specialized-training-data-task, evaluation-dataset-design-task]
---

## Multi-Task Learning

### Task Interference Detection

```python
def detect_task_interference(model, task_pairs, eval_data):
    """When does training task A hurt task B?"""
    interference = {}
    for task_a, task_b in task_pairs:
        perf_b_before = evaluate(model, eval_data[task_b])
        model.fit(eval_data[task_a], epochs=1)
        perf_b_after = evaluate(model, eval_data[task_b])
        delta = perf_b_after - perf_b_before
        interference[(task_a, task_b)] = {"delta": delta, 
            "type": "interference" if delta < -0.02 else "synergy" if delta > 0.02 else "neutral"}
    return interference

def find_synergistic_tasks(tasks, eval_data, model_fn):
    """Which tasks benefit from joint training?"""
    synergies = []
    for task_a, task_b in itertools.combinations(tasks, 2):
        joint = train_jointly(model_fn(), [eval_data[task_a], eval_data[task_b]])
        separate = {t: train(model_fn(), eval_data[t]) for t in [task_a, task_b]}
        if evaluate(joint, eval_data[task_a]) > evaluate(separate[task_a], eval_data[task_a]):
            synergies.append((task_a, task_b))
    return synergies
```

### Task Priority Weighting

```python
def optimize_task_weights(tasks, eval_data, model_fn, n_trials=50):
    """Learn optimal loss weights using grid search or Bayesian optimization."""
    best_weights, best_score = None, -float("inf")
    for _ in range(n_trials):
        weights = {t: np.random.uniform(0.5, 2.0) for t in tasks}
        weights = {k: v/sum(weights.values()) for k, v in weights.items()}
        model = train_weighted(model_fn(), eval_data, weights)
        score = np.mean([evaluate(model, eval_data[t]) for t in tasks])
        if score > best_score: best_weights, best_score = weights, score
    return best_weights
```

## Transfer Learning

### Transfer Success Validation

```python
def validate_transfer(source_model, target_data, target_eval, fine_tune_configs):
    """Which fine-tuning strategy works best?"""
    results = {}
    for config_name, config in fine_tune_configs.items():
        model = fine_tune(source_model, target_data, **config)
        perf = evaluate(model, target_eval)
        results[config_name] = perf
    
    best = max(results, key=results.get)
    return {"results": results, "best_strategy": best, "best_performance": results[best]}
```

### Negative Transfer Detection

```python
def detect_negative_transfer(source_model, target_data, target_eval):
    """Did transfer hurt compared to training from scratch?"""
    transferred = fine_tune(source_model, target_data)
    from_scratch = train(Model(), target_data)
    delta = evaluate(transferred, target_eval) - evaluate(from_scratch, target_eval)
    return {"negative_transfer": delta < -0.02, "delta": delta,
            "recommendation": "TRAIN_FROM_SCRATCH" if delta < -0.02 else "TRANSFER_OK"}
```

### Fine-Tuning Stability

```python
def measure_fine_tuning_stability(source_model, target_data, n_runs=10):
    """Does fine-tuning destroy source capabilities?"""
    source_perf_before = evaluate(source_model, source_eval)
    results = []
    for _ in range(n_runs):
        model = fine_tune(copy.deepcopy(source_model), target_data)
        results.append({"target": evaluate(model, target_eval),
                         "source_retained": evaluate(model, source_eval)})
    return {"target_perf": np.mean([r["target"] for r in results]),
            "source_retention": np.mean([r["source_retained"] for r in results]) / source_perf_before,
            "catastrophic_forgetting": np.mean([r["source_retained"] for r in results]) / source_perf_before < 0.7}
```

## Quality Gate

- Task interference matrix computed for all task pairs.
- Synergistic tasks identified for joint training.
- Transfer validated across ≥ 3 fine-tuning strategies.
- Negative transfer detected and flagged.
- Fine-tuning stability: source retention > 70%.
