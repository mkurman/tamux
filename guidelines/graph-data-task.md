---
name: graph-data-task
description: Curate graph and network datasets for ML — node/edge dedup, degree distribution QC, connected component analysis, edge-level splitting, and negative sampling for link prediction.
recommended_skills:
  - networkx
  - dgl
  - torch-geometric
  - embedding-analysis
  - dataset-splitting
recommended_guidelines:
  - training-data-design-principles
  - data-contamination-task
  - data-visualization-task
---

## Overview

Graph data violates IID assumptions even more severely than tabular data. Nodes are interdependent by design. Splitting must respect the graph structure, not random row indices. This guideline covers the unique curation challenges.

## Phase 1: Graph Integrity Checks

```python
import networkx as nx
import numpy as np

def audit_graph(G):
    issues = []
    
    # Self-loops
    n_selfloops = nx.number_of_selfloops(G)
    if n_selfloops > 0:
        issues.append({"type": "self_loops", "n": n_selfloops, "action": "remove_or_document"})
    
    # Multi-edges
    if isinstance(G, nx.MultiGraph):
        n_multiedges = sum(1 for _ in G.edges if G.number_of_edges(*_[:2]) > 1)
        if n_multiedges > 0:
            issues.append({"type": "multi_edges", "n": n_multiedges})
    
    # Isolated nodes
    isolated = list(nx.isolates(G))
    if isolated:
        issues.append({"type": "isolated_nodes", "n": len(isolated), "action": "remove"})
    
    # Duplicate edges
    edge_set = set()
    dups = 0
    for u, v in G.edges:
        e = (min(u, v), max(u, v)) if not G.is_directed() else (u, v)
        if e in edge_set: dups += 1
        edge_set.add(e)
    if dups > 0:
        issues.append({"type": "duplicate_edges", "n": dups})
    
    return issues
```

## Phase 2: Degree Distribution

```python
def degree_analysis(G):
    degrees = [d for _, d in G.degree()]
    
    stats = {
        "n_nodes": G.number_of_nodes(),
        "n_edges": G.number_of_edges(),
        "mean_degree": np.mean(degrees),
        "median_degree": np.median(degrees),
        "max_degree": max(degrees),
        "min_degree": min(degrees),
        "std_degree": np.std(degrees),
    }
    
    # Power-law fit (common in real networks)
    log_degrees = np.log10([d + 1 for d in degrees])
    log_counts = np.log10(np.bincount([int(d) for d in degrees])[1:] + 1)
    # If linear on log-log, network is scale-free
    
    # Hub detection (> 2 SD above mean)
    hub_threshold = np.mean(degrees) + 2 * np.std(degrees)
    hubs = [n for n, d in G.degree() if d > hub_threshold]
    stats["n_hubs"] = len(hubs)
    stats["hub_edges_share"] = sum(G.degree(h) for h in hubs) / (2 * G.number_of_edges())
    
    return stats
```

## Phase 3: Connected Components

```python
def component_analysis(G):
    if G.is_directed():
        scc = list(nx.strongly_connected_components(G))
        wcc = list(nx.weakly_connected_components(G))
        return {
            "n_scc": len(scc),
            "largest_scc_share": max(len(c) for c in scc) / G.number_of_nodes(),
            "n_wcc": len(wcc),
            "largest_wcc_share": max(len(c) for c in wcc) / G.number_of_nodes(),
            "isolated_in_scc": sum(1 for c in scc if len(c) == 1),
        }
    else:
        cc = list(nx.connected_components(G))
        sizes = sorted([len(c) for c in cc], reverse=True)
        return {
            "n_components": len(cc),
            "largest_share": sizes[0] / G.number_of_nodes() if sizes else 0,
            "component_sizes": sizes[:10],
            "isolated_fraction": sum(1 for c in cc if len(c) == 1) / G.number_of_nodes(),
        }
```

## Phase 4: Graph Splitting (The Hard Part)

### Edge-Level Split (Link Prediction)

```python
import random

def edge_split(G, test_frac=0.1, val_frac=0.1, seed=42):
    rng = random.Random(seed)
    
    # Get all edges
    all_edges = list(G.edges)
    rng.shuffle(all_edges)
    
    n_test = int(len(all_edges) * test_frac)
    n_val = int(len(all_edges) * val_frac)
    
    test_edges = all_edges[:n_test]
    val_edges = all_edges[n_test:n_test + n_val]
    train_edges = all_edges[n_test + n_val:]
    
    # CRITICAL: ensure train graph is still connected
    train_G = G.edge_subgraph(train_edges).copy()
    
    # Generate negative edges (non-existent edges)
    neg_edges = _sample_negatives(G, len(test_edges) + len(val_edges), rng)
    
    return {
        "train_G": train_G,
        "val_pos": val_edges,
        "val_neg": neg_edges[:len(val_edges)],
        "test_pos": test_edges,
        "test_neg": neg_edges[len(val_edges):],
    }

def _sample_negatives(G, n, rng):
    nodes = list(G.nodes)
    existing = set(G.edges)
    if not G.is_directed():
        existing = set((min(u, v), max(u, v)) for u, v in existing)
    
    negs = set()
    while len(negs) < n:
        u, v = rng.sample(nodes, 2)
        if not G.is_directed():
            u, v = min(u, v), max(u, v)
        if (u, v) not in existing and u != v:
            negs.add((u, v))
    return list(negs)
```

### Node-Level Split (Node Classification)

```python
def node_split(G, test_frac=0.2, val_frac=0.1, seed=42):
    """Split NODES, not edges. Maintain connectivity in training."""
    rng = random.Random(seed)
    nodes = list(G.nodes)
    rng.shuffle(nodes)
    
    n_test = int(len(nodes) * test_frac)
    n_val = int(len(nodes) * val_frac)
    
    test_nodes = set(nodes[:n_test])
    val_nodes = set(nodes[n_test:n_test + n_val])
    train_nodes = set(nodes[n_test + n_val:])
    
    # Verify no edges from train to test (message-passing leakage)
    leakage = 0
    for u, v in G.edges:
        if (u in train_nodes and v in test_nodes) or (u in test_nodes and v in train_nodes):
            leakage += 1
    
    return {
        "train_nodes": train_nodes,
        "val_nodes": val_nodes,
        "test_nodes": test_nodes,
        "leakage_edges": leakage,
        "is_clean": leakage == 0,
    }
```

## Phase 5: Negative Sampling Quality

```python
def assess_negative_quality(G, pos_edges, neg_edges, embeddings=None):
    """
    Good negatives: structurally similar to positives but not real edges.
    Bad negatives: trivially distinguishable (random node pairs with no proximity).
    """
    if embeddings is not None:
        pos_scores = [np.dot(embeddings[u], embeddings[v]) for u, v in pos_edges[:1000]]
        neg_scores = [np.dot(embeddings[u], embeddings[v]) for u, v in neg_edges[:1000]]
        overlap = sum(1 for ns in neg_scores if ns > np.median(pos_scores))
    
    # Jaccard-based hardness
    pos_jaccard = []
    neg_jaccard = []
    for u, v in pos_edges[:1000]:
        nu = set(G.neighbors(u))
        nv = set(G.neighbors(v))
        pos_jaccard.append(len(nu & nv) / len(nu | nv) if len(nu | nv) > 0 else 0)
    for u, v in neg_edges[:1000]:
        nu = set(G.neighbors(u))
        nv = set(G.neighbors(v))
        neg_jaccard.append(len(nu & nv) / len(nu | nv) if len(nu | nv) > 0 else 0)
    
    return {
        "negatives_too_easy": np.mean(neg_jaccard) < 0.01,  # random unrelated pairs
        "pos_neg_overlap_jaccard": np.mean(pos_jaccard) - np.mean(neg_jaccard),
    }
```

## Quality Gate

- No isolated nodes in training graph (unless explicitly intended).
- Largest component > 80% of nodes (otherwise, analyze components separately).
- Edge split: train graph remains connected after removing test/val edges.
- Node split: ZERO edges between train and test nodes.
- Negative samples not trivially distinguishable from positives.
- Degree distribution documented; hubs identified.
