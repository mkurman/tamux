---
name: robustness-engineering-task
description: Engineer dataset-driven robustness — stress test catalogs with hardness tiers, robustness envelope mapping, failure mode genealogy, and recovery test sets.
recommended_skills:
  - embedding-analysis
  - bias-audit
  - benchmark-contamination-scan
recommended_guidelines:
  - training-data-design-principles
  - data-contamination-task
  - evaluation-dataset-design-task
---

## Overview

Robustness isn't a metric — it's an engineering discipline. This guideline builds stress test catalogs, maps performance under perturbation, traces failure to root cause, and measures recovery capability.

## Phase 1: Stress Test Catalog

### Hardness Tier System

```python
STRESS_TEST_CATALOG = {
    "tier_1_mild": {
        "description": "Minor perturbations — model should be invariant",
        "pass_threshold": 0.95,  # 95% of original performance
        "tests": {
            "whitespace_noise": "Add/remove spaces, tabs, newlines",
            "casing": "randomize CASE of input text",
            "punctuation": "Add/remove punctuation",
            "synonym_swap": "Replace words with synonyms (WordNet)",
            "active_passive": "Convert between active and passive voice",
            "image_brightness": "±10% brightness",
            "image_rotation": "±5° rotation",
            "audio_volume": "±3dB volume",
        },
    },
    "tier_2_moderate": {
        "description": "Meaningful perturbations — model should degrade gracefully",
        "pass_threshold": 0.80,
        "tests": {
            "negation": "Negate factual statements",
            "entity_swap": "Swap named entities (Paris→London, Tesla→Ford)",
            "numerical_perturbation": "Multiply numbers by 0.5 or 2.0",
            "paraphrase": "Full sentence paraphrase (back-translation)",
            "image_crop": "Crop 20% of image",
            "image_blur": "Gaussian blur σ=2",
            "audio_noise": "Add background noise at SNR=10dB",
            "code_variable_rename": "Rename all variables in code",
        },
    },
    "tier_3_severe": {
        "description": "Adversarial — designed to break the model",
        "pass_threshold": 0.50,
        "tests": {
            "adversarial_attack": "PGD/FGSM adversarial examples",
            "out_of_distribution": "Input from completely different domain",
            "long_context_overflow": "10x normal input length",
            "recursive_self_reference": "Input contains its own output",
            "contradictory_premises": "Input with internal contradictions",
            "impossible_requests": "Tasks the model cannot do",
            "jailbreak_attempts": "Known jailbreak templates",
        },
    },
}
```

### Stress Test Execution

```python
def run_stress_suite(model, test_catalog, original_score):
    results = {}
    
    for tier_name, tier_config in test_catalog.items():
        tier_results = {}
        for test_name, test_fn in tier_config["tests"].items():
            perturbed_data = test_fn(test_set)
            perturbed_score = evaluate(model, perturbed_data)
            
            relative_score = perturbed_score / original_score
            passed = relative_score >= tier_config["pass_threshold"]
            
            tier_results[test_name] = {
                "original_score": original_score,
                "perturbed_score": perturbed_score,
                "relative_score": relative_score,
                "passed": passed,
                "degradation": original_score - perturbed_score,
            }
        
        tier_pass_rate = sum(1 for r in tier_results.values() if r["passed"]) / len(tier_results)
        
        results[tier_name] = {
            "tests": tier_results,
            "pass_rate": tier_pass_rate,
            "passed": tier_pass_rate >= 0.8,
            "mean_relative_score": np.mean([r["relative_score"] for r in tier_results.values()]),
        }
    
    return results
```

## Phase 2: Robustness Envelope Mapping

### 2D: Performance vs. Perturbation Intensity

```python
def map_robustness_envelope(model, test_set, perturbation_fn, intensity_range):
    """
    Map model performance as perturbation intensity increases.
    Produces a 2D envelope: X = intensity, Y = performance.
    """
    intensities = np.linspace(intensity_range[0], intensity_range[1], 20)
    performances = []
    
    for intensity in intensities:
        perturbed = perturbation_fn(test_set, intensity)
        score = evaluate(model, perturbed)
        performances.append(score)
    
    # Find the "breaking point" — where performance drops below threshold
    threshold = np.max(performances) * 0.8
    breaking_point = None
    for i, score in enumerate(performances):
        if score < threshold:
            breaking_point = intensities[i]
            break
    
    # Area under the robustness curve (AuRC)
    from scipy.integrate import trapezoid
    auroc = trapezoid(performances, intensities)
    max_auc = (intensity_range[1] - intensity_range[0]) * np.max(performances)
    
    return {
        "intensities": intensities.tolist(),
        "performances": performances,
        "breaking_point": breaking_point,
        "max_degradation": float(np.max(performances) - np.min(performances)),
        "auroC_ratio": auroc / max_auc,  # 1.0 = perfectly robust
        "robustness_profile": (
            "flat" if np.std(performances) / np.mean(performances) < 0.05
            else "graceful" if breaking_point and breaking_point > intensities[len(intensities)//2]
            else "brittle"
        ),
    }
```

### Multi-Axis Envelope

```python
def multi_axis_robustness_radar(model, test_set, perturbation_axes):
    """
    Radar plot: each axis = a perturbation type, radius = robustness score.
    """
    radar_data = {}
    for axis_name, (perturb_fn, min_intensity, max_intensity) in perturbation_axes.items():
        envelope = map_robustness_envelope(model, test_set, perturb_fn, 
                                           (min_intensity, max_intensity))
        radar_data[axis_name] = {
            "auroC_ratio": envelope["auroC_ratio"],
            "breaking_point": envelope["breaking_point"],
            "profile": envelope["robustness_profile"],
        }
    
    # Overall robustness score = mean across axes
    overall = np.mean([d["auroC_ratio"] for d in radar_data.values()])
    
    return {
        "axes": radar_data,
        "overall_robustness": overall,
        "weakest_axis": min(radar_data.items(), key=lambda x: x[1]["auroC_ratio"])[0],
        "brittle_axes": [k for k, v in radar_data.items() if v["profile"] == "brittle"],
    }
```

## Phase 3: Failure Mode Genealogy

### Root Cause → Symptom → Downstream Effect

```python
def trace_failure_genealogy(model, failure_examples, attribution_model):
    """
    For each failure:
    1. What went wrong? (symptom)
    2. Which training examples taught this? (root cause, via attribution)
    3. What downstream task does this affect? (effect)
    """
    genealogy = []
    
    for failure in failure_examples:
        # Symptom analysis
        symptom = classify_failure_type(model, failure)
        
        # Trace to training data via attribution
        root_causes = attribution_model.trace(failure)
        
        # What downstream tasks share these training examples?
        affected_tasks = _find_affected_tasks(root_causes["top_train_examples"])
        
        genealogy.append({
            "failure_id": failure["id"],
            "symptom": {
                "type": symptom["type"],          # e.g., "negation_insensitivity"
                "description": symptom["description"],
                "severity": symptom["severity"],
            },
            "root_cause": {
                "training_examples": root_causes["top_train_examples"][:10],
                "common_pattern": root_causes["pattern"],
                "source": root_causes["data_source"],
            },
            "downstream_effect": {
                "affected_tasks": affected_tasks,
                "estimated_impact": _estimate_impact(failure, affected_tasks),
            },
        })
    
    # Aggregated failure patterns
    pattern_counts = {}
    for g in genealogy:
        key = g["symptom"]["type"]
        if key not in pattern_counts:
            pattern_counts[key] = {"count": 0, "root_causes": set()}
        pattern_counts[key]["count"] += 1
        pattern_counts[key]["root_causes"].add(g["root_cause"]["common_pattern"])
    
    return {
        "failures": genealogy,
        "patterns": pattern_counts,
        "most_common_failure": max(pattern_counts.items(), key=lambda x: x[1]["count"])[0],
        "most_damaging_root_cause": _find_most_damaging_root(genealogy),
    }
```

## Phase 4: Recovery Test Sets

### Can the Model Recover from Error?

```python
def build_recovery_test_set(model, tasks, error_types):
    """
    For each task + error type:
    1. Give the model a task
    2. Let it make a mistake
    3. Give corrective feedback
    4. Measure: can it recover?
    """
    recovery_tests = []
    
    for task in tasks:
        for error_type in error_types:
            # Induce the error
            result = model.execute(task)
            
            if not _is_error(result, error_type):
                continue
            
            # Provide corrective feedback
            feedback = _generate_correction(task, result, error_type)
            
            # Measure recovery
            recovery_result = model.execute(task, context=[result, feedback])
            recovered = _is_correct(recovery_result, task)
            
            recovery_tests.append({
                "task_id": task["id"],
                "error_type": error_type,
                "original_output": result,
                "feedback": feedback,
                "recovery_output": recovery_result,
                "recovered": recovered,
                "recovery_time_steps": len(recovery_result.get("steps", [])),
            })
    
    return {
        "tests": recovery_tests,
        "recovery_rate": np.mean([t["recovered"] for t in recovery_tests]),
        "by_error_type": {
            et: np.mean([t["recovered"] for t in recovery_tests if t["error_type"] == et])
            for et in error_types
        },
        "unrecoverable_errors": [t for t in recovery_tests if not t["recovered"]],
    }
```

## Quality Gate

- Stress test catalog covers all three hardness tiers.
- Tier 1 (mild) pass rate > 95%. Fails indicate fundamental fragility.
- Robustness envelope mapped for ≥ 5 perturbation axes.
- Failure mode genealogy traced for ≥ 90% of validation failures.
- Recovery rate measured; unrecoverable errors documented and escalated.
- Weakest robustness axis identified and targeted for data augmentation.
