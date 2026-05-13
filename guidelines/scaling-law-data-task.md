---
name: scaling-law-data-task
description: Design data strategies for scaling laws — calibration curve fitting, compute-optimal data selection, early-stage performance prediction, anomaly detection, and multi-modal scaling coordination.
recommended_skills: [dataset-splitting, cost-model-task, embedding-analysis]
recommended_guidelines: [data-strategy-foundation-models-task, data-portfolio-theory-task]
---

## Overview

Scaling laws predict model performance from data and compute. But bad calibration data produces bad predictions. This guideline covers how to collect, validate, and use the data that scaling laws depend on.

## Phase 1: Scaling Curve Calibration

```python
def calibrate_scaling_law(model_sizes, data_sizes, performances):
    """
    Fit the Chinchilla-style law: L(N, D) = E + A/N^α + B/D^β
    N = parameters, D = tokens, L = loss
    """
    from scipy.optimize import minimize
    
    def chinchilla(params, N, D):
        E, A, alpha, B, beta = params
        return E + A / (N ** alpha) + B / (D ** beta)
    
    def loss(params):
        pred = chinchilla(params, model_sizes, data_sizes)
        return np.mean((pred - performances) ** 2)
    
    result = minimize(loss, [1.0, 100, 0.34, 100, 0.28], bounds=[
        (0.1, 10), (1, 1000), (0.1, 0.5), (1, 1000), (0.1, 0.5)
    ])
    
    E, A, alpha, B, beta = result.x
    return {"irreducible_loss": E, "param_exponent": alpha, "data_exponent": beta,
            "compute_optimal_ratio": f"D/N ≈ {beta/alpha:.1f}"}
```

**Requirements**: At least 3 model sizes × 3 data sizes = 9 points minimum. More is better. Vary ONE variable at a time for clean measurements.

## Phase 2: Compute-Optimal Data Selection

```python
def select_compute_optimal_subset(dataset, compute_budget, small_model, eval_task):
    """Select the subset that maximizes performance per FLOP."""
    subset_sizes = [int(compute_budget / k) for k in [1, 2, 5, 10, 20]]
    
    results = []
    for size in subset_sizes:
        subset = dataset.sample(min(size, len(dataset)))
        small_model.fit(subset)
        perf = evaluate(small_model, eval_task)
        flops = small_model.estimate_flops()
        results.append({"size": size, "perf": perf, "flops": flops, "efficiency": perf / flops})
    
    best = max(results, key=lambda r: r["efficiency"])
    return best
```

## Phase 3: Early-Stage Prediction

Use small-scale runs to predict large-scale performance:

| Small Run | Extrapolation | Prediction | Actual | Error |
|-----------|--------------|------------|--------|-------|
| 10M params, 200M tokens | ×100 scale | 2.1 loss | 2.05 loss | 2.4% |
| 50M params, 1B tokens | ×20 scale | 1.85 loss | 1.88 loss | 1.6% |

**Rule**: If prediction error > 5%, your scaling law is miscalibrated — recalibrate before committing to large-scale training.

## Phase 4: Scaling Anomaly Detection

```python
def detect_scaling_anomaly(actual_loss, predicted_loss, threshold=0.1):
    error = abs(actual_loss - predicted_loss) / predicted_loss
    if error > threshold:
        return {"anomaly": True, "error": error, "possible_causes": [
            "Data distribution shift at scale",
            "Optimization instability",
            "Scaling law miscalibration",
            "Benchmark contamination in larger training set"
        ]}
    return {"anomaly": False}
```

## Quality Gate

- Scaling law fit on ≥ 9 data points (3 model sizes × 3 data sizes).
- Extrapolation error < 5% on held-out scale point.
- Compute-optimal subset identified before full training.
- Anomalies flagged before committing large compute budgets.
