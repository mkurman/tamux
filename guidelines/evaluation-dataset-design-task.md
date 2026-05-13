---
name: evaluation-dataset-design-task
description: Design evaluation/holdout datasets that actually measure what you think they do — IID vs OOD splits, adversarial test construction, counterfactual evaluation, multi-metric tradeoff, and minimum detectable effect sizing.
recommended_skills:
  - dataset-splitting
  - embedding-analysis
  - bias-audit
recommended_guidelines:
  - training-data-design-principles
  - data-contamination-task
---

## Overview

Your model is only as good as your evaluation. A poorly designed test set produces numbers that look great in the paper and fail in production. This guideline covers how to design evaluation datasets that measure real capability, not dataset-specific memorization.

## Phase 1: The Evaluation Pyramid

```
Production (can't measure directly)
    ↑
Adversarial / stress tests (hardest, most revealing)
    ↑
OOD generalization (different distribution from training)
    ↑
Stratified holdout (same distribution, strict leakage control)
    ↑
Random split (baseline — easy to ace, says the least)
```

Every model should report on at least 3 levels. Random split alone is not sufficient.

## Phase 2: IID Holdout Design

### Minimum Requirements

```python
def design_iid_holdout(dataset, label_col, test_size=0.2, seed=42):
    """
    Stratified random split with:
    - Fixed seed (reproducible)
    - Stratification (maintains class balance)
    - No entity leakage (same entity in only one split)
    """
    from sklearn.model_selection import train_test_split
    
    train, test = train_test_split(
        dataset, test_size=test_size, 
        stratify=dataset[label_col], random_state=seed
    )
    
    # Power analysis: can we detect meaningful differences?
    n_per_class_test = test[label_col].value_counts()
    min_class_size = n_per_class_test.min()
    
    issues = []
    if min_class_size < 30:
        issues.append(f"Class with only {min_class_size} test examples — unreliable metrics")
    if test_size * len(dataset) < 100:
        issues.append("Test set < 100 examples — large confidence intervals")
    
    return train, test, issues
```

## Phase 3: OOD Evaluation Design

### Distribution Shift Strategies

| Shift Type | How to Construct | What It Measures |
|-------|-------|-------|
| **Temporal** | Train on pre-2023, test on 2024+ | Real-world deployment readiness |
| **Geographic** | Train on US hospitals, test on EU hospitals | Cross-site generalization |
| **Demographic** | Train on young adults, test on elderly | Fairness across age |
| **Domain** | Train on news, test on scientific papers | Domain transfer |
| **Label distribution** | Train on balanced, test on natural distribution | Real-world class skew |
| **Adversarial** | Generate examples that break the current model | Worst-case robustness |

```python
def construct_ood_split(dataset, split_col, train_values, test_values):
    train = dataset[dataset[split_col].isin(train_values)]
    test = dataset[dataset[split_col].isin(test_values)]
    
    # Verify: no overlap
    assert len(set(train[split_col]) & set(test[split_col])) == 0
    
    # Measure: distribution shift magnitude
    from scipy.spatial.distance import jensenshannon
    shift = {}
    for col in dataset.select_dtypes(include=np.number).columns:
        train_hist, _ = np.histogram(train[col].dropna(), bins=50, density=True)
        test_hist, _ = np.histogram(test[col].dropna(), bins=50, density=True)
        shift[col] = {
            "js_divergence": float(jensenshannon(train_hist + 1e-10, test_hist + 1e-10)),
        }
    
    return train, test, shift
```

## Phase 4: Adversarial Test Sets

### Construction Methods

```python
# 1. Contrast set generation
# For each test example, create a minimally different negative
# "The movie was great" → "The movie was not great"
# Measures: sensitivity to negation, fine-grained distinctions

# 2. Invariance testing
# For each test example, apply label-preserving transformations
# Image: rotate 5°, shift 2px, adjust brightness ±10%
# Text: synonym replacement, active/passive voice swap
# Result: metric drop = fragility

# 3. Behavioral testing (CheckList-style)
# Minimum Functionality Tests (MFT): simple cases any model should get
# Invariance Tests (INV): perturbations should not change output
# Directional Expectation Tests (DIR): perturbation should change output predictably
```

## Phase 5: Multi-Metric Design

### What to Report (Minimum)

| Metric | What It Catches |
|-------|-------|
| **Accuracy / F1** | Overall performance (can be misleading) |
| **Per-class metrics** | Which classes the model fails on |
| **Confidence calibration** (ECE) | Is the model honest about uncertainty? |
| **Worst-group accuracy** | Does it fail on a specific subgroup? |
| **Pareto frontier** | Accuracy vs. latency vs. model size |

```python
# Confidence calibration check
from sklearn.calibration import calibration_curve

def audit_calibration(y_true, y_prob, n_bins=10):
    prob_true, prob_pred = calibration_curve(y_true, y_prob, n_bins=n_bins)
    ece = np.mean(np.abs(prob_true - prob_pred))  # Expected Calibration Error
    
    issues = []
    if ece > 0.1:
        issues.append(f"ECE={ece:.3f} — model is poorly calibrated")
    
    # Check for over/under-confidence
    overconfidence = (prob_pred > prob_true).mean()
    if overconfidence > 0.6:
        issues.append("Model is systematically overconfident")
    
    return {"ece": ece, "overconfidence_ratio": overconfidence, "issues": issues}
```

## Phase 6: Minimum Detectable Effect

```python
def minimum_detectable_effect(n_test, baseline_accuracy=0.85, alpha=0.05, power=0.8):
    """
    How small an improvement can you reliably detect?
    If your test set is too small, you can't prove your model is better.
    """
    from statsmodels.stats.proportion import proportion_effectsize
    from statsmodels.stats.power import zt_ind_solve_power
    
    # Two-proportion z-test power analysis
    effect_size = zt_ind_solve_power(effect_size=None, nobs1=n_test, 
                                      alpha=alpha, power=power, ratio=1.0,
                                      alternative="larger")
    # Convert effect size to accuracy difference
    # This is approximate for proportions
    return {
        "n_test": n_test,
        "baseline": baseline_accuracy,
        "min_detectable_improvement": effect_size,
        "min_detectable_accuracy": baseline_accuracy + effect_size,
        "adequate_power": effect_size < 0.02,  # can detect < 2pp improvement
    }
```

## Quality Gate

- Evaluation design documented before seeing test results.
- At least 3 levels of the pyramid reported (IID + OOD + one more).
- Per-class metrics reported (not just aggregate).
- Minimum detectable effect computed and discussed.
- No test set used for any decision (hyperparameter, feature selection, early stopping).
- Test set version-controlled and never updated after initial design.
