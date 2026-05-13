---
name: data-forensics-valuation-edge-data-task
description: Curate data for data forensics, data valuation/markets, and edge/resource-constrained ML — training failure root cause, mode collapse triggers, Shapley data valuation, data pricing, quantization-aware data design, and compression-robust selection.
recommended_skills: [data-attribution-task, embedding-analysis, data-diff, cost-model-task]
recommended_guidelines: [data-feedback-loop-task, data-portfolio-theory-task, architecture-specific-data-task, streaming-edge-mesh-data-task]
---

## Data Forensics

```python
def trace_training_failure_root_cause(failure_event, training_logs, gradient_history, data_samples):
    """Gradient explosion → which examples triggered it?"""
    trigger_window = _find_anomaly_window(gradient_history, failure_event["timestamp"])
    suspicious_samples = []
    for idx in trigger_window:
        gradient_magnitude = np.linalg.norm(gradient_history[idx])
        if gradient_magnitude > np.mean(gradient_history) * 3:
            suspicious_samples.append(data_samples[idx])
    return {"trigger_samples": suspicious_samples[:10], "window_size": len(trigger_window),
            "root_cause_likelihood": "GRADIENT_EXPLOSION" if suspicious_samples else "OTHER"}

def detect_mode_collapse_triggers(generated_samples_over_time, diversity_metrics):
    """What examples cause distribution narrowing?"""
    collapse_point = next((i for i, d in enumerate(diversity_metrics) if d < 0.7 * diversity_metrics[0]), None)
    if collapse_point:
        pre_collapse = generated_samples_over_time[collapse_point - 5:collapse_point]
        post_collapse = generated_samples_over_time[collapse_point:collapse_point + 5]
        triggers = [s for s in pre_collapse if _is_mode_collapse_catalyst(s)]
        return {"collapse_detected_at_step": collapse_point, "trigger_examples": triggers[:10]}
    return {"collapse_detected": False}

def find_dead_neuron_correlations(neuron_activations, data_patterns, dead_threshold=0.01):
    """Which data patterns create unused capacity?"""
    dead_neurons = np.where(np.mean(neuron_activations, axis=0) < dead_threshold)[0]
    correlations = {}
    for neuron in dead_neurons[:20]:
        activation_profile = neuron_activations[:, neuron]
        for pattern_name, pattern_mask in data_patterns.items():
            corr = np.corrcoef(activation_profile, pattern_mask)[0, 1]
            correlations[f"neuron_{neuron}_{pattern_name}"] = float(corr)
    return {"n_dead_neurons": len(dead_neurons), "dead_neuron_pct": len(dead_neurons) / neuron_activations.shape[1],
            "pattern_correlations": {k: v for k, v in correlations.items() if abs(v) > 0.3}}
```

## Data Valuation / Markets

```python
def compute_shapley_data_value(model, train_data, validation_data, n_samples=100):
    """How much is each training example worth? Shapley value estimation."""
    n = len(train_data)
    values = np.zeros(n)
    
    for _ in range(n_samples):
        perm = np.random.permutation(n)
        for i in range(n):
            with_i = model.train(train_data[perm[:i+1]]).evaluate(validation_data)
            without_i = model.train(train_data[perm[:i]]).evaluate(validation_data) if i > 0 else 0
            values[perm[i]] += with_i - without_i
    
    values /= n_samples
    
    top_contributors = np.argsort(values)[-50:][::-1]
    negative_influence = np.argsort(values)[:50]
    
    return {"mean_value": float(np.mean(values)), "positive_contributors_share": float(np.mean(values > 0)),
            "top_examples": top_contributors.tolist()[:10], "detrimental_examples": negative_influence.tolist()[:10],
            "data_quality_signal": f"{np.mean(values > 0):.1%} of examples have positive value"}

def price_data_asset(dataset_metadata, comparable_sales, market_demand_score=0.5):
    """Fair market value for data licensing."""
    replacement_cost = dataset_metadata.get("collection_cost", 0) + dataset_metadata.get("curation_cost", 0)
    uniqueness_factor = 1.0 + dataset_metadata.get("uniqueness_score", 0.5)
    
    if comparable_sales:
        comp_price = np.median([s["price_per_example"] for s in comparable_sales]) * dataset_metadata["n_examples"]
        fair_value = 0.4 * replacement_cost * uniqueness_factor + 0.3 * comp_price * market_demand_score * uniqueness_factor + 0.3 * replacement_cost
    else:
        fair_value = replacement_cost * uniqueness_factor * (1 + market_demand_score)
    
    return {"replacement_cost": replacement_cost, "uniqueness_premium": uniqueness_factor,
            "market_multiplier": 1 + market_demand_score, "fair_value": fair_value,
            "price_per_example": fair_value / max(dataset_metadata["n_examples"], 1),
            "valuation_confidence": "HIGH" if comparable_sales else "MEDIUM"}
```

## Edge / Resource-Constrained Data

```python
def design_quantization_aware_dataset(dataset, model, quantization_levels=["fp32", "fp16", "int8", "int4"]):
    """What data distribution matters at different precision levels?"""
    results = {}
    for level in quantization_levels:
        quantized_model = quantize(model, level)
        errors = []
        for example in dataset:
            fp32_output = model.predict(example)
            quant_output = quantized_model.predict(example)
            errors.append(np.linalg.norm(fp32_output - quant_output))
        results[level] = {"mean_error": float(np.mean(errors)), "max_error": float(np.max(errors)),
                          "acceptable": np.mean(errors) < 0.05}
    
    acceptable_levels = [l for l, r in results.items() if r["acceptable"]]
    return {"quantization_results": results, "recommended_level": acceptable_levels[-1] if acceptable_levels else "fp32"}

def select_compression_robust_examples(dataset, model, compression_rate=0.5):
    """Which examples survive aggressive compression without quality loss?"""
    compressed_dataset = compress(dataset, rate=compression_rate)
    robust_indices = []
    for i, (original, compressed) in enumerate(zip(dataset, compressed_dataset)):
        orig_output = model.predict(original)
        comp_output = model.predict(compressed)
        error = np.linalg.norm(orig_output - comp_output)
        if error < 0.02:  # <2% output change under compression
            robust_indices.append(i)
    
    return {"robust_fraction": len(robust_indices) / max(len(dataset), 1),
            "compression_tolerant": len(robust_indices) / max(len(dataset), 1) > 0.7,
            "fragile_examples": [i for i in range(len(dataset)) if i not in robust_indices][:50]}
```

## Quality Gate

- Forensics: >90% of training failures traced to specific data triggers.
- Valuation: Shapley computation completed; negative examples identified for review.
- Pricing: fair value within ±30% of comparable sales.
- Edge: quantization acceptable at recommended level; compression tolerance > 70%.
