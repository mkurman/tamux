---
name: active-learning-data-task
description: Design active learning data strategies — uncertainty sampling, diversity sampling, query-by-committee, expected model change, and cost-sensitive acquisition. Maximize model improvement per annotation dollar.
recommended_skills:
  - embedding-analysis
  - label-quality-audit
  - llm-assisted-curation
recommended_guidelines:
  - annotation-management-task
  - cost-model-task
  - training-data-design-principles
---

## Overview

Annotation is expensive. Active learning makes every annotation count by choosing the MOST informative examples to label next. This guideline covers acquisition strategies, when to use each, and how to measure their effectiveness.

## Core Loop

```
Unlabeled Pool → Acquisition Function → Human Annotator → Labeled Training Set → Model
       ↑                                                                              |
       └──────────────────────────── retrain ←────────────────────────────────────────┘
```

## Acquisition Strategies

### 1. Uncertainty Sampling

```python
def uncertainty_sampling(model, unlabeled_pool, n_select, strategy="entropy"):
    proba = model.predict_proba(unlabeled_pool)
    
    if strategy == "entropy":
        # Maximum entropy = most uncertain
        entropy = -np.sum(proba * np.log(proba + 1e-10), axis=1)
        idx = np.argsort(entropy)[-n_select:]
    
    elif strategy == "margin":
        # Smallest margin between top 2 classes
        sorted_proba = np.sort(proba, axis=1)
        margins = sorted_proba[:, -1] - sorted_proba[:, -2]
        idx = np.argsort(margins)[:n_select]
    
    elif strategy == "confidence":
        # Lowest confidence
        confidence = proba.max(axis=1)
        idx = np.argsort(confidence)[:n_select]
    
    return idx
```

### 2. Diversity Sampling

```python
def diversity_sampling(embeddings, n_select, n_diverse=5):
    """
    Coreset: select examples that cover the embedding space.
    Avoids selecting 100 nearly identical uncertain examples.
    """
    from sklearn.metrics.pairwise import euclidean_distances
    
    # Start with random example, then greedily add farthest
    selected = [np.random.randint(len(embeddings))]
    dists = euclidean_distances(embeddings, embeddings[selected])
    
    while len(selected) < n_select:
        # Each unselected: minimum distance to any selected
        min_dists = dists[:, selected].min(axis=1)
        # Pick the farthest
        next_idx = np.argmax(min_dists)
        selected.append(next_idx)
    
    return selected
```

### 3. Query-by-Committee

```python
def query_by_committee(committee, unlabeled_pool, n_select):
    """
    Ensemble disagreement: where do the experts differ most?
    """
    predictions = np.array([m.predict(unlabeled_pool) for m in committee])
    # Vote entropy: high = disagreement
    n_models = len(committee)
    vote_entropy = np.zeros(len(unlabeled_pool))
    
    for i in range(len(unlabeled_pool)):
        votes = predictions[:, i]
        _, counts = np.unique(votes, return_counts=True)
        probs = counts / n_models
        vote_entropy[i] = -np.sum(probs * np.log(probs + 1e-10))
    
    return np.argsort(vote_entropy)[-n_select:]
```

### 4. Expected Model Change

```python
def expected_model_change(model, unlabeled_pool, n_select):
    """
    Select examples that would change the model the most if labeled.
    Approximation: gradient magnitude.
    """
    if hasattr(model, "gradient_norm"):
        grad_norms = model.gradient_norm(unlabeled_pool)
        return np.argsort(grad_norms)[-n_select:]
    # Fallback: use uncertainty + diversity hybrid
    return hybrid_sampling(model, unlabeled_pool, n_select)
```

## Strategy Selection by Scenario

| Scenario | Best Strategy | Why |
|------|-------|-------|
| **Cold start** (no labels yet) | Diversity (coreset) | Explore the space first |
| **Warm start** (some labels) | Uncertainty + diversity hybrid | Exploit what the model doesn't know |
| **Label noise suspected** | Query-by-committee | Disagreement signals ambiguous examples |
| **Severe class imbalance** | Uncertainty weighted by class rarity | Don't oversample the majority |
| **Cost-sensitive** (some labels expensive) | Expected value of information | ROI per annotation dollar |
| **LLM-assisted** | LLM pre-label → uncertain to human | LLM does easy, human does hard |

## Measuring Active Learning ROI

```python
def active_learning_roi(baseline_model, al_model, n_labeled, cost_per_label):
    """
    Compare random sampling baseline to active learning.
    """
    random_perf = evaluate(baseline_model, test_set)
    al_perf = evaluate(al_model, test_set)
    
    perf_gain = al_perf - random_perf
    total_cost = n_labeled * cost_per_label
    roi = perf_gain / total_cost if total_cost > 0 else 0
    
    return {
        "random_performance": random_perf,
        "al_performance": al_perf,
        "absolute_gain": perf_gain,
        "relative_gain": f"{perf_gain / random_perf:.1%}" if random_perf > 0 else "N/A",
        "n_labeled": n_labeled,
        "cost_per_label": cost_per_label,
        "total_cost": total_cost,
        "roi": roi,
    }
```

## Common Pitfalls

| Pitfall | Fix |
|-------|-------|
| Selecting 100 copies of the same example | Use diversity in acquisition |
| Biasing toward the majority class | Weight by inverse class frequency |
| Training too infrequently | Retrain after every batch (or use online learning) |
| Ignoring annotation cost differences | Use cost-sensitive acquisition |
| Human bias in annotation | Measure inter-annotator agreement continuously |
| Drift in unlabeled pool | Re-embed periodically as pool changes |

## Quality Gate

- Acquisition strategy documented with rationale.
- Random sampling baseline compared (not assumed worse).
- Diversity of selected examples measured (embedding coverage).
- Annotation cost tracked per example.
- Model retrained after each batch and performance curve plotted.
- Active learning ROI reported (absolute and relative to random).
