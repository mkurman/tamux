---
name: mechanistic-interpretability-data-task
description: Curate datasets for mechanistic interpretability — sparse autoencoder training data, circuit discovery data, and activation patching. Design data that reveals how models think, not just what they predict.
recommended_skills:
  - embedding-analysis
  - llm-assisted-curation
  - hf-datasets
recommended_guidelines:
  - training-data-design-principles
  - data-contamination-task
  - evaluation-dataset-design-task
---

## Overview

Mechanistic interpretability needs different data than model training. You're not optimizing for accuracy — you're optimizing for REVEALING the model's internal algorithms. This guideline covers data design for SAEs, circuit discovery, and activation analysis.

## Phase 1: Sparse Autoencoder (SAE) Training Data

### What Makes Good SAE Data

SAEs learn to decompose model activations into interpretable features. The data must:
- Cover the model's full activation distribution (not just easy examples).
- Include edge cases that activate rare features.
- Be diverse enough to force feature separation.

```python
def design_sae_dataset(model, candidate_texts, n_samples=100000):
    """
    Select data that maximizes activation diversity for SAE training.
    """
    # Sample initial pool
    texts = sample(candidate_texts, n_samples * 2)
    
    # Run model, collect activations
    activations = []
    for text in texts:
        with torch.no_grad():
            _, cache = model.run_with_cache(text)
            layer_acts = cache["resid_post", 8]  # middle layer
            activations.append(layer_acts.mean(dim=0).detach())
    activations = torch.stack(activations)
    
    # Select for activation diversity
    from sklearn.metrics.pairwise import cosine_similarity
    selected = [0]
    sim_matrix = cosine_similarity(activations.numpy())
    
    while len(selected) < n_samples:
        # Find example that is LEAST similar to anything selected
        min_sims = sim_matrix[:, selected].min(axis=1)
        next_idx = np.argmax(min_sims)
        selected.append(next_idx)
        if len(selected) % 10000 == 0:
            print(f"Selected {len(selected)} examples")
    
    return [texts[i] for i in selected]
```

### SAE Data Composition

| Component | Fraction | Purpose |
|-------|-------|-------|
| Diverse natural text | 70% | Cover the model's normal operating range |
| Adversarial / edge cases | 15% | Activate rare features, stress-test separation |
| Contrastive pairs | 10% | Force features to distinguish similar-but-different concepts |
| Synthetic probing text | 5% | Deliberately activate suspected features |

### Contrastive Pairs for SAEs

```python
def generate_contrastive_pairs(text, model, n_variations=3):
    """
    Create minimally different texts to force feature separation.
    """
    pairs = []
    
    # 1. Gender swap
    variants = [text.replace("he", "she"), text.replace("man", "woman")]
    pairs.extend([(text, v) for v in variants if v != text])
    
    # 2. Negation
    if " is " in text:
        neg = text.replace(" is ", " is not ")
        pairs.append((text, neg))
    
    # 3. Entity substitution (same type, different instance)
    # "Paris is the capital of France" → "London is the capital of England"
    # Requires entity recognition + same-type substitution
    
    return pairs[:n_variations]
```

## Phase 2: Circuit Discovery Data

### Activation Patching

```python
def design_patching_dataset(model, clean_examples, corrupted_examples):
    """
    Clean examples: model gets right.
    Corrupted examples: model gets wrong.
    Patching: replace clean activations with corrupted ones to find which
    components matter for which behaviors.
    """
    patches = []
    
    for clean, corrupt in zip(clean_examples, corrupted_examples):
        # Verify: model is correct on clean, incorrect on corrupt
        clean_pred = model(clean["text"])
        corrupt_pred = model(corrupt["text"])
        
        if clean_pred != clean["label"] or corrupt_pred == corrupt["label"]:
            continue
        
        patches.append({
            "clean": clean,
            "corrupt": corrupt,
            "expected_behavior": clean["label"],
            "corrupted_behavior": corrupt_pred,
        })
    
    return patches
```

### Minimal Pair Design

For circuit analysis, construct pairs where EXACTLY ONE concept changes:

```
Prompt: "The capital of France is"
Answer A: " Paris" (correct)
Answer B: " London" (incorrect)

→ Patches that flip A→B reveal the "France→Paris" circuit
→ Patches that flip B→A reveal the "capital retrieval" circuit
```

## Phase 3: Feature Visualization Data

### Maximally Activating Examples

```python
def find_max_activating_examples(feature_idx, sae, dataset, top_k=20):
    """
    Find the text that most strongly activates a specific SAE feature.
    These define "what the feature means."
    """
    activations = []
    for example in dataset:
        with torch.no_grad():
            acts = sae.encode(model.get_activations(example["text"]))
            feature_act = acts[:, feature_idx].max().item()
            activations.append((example, feature_act))
    
    activations.sort(key=lambda x: -x[1])
    
    # Show top and also random for contrast
    return {
        "feature_idx": feature_idx,
        "top_examples": activations[:top_k],
        "random_examples": random.sample(activations, min(top_k, len(activations))),
        "activation_distribution": [a[1] for a in activations],
    }
```

## Phase 4: Interpretability Benchmarks

| Benchmark | What It Tests | Data Requirements |
|-------|-------|-------|
| **Tracr** | Can SAEs recover known ground-truth features? | Synthetic programs with known structure |
| **SPAR** | Sparse probing — can a linear probe recover features? | Labeled feature datasets |
| **BABILong** | Long-context reasoning circuits | Synthetic reasoning tasks at various lengths |
| **Causal scrubbing** | Does the hypothesized circuit actually explain behavior? | Counterfactual test examples |

## Phase 5: Data Design Principles for Interpretability

| Principle | Why | Implementation |
|-------|-------|-------|
| **Activation diversity** | SAEs need to see the full activation range | Coreset selection on activation space |
| **Minimal pairs** | Isolate individual circuits | Automated contrastive pair generation |
| **Ground-truth features** | Validate that SAEs recover known structure | Synthetic data with injected features |
| **Distribution coverage** | Don't miss rare but important features | Stratified sampling by activation magnitude |
| **Temporal structure** | Some features only activate in sequence | Multi-token contexts, not single tokens |
| **Cross-model** | Compare features across models | Same data, different model checkpoints |

## Quality Gate

- SAE training data covers the full activation distribution (diversity check).
- Contrastive pairs exist for suspected feature families.
- Activation patching dataset contains clean/corrupt pairs with clear behavioral difference.
- Interpretability benchmark results reported alongside SAE metrics.
- Feature visualization examples reviewed by human for semantic coherence.
