---
name: data-visualization-task
description: Visualize datasets for curation QC — distribution plots, missingness heatmaps, embedding projections, split comparison, label quality, and interactive dashboards. What to plot at every stage of the data pipeline.
recommended_skills:
  - seaborn
  - matplotlib
  - scientific-visualization
  - embedding-analysis
recommended_guidelines:
  - dataset-creation-curation-task
  - training-data-design-principles
  - data-contamination-task
---

## Overview

Numbers in a table don't reveal data problems — visualizations do. Every dataset QC pipeline needs a standard visualization protocol. This guideline defines what to plot at each stage: raw inspection, cleaning, splitting, quality audit, and release.

---

## Stage 1: Raw Data Inspection

### Distribution Overview

```python
import seaborn as sns
import matplotlib.pyplot as plt
import numpy as np

# Numeric columns: distribution grid
fig, axes = plt.subplots(3, 3, figsize=(14, 10))
for ax, col in zip(axes.flat, numeric_cols[:9]):
    sns.histplot(df[col], kde=True, ax=ax, bins=50)
    ax.set_title(f"{col}\nskew={df[col].skew():.2f}, nulls={df[col].isnull().mean():.1%}")
plt.tight_layout()
```

**What to look for**: skew, multi-modality, long tails, suspicious spikes at zero, impossible negative values.

### Categorical Balance

```python
# Bar chart with counts + percentages
fig, axes = plt.subplots(1, len(cat_cols[:4]), figsize=(16, 4))
for ax, col in zip(axes, cat_cols[:4]):
    counts = df[col].value_counts().head(20)
    sns.barplot(x=counts.values, y=counts.index, ax=ax)
    ax.set_title(f"{col} ({df[col].nunique()} unique)")
```

**What to look for**: severe imbalance (> 20:1), unexpected categories, "Other" dominating, missing encoded as "Unknown".

### Missingness Heatmap

```python
# Requires: pip install missingno
import missingno as msno

# Matrix view — rows = samples, columns = features
msno.matrix(df.sample(500), figsize=(14, 6))
plt.title("Missing Data Pattern — white = missing")

# Correlation of missingness
msno.heatmap(df)
```

**What to look for**: columns that are always missing together (suggests data source issue), rows with systematic missingness (entire modality missing), missingness correlated with a label (bias).

---

## Stage 2: Post-Cleaning Verification

### Before/After Distributions

```python
# Overlay before and after histograms
fig, axes = plt.subplots(1, 3, figsize=(16, 4))
for ax, col in zip(axes, ["age", "income", "score"]):
    sns.kdeplot(df_raw[col].dropna(), ax=ax, label="Before", linewidth=2, color="red", alpha=0.5)
    sns.kdeplot(df_clean[col].dropna(), ax=ax, label="After", linewidth=2, color="blue")
    ax.set_title(f"{col}: {len(df_raw)-len(df_clean)} rows removed")
    ax.legend()
```

### Deduplication Audit

```python
# Shows how many duplicates were removed
fig, ax = plt.subplots(figsize=(6, 4))
categories = ["Kept", "Exact Dups", "Near Dups", "Other Removed"]
counts = [n_kept, n_exact, n_near, n_removed - n_exact - n_near]
ax.pie(counts, labels=categories, autopct="%1.1f%%", 
       colors=["#2ecc71", "#e74c3c", "#f39c12", "#95a5a6"])
ax.set_title(f"Dataset Reduction: {n_raw:,} → {n_clean:,}")
```

---

## Stage 3: Split Validation

### Distribution Comparison Across Splits

```python
# KDE overlay for train/val/test
fig, axes = plt.subplots(2, 4, figsize=(18, 8))
split_colors = {"train": "#3498db", "val": "#2ecc71", "test": "#e74c3c"}

for ax, col in zip(axes.flat, key_columns[:8]):
    for split_name, split_df in [("train", train), ("val", val), ("test", test)]:
        sns.kdeplot(split_df[col].dropna(), ax=ax, 
                    label=split_name, color=split_colors[split_name], linewidth=2)
    ax.set_title(col)
    if ax == axes.flat[0]:
        ax.legend(fontsize=8)
```

**What to look for**: any split with a visibly different distribution — indication of data leakage or bad split.

### Target Distribution by Split

```python
fig, ax = plt.subplots(figsize=(10, 4))
for split_name, split_df in [("train", train), ("val", val), ("test", test)]:
    dist = split_df[target_col].value_counts(normalize=True).sort_index()
    ax.plot(dist.index, dist.values, "o-", label=split_name, linewidth=2, markersize=8)
ax.set_xlabel(target_col)
ax.set_ylabel("Proportion")
ax.legend()
ax.set_title("Target distribution across splits — should be nearly identical")
```

---

## Stage 4: Embedding Space Diagnostics

### UMAP / t-SNE Projection

```python
from umap import UMAP
import matplotlib.pyplot as plt

def plot_embedding_space(embeddings, labels=None, splits=None, 
                         n_neighbors=15, min_dist=0.1, figsize=(10, 8)):
    reducer = UMAP(n_neighbors=n_neighbors, min_dist=min_dist, random_state=42)
    projection = reducer.fit_transform(embeddings)
    
    fig, axes = plt.subplots(1, 2, figsize=(18, 7)) if splits is not None else plt.subplots(1, 1, figsize=(10, 8))
    if splits is None:
        axes = [axes]
    
    # Color by label
    scatter = axes[0].scatter(projection[:, 0], projection[:, 1], 
                              c=labels, cmap="tab20", s=2, alpha=0.6)
    axes[0].set_title("Embedding Space — colored by label")
    axes[0].set_xlabel("UMAP 1"); axes[0].set_ylabel("UMAP 2")
    plt.colorbar(scatter, ax=axes[0])
    
    # Color by split (if available)
    if splits is not None:
        for split_name, color in [("train", "#3498db"), ("val", "#2ecc71"), ("test", "#e74c3c")]:
            mask = splits == split_name
            axes[1].scatter(projection[mask, 0], projection[mask, 1], 
                           c=color, label=split_name, s=2, alpha=0.4)
        axes[1].set_title("Embedding Space — colored by split")
        axes[1].legend()
    
    return projection
```

**What to look for**: 
- Clusters not separated → model will struggle.
- One split dominating a region → split bias.
- Outliers far from everything → annotation errors, noise.
- Training examples in test-only clusters → data leakage.

### Embedding Density by Class

```python
# Per-class embedding density — identifies poorly covered classes
from scipy.stats import gaussian_kde

fig, axes = plt.subplots(2, 4, figsize=(18, 8))
for ax, cls in zip(axes.flat, top_classes[:8]):
    mask = labels == cls
    if mask.sum() < 10:
        continue
    xy = projection[mask].T
    z = gaussian_kde(xy)(xy)
    ax.scatter(projection[mask, 0], projection[mask, 1], c=z, s=3, cmap="viridis")
    ax.set_title(f"Class {cls} (n={mask.sum():,})")
```

---

## Stage 5: Label Quality Visualization

### Confusion Between Labels

```python
# Confident learning joint matrix
fig, ax = plt.subplots(figsize=(8, 6))
sns.heatmap(confident_joint, annot=True, fmt=".0f", cmap="YlOrRd", ax=ax,
            xticklabels=class_names, yticklabels=class_names)
ax.set_xlabel("Predicted Label"); ax.set_ylabel("Given Label")
ax.set_title("Confident Joint — off-diagonal = likely mislabeled")
```

### Per-Example Uncertainty

```python
# Histogram of prediction confidence
fig, ax = plt.subplots(figsize=(8, 4))
sns.histplot(max_proba, bins=50, ax=ax)
ax.axvline(x=0.5, color="red", linestyle="--", label="50% threshold")
ax.axvline(x=0.8, color="orange", linestyle="--", label="80% threshold")
ax.set_xlabel("Model Confidence"); ax.set_title("Prediction Confidence Distribution")
ax.legend()
# Many examples with low confidence → many ambiguous labels
```

---

## Stage 6: Interactive Exploration

For large datasets where static plots miss patterns:

```python
# Bokeh interactive scatter
from bokeh.plotting import figure, show, output_notebook
from bokeh.models import HoverTool
output_notebook()

p = figure(width=800, height=600, title="Interactive Embedding Explorer",
           tools="pan,wheel_zoom,box_select,reset")
p.scatter(x=projection[:, 0], y=projection[:, 1], 
          source=ColumnDataSource({
              "x": projection[:, 0], "y": projection[:, 1],
              "text": texts, "label": labels, "id": ids,
          }), size=3, alpha=0.5)
p.add_tools(HoverTool(tooltips=[("ID", "@id"), ("Label", "@label"), ("Text", "@text")]))
show(p)
```

```python
# Plotly parallel coordinates for multi-dimensional QC
import plotly.express as px

fig = px.parallel_coordinates(
    df.sample(1000), 
    color=target_col,
    dimensions=key_numeric_cols[:8],
    color_continuous_scale=px.colors.diverging.Tealrose
)
fig.show()
```

---

## Visualization Protocol Per Dataset Stage

| Stage | Must Plot | Optional |
|------|-------|-------|
| **Raw** | Distribution grid (numeric), frequency bars (cat), missingness heatmap | Box plots by group, pairwise scatter matrix |
| **Cleaned** | Before/after KDE overlay, dedup pie chart | Outlier flag distribution |
| **Split** | KDE overlay by split, target distribution by split, split proportions bar | Per-split class balance |
| **Embedding** | UMAP colored by label + split, density per class | t-SNE, PCA scree plot, trustworthiness score |
| **Labels** | Confident joint heatmap, confidence histogram | Per-class noise rate bar chart |
| **Release** | All of the above, saved as PNG + HTML dashboard | Interactive explorer |

## Quality Gate

- Every numeric column has a distribution plot generated.
- Train/val/test distributions visibly overlap (no split looks different).
- Embedding space shows reasonable cluster structure.
- Label quality visualization shows < 10% suspected mislabeling.
- All plots saved alongside the dataset release.
- At least one interactive visualization for spot-checking individual examples.
