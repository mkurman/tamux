---
name: reproducibility-science-task
description: Design datasets for reproducibility validation — exact reproduction capture, partial reproducibility measurement, environment influence quantification, and author-provided reproduction package validation.
recommended_skills: [data-diff, dataset-versioning, embedding-analysis]
recommended_guidelines: [evaluation-dataset-design-task, data-contamination-task]
---

## Overview

ML reproducibility is abysmal. Claims of "state-of-the-art" often fail to reproduce. This guideline covers how to design datasets that validate reproducibility — not just report metrics once, but prove they can be reproduced.

## Phase 1: Exact Reproducibility Data Capture

```python
def capture_reproducibility_package(model, training_data, eval_data, seeds, hyperparams):
    return {
        "dataset_hash": hashlib.sha256(str(training_data).encode()).hexdigest(),
        "dataset_version": training_data.version,
        "split_seeds": seeds, "all_hyperparams": hyperparams,
        "environment": {"python": sys.version, "packages": _freeze_packages()},
        "random_state_snapshot": np.random.get_state(),  # for NumPy reproducibility
        "hardware": {"gpu": torch.cuda.get_device_name() if torch.cuda.is_available() else "CPU"},
    }

def verify_reproduction(package, reproduction_run):
    """Compare claimed reproduction to original."""
    checks = {
        "dataset_match": reproduction_run["dataset_hash"] == package["dataset_hash"],
        "hyperparams_match": reproduction_run["hyperparams"] == package["all_hyperparams"],
        "seeds_match": reproduction_run["seeds"] == package["split_seeds"],
    }
    return {"checks": checks, "reproduced": all(checks.values()),
            "failures": [k for k, v in checks.items() if not v]}
```

## Phase 2: Partial Reproducibility

```python
def measure_partial_reproducibility(original_results, reproduction_attempts):
    """What fraction of runs reproduce the claimed result within tolerance?"""
    within_tolerance = 0
    for attempt in reproduction_attempts:
        delta = abs(attempt["metric"] - original_results["metric"])
        if delta <= original_results.get("claimed_tolerance", 0.02):
            within_tolerance += 1
    
    return {"reproduction_rate": within_tolerance / len(reproduction_attempts),
            "reliable": within_tolerance / len(reproduction_attempts) > 0.8,
            "needs_investigation": within_tolerance / len(reproduction_attempts) < 0.5}
```

## Phase 3: Environment Influence

```python
def measure_environment_influence(model_fn, dataset, environments):
    """Does result vary across hardware/software configs?"""
    results = {}
    for env_name, env_config in environments.items():
        with _set_environment(env_config):
            model = model_fn()
            results[env_name] = evaluate(model, dataset)
    
    variance = np.var(list(results.values()))
    return {"results": results, "variance": variance,
            "environment_sensitive": variance > 0.001,
            "stable": variance < 0.0005}
```

## Quality Gate

- Reproducibility package captures all seeds, hyperparams, environment.
- Partial reproducibility > 80% across ≥ 5 independent reproduction attempts.
- Environment influence variance < 0.001 (stable across configs).
