---
name: data-feedback-loop-task
description: Monitor and govern self-training feedback loops — quality drift detection, pseudo-label confidence decay modeling, model-generated data validation, and iterative refinement stopping criteria.
recommended_skills:
  - embedding-analysis
  - data-diff
  - label-quality-audit
  - llm-assisted-curation
recommended_guidelines:
  - data-contamination-task
  - synthetic-data-generation-task
  - training-data-design-principles
---

## Overview

Self-training creates feedback loops. The model generates data, trains on it, generates more data. Without monitoring, these loops amplify model biases, collapse diversity, and inject noise disguised as signal. This guideline detects divergence before it contaminates the model.

## Phase 1: Quality Drift Detection

### Distribution Divergence Over Iterations

```python
def monitor_self_training_drift(reference_data, iteration_data, iteration):
    """Detect distribution drift from original data to self-generated data."""
    
    # 1. Embedding space shift
    from scipy.spatial.distance import jensenshannon
    
    ref_emb = embed(reference_data)
    iter_emb = embed(iteration_data)
    
    # Per-dimension JS divergence
    divergences = []
    for d in range(ref_emb.shape[1]):
        ref_hist, _ = np.histogram(ref_emb[:, d], bins=50, density=True)
        iter_hist, _ = np.histogram(iter_emb[:, d], bins=50, density=True)
        divergences.append(jensenshannon(ref_hist + 1e-10, iter_hist + 1e-10))
    
    mean_divergence = np.mean(divergences)
    
    # 2. Vocabulary / token distribution shift
    ref_tokens = tokenize_and_count(reference_data)
    iter_tokens = tokenize_and_count(iteration_data)
    token_overlap = len(set(ref_tokens) & set(iter_tokens)) / len(set(iter_tokens))
    
    # 3. Diversity collapse detection
    ref_diversity = _pairwise_diversity(ref_emb)
    iter_diversity = _pairwise_diversity(iter_emb)
    diversity_ratio = iter_diversity / ref_diversity if ref_diversity > 0 else 1.0
    
    drift = {
        "iteration": iteration,
        "js_divergence": float(mean_divergence),
        "token_overlap": token_overlap,
        "diversity_ratio": diversity_ratio,
        "diversity_collapse": diversity_ratio < 0.7,
        "drift_severity": (
            "critical" if mean_divergence > 0.3 or diversity_ratio < 0.5
            else "warning" if mean_divergence > 0.15 or diversity_ratio < 0.7
            else "stable"
        ),
    }
    
    return drift

def _pairwise_diversity(embeddings, sample_size=1000):
    """Average pairwise cosine distance = diversity metric."""
    idx = np.random.choice(len(embeddings), min(sample_size, len(embeddings)), replace=False)
    sims = cosine_similarity(embeddings[idx])
    mask = ~np.eye(len(idx), dtype=bool)
    return float(1 - sims[mask].mean())
```

### Drift Intervention Triggers

| Signal | Threshold | Intervention |
|-------|-------|-------|
| JS divergence > 0.15 | Warning | Increase generation temperature, add diversity sampling |
| JS divergence > 0.3 | Critical | Stop self-training, mix in fresh real data |
| Diversity ratio < 0.7 | Warning | Add synthetic diversity, broaden generation prompts |
| Diversity ratio < 0.5 | Critical | Revert to previous iteration's model as teacher |
| Token overlap < 0.6 | Warning | Model is generating out-of-domain text |
| Token overlap < 0.4 | Critical | Model has catastrophically drifted — restart |

## Phase 2: Pseudo-Label Confidence Decay

### When Does Self-Supervision Become Noise?

```python
def model_confidence_decay_curve(model, unlabeled_data, n_iterations=10):
    """
    Track how model confidence on pseudo-labels changes over iterations.
    Confidence naturally increases (model gets "better") but then DECREASES
    as self-training noise accumulates.
    """
    confidence_trajectory = []
    
    for iteration in range(n_iterations):
        # Generate pseudo-labels
        proba = model.predict_proba(unlabeled_data)
        pseudo_labels = proba.argmax(axis=1)
        confidences = proba.max(axis=1)
        
        # Train on pseudo-labels
        model.fit(unlabeled_data, pseudo_labels, sample_weight=confidences)
        
        # Measure
        stats = {
            "iteration": iteration,
            "mean_confidence": float(np.mean(confidences)),
            "median_confidence": float(np.median(confidences)),
            "low_confidence_fraction": float(np.mean(confidences < 0.6)),
            "confidence_std": float(np.std(confidences)),
        }
        confidence_trajectory.append(stats)
        
        # Detect the inflection point
        if iteration >= 3:
            recent = [c["mean_confidence"] for c in confidence_trajectory[-3:]]
            if recent[-1] < recent[-2] < recent[-3]:
                stats["stopping_signal"] = True
                stats["stopping_reason"] = "confidence_declining"
                break
    
    return confidence_trajectory
```

### Optimal Stopping Criteria

```python
def compute_optimal_stopping_point(trajectory, real_validation_set, model):
    """
    The optimal stop is when validation performance on REAL data peaks.
    After this, self-training improves on self-generated data but degrades on real data.
    """
    val_scores = []
    model_state = copy.deepcopy(model.state_dict())
    
    for iter_stats in trajectory:
        model.load_state_dict(model_state)
        score = evaluate(model, real_validation_set)
        val_scores.append(score)
        
        # Train one more iteration
        model.train_one_iteration()
        model_state = copy.deepcopy(model.state_dict())
    
    optimal_iteration = np.argmax(val_scores)
    
    return {
        "optimal_iteration": optimal_iteration,
        "peak_score": val_scores[optimal_iteration],
        "current_score": val_scores[-1],
        "overshot": optimal_iteration < len(val_scores) - 1,
        "degradation_from_peak": val_scores[optimal_iteration] - val_scores[-1],
        "val_curve": val_scores,
    }
```

## Phase 3: Model-Generated Data Validation

### The Meta-Evaluation Problem

When the model generates training data, who validates the generator?

```python
def validate_model_generated_data(generated_examples, validation_panel):
    """
    Multi-dimensional validation of LLM-generated training data.
    validation_panel: list of (validator_model, validator_prompt, weight)
    """
    results = {}
    
    for example in generated_examples:
        scores = {}
        for validator_name, validator_fn, weight in validation_panel:
            score = validator_fn(example)
            scores[validator_name] = {
                "score": score,
                "weighted": score * weight,
            }
        
        # Ensemble score
        total_weight = sum(v[2] for v in validation_panel)
        ensemble = sum(s["weighted"] for s in scores.values()) / total_weight
        
        example_id = example.get("id", hash(str(example)))
        results[example_id] = {
            "per_validator": scores,
            "ensemble_score": ensemble,
            "pass": ensemble > 0.5,
            "disagreement": max(s["score"] for s in scores.values()) - 
                           min(s["score"] for s in scores.values()),
        }
    
    # Disagreement analysis
    high_disagreement = [k for k, v in results.items() if v["disagreement"] > 0.4]
    
    return {
        "results": results,
        "pass_rate": np.mean([r["pass"] for r in results.values()]),
        "mean_ensemble_score": np.mean([r["ensemble_score"] for r in results.values()]),
        "high_disagreement_examples": len(high_disagreement),
        "disagreement_rate": len(high_disagreement) / len(generated_examples),
    }
```

## Phase 4: Feedback Loop Observability

### The Self-Training Dashboard

```python
def feedback_loop_dashboard(iterations_history):
    """
    What to track across self-training iterations:
    """
    fig, axes = plt.subplots(2, 3, figsize=(18, 10))
    
    # 1. Quality drift: JS divergence from original data
    axes[0, 0].plot([h["js_divergence"] for h in iterations_history])
    axes[0, 0].axhline(0.15, color="orange", linestyle="--", label="warning")
    axes[0, 0].axhline(0.3, color="red", linestyle="--", label="critical")
    axes[0, 0].set_title("Distribution Drift from Origin")
    
    # 2. Diversity collapse
    axes[0, 1].plot([h["diversity_ratio"] for h in iterations_history])
    axes[0, 1].axhline(0.7, color="orange", linestyle="--")
    axes[0, 1].set_title("Diversity Ratio (vs origin)")
    
    # 3. Pseudo-label confidence
    axes[0, 2].plot([h["mean_confidence"] for h in iterations_history])
    axes[0, 2].set_title("Mean Pseudo-Label Confidence")
    
    # 4. Real validation performance
    axes[1, 0].plot([h.get("real_val_score", 0) for h in iterations_history])
    axes[1, 0].set_title("Performance on REAL Validation Set")
    
    # 5. Synthetic quality pass rate
    axes[1, 1].plot([h.get("quality_pass_rate", 0) for h in iterations_history])
    axes[1, 1].set_title("Generated Data Quality Pass Rate")
    
    # 6. Stopping signal
    if any(h.get("stopping_signal") for h in iterations_history):
        stop_iter = next(i for i, h in enumerate(iterations_history) if h.get("stopping_signal"))
        for ax in axes.flat:
            ax.axvline(stop_iter, color="red", linestyle=":", linewidth=2, label=f"stop@{stop_iter}")
    
    return fig
```

## Quality Gate

- JS divergence from original data tracked every iteration; critical threshold halts training.
- Pseudo-label confidence curve monitored for inflection point (decline = noise injection).
- Optimal stopping point computed via real validation set (not self-generated metrics).
- Model-generated data validated by ensemble of independent validators, not the generator.
- Disagreement between validators flagged for human review (> 0.4 spread).
- All self-training iterations versioned with full trajectory logs.
