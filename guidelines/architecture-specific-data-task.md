---
name: architecture-specific-data-task
description: Design data optimized for specific neural architectures — transformer sequence length distribution, CNN spatial locality, sparse attention patterns, memory-augmented network patterns, and architecture-appropriate data formats.
recommended_skills: [embedding-analysis, dataset-splitting, scaling-law-data-task]
recommended_guidelines: [specialized-training-data-task, data-compression-learning-task]
---

## Overview

Different architectures learn differently from the same data. Transformers prefer certain sequence distributions. CNNs benefit from spatial locality. Sparse attention systems need genuinely sparse patterns. This guideline covers architecture-aware data design.

## Transformer-Optimized Data

```python
def audit_transformer_data_fitness(dataset_texts, model_max_length=4096):
    lengths = [len(t.split()) for t in dataset_texts]
    
    # Ideal: most examples near max_length (efficient padding)
    efficiency = np.mean(lengths) / model_max_length
    wasted_compute = np.mean([(model_max_length - l) / model_max_length for l in lengths if l < model_max_length])
    
    return {"mean_length": np.mean(lengths), "median_length": np.median(lengths),
            "efficiency": efficiency, "wasted_compute_pct": wasted_compute * 100,
            "too_short": np.mean([l < 100 for l in lengths]),  # >20% short = inefficient
            "truncated": np.mean([l >= model_max_length for l in lengths]),  # >30% truncated = lost info
            "recommendation": "PACK_SEQUENCES" if efficiency < 0.5 else "GOOD"}
```

## CNN-Optimized Data

```python
def audit_cnn_data_fitness(image_dataset):
    """CNN-friendly: consistent resolution, centered objects, moderate detail."""
    issues = []
    resolutions = [img.size for img in image_dataset]
    if np.std([r[0] for r in resolutions]) / np.mean([r[0] for r in resolutions]) > 0.2:
        issues.append("inconsistent_resolution")
    
    # Check aspect ratios — extreme ratios waste compute
    aspect_ratios = [img.size[0] / max(img.size[1], 1) for img in image_dataset]
    if np.mean([abs(ar - 1.0) > 0.5 for ar in aspect_ratios]) > 0.3:
        issues.append("extreme_aspect_ratios")
    
    return {"issues": issues, "ready": len(issues) == 0,
            "recommended_input_size": _recommend_size(resolutions)}
```

## Sparse Attention Data

```python
def audit_sparse_attention_fitness(dataset_texts, model):
    """Does data naturally have sparse attention patterns?"""
    # Measure attention entropy — low entropy = genuinely sparse
    sparsities = []
    for text in dataset_texts[:100]:
        attn = model.get_attention(text)
        entropy = -np.sum(attn * np.log(attn + 1e-10), axis=-1).mean()
        sparsities.append(entropy)
    
    mean_entropy = np.mean(sparsities)
    return {"mean_attention_entropy": float(mean_entropy),
            "naturally_sparse": mean_entropy < 2.0,
            "sparse_attention_beneficial": mean_entropy < 1.5,
            "recommendation": "USE_SPARSE_ATTENTION" if mean_entropy < 1.5 else "STANDARD_ATTENTION_OK"}
```

## Memory-Augmented Data

```python
def audit_memory_network_fitness(dataset):
    """What patterns benefit from external memory?"""
    # Long-range dependencies: does information from 500+ tokens ago matter?
    long_range_ratio = _measure_long_range_dependency(dataset)
    # Repeated patterns: would caching help?
    pattern_repetition = _measure_pattern_repetition(dataset)
    
    return {"long_range_dependency": long_range_ratio,
            "pattern_repetition": pattern_repetition,
            "memory_beneficial": long_range_ratio > 0.3 or pattern_repetition > 0.5,
            "recommendation": "USE_MEMORY_NETWORK" if long_range_ratio > 0.3 else "STANDARD_OK"}
```

## Quality Gate

- Transformer data efficiency > 50% (mean length / max length).
- CNN data has consistent resolution and aspect ratios.
- Sparse attention data has low attention entropy (< 2.0).
- Memory-augmented data has genuine long-range dependencies.
