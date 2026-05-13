---
name: dataset-cleaning
description: Clean, normalize, and prepare raw datasets for analysis or ML. Covers missing value handling, deduplication, outlier treatment, type normalization, categorical encoding, and transformation logging.
tags: [data-cleaning, data-preparation, data-quality, imputation, deduplication, normalization, dataset-curation]
---

# Dataset Cleaning

## Overview

Dataset cleaning transforms raw, messy data into a consistent, analysis-ready form. Every operation must be documented: what was changed, why, and how many rows/columns were affected. This is not a one-shot script — it's a reproducible pipeline.

## When to Use

Use this skill when:
- Raw data has missing values, duplicates, inconsistent types, or outliers.
- Data comes from multiple sources with conflicting formats or encodings.
- You need a documented cleaning pipeline, not silent `dropna()`.
- The dataset will be used for modeling, analysis, or sharing.

Do not use for:
- Exploratory analysis without transformation — use `exploratory-data-analysis` first.
- Splitting strategies — use `dataset-splitting`.
- Version tracking — use `dataset-versioning`.

## Cleaning Workflow

### 1. Missing Value Strategy

Choose and document ONE strategy per column:

| Strategy | When to use | Risk |
||-------------|------|
| **Drop rows** | < 5% missing, rows are independent | Loss of rare cases |
| **Drop column** | > 40% missing and not critical | Loss of signal |
| **Mean/median imputation** | Continuous, symmetric distribution | Underestimates variance |
| **Mode imputation** | Categorical, dominant class clear | Over-represents majority |
| **Constant fill** | Domain-knowledge default exists | May introduce bias |
| **Model-based imputation** | High missingness, strong predictors | Leakage if not careful |
| **Indicator flag** | Missingness itself is informative | Adds dimensionality |

```python
import pandas as pd
import numpy as np

# NEVER do this silently:
# df.dropna(inplace=True)

# Instead — audit first:
missing = df.isnull().sum()
missing_pct = df.isnull().mean() * 100
print(missing_pct[missing_pct > 0].sort_values(ascending=False))

# Documented imputation with audit trail:
audit = {}
mask = df['age'].isnull()
audit['age_imputed_count'] = mask.sum()
df.loc[mask, 'age'] = df['age'].median()
df['age_imputed'] = mask.astype(int)  # flag for downstream awareness
```

### 2. Deduplication

```python
# Define identity columns explicitly before deduping
identity_cols = ['user_id', 'timestamp']
n_before = len(df)
df = df.drop_duplicates(subset=identity_cols, keep='first')
audit['duplicates_removed'] = n_before - len(df)

# Near-duplicate detection (fuzzy):
from difflib import SequenceMatcher
# Use for text fields where exact match is too strict
```

### 3. Type Normalization

```python
# Date/time normalization
df['created_at'] = pd.to_datetime(df['created_at'], utc=True, errors='coerce')

# String normalization
df['category'] = df['category'].str.strip().str.lower().str.replace(r'\s+', '_', regex=True)

# Numeric coercion with audit
original = df['price'].copy()
df['price'] = pd.to_numeric(df['price'], errors='coerce')
audit['price_coerced_nulls'] = df['price'].isnull().sum() - original.isnull().sum()
```

### 4. Outlier Handling

```python
# Domain-based capping, not arbitrary percentiles
# Example: age cannot be < 0 or > 120
df.loc[df['age'] < 0, 'age'] = np.nan
df.loc[df['age'] > 120, 'age'] = 120  # cap, don't drop

# For statistical outliers — use IQR with domain validation:
Q1 = df['value'].quantile(0.25)
Q3 = df['value'].quantile(0.75)
IQR = Q3 - Q1
lower = Q1 - 3.0 * IQR  # wider fence for less aggressive removal
upper = Q3 + 3.0 * IQR
outliers = (df['value'] < lower) | (df['value'] > upper)
audit['outliers_flagged'] = outliers.sum()
df['outlier_flag'] = outliers.astype(int)
```

### 5. Categorical Encoding

```python
# One-hot for < 20 categories, label encoding otherwise
n_unique = df['category'].nunique()
if n_unique <= 20:
    df = pd.get_dummies(df, columns=['category'], drop_first=True)
else:
    df['category_code'] = df['category'].astype('category').cat.codes
```

## Audit Trail

Always produce a structured audit log:

```python
audit = {
    'rows_before': n_before,
    'rows_after': n_after,
    'columns_before': cols_before,
    'columns_after': cols_after,
    'missing_handled': {col: strategy for col, strategy in missing_strategies.items()},
    'duplicates_removed': dupes,
    'outliers_flagged': outliers,
    'type_coercions': type_changes,
}
```

Save `audit` alongside the cleaned dataset as `cleaning_audit.json`.

## Quality Check

After cleaning, verify:
- No column has > 5% missingness (unless documented as acceptable).
- All dtypes match the specification.
- No duplicate identity rows exist.
- Categorical columns have consistent, normalized values.
- The audit log is complete and saved with the dataset.


## Advanced Techniques (2025-2026)

For datasets beyond simple tabular cleaning, combine with:

- **Semantic deduplication** — `embedding-analysis` skill for meaning-based near-duplicate removal at scale (NeMo Curator SemDedup, LSHBloom).
- **Perplexity-based filtering** — `embedding-analysis` skill for GRAPE-style quality scoring with a reference language model.
- **LLM-assisted quality scoring** — `llm-assisted-curation` skill for clarity/correctness/usefulness scoring per example.
- **Streaming at scale** — `hf-datasets` skill for Arrow-backed streaming when data exceeds RAM.
- **Distribution validation** — `embedding-analysis` skill for JS divergence and Wasserstein distance between splits.

These are referenced in the parent guideline `dataset-creation-curation-task` and should be applied after standard cleaning when dataset size or quality demands warrant.
