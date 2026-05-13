---
name: curriculum-learning-data-task
description: Design curriculum learning datasets — difficulty scoring validation, progressive difficulty ordering, prerequisite learning validation, adaptive curricula, and anti-curriculum (hard-first) strategies.
recommended_skills: [embedding-analysis, llm-assisted-curation, dataset-splitting]
recommended_guidelines: [specialized-training-data-task, data-mixture-optimization-task]
---

## Overview

Not all examples are equally learnable at all stages. Curriculum learning orders examples from easy to hard, accelerating convergence and improving final performance. This guideline covers how to score difficulty, validate the ordering, and adapt the curriculum.

## Phase 1: Difficulty Scoring

```python
def score_difficulty(examples, model=None, metrics=["length", "perplexity", "entropy"]):
    scores = np.zeros(len(examples))
    weights = {"length": 0.2, "perplexity": 0.5, "entropy": 0.3}
    
    for metric in metrics:
        if metric == "length": scores += weights[metric] * _normalize([len(str(ex)) for ex in examples])
        elif metric == "perplexity" and model: scores += weights[metric] * _normalize(model.perplexity(examples))
        elif metric == "entropy": scores += weights[metric] * _normalize([_text_entropy(str(ex)) for ex in examples])
    
    return scores  # 0 = easiest, 1 = hardest

def validate_difficulty_ordering(ordered_examples, model):
    """Verify: does the model get easy examples right and hard ones wrong?"""
    easy = ordered_examples[:len(ordered_examples)//3]
    hard = ordered_examples[-len(ordered_examples)//3:]
    easy_acc = evaluate(model, easy)
    hard_acc = evaluate(model, hard)
    return {"easy_accuracy": easy_acc, "hard_accuracy": hard_acc,
            "valid": easy_acc > hard_acc + 0.1,  # meaningful gap
            "difficulty_separation": easy_acc - hard_acc}
```

## Phase 2: Curriculum Strategies

| Strategy | Order | Best For |
|----------|-------|----------|
| **Standard** | Easy → Hard | Stable convergence |
| **Anti-curriculum** | Hard → Easy | Robustness, noisy data |
| **Competence-based** | Model decides — learn examples with predicted success probability 0.5-0.7 | Optimal sample efficiency |
| **Interleaved** | Mix easy and hard | Prevents forgetting |
| **Adaptive** | Difficulty updated as model improves | Dynamic curricula |

## Phase 3: Prerequisite Validation

```python
def detect_prerequisites(examples, labels, model):
    """Does the model need to learn concept A before concept B?"""
    a_examples = [ex for ex, lbl in zip(examples, labels) if "A" in str(ex)]
    b_examples = [ex for ex, lbl in zip(examples, labels) if "B" in str(ex)]
    
    # Train on A only → test on B
    model_a = train(model, a_examples)
    perf_b_with_a = evaluate(model_a, b_examples)
    
    # Train on B only → test on B
    model_b = train(model, b_examples)
    perf_b_direct = evaluate(model_b, b_examples)
    
    # A is prerequisite if learning A helps B
    return {"a_helps_b": perf_b_with_a > perf_b_direct,
            "prerequisite_strength": perf_b_with_a - perf_b_direct}
```

## Quality Gate

- Difficulty ordering validated: easy accuracy > hard accuracy + 10pp.
- Curriculum strategy documented with rationale.
- Adaptive curriculum updates difficulty at least every N epochs.
- Prerequisite relationships validated where applicable.
