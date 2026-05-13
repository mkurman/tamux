---
name: efficient-inference-data-task
description: Curate data for efficient inference — speculative decoding calibration corpora, KV cache optimization profiling data, dynamic batching composition data, quantization calibration sets, and inference-aware training data selection. Fast inference is a data problem.
recommended_skills: [dataset-splitting, embedding-analysis, data-pipeline-monitoring-task]
recommended_guidelines: [model-compression-data-task, scaling-law-data-task, data-compression-learning-task]
---

## Overview

A model that answers correctly in 10 seconds loses to a model that answers correctly in 1 second. Inference efficiency is not just an engineering problem — it's a data problem. Speculative decoding needs calibration corpora that match deployment query distributions. KV cache optimization needs profiling data to decide what to evict. Dynamic batching needs composition data to group requests efficiently. This guideline treats every inference optimization as a data curation problem.

## Speculative Decoding Data

```python
SPECULATIVE_DECODING_DATA = {
    "draft_model_calibration": {
        "description": "Data to align draft model outputs with target model acceptance",
        "data_requirement": "query_distribution_sample — must match deployment query patterns",
        "curation": "sample from production query logs or synthetic queries matching distribution",
        "size": "10K-100K representative queries",
        "quality_check": "distribution_match_to_production — KL divergence < 0.1",
    },
    "acceptance_rate_optimization": {
        "description": "Data to tune draft model for higher acceptance rates",
        "data_requirement": "sequences_where_draft_and_target_diverge",
        "curation": "collect rejection cases — where draft was wrong, target was right",
        "size": "all_rejection_cases_from_calibration_run",
    },
    "tree_attention_calibration": {
        "description": "Data to optimize speculative tree topology",
        "data_requirement": "branching_patterns_from_deployment_queries",
        "curation": "track which speculative branches succeed/fail per position",
        "size": "statistically_significant_branching_sample",
    },
}

def curate_speculative_decoding_calibration(production_queries, target_model, draft_model, n_samples=50000):
    """Build a calibration corpus that matches production query distribution."""
    
    # Sample matching production distribution
    calibration = _distribution_matching_sample(production_queries, n_samples)
    
    # Generate draft and target outputs for alignment
    alignment_data = []
    for query in calibration:
        draft_output = draft_model.generate(query, max_tokens=5)
        target_output = target_model.generate(query, max_tokens=5)
        
        acceptance = _compute_token_acceptance(draft_output, target_output)
        
        alignment_data.append({
            "query": query,
            "draft_tokens": draft_output["tokens"],
            "target_tokens": target_output["tokens"],
            "acceptance_rate": acceptance["rate"],
            "rejection_positions": acceptance["rejection_positions"],
        })
    
    mean_acceptance = sum(d["acceptance_rate"] for d in alignment_data) / max(len(alignment_data), 1)
    
    return {
        "calibration_corpus": alignment_data,
        "corpus_size": len(alignment_data),
        "mean_acceptance_rate": mean_acceptance,
        "speculative_efficiency": mean_acceptance * draft_model.get("speedup_factor", 1),
        "sufficient": mean_acceptance >= 0.7,
        "rejection_patterns": _aggregate_rejection_patterns(alignment_data),
    }
```

## KV Cache Optimization

```python
KV_CACHE_EVICTION_STRATEGIES = {
    "sliding_window": {
        "description": "Keep last W tokens; evict older tokens",
        "data_need": "window_size_calibration — what W minimizes performance loss?",
        "calibration_data": "long_context_benchmarks at varying context lengths",
        "risk": "loses_critical_early_context — first instruction, system prompt, few-shot examples",
    },
    "attention_sink": {
        "description": "Keep first N tokens as attention sinks + last W tokens",
        "data_need": "sink_size_calibration — how many initial tokens act as sinks?",
        "calibration_data": "multi_turn_conversation_logs with attention map analysis",
        "risk": "overly_conservative — keeping too many sinks wastes cache",
    },
    "importance_scored": {
        "description": "Score each token's importance; evict lowest scores",
        "data_need": "importance_scoring_model_calibration — learn what matters",
        "calibration_data": "token_level_attention_patterns from production traffic",
        "risk": "scoring_model_quality — bad scores → bad evictions",
    },
    "heavy_hitter": {
        "description": "Identify and preserve heavy hitter tokens that dominate attention",
        "data_need": "heavy_hitter_detection_threshold_calibration",
        "calibration_data": "attention_matrix_from_diverse_queries",
        "risk": "threshold_too_high → evicts important tokens; too_low → wastes cache",
    },
}

def calibrate_kv_cache_strategy(strategy, production_traffic_sample, target_model):
    """Use production traffic data to calibrate KV cache eviction parameters."""
    
    if strategy == "sliding_window":
        # Sweep window sizes, measure performance degradation
        results = []
        for window_size in [512, 1024, 2048, 4096, 8192]:
            perf = _evaluate_with_window(target_model, production_traffic_sample, window_size)
            cache_savings = 1 - (window_size / target_model.get("max_context", 8192))
            results.append({
                "window_size": window_size,
                "performance": perf,
                "cache_savings_pct": cache_savings * 100,
                "efficiency_score": perf * cache_savings,  # optimize both
            })
        
        best = max(results, key=lambda r: r["efficiency_score"])
        return {"optimal_window": best["window_size"], "sweep_results": results}
    
    elif strategy == "attention_sink":
        # Analyze attention maps to find natural sink tokens
        attention_maps = _collect_attention_maps(production_traffic_sample, target_model)
        
        # First N positions consistently receive high attention
        sink_scores = []
        for pos in range(1, 51):
            mean_attention = attention_maps[:, :, pos].mean()
            sink_scores.append({"position": pos, "mean_attention": mean_attention})
        
        sink_threshold = _elbow_point([s["mean_attention"] for s in sink_scores])
        
        return {
            "optimal_sink_count": sink_threshold,
            "sink_attention_decay": sink_scores[:sink_threshold],
            "calibration_quality": "GOOD" if sink_threshold < 10 else "SINK_REGION_TOO_LARGE",
        }
```

## Dynamic Batching Composition

```python
BATCHING_COMPOSITION_DATA = {
    "sequence_length_distribution": {
        "description": "Distribution of input+output lengths in production",
        "data_need": "sequence_lengths_from_production_logs",
        "optimization": "group_similar_lengths — minimize padding waste",
    },
    "query_arrival_pattern": {
        "description": "Temporal pattern of query arrivals",
        "data_need": "timestamped_query_logs",
        "optimization": "dynamic_batch_accumulation — wait for optimal batch size vs latency trade-off",
    },
    "task_composition": {
        "description": "Which task types arrive together",
        "data_need": "task_labels_from_production_traffic",
        "optimization": "task_aware_batching — some tasks benefit from shared context",
    },
}

def optimize_batching_strategy(production_logs, latency_sla_ms, throughput_target):
    """Use production data to optimize batching."""
    
    seq_lengths = [log["input_tokens"] + log["output_tokens"] for log in production_logs]
    
    # Find optimal batch size that meets SLA
    optimal_config = None
    for batch_size in [1, 2, 4, 8, 16, 32, 64]:
        padding_waste = _estimate_padding_waste(seq_lengths, batch_size)
        batch_latency = _estimate_batch_latency(batch_size, seq_lengths)
        throughput = batch_size / max(batch_latency, 0.001)
        
        if batch_latency <= latency_sla_ms:
            optimal_config = {
                "batch_size": batch_size,
                "batch_latency_ms": batch_latency,
                "padding_waste_pct": padding_waste,
                "throughput_qps": throughput,
                "meets_sla": True,
            }
    
    return optimal_config or {"error": "no_batch_size_meets_sla", 
                               "recommendation": "reduce_max_sequence_length_or_quantize"}
```

## Inference-Aware Training Data

```python
def inference_aware_data_selection(training_corpus, deployment_query_distribution, model):
    """Select training data that improves inference performance, not just accuracy."""
    
    # Score training examples by inference relevance
    scored = []
    for example in training_corpus:
        # Inference-cost-aware score: reward examples that teach patterns with high inference ROI
        inference_frequency = _query_pattern_frequency(example, deployment_query_distribution)
        inference_cost = _estimate_inference_cost(example, model)
        
        score = inference_frequency / max(inference_cost, 1)
        scored.append({"example": example["id"], "score": score})
    
    # Select top-scoring examples
    scored.sort(key=lambda x: -x["score"])
    budget = len(training_corpus) * 0.3  # 30% of corpus selected for inference-aware training
    
    return {
        "selected_examples": scored[:int(budget)],
        "selection_ratio": budget / max(len(training_corpus), 1),
        "principle": "train_on_patterns_that_appear_frequently_at_inference_time",
    }
```

## Quality Gate

- Speculative decoding calibration corpus matches production query distribution (KL < 0.1).
- Mean draft acceptance rate ≥ 0.7; rejection patterns analyzed and addressed.
- KV cache strategy calibrated on production traffic — optimal window size or sink count determined.
- Batching configuration meets latency SLA while minimizing padding waste.
- Inference-aware training data selection applied — 30% of training corpus scored by inference ROI.
- All calibration data free of benchmark contamination.
