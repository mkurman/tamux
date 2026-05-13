---
name: data-mixture-optimization-task
description: Optimize data mixtures automatically — DoReMi, DoGE, auto-curricula, and learned mixing weights. Go from manual guesswork to learned data composition.
recommended_skills:
  - embedding-analysis
  - llm-assisted-curation
  - hf-datasets
recommended_guidelines:
  - training-data-design-principles
  - data-attribution-task
  - llm-training-data-task
---

## Overview

Manual data mixing ("40% web, 20% code, 15% books…") is guesswork. Learned data mixture optimization discovers the optimal composition automatically. This guideline covers methods, implementation, and validation.

## Phase 1: DoReMi (Domain Re-weighting with Minimax)

DoReMi (Xie et al., 2024) uses a small proxy model to find optimal domain weights before training the large model:

```python
# DoReMi-style domain weight optimization
def doremi_optimize(domains, proxy_model, n_steps=10000, lr=1e-3):
    """
    Optimize domain weights using minimax:
    - Outer loop: find weights that maximize proxy model's worst-domain loss
    - Inner loop: train proxy model with current weights
    """
    n_domains = len(domains)
    domain_weights = torch.ones(n_domains) / n_domains  # uniform start
    
    for step in range(n_steps):
        # Sample batch with current weights
        domain_idx = torch.multinomial(domain_weights, batch_size, replacement=True)
        batch = sample_from_domains(domains, domain_idx)
        
        # Train proxy model
        loss = proxy_model(batch)
        
        # Update domain weights: upweight domains with higher loss
        per_domain_loss = compute_per_domain_loss(proxy_model, domains)
        domain_weights *= torch.exp(lr * per_domain_loss)
        domain_weights /= domain_weights.sum()
        
        # Track
        if step % 500 == 0:
            print(f"Step {step}: weights={domain_weights.tolist()}")
    
    return domain_weights

# Key insight: domains that the proxy model struggles with get MORE weight
# This automatically discovers that books/code need higher weight than raw web
```

### DoReMi vs Manual Mixing

| Aspect | Manual | DoReMi |
|-------|-------|-------|
| Book proportion | Designer guesses 15% | Model discovers it needs 18% |
| Low-quality web | Designer filters by heuristic | Model naturally down-weights it |
| New domain added | Designer must guess weight | Re-optimize automatically |
| Compute cost | Negligible | ~10% of full training (proxy model) |
| Theoretical grounding | Vibes | Minimax optimality |

## Phase 2: DoGE (Domain Generalization via Evolution)

DoGE dynamically adjusts domain weights DURING training, not just at the start:

```python
# DoGE-style dynamic mixing (simplified)
def doge_training_loop(model, domains, total_steps):
    domain_weights = torch.ones(len(domains)) / len(domains)
    loss_history = {d: [] for d in domains}
    
    for step in range(total_steps):
        # Sample domain
        d_idx = torch.multinomial(domain_weights, 1).item()
        batch = domains[d_idx].sample()
        
        # Train
        loss = model.train_step(batch)
        loss_history[d_idx].append(loss)
        
        # Every K steps: update weights based on recent loss trajectory
        if step % 100 == 0:
            for d in domains:
                if len(loss_history[d]) >= 50:
                    # Domains with IMPROVING loss get MORE weight
                    # Domains with STAGNATING loss get LESS weight
                    recent = loss_history[d][-50:]
                    improvement = recent[0] - recent[-1]
                    domain_weights[d] *= max(0.5, min(2.0, 1.0 + improvement))
            domain_weights /= domain_weights.sum()
    
    return model
```

## Phase 3: Data Mixture Evaluation

### How to Know Your Mixture Works

```python
def evaluate_mixture(mixture_weights, domains, eval_tasks, proxy_model_fn):
    """Does this mixture produce a better model than uniform?"""
    
    # Train with proposed mixture
    model_mix = train_with_mixture(proxy_model_fn(), domains, mixture_weights)
    results_mix = evaluate_on_tasks(model_mix, eval_tasks)
    
    # Train with uniform baseline
    uniform_weights = torch.ones(len(domains)) / len(domains)
    model_uniform = train_with_mixture(proxy_model_fn(), domains, uniform_weights)
    results_uniform = evaluate_on_tasks(model_uniform, eval_tasks)
    
    # Compare
    comparison = {}
    for task in eval_tasks:
        delta = results_mix[task] - results_uniform[task]
        comparison[task] = {
            "uniform": results_uniform[task],
            "optimized": results_mix[task],
            "delta": delta,
            "improved": delta > 0,
        }
    
    # Per-domain breakdown: which domains gained/lost weight?
    weight_comparison = {
        dom: {"uniform": 1/len(domains), "optimized": w.item()}
        for dom, w in zip(domains, mixture_weights)
    }
    
    return comparison, weight_comparison
```

## Phase 4: Auto-Curricula

The model learns which order to train on data, not just which data:

```python
# Anti-curriculum: start with hard examples, finish with easy
# Curriculum: start with easy examples, finish with hard
# Learned curriculum: model decides the order

def learned_curriculum(dataset, model, n_epochs):
    """The model learns what to learn next."""
    difficulties = compute_per_example_difficulty(model, dataset)
    
    # Sort by learnability: examples model is JUST starting to get right
    # These are in the "zone of proximal development"
    predictions = model.predict_proba(dataset)
    correctness = (predictions.argmax(axis=1) == dataset.labels)
    confidence = predictions.max(axis=1)
    
    # Learning signal: correct BUT low confidence → consolidating
    #                  wrong BUT moderate confidence → almost there
    learnability = np.where(
        correctness,
        1 - confidence,           # correct + low confidence = good to reinforce
        confidence - 0.5           # wrong + moderate confidence = close to getting it
    )
    
    return np.argsort(-learnability)  # highest learnability first
```

## Phase 5: Mixture Observability

### What to Track

```python
def mixture_dashboard(mixture_history, eval_history):
    """Live dashboard of mixture evolution."""
    
    # 1. Weight trajectory: how did domain weights evolve over training?
    # Plot: weight ~ training step, one line per domain
    
    # 2. Weight-entropy: is the mixture concentrating or diversifying?
    entropy = -np.sum(weights * np.log(weights + 1e-10), axis=1)
    # Declining entropy → model is specializing (good if late in training)
    # Increasing entropy → model is diversifying (good if early in training)
    
    # 3. Eval correlation: does weight change correlate with eval improvement?
    for eval_task in eval_tasks:
        corr = np.corrcoef(weights[:, domain_idx], eval_history[eval_task])[0, 1]
        # Positive correlation: more of this domain → better on this task
    
    # 4. Domain saturation: is more data from this domain still helping?
    # Measure: derivative of eval w.r.t. domain weight
    # If flat → domain is saturated, reduce weight
```

## Quality Gate

- Optimized mixture beats uniform baseline on ≥ 80% of evaluation tasks.
- Weight entropy monitored — extreme concentration (one domain > 60%) flagged for review.
- Domain saturation checked — down-weight saturated domains, up-weight unsaturated.
- Mixture optimization reproduced with different random seeds (weights should be stable).
- Mixing ratio evolution logged for every training run.
