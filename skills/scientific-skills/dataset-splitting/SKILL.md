---
name: dataset-splitting
description: Create reproducible train/validation/test splits with stratification, leakage prevention, and distribution validation. Covers random, stratified, grouped, and time-series split strategies.
tags: [data-splitting, train-test-split, cross-validation, stratification, leakage-prevention, dataset-curation]
---

# Dataset Splitting

## Overview

Train/validation/test splits are the single most important guardrail against overfitting and data leakage. A bad split invalidates everything downstream. Split once, lock the split, and never let test data influence any decision.

## When to Use

Use this skill when:
- Preparing data for supervised learning.
- Designing evaluation protocols for model comparison.
- Setting up cross-validation folds.
- Ensuring no data leakage between splits.

Do not use for:
- Unsupervised learning evaluation — different rules apply.
- Time-series forecasting with backtesting — use `darts` or `prophet` skills.
- Data cleaning — use `dataset-cleaning` before splitting.

## Split Strategies

### 1. Standard Random Split (IID)

```python
from sklearn.model_selection import train_test_split

# Single split (fixed seed, stratified)
X_train, X_test, y_train, y_test = train_test_split(
    X, y,
    test_size=0.2,
    stratify=y,
    random_state=42  # NEVER change this casually
)

# Train / validation / test (two-step)
X_temp, X_test, y_temp, y_test = train_test_split(
    X, y, test_size=0.15, stratify=y, random_state=42
)
X_train, X_val, y_train, y_val = train_test_split(
    X_temp, y_temp, test_size=0.1765, stratify=y_temp, random_state=42
)
# Result: 70% train, 15% val, 15% test
```

### 2. Stratified Split (Class Imbalance)

```python
# Stratify on target AND protected attributes
X_train, X_test, y_train, y_test = train_test_split(
    X, y,
    test_size=0.2,
    stratify=df[['target', 'gender', 'region']].apply(tuple, axis=1),
    random_state=42
)
```

### 3. Group-Level Split (No Cross-Contamination)

```python
from sklearn.model_selection import GroupShuffleSplit

# When rows from the same group MUST stay together
# Example: multiple samples per patient
gss = GroupShuffleSplit(n_splits=1, test_size=0.2, random_state=42)
train_idx, test_idx = next(gss.split(X, y, groups=df['patient_id']))
X_train, X_test = X.iloc[train_idx], X.iloc[test_idx]
```

### 4. Time-Series Split (No Future Leakage)

```python
# Chronological split — NEVER shuffle time data
df = df.sort_values('timestamp')
split_idx = int(len(df) * 0.8)
train = df.iloc[:split_idx]
test = df.iloc[split_idx:]

# For multiple backtest windows:
from sklearn.model_selection import TimeSeriesSplit
tscv = TimeSeriesSplit(n_splits=5)
for train_idx, test_idx in tscv.split(X):
    X_train, X_test = X.iloc[train_idx], X.iloc[test_idx]
    # Each fold uses older data for training, newer for testing
```

## Cross-Validation Setup

```python
from sklearn.model_selection import StratifiedKFold, RepeatedStratifiedKFold

# Standard 5-fold
cv = StratifiedKFold(n_splits=5, shuffle=True, random_state=42)

# Repeated for small datasets
cv = RepeatedStratifiedKFold(n_splits=5, n_repeats=3, random_state=42)
```

## Leakage Prevention Checklist

Before locking the split, verify:

1. **No ID leakage**: same entity does not appear in train and test.
2. **No temporal leakage**: all train timestamps precede all test timestamps.
3. **No target leakage**: no feature derived from test data (including imputation, scaling, encoding).
4. **No group leakage**: groups (patients, users, experiments) are fully in one split.
5. **Stratification preserved**: target distribution similar across splits.

```python
# Split distribution validation
for split_name, split_df in [('train', train_df), ('val', val_df), ('test', test_df)]:
    print(f"{split_name}: {split_df['target'].value_counts(normalize=True).to_dict()}")

# Group integrity check
train_groups = set(train_df['group_id'])
test_groups = set(test_df['group_id'])
assert len(train_groups & test_groups) == 0, "Group leakage detected!"
```

## Locking the Split

Once created, save split assignments immutably:

```python
# Add split column and save
df['split'] = 'train'
df.loc[val_idx, 'split'] = 'val'
df.loc[test_idx, 'split'] = 'test'

# Save with version
df.to_parquet('dataset_v1.0.0_with_splits.parquet', index=False)

# Save split indices for reproducibility
np.savez('split_indices_v1.0.0.npz',
         train=train_idx, val=val_idx, test=test_idx)
```

## Quality Gate

A split is valid when:
- Every row belongs to exactly one split.
- No group, entity, or timestamp leakage exists.
- Target distribution is consistent across splits (within tolerance).
- Split artifacts (indices, assignments) are saved and versioned.
- All preprocessing decisions are made using only the training set.
