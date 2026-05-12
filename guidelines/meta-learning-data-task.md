---
name: meta-learning-data-task
description: Curate data for meta-learning — task distribution design, support/query split construction with leakage prevention, task similarity measurement, and meta-validation set design for measuring transfer.
recommended_skills: [dataset-splitting, embedding-analysis, evaluation-dataset-design-task]
recommended_guidelines: [specialized-training-data-task, cross-validation-strategy-task]
---

## Meta-Learning Data

### Task Distribution Design

```python
def design_meta_tasks(dataset, n_train_tasks=1000, n_test_tasks=200, n_way=5, k_shot=5):
    classes = np.unique(dataset.labels)
    assert len(classes) >= n_way * 2, "Need enough classes for train+test split"
    
    rng = np.random.default_rng(42)
    train_classes = rng.choice(classes, len(classes) // 2, replace=False)
    test_classes = np.setdiff1d(classes, train_classes)
    
    def sample_tasks(class_pool, n_tasks):
        tasks = []
        for _ in range(n_tasks):
            task_classes = rng.choice(class_pool, n_way, replace=False)
            support, query = [], []
            for cls in task_classes:
                cls_examples = dataset.data[dataset.labels == cls]
                idx = rng.permutation(len(cls_examples))
                support.append({"data": cls_examples[idx[:k_shot]], "label": cls})
                query.append({"data": cls_examples[idx[k_shot:k_shot+5]], "label": cls})
            tasks.append({"support": support, "query": query})
        return tasks
    
    return {"train": sample_tasks(train_classes, n_train_tasks),
            "test": sample_tasks(test_classes, n_test_tasks),
            "class_disjoint": len(set(train_classes) & set(test_classes)) == 0}

# CRITICAL: Test tasks must use UNSEEN classes for meta-learning
# Using same classes = memorization, not meta-learning
```

### Task Similarity

```python
def measure_task_similarity(tasks, embedding_model):
    """How similar are tasks? High similarity good for few-shot transfer."""
    task_embeddings = []
    for task in tasks:
        support_data = np.concatenate([s["data"] for s in task["support"]])
        task_embeddings.append(embedding_model.encode(support_data).mean(axis=0))
    
    sims = cosine_similarity(task_embeddings)
    mask = ~np.eye(len(sims), dtype=bool)
    return {"mean_inter_task_similarity": float(sims[mask].mean()),
            "too_diverse": float(sims[mask].mean()) < 0.3,  # tasks unrelated
            "too_redundant": float(sims[mask].mean()) > 0.9}  # tasks identical
```

## Negative Result Data

```python
def archive_negative_result(experiment_id, hypothesis, method, result, controls):
    """Publish a properly controlled negative result."""
    return {"experiment_id": experiment_id, "hypothesis": hypothesis,
            "method": method, "result": result, "controls_passed": all(controls.values()),
            "interpretation": ("VALID_NEGATIVE" if all(controls.values()) else "INCONCLUSIVE"),
            "value": "Publishing saves others from repeating this experiment"}

def validate_null_result(experiment, power_analysis, effect_size_threshold=0.01):
    """Is this a genuine null result or underpowered?"""
    return {"genuine_null": power_analysis["detectable_effect"] < effect_size_threshold,
            "underpowered": power_analysis["detectable_effect"] > effect_size_threshold * 2,
            "recommendation": "PUBLISH" if power_analysis["detectable_effect"] < effect_size_threshold else "INCREASE_POWER"}
```

## Quality Gate

- Meta-learning tasks use DISJOINT classes for train/test.
- Task similarity measured — not too diverse (similarity < 0.3) nor redundant (> 0.9).
- Negative results pass control checks before archiving.
- Power analysis confirms genuine null or identifies underpowered experiments.
