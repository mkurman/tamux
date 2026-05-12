---
name: sim-to-real-bridge-task
description: Bridge synthetic and real data — validation gap measurement, domain randomization parameter tuning, sim validity audits, and synthetic failure mode detection.
recommended_skills:
  - embedding-analysis
  - data-diff
  - llm-assisted-curation
recommended_guidelines:
  - synthetic-data-generation-task
  - data-contamination-task
  - training-data-design-principles
---

## Overview

Synthetic data is only as good as its alignment to reality. This guideline measures the gap, tunes the bridge, and detects what simulation teaches WRONG.

## Phase 1: Sim-Real Gap Measurement

### Multi-Axis Gap Analysis

```python
def measure_sim_real_gap(sim_data, real_data, axes):
    """
    Measure distribution gap between sim and real across multiple axes.
    Returns per-axis gap and overall alignment score.
    """
    gaps = {}
    
    for axis_name, axis_config in axes.items():
        if axis_config["type"] == "numeric":
            # Wasserstein distance (range-normalized)
            sim_vals = sim_data[axis_config["column"]].dropna()
            real_vals = real_data[axis_config["column"]].dropna()
            
            w_dist = wasserstein_distance(sim_vals, real_vals)
            # Normalize by data range for comparability
            data_range = max(real_vals.max() - real_vals.min(), 1e-10)
            normalized_gap = w_dist / data_range
            
            gaps[axis_name] = {
                "type": "numeric",
                "wasserstein": float(w_dist),
                "normalized_gap": float(normalized_gap),
                "severity": (
                    "critical" if normalized_gap > 0.2
                    else "warning" if normalized_gap > 0.1
                    else "acceptable"
                ),
                "sim_mean": float(sim_vals.mean()),
                "real_mean": float(real_vals.mean()),
                "mean_shift": float(abs(sim_vals.mean() - real_vals.mean()) / real_vals.std()),
            }
        
        elif axis_config["type"] == "categorical":
            sim_dist = sim_data[axis_config["column"]].value_counts(normalize=True)
            real_dist = real_data[axis_config["column"]].value_counts(normalize=True)
            
            # Jensen-Shannon divergence on categorical
            all_cats = sorted(set(sim_dist.index) | set(real_dist.index))
            sim_vec = np.array([sim_dist.get(c, 0) for c in all_cats])
            real_vec = np.array([real_dist.get(c, 0) for c in all_cats])
            
            js = jensenshannon(sim_vec + 1e-10, real_vec + 1e-10)
            
            gaps[axis_name] = {
                "type": "categorical",
                "js_divergence": float(js),
                "severity": "critical" if js > 0.3 else "warning" if js > 0.15 else "acceptable",
                "missing_categories": list(set(real_dist.index) - set(sim_dist.index)),
                "extra_categories": list(set(sim_dist.index) - set(real_dist.index)),
            }
        
        elif axis_config["type"] == "embedding":
            # Fréchet distance in embedding space
            sim_emb = embed(sim_data[axis_config["text_col"]])
            real_emb = embed(real_data[axis_config["text_col"]])
            
            fd = _frechet_distance(sim_emb, real_emb)
            
            # Per-cluster coverage (does sim cover all real clusters?)
            from sklearn.cluster import KMeans
            k = axis_config.get("n_clusters", 10)
            real_clusters = KMeans(k, random_state=42).fit_predict(real_emb)
            
            # For each real cluster, find nearest sim example
            sim_emb_mean = sim_emb.mean(axis=0)
            cluster_coverages = {}
            for c in range(k):
                mask = real_clusters == c
                if mask.sum() < 10:
                    continue
                cluster_center = real_emb[mask].mean(axis=0)
                # Distance from cluster center to nearest sim example
                sim_dists = np.linalg.norm(sim_emb - cluster_center, axis=1)
                cluster_coverages[c] = float(sim_dists.min())
            
            gaps[axis_name] = {
                "type": "embedding",
                "frechet_distance": float(fd),
                "mean_cluster_coverage": float(np.mean(list(cluster_coverages.values()))),
                "worst_cluster": int(np.argmax(list(cluster_coverages.values()))),
                "uncovered_clusters": sum(1 for v in cluster_coverages.values() if v > 1.0),
            }
    
    # Overall alignment score
    severities = [g.get("severity") for g in gaps.values()]
    overall = (
        "aligned" if all(s == "acceptable" for s in severities)
        else "partial" if sum(1 for s in severities if s == "critical") == 0
        else "misaligned"
    )
    
    return {"axes": gaps, "overall": overall}
```

## Phase 2: Domain Randomization Tuning

### What Variation Matters?

```python
def tune_domain_randomization(sim_env, real_data, parameters, n_trials=100):
    """
    Find randomization ranges that maximize sim-real transfer.
    Too little variation → overfit to sim. Too much → unrealistic noise.
    """
    def objective(randomization_ranges):
        # Generate sim data with these ranges
        sim_data = sim_env.generate(randomization_ranges)
        
        # Train model on sim
        model = train_model(sim_data)
        
        # Evaluate on real
        real_score = evaluate(model, real_data)
        
        # Also measure sim-real gap (lower = better)
        sim_gap = measure_sim_real_gap(sim_data, real_data, _get_axes(parameters))
        
        # Combined objective: real score + gap penalty
        gap_penalty = sum(
            1.0 for g in sim_gap["axes"].values()
            if g.get("severity") == "critical"
        )
        return real_score - 0.2 * gap_penalty  # trade off
    
    # Bayesian optimization over parameter ranges
    from sklearn.gaussian_process import GaussianProcessRegressor
    
    best_ranges = None
    best_score = -float("inf")
    
    for trial in range(n_trials):
        ranges = sample_randomization_ranges(parameters)
        if best_ranges is None:
            best_ranges = ranges
        else:
            # Perturb around current best
            ranges = perturb_ranges(best_ranges, parameters, trial)
        
        score = objective(ranges)
        if score > best_score:
            best_score = score
            best_ranges = ranges
    
    return best_ranges
```

## Phase 3: Sim Validity Audit

### Does Simulation Match Reality on Key Axes?

```python
def audit_sim_validity(sim_env, real_data, validity_axes):
    """
    For each validity axis, check if sim produces realistic data.
    """
    sim_data = sim_env.generate(n_samples=len(real_data))
    
    results = {}
    for axis_name, axis_check in validity_axes.items():
        if axis_check["type"] == "statistical":
            # Two-sample test: are sim and real from same distribution?
            sim_vals = sim_data[axis_check["column"]]
            real_vals = real_data[axis_check["column"]]
            
            stat, p_value = ks_2samp(sim_vals, real_vals)
            
            results[axis_name] = {
                "type": "statistical",
                "ks_statistic": float(stat),
                "p_value": float(p_value),
                "same_distribution": p_value > 0.05,
                "recommendation": (
                    "Sim matches reality — no adjustment needed"
                    if p_value > 0.05
                    else "Adjust sim parameter '" + axis_check["param"] + "' to match real distribution"
                ),
            }
        
        elif axis_check["type"] == "correlation":
            # Do variables correlate similarly in sim and real?
            sim_corr = sim_data[axis_check["columns"]].corr()
            real_corr = real_data[axis_check["columns"]].corr()
            
            corr_diff = np.abs(sim_corr - real_corr).values
            max_diff_idx = np.unravel_index(corr_diff.argmax(), corr_diff.shape)
            
            results[axis_name] = {
                "type": "correlation",
                "mean_abs_diff": float(corr_diff.mean()),
                "max_diff": float(corr_diff.max()),
                "max_diff_pair": (
                    axis_check["columns"][max_diff_idx[0]],
                    axis_check["columns"][max_diff_idx[1]],
                ),
                "realistic_correlations": corr_diff.mean() < 0.1,
            }
    ]
    
    return results
```

## Phase 4: Synthetic Failure Mode Detection

### What Does Sim Teach That's WRONG?

```python
def detect_synthetic_failures(model_trained_on_sim, real_test_data, 
                               model_trained_on_real, sim_test_data):
    """
    Find systematic failures introduced by sim training.
    """
    # Predictions from sim-trained model
    sim_predictions = model_trained_on_sim.predict(real_test_data)
    sim_accuracy = accuracy_score(real_test_data.labels, sim_predictions)
    
    # Predictions from real-trained model (upper bound)
    real_predictions = model_trained_on_real.predict(real_test_data)
    real_accuracy = accuracy_score(real_test_data.labels, real_predictions)
    
    # Where does sim-trained model fail but real-trained succeeds?
    sim_wrong_real_right = (sim_predictions != real_test_data.labels) & \
                            (real_predictions == real_test_data.labels)
    
    failure_patterns = _analyze_failure_clusters(
        real_test_data[sim_wrong_real_right],
        sim_predictions[sim_wrong_real_right],
    )
    
    # What features characterize sim-specific failures?
    sim_confidence = model_trained_on_sim.predict_proba(real_test_data).max(axis=1)
    overconfidence_failures = (sim_wrong_real_right) & (sim_confidence > 0.9)
    
    return {
        "sim_accuracy": sim_accuracy,
        "real_accuracy": real_accuracy,
        "sim_real_gap": real_accuracy - sim_accuracy,
        "n_sim_specific_failures": sim_wrong_real_right.sum(),
        "failure_rate": sim_wrong_real_right.mean(),
        "overconfident_failures": overconfidence_failures.sum(),
        "failure_patterns": failure_patterns,
    }
```

## Quality Gate

- Sim-real gap measured on all major axes; critical gaps (> 0.2 normalized) block training.
- Domain randomization parameters tuned via Bayesian optimization.
- Sim validity audit passes: distributions match (KS p > 0.05), correlations realistic (mean diff < 0.1).
- Synthetic failure modes catalogued: what sim teaches wrong, documented.
- Model trained on sim only deployed if sim-real accuracy gap < 10%.
