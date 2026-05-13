---
name: cross-validation-strategy-task
description: Design cross-validation protocols — stratified, group, temporal, spatial, leave-one-cluster-out, nested CV for tuning, and what each strategy actually estimates.
recommended_skills:
  - dataset-splitting
  - bias-audit
recommended_guidelines:
  - training-data-design-principles
  - data-contamination-task
  - evaluation-dataset-design-task
---

## Overview

Cross-validation is the most misused tool in ML. Random k-fold on time series data. Group k-fold on IID data. Reporting the mean without the variance. This guideline fixes that.

## The Strategy Matrix

| Data Structure | Use This CV | Why |
|------|-------|-------|
| **IID** | Stratified k-fold | Preserves class balance per fold |
| **Groups** (patients, users) | Group k-fold | All samples from one group in the same fold |
| **Time series** | TimeSeriesSplit | Train always before test |
| **Spatial** | Spatial k-fold (blocked) | Accounts for spatial autocorrelation |
| **Hierarchical** (schools → classes → students) | Leave-one-cluster-out | Test on unseen clusters |
| **Imbalanced** | Stratified + repeated | Reliable per-class metrics |
| **Small dataset (< 500)** | Leave-one-out (LOO) or 5x repeated 5-fold | Maximize training data |
| **Large dataset (> 100K)** | Single holdout suffices | CV adds compute without value |

## When Random k-fold Is Wrong

```python
def cv_compatibility_check(dataset_info):
    issues = []
    
    if dataset_info.get("temporal"):
        issues.append({
            "severity": "critical",
            "message": "Random k-fold on temporal data = training on the future. Use TimeSeriesSplit."
        })
    
    if dataset_info.get("has_groups"):
        issues.append({
            "severity": "critical",
            "message": "Same group across folds = inflated scores. Use GroupKFold."
        })
    
    if dataset_info.get("spatial_autocorrelation"):
        issues.append({
            "severity": "high",
            "message": "Nearby samples in different folds = leakage. Use spatial blocking."
        })
    
    if dataset_info.get("hierarchical"):
        issues.append({
            "severity": "high",
            "message": "Nested structure ignored = overoptimistic. Use leave-one-cluster-out."
        })
    
    return issues
```

## Protocol Templates

### Stratified Group k-fold

```python
from sklearn.model_selection import StratifiedGroupKFold

cv = StratifiedGroupKFold(n_splits=5, shuffle=True, random_state=42)
for fold, (train_idx, test_idx) in enumerate(cv.split(X, y, groups=groups)):
    # Verify: no group overlap
    train_groups = set(groups[train_idx])
    test_groups = set(groups[test_idx])
    assert len(train_groups & test_groups) == 0, f"Fold {fold}: group leakage!"
```

### Nested CV (Hyperparameter Tuning)

```python
from sklearn.model_selection import GridSearchCV, cross_val_score, StratifiedKFold

inner_cv = StratifiedKFold(n_splits=3, shuffle=True, random_state=42)
outer_cv = StratifiedKFold(n_splits=5, shuffle=True, random_state=42)

# Inner loop: hyperparameter tuning
clf = GridSearchCV(estimator, param_grid, cv=inner_cv)

# Outer loop: unbiased performance estimate
nested_scores = cross_val_score(clf, X, y, cv=outer_cv)
print(f"Nested CV: {nested_scores.mean():.3f} ± {nested_scores.std():.3f}")

# WARNING: non-nested CV on GridSearchCV gives OPTIMISTICALLY BIASED scores
```

### Spatial Block Cross-Validation

```python
from sklearn.model_selection import KFold
import numpy as np

def spatial_block_cv(coords, n_splits=5, block_size_km=10):
    """Block spatial CV: split by geographic blocks, not random points."""
    # Convert lat/lon to approximate grid blocks
    lat_block = (coords[:, 0] // (block_size_km / 111.0)).astype(int)
    lon_block = (coords[:, 1] // (block_size_km / (111.0 * np.cos(np.radians(coords[:, 0].mean()))))).astype(int)
    blocks = np.unique(list(zip(lat_block, lon_block)), axis=0)
    
    # Split blocks, not points
    kf = KFold(n_splits=n_splits, shuffle=True, random_state=42)
    splits = []
    for train_blocks_idx, test_blocks_idx in kf.split(blocks):
        train_mask = np.zeros(len(coords), dtype=bool)
        test_mask = np.zeros(len(coords), dtype=bool)
        for bi in train_blocks_idx:
            train_mask |= ((lat_block == blocks[bi][0]) & (lon_block == blocks[bi][1]))
        for bi in test_blocks_idx:
            test_mask |= ((lat_block == blocks[bi][0]) & (lon_block == blocks[bi][1]))
        splits.append((train_mask, test_mask))
    return splits
```

## Reporting Standards

```markdown
## Cross-Validation Report
- Strategy: Stratified 5-fold, repeated 3x
- Total folds: 15
- Metric: F1 (macro-averaged)
- Mean ± SD: 0.823 ± 0.014
- Range: [0.805, 0.842]
- Per-fold: [0.841, 0.842, 0.823, 0.812, 0.805, ...]

### Diagnostics
- Fold consistency: SD / mean = 1.7% (< 5% = consistent)
- Per-class metrics: class_3 F1 = 0.61 ± 0.08 (high variance — investigate)
- Train/test gap: train F1 = 0.91, test F1 = 0.82 (gap = 9pp — moderate overfit)
```

## Common Mistakes

| Mistake | Consequence | Fix |
|-------|-------|-------|
| CV on preprocessed data | Information leakage from all folds | Preprocess inside CV loop |
| CV before feature selection | Selected features use all data | Feature selection inside CV loop |
| Reporting CV mean only | Hides unstable performance | Always report ± SD and range |
| k-fold on time series | Future leaks into training | TimeSeriesSplit |
| 10-fold CV on 50 examples | Folds too small, high variance | LOO or repeated 5-fold |
| Non-nested CV for tuning | Optimistic bias in reported score | Nested CV |

## Quality Gate

- CV strategy matches data structure (temporal, grouped, spatial, hierarchical).
- No information leakage across folds (preprocessing, feature selection inside loop).
- Per-fold metrics reported, not just mean.
- Train/test gap reported per fold.
- Nested CV used when hyperparameters are tuned.
- Minimum 5 folds for reliable variance estimates.
