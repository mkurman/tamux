---
name: data-attribution-task
description: Trace which training examples cause which model behaviors — TRAK, influence functions, datamodels, and example-level attribution. Move from "is this data good" to "this example caused that prediction."
recommended_skills:
  - embedding-analysis
  - data-diff
  - benchmark-contamination-scan
recommended_guidelines:
  - training-data-design-principles
  - evaluation-dataset-design-task
---

## Overview

Data curation today answers: "is this dataset good?" Data attribution answers: "this specific training example caused this specific model behavior on this specific test input." It's the difference between correlation and causation in data engineering.

## Phase 1: Influence Functions

### What They Measure

How would the model's prediction on test example z change if training example x were removed and the model retrained?

```python
# Influence function approximation (Koh & Liang, 2017)
# I(x, z) ≈ -∇_θ L(z, θ̂)ᵀ · H⁻¹ · ∇_θ L(x, θ̂)
# Where H is the Hessian of the training loss at θ̂

def compute_influence(model, train_loader, test_example, top_k=100):
    """
    Approximate influence of each training example on a test prediction.
    Uses LiSSA (Agarwal et al., 2017) for efficient inverse-Hessian-vector products.
    """
    # Get test gradient
    test_grad = get_gradient(model, test_example)
    
    # LiSSA: estimate H⁻¹ · test_grad without computing H
    ihvp = estimate_ihvp_lissa(model, train_loader, test_grad, num_steps=50)
    
    # Score each training example
    influences = []
    for batch_idx, (x, y, idx) in enumerate(train_loader):
        train_grad = get_gradient(model, (x, y))
        score = -torch.dot(train_grad.flatten(), ihvp.flatten())
        influences.extend(zip(idx.tolist(), score.tolist()))
    
    # Top proponent (positive influence) and opponent (negative)
    influences.sort(key=lambda x: x[1], reverse=True)
    return {
        "test_example_id": test_example["id"],
        "top_proponents": influences[:top_k],     # helped correct prediction
        "top_opponents": influences[-top_k:],      # hurt correct prediction
    }
```

### What To Do With Influence Scores

| Finding | Action |
|-------|-------|
| Many highly influential mislabeled examples | Fix labels (confident learning already flagged them) |
| A single example dominates influence | Check for memorization, benchmark contamination |
| Influence concentrated in one data source | Audit that source's quality |
| Test example has zero influential training examples | Test example may be OOD — model is guessing |

## Phase 2: TRAK (Tracing with Randomly-projected After Kernel)

TRAK (Park et al., 2023) scales influence to large models by using random projections and ensembling:

```python
# TRAK-style attribution (simplified)
def trak_attribution(model, train_dataset, test_dataset, 
                     projection_dim=1024, n_models=10):
    """
    Train n_models on subsets, project gradients, attribute.
    Scales to foundation models unlike exact influence.
    """
    P = torch.randn(model.output_dim, projection_dim) / np.sqrt(projection_dim)
    
    # Projected training gradients
    train_features = []
    for model_i in range(n_models):
        model_i = train_subset(model, train_dataset, seed=model_i)
        for x, y, idx in train_dataset:
            grad = get_gradient(model_i, (x, y))
            train_features.append({
                "model": model_i, "idx": idx,
                "feature": (grad @ P).detach(),
            })
    
    # Projected test gradients
    test_features = []
    for z, z_label in test_dataset:
        test_features.append({
            "id": z["id"],
            "features": [(get_gradient(model_i, (z, z_label)) @ P).detach() 
                         for model_i in range(n_models)],
        })
    
    # Attribution: cosine similarity in projected space
    attributions = []
    for tf in test_features:
        scores = {}
        for trf in train_features:
            score = F.cosine_similarity(tf["features"][trf["model"]], trf["feature"], dim=0)
            scores[trf["idx"]] = scores.get(trf["idx"], 0) + score.item()
        attributions.append({
            "test_id": tf["id"],
            "top_train_examples": sorted(scores.items(), key=lambda x: -x[1])[:50],
        })
    
    return attributions
```

## Phase 3: Datamodels

Datamodels (Ilyas et al., 2022) learn a linear model that predicts model behavior from training set membership:

```python
# Datamodel estimation
# For each training example i, learn weight w_i such that:
#   ŷ(z) ≈ Σ_i w_i · 1[i in training set]
# across many models trained on random subsets

def estimate_datamodels(n_examples, n_models=1000, subset_size=0.5):
    """
    Train many models on random subsets, estimate per-example weights.
    """
    subsets = np.random.binomial(1, subset_size, size=(n_models, n_examples))
    
    outcomes = []
    for model_i in range(n_models):
        train_mask = subsets[model_i].astype(bool)
        model = train_model(train_data[train_mask])
        outcomes.append(model.evaluate(test_data))
    
    # Solve: outcomes ≈ subsets @ weights
    weights, residuals = np.linalg.lstsq(subsets, outcomes, rcond=None)[:2]
    
    return {
        "weights": weights,
        "r_squared": 1 - residuals / np.var(outcomes),
        "top_positive": np.argsort(weights)[-100:][::-1],  # most helpful
        "top_negative": np.argsort(weights)[:100],          # most harmful
    }
```

## Phase 4: Attribution-Driven Data Curation

### The Attribution Curation Loop

```
1. Train model on current dataset
2. Run TRAK/influence on validation examples
3. For each WRONG prediction:
   a. Find training examples with strongest negative influence
   b. Inspect these examples — are they mislabeled, noisy, or adversarial?
4. Remove/fix the harmful examples
5. For each RIGHT prediction:
   a. Find training examples with strongest positive influence
   b. These are your "gold" examples — protect them, find more like them
6. Retrain on curated dataset
7. Compare attribution patterns — did curation remove harmful influence?
```

### Attribution Metrics

```python
def attribution_metrics(influence_results, dataset):
    """What does the attribution pattern tell us about the dataset?"""
    
    # Concentration: does a few examples dominate all predictions?
    influence_counts = {}
    for result in influence_results:
        for idx, score in result["top_proponents"][:10]:
            influence_counts[idx] = influence_counts.get(idx, 0) + 1
    
    top_examples = sorted(influence_counts.values(), reverse=True)
    concentration = sum(top_examples[:100]) / sum(top_examples) if top_examples else 0
    
    # Memorization: do training examples have outsized self-influence?
    memorization = []
    for train_idx in sample_train_indices:
        self_influence = compute_self_influence(model, train_idx)
        if self_influence > threshold:
            memorization.append(train_idx)
    
    return {
        "concentration_ratio": concentration,
        "concentration_warning": concentration > 0.3,  # 100 examples account for >30% of influence
        "memorized_examples": len(memorization),
        "memorization_rate": len(memorization) / len(dataset),
    }
```

## Phase 5: What This Unlocks

| Capability | Without Attribution | With Attribution |
|-------|-------|-------|
| Remove toxic behavior | Guess which data caused it | Remove exactly the 17 examples that taught the model to be toxic |
| Fix a specific wrong prediction | Retrain on "better data" | Find the 3 training examples that confused the model |
| Data valuation | Treat all data as equal | Pay data providers based on their examples' actual influence |
| Debug model failures | Stare at loss curves | Trace the exact training provenance of every wrong answer |
| Comply with "right to erasure" | Retrain from scratch | Remove specific training examples and measure the residual influence |
| Optimize data budget | Guess which 20% of data matters most | Compute which 20% of data carries 80% of influence |

## Quality Gate

- Attribution method documented (influence, TRAK, or datamodels).
- Concentration ratio computed — if a few examples dominate, the dataset is brittle.
- Memorized examples identified and audited for benchmark contamination.
- Top harmful examples (negative influence on validation) reviewed by human.
- Attribution results versioned alongside the dataset.
