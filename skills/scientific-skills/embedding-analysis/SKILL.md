---
name: embedding-analysis
description: Compute and analyze embeddings for dataset quality, distribution comparison, semantic deduplication, diversity measurement, and similarity-based filtering. Covers sentence-transformers, embedding space diagnostics, and 2025-2026 literature techniques (NeMo Curator semantic dedup, embedding similarity metrics for data selection, DataRater-style quality scoring).
tags: [embeddings, semantic-dedup, distribution-analysis, sentence-transformers, data-quality, metric-learning, dataset-curation]
---

# Embedding Analysis for Dataset Curation

## Overview

Embeddings turn unstructured text/images into a vector space where quality, diversity, and redundancy become measurable. This skill covers embedding-based dataset curation using sentence-transformers, distribution comparison metrics, semantic deduplication, and quality scoring techniques from the 2025-2026 literature.

## When to Use

Use this skill when:
- Detecting near-duplicate or semantically redundant examples.
- Comparing embedding distributions across dataset splits (train vs. val vs. test).
- Measuring dataset diversity or coverage in embedding space.
- Filtering examples based on embedding distance to a "gold" reference set.
- Applying NeMo Curator-style semantic dedup at scale.
- Computing DataRater-style quality signals from embedding neighborhoods.

Do not use for:
- Raw text deduplication (exact match) — use simple hashing.
- Model training — use `transformers` or `trl` skills.
- Vector database persistence — use `chromadb`, `milvus`, or `qdrant` skills.

## Installation

```bash
uv pip install sentence-transformers umap-learn scikit-learn numpy
```

## Core Workflow

### 1. Generate Embeddings

```python
from sentence_transformers import SentenceTransformer
import numpy as np

# Choose model based on domain
model = SentenceTransformer("all-MiniLM-L6-v2")  # fast, general
# model = SentenceTransformer("all-mpnet-base-v2")  # higher quality
# model = SentenceTransformer("BAAI/bge-large-en-v1.5")  # retrieval-optimized
# model = SentenceTransformer("intfloat/multilingual-e5-large")  # multilingual

# Batched encoding (memory-efficient)
embeddings = model.encode(
    texts,
    batch_size=64,
    show_progress_bar=True,
    normalize_embeddings=True,  # cosine similarity via dot product
    convert_to_numpy=True,
)
```

### 2. Semantic Deduplication

Semantic dedup removes examples that are meaning-equivalent but not text-identical. From NeMo Curator (NVIDIA, 2024-2025) and academic literature.

#### Clustering-Based Approach (NeMo Curator SemDedup)

```python
from sklearn.cluster import MiniBatchKMeans
from sklearn.metrics.pairwise import cosine_similarity

def semantic_dedup(embeddings, threshold=0.95, n_clusters=100):
    """
    Cluster embeddings, then remove near-duplicates within each cluster.
    threshold: cosine similarity above which two examples are duplicates.
    Returns: boolean mask (True = keep, False = duplicate).
    """
    n = len(embeddings)
    keep = np.ones(n, dtype=bool)

    # Coarse clustering for efficiency (O(n * k) instead of O(n^2))
    k = min(n_clusters, n // 10)
    clusters = MiniBatchKMeans(n_clusters=k, random_state=42, batch_size=1024).fit_predict(embeddings)

    for c in range(k):
        idx = np.where(clusters == c)[0]
        if len(idx) < 2:
            continue
        sim = cosine_similarity(embeddings[idx])
        # Mark duplicates (lower-index kept, higher-index removed)
        for i in range(len(idx)):
            if not keep[idx[i]]:
                continue
            dupes = np.where(sim[i, i+1:] > threshold)[0]
            for d in dupes:
                keep[idx[i + 1 + d]] = False

    return keep
```

#### Connected Components Approach (LSHBloom, 2025)

```python
# For internet-scale dedup (>100M examples):
# Use LSH for approximate nearest neighbor + union-find
from sklearn.neighbors import NearestNeighbors

def lsh_semantic_dedup(embeddings, threshold=0.90, n_neighbors=10):
    """Finds clusters of semantically similar examples via NN graph."""
    nn = NearestNeighbors(n_neighbors=n_neighbors, metric="cosine", n_jobs=-1)
    nn.fit(embeddings)
    distances, indices = nn.kneighbors(embeddings)

    # Union-find to merge connected components
    parent = np.arange(len(embeddings))
    def find(x):
        while parent[x] != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x
    def union(a, b):
        parent[find(a)] = find(b)

    for i in range(len(embeddings)):
        for j, d in zip(indices[i], distances[i]):
            if i != j and (1 - d) > threshold:  # cosine dist → similarity
                union(i, j)

    # Keep one per component
    components = {}
    for i in range(len(embeddings)):
        root = find(i)
        if root not in components:
            components[root] = i

    return list(components.values())  # indices to keep
```

### 3. Distribution Comparison Across Splits

Compare embedding distributions between train/val/test to detect drift or bias.

```python
from scipy.spatial.distance import jensenshannon
from scipy.stats import wasserstein_distance

def embedding_distribution_shift(train_emb, test_emb, n_bins=50):
    """Quantify shift between train and test embedding distributions."""

    # Project to 1D for distribution comparison
    from sklearn.decomposition import PCA
    pca = PCA(n_components=1, random_state=42).fit(
        np.concatenate([train_emb, test_emb])
    )
    train_1d = pca.transform(train_emb).ravel()
    test_1d = pca.transform(test_emb).ravel()

    # Histogram-based metrics
    hist_range = (min(train_1d.min(), test_1d.min()),
                   max(train_1d.max(), test_1d.max()))
    train_hist, _ = np.histogram(train_1d, bins=n_bins, range=hist_range, density=True)
    test_hist, _ = np.histogram(test_1d, bins=n_bins, range=hist_range, density=True)

    # Jensen-Shannon divergence (0 = identical, 1 = maximally different)
    js_div = jensenshannon(train_hist + 1e-10, test_hist + 1e-10)

    # Wasserstein (Earth Mover's) distance
    w_dist = wasserstein_distance(train_1d, test_1d)

    return {"js_divergence": js_div, "wasserstein": w_dist}

def per_dimension_shift(train_emb, test_emb):
    """Per-dimension mean shift — identifies which semantic axes drifted."""
    train_mean = train_emb.mean(axis=0)
    test_mean = test_emb.mean(axis=0)
    cos_sim = np.dot(train_mean, test_mean) / (
        np.linalg.norm(train_mean) * np.linalg.norm(test_mean)
    )
    max_dim_shift = np.argmax(np.abs(train_mean - test_mean))
    return {"mean_cosine": cos_sim, "max_shift_dim": int(max_dim_shift)}
```

### 4. Embedding Quality Scoring (DataRater-Inspired, 2025)

DataRater (Calian et al., NeurIPS 2025) meta-learns a quality function over embedding neighborhoods. Below is a practical approximation.

```python
def embedding_quality_score(embeddings, k=20):
    """
    Score each example by embedding neighborhood coherence.
    Low coherence → outlier / potential noise.
    High coherence → in-distribution, likely high quality.
    """
    nn = NearestNeighbors(n_neighbors=k + 1, metric="cosine")
    nn.fit(embeddings)
    distances, _ = nn.kneighbors(embeddings)

    # Exclude self-distance (index 0)
    mean_dist = distances[:, 1:].mean(axis=1)

    # Normalize to [0, 1] — lower distance = higher quality
    scores = 1 - (mean_dist - mean_dist.min()) / (mean_dist.max() - mean_dist.min() + 1e-10)
    return scores
```

### 5. Diversity Measurement

```python
def embedding_diversity(embeddings, n_clusters=50):
    """
    Measure dataset diversity via cluster coverage.
    Returns: coverage ratio and entropy of cluster assignments.
    """
    k = min(n_clusters, len(embeddings) // 10)
    clusters = MiniBatchKMeans(n_clusters=k, random_state=42, batch_size=1024).fit_predict(embeddings)

    # Cluster coverage: fraction of clusters that are non-empty
    unique, counts = np.unique(clusters, return_counts=True)
    coverage = len(unique) / k

    # Entropy: higher = more uniform distribution across clusters
    probs = counts / counts.sum()
    entropy = -np.sum(probs * np.log(probs + 1e-10)) / np.log(k)  # normalized

    return {"coverage": coverage, "normalized_entropy": entropy}

def embedding_redundancy_score(embeddings, sample_size=5000):
    """
    Estimate redundancy: average pairwise cosine similarity.
    High score → dataset is homogeneous; needs diversification.
    """
    idx = np.random.choice(len(embeddings), min(sample_size, len(embeddings)), replace=False)
    sample = embeddings[idx]
    sim = cosine_similarity(sample)
    # Exclude diagonal
    mask = ~np.eye(len(sample), dtype=bool)
    return float(sim[mask].mean())
```

## Perplexity-Based Filtering (GRAPE Score, 2025)

Use a small reference model to score data quality by perplexity. High perplexity = unfamiliar/hard/noisy.

```python
from transformers import AutoModelForCausalLM, AutoTokenizer
import torch

def grape_score(texts, model_name="gpt2", batch_size=8):
    """
    GRAPE-style perplexity scoring.
    Lower perplexity → more in-distribution, likely higher quality.
    Very high perplexity → gibberish, noise, or out-of-domain.
    """
    tokenizer = AutoTokenizer.from_pretrained(model_name)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token
    model = AutoModelForCausalLM.from_pretrained(model_name).eval()

    scores = []
    for i in range(0, len(texts), batch_size):
        batch = texts[i:i + batch_size]
        enc = tokenizer(batch, return_tensors="pt", padding=True, truncation=True, max_length=512)
        with torch.no_grad():
            outputs = model(**enc, labels=enc["input_ids"])
            loss = outputs.loss  # average NLL across batch
            ppl = torch.exp(loss).item()
            scores.append(ppl)

    return np.array(scores)
```

## Quality Gate

An embedding-based curation pass is complete when:
- Semantic dedup threshold is justified and documented.
- Train/val/test embedding distributions are compared (JS divergence, Wasserstein).
- Diversity and redundancy metrics are computed before and after curation.
- Outlier scores are reviewed — low neighborhood coherence may indicate noise or rare-but-valuable data.
- Results are reproducible (fixed seeds, saved model versions).
