---
name: data-model-coevolution-task
description: Govern data-model feedback loops across generations — successor model data requirements, capability inheritance validation, generation-to-generation drift tracking, and cross-generation contamination detection.
recommended_skills: [data-diff, embedding-analysis, benchmark-contamination-scan]
recommended_guidelines: [data-feedback-loop-task, data-attribution-task, data-contamination-task]
---

## Overview

Models generate data. Data trains new models. Without governance, this loop amplifies bias, collapses diversity, and creates invisible cross-generation contamination. This guideline detects and prevents co-evolution pathologies.

## Phase 1: Capability Inheritance Validation

```python
def audit_capability_inheritance(model_v1, model_v2, test_suite):
    """Verify v2 retained all v1 capabilities before shipping."""
    regressions = []
    for task_name, task_data in test_suite.items():
        perf_v1 = evaluate(model_v1, task_data)
        perf_v2 = evaluate(model_v2, task_data)
        delta = perf_v2 - perf_v1
        
        if delta < -0.02:  # >2% regression
            regressions.append({"task": task_name, "v1": perf_v1, "v2": perf_v2, 
                                "delta": delta, "severity": "HARD_REGRESSION"})
        elif delta < 0:
            regressions.append({"task": task_name, "v1": perf_v1, "v2": perf_v2,
                                "delta": delta, "severity": "SOFT_REGRESSION"})
    
    return {"regressions": regressions, "n_regressed": len(regressions),
            "inheritance_score": 1 - len([r for r in regressions if r["severity"]=="HARD_REGRESSION"]) / max(len(test_suite), 1),
            "safe_to_ship": len([r for r in regressions if r["severity"]=="HARD_REGRESSION"]) == 0}
```

## Phase 2: Cross-Generation Contamination

```python
def detect_cross_gen_contamination(gen1_training, gen2_test):
    """Does v2's test set contain v1's training data?"""
    gen1_ngrams = set()
    for ex in gen1_training:
        tokens = str(ex).split()
        gen1_ngrams.update(" ".join(tokens[i:i+13]) for i in range(len(tokens)-12))
    
    contaminated = []
    for i, ex in enumerate(gen2_test):
        tokens = str(ex).split()
        test_ngrams = set(" ".join(tokens[j:j+13]) for j in range(len(tokens)-12))
        overlap = test_ngrams & gen1_ngrams
        if len(overlap) >= 3:
            contaminated.append(i)
    
    return {"contaminated_examples": len(contaminated),
            "contamination_rate": len(contaminated) / max(len(gen2_test), 1),
            "valid": len(contaminated) == 0}
```

## Phase 3: Generation Drift Tracking

```python
def track_generational_drift(generations_embeddings):
    """Track how data distribution shifts across model generations."""
    drift_metrics = []
    for i in range(1, len(generations_embeddings)):
        js = jensenshannon(
            np.histogram(generations_embeddings[i-1], bins=50, density=True)[0],
            np.histogram(generations_embeddings[i], bins=50, density=True)[0]
        )
        drift_metrics.append({"from_gen": i-1, "to_gen": i, "js_divergence": float(js),
                               "drift_severity": "CRITICAL" if js > 0.3 else "WARNING" if js > 0.15 else "STABLE"})
    return drift_metrics
```

## Quality Gate

- Zero hard regressions on the inheritance test suite.
- Cross-generation contamination scan clean (rate = 0).
- Generational drift JS divergence < 0.2 for consecutive generations.
- Every generation's training data versioned with provenance linking to parent generation.
