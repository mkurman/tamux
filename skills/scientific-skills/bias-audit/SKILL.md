---
name: bias-audit
description: Audit dataset bias across protected attributes — demographic parity, equalized odds, representation gaps, and intersectional bias. Reports actionable gaps with per-group metrics.
tags: [bias-audit, fairness, demographics, representation, equalized-odds, dataset-curation]
---

# Bias Audit

## Overview

Bias in training data produces biased models, full stop. This audit measures representation, outcome disparities, and intersectional gaps so you can fix problems before training.

## When to Use

Use when: building models that make decisions about people, deploying in regulated domains, or when protected attributes (gender, race, age, etc.) are available.

## Core Metrics

### Representation Audit

```python
import pandas as pd
import numpy as np

def representation_audit(df, protected_cols, population_benchmark=None):
    """Check if dataset representation matches population."""
    n = len(df)
    results = {}
    
    for col in protected_cols:
        dist = df[col].value_counts(normalize=True).to_dict()
        results[col] = {
            "distribution": dist,
            "n_groups": len(dist),
            "min_group_pct": min(dist.values()),
            "max_group_pct": max(dist.values()),
            "imbalance_ratio": max(dist.values()) / (min(dist.values()) + 1e-10),
        }
    
    # Intersectional audit
    if len(protected_cols) >= 2:
        intersectional = df.groupby(protected_cols).size() / n
        min_intersection = intersectional.min()
        results["intersectional"] = {
            "n_intersections": len(intersectional),
            "min_pct": min_intersection,
            "empty_groups": (intersectional == 0).sum(),
        }
    
    return results
```

### Outcome Parity Audit

```python
def outcome_audit(df, label_col, protected_col, positive_label=1):
    """Check if outcomes differ across protected groups."""
    groups = df.groupby(protected_col)
    
    metrics = {}
    for group, data in groups:
        metrics[group] = {
            "n": len(data),
            "positive_rate": (data[label_col] == positive_label).mean(),
            "label_distribution": data[label_col].value_counts().to_dict(),
        }
    
    # Disparity metrics
    pos_rates = [m["positive_rate"] for m in metrics.values()]
    disparity = max(pos_rates) - min(pos_rates)
    
    return {
        "per_group": metrics,
        "max_disparity": disparity,
        "disparity_ratio": max(pos_rates) / (min(pos_rates) + 1e-10),
    }
```

## Thresholds That Matter

| Metric | Green | Yellow | Red |
|------|-------|-------|-------|
| Group size ratio (max/min) | < 3:1 | 3:1-10:1 | > 10:1 |
| Outcome disparity | < 5pp | 5-15pp | > 15pp |
| Min intersection group | > 1% | 0.1-1% | < 0.1% |

## Remediation Plan

1. **Under-represented groups**: Oversample, collect more data, or use synthetic augmentation.
2. **Outcome disparity**: Check if label quality differs across groups. Check if label definition is biased.
3. **Intersectional gaps**: Report even if you can't fix — don't hide zero-count cells.
4. **Document**: What you measured, what you found, what you did about it. Transparency is the minimum bar.
