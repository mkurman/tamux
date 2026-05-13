---
name: label-quality-audit
description: Audit label quality using confident learning (Northcutt et al.), cross-validation noise detection, and per-class error analysis. Identifies mislabeled examples for review.
tags: [label-quality, confident-learning, noise-detection, data-cleaning, dataset-curation, ml-ops]
---

# Label Quality Audit

## Overview

Label noise is the most insidious data quality problem — it's invisible until the model learns the wrong thing. Confident learning (Northcutt et al., 2021) identifies likely mislabeled examples using out-of-sample predicted probabilities.

## When to Use

Use when: training data labels come from crowd workers, automated systems, or weak supervision. Do not use on expert-validated reference data unless auditing for drift.

## Confident Learning Pipeline

```python
import numpy as np
from sklearn.model_selection import cross_val_predict
from sklearn.ensemble import RandomForestClassifier

def confident_learning_audit(X, y, n_folds=5):
    """
    Returns indices of likely mislabeled examples.
    Based on Northcutt et al. "Confident Learning: Estimating
    Uncertainty in Dataset Labels" (JMLR 2021).
    """
    n_classes = len(np.unique(y))
    
    # 1. Out-of-sample predicted probabilities
    proba = cross_val_predict(
        RandomForestClassifier(n_estimators=100, random_state=42),
        X, y, cv=n_folds, method="predict_proba"
    )
    
    # 2. Compute confident joint
    # Estimated joint distribution of noisy labels × true labels
    confident_joint = np.zeros((n_classes, n_classes))
    for i in range(len(y)):
        true_class = y[i]
        pred_class = np.argmax(proba[i])
        confidence = proba[i][pred_class]
        
        # Count if predicted class has confidence above per-class threshold
        class_threshold = np.percentile(proba[:, pred_class], 70)
        if confidence > class_threshold:
            confident_joint[true_class][pred_class] += 1
    
    # 3. Find label issues: examples where predicted ≠ given AND confident
    issues = []
    per_class_thresholds = {
        k: np.percentile(proba[:, k], 70) for k in range(n_classes)
    }
    
    for i in range(len(y)):
        pred_class = np.argmax(proba[i])
        if (pred_class != y[i] and 
            proba[i][pred_class] > per_class_thresholds[pred_class]):
            issues.append(i)
    
    # 4. Per-class noise estimates
    noise_rates = {}
    for k in range(n_classes):
        n_in_class = np.sum(y == k)
        n_noisy = np.sum((np.array(issues) != y[np.array(issues)]) & 
                         (y[np.array(issues) == k]))
        noise_rates[k] = n_noisy / n_in_class if n_in_class > 0 else 0
    
    return {
        "issue_indices": issues,
        "n_issues": len(issues),
        "issue_fraction": len(issues) / len(y),
        "noise_rates": noise_rates,
        "confident_joint": confident_joint,
    }
```

## Per-Class Analysis

| Class | Total | Mislabeled | Noise Rate | Action |
|------|-------|-------|-------|-------|
| High noise class | N | M | > 0.10 | Review annotation guidelines |
| Medium noise | N | M | 0.05-0.10 | Spot-check 50 examples |
| Low noise | N | M | < 0.05 | OK |

## What to Do with Detected Issues

1. **Never auto-correct** based on model predictions — that reinforces model bias.
2. Flag for human review. If impossible, remove from training (not from test).
3. Re-annotate a stratified sample to estimate true noise rate.
4. If noise rate > 20%, consider re-annotation rather than cleanup.
