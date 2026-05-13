---
name: data-compression-learning-task
description: Design data strategies for compression-aware training — information-theoretic minimality, redundancy taxonomy, compression-aware curricula, lossy compression tolerance per modality, and progressive decompression.
recommended_skills: [embedding-analysis, dataset-splitting]
recommended_guidelines: [scaling-law-data-task, synthetic-data-generation-task]
---

## Overview

Not all data is equally compressible, and not all compression is equally harmful. This guideline covers how to measure redundancy, determine what can be safely compressed, and design compression-aware training curricula.

## Phase 1: Redundancy Taxonomy

```python
REDUNDANCY_TYPES = {
    "benign_redundancy": {"description": "Same concept, different phrasing — helps generalization", "action": "KEEP"},
    "exact_duplicate": {"description": "Byte-identical examples — pure waste", "action": "REMOVE"},
    "near_duplicate": {"description": "Minimally different examples — diminishing returns", "action": "DEDUP_AT_THRESHOLD"},
    "informational_redundancy": {"description": "Different examples, same information content", "action": "COMPRESS"},
    "harmful_redundancy": {"description": "Memorized benchmark examples — contamination", "action": "REMOVE_IMMEDIATELY"},
}

def measure_redundancy_distribution(dataset_embeddings, similarity_threshold=0.95):
    from sklearn.metrics.pairwise import cosine_similarity
    sim = cosine_similarity(dataset_embeddings[:5000])
    mask = ~np.eye(len(sim), dtype=bool)
    
    exact = (sim > 0.999).sum() // 2
    near = ((sim > similarity_threshold) & (sim <= 0.999)).sum() // 2
    unique = len(dataset_embeddings) - exact - near
    
    return {"exact_dup_pct": exact / len(dataset_embeddings), 
            "near_dup_pct": near / len(dataset_embeddings),
            "effective_unique_ratio": unique / len(dataset_embeddings),
            "compression_potential": f"{(exact + near) / len(dataset_embeddings):.1%}"}
```

## Phase 2: Modality-Specific Lossy Tolerance

```python
COMPRESSION_TOLERANCE = {
    "text": {"max_compression": 0.5, "safe_methods": ["dedup", "token_pruning", "entropy_filtering"],
             "dangerous_methods": ["random_drop", "truncation"]},
    "image": {"max_compression": 0.7, "safe_methods": ["JPEG_quality>60", "resolution_downsample_2x"],
              "dangerous_methods": ["heavy_blur", "color_quantization"]},
    "audio": {"max_compression": 0.6, "safe_methods": ["MP3_128kbps", "downsample_16kHz"],
              "dangerous_methods": ["MP3_32kbps", "aggressive_noise_gate"]},
    "tabular": {"max_compression": 0.3, "safe_methods": ["column_pruning", "float32_to_float16"],
                "dangerous_methods": ["row_sampling", "value_quantization"]},
}

def find_compression_limit(model, dataset, modality, eval_task):
    """Find maximum compression before performance degrades >2%."""
    tolerance = COMPRESSION_TOLERANCE[modality]
    baseline = evaluate(model, eval_task)
    
    for rate in [0.1, 0.3, 0.5, 0.7, 0.9]:
        compressed = compress_dataset(dataset, rate, tolerance["safe_methods"])
        perf = evaluate(model_retrained_on(compressed), eval_task)
        if (baseline - perf) / baseline > 0.02:
            return {"max_compression_rate": rate - 0.1, "max_before_degradation": True}
    return {"max_compression_rate": tolerance["max_compression"], "max_before_degradation": False}
```

## Phase 3: Progressive Decompression Curricula

```python
def decompression_curriculum(dataset, compression_levels=[0.9, 0.7, 0.5, 0.3, 0.0]):
    """Train on compressed data first, progressively decompress."""
    curriculum = []
    for level in compression_levels:
        curriculum.append({"stage": len(curriculum), "compression": level,
                           "n_examples": int(len(dataset) * (1 - level)),
                           "description": f"Train at {level:.0%} compression"})
    return curriculum
```

## Quality Gate

- Redundancy distribution measured; exact duplicates removed (<1% remaining).
- Compression limit identified per modality before degradation.
- Progressive decompression curriculum documented.
- Storage savings quantified: `(1 - compressed_size / original_size) * 100%`.
