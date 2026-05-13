---
name: adversarial-data-design-task
description: Design adversarial datasets for robustness testing — targeted failure induction, universal adversarial patterns, severity calibration, counter-adversarial validation, and red-teaming data construction.
recommended_skills: [robustness-engineering-task, embedding-analysis, benchmark-contamination-scan]
recommended_guidelines: [data-contamination-task, evaluation-dataset-design-task, synthetic-data-generation-task]
---

## Overview

Adversarial data design is deliberate — you construct examples to break models, not to train them. This guideline covers how to build adversarial test sets that reveal genuine vulnerability, not just noise sensitivity.

## Phase 1: Targeted Failure Induction

```python
def induce_targeted_failure(model, target_class, original_examples, perturbation_budget=0.1):
    """Find minimal perturbations that flip a specific class prediction."""
    failures = []
    for ex in original_examples:
        if model.predict(ex) != target_class:
            continue
        
        # Binary search for minimal perturbation
        lo, hi = 0, perturbation_budget
        best_perturbed = None
        for _ in range(10):
            mid = (lo + hi) / 2
            perturbed = apply_perturbation(ex, magnitude=mid)
            if model.predict(perturbed) != target_class:
                best_perturbed = perturbed
                hi = mid
            else:
                lo = mid
        
        if best_perturbed:
            failures.append({"original": ex, "perturbed": best_perturbed,
                             "min_perturbation": hi, "original_class": target_class,
                             "flipped_to": model.predict(best_perturbed)})
    
    return {"failures": failures, "success_rate": len(failures) / max(len(original_examples), 1),
            "mean_perturbation_needed": np.mean([f["min_perturbation"] for f in failures])}
```

## Phase 2: Universal Adversarial Patterns

```python
def find_universal_patterns(models, examples, pattern_types, min_victim_models=2):
    """Find patterns that fail MULTIPLE models — not model-specific quirks."""
    universal = []
    for pattern_type, pattern_fn in pattern_types.items():
        for ex in examples:
            perturbed = pattern_fn(ex)
            n_failed = sum(1 for m in models if m.predict(perturbed) != m.predict(ex))
            if n_failed >= min_victim_models:
                universal.append({"example": ex, "pattern": pattern_type,
                                   "models_failed": n_failed, "total_models": len(models)})
    return universal

PATTERN_TYPES = {
    "negation": lambda text: text.replace(" is ", " is not "),
    "entity_flip": lambda text: text.replace("Paris", "London"),
    "numerical_shift": lambda text: re.sub(r"\d+", lambda m: str(int(m.group())*2), text),
    "logical_inversion": lambda text: text.replace("always", "never").replace("all", "none"),
}
```

## Phase 3: Adversarial Severity Calibration

```python
def calibrate_adversarial_severity(failures, impact_assessment):
    """Not all adversarial failures matter equally."""
    severity_levels = {"CRITICAL": [], "HIGH": [], "MEDIUM": [], "LOW": []}
    
    for failure in failures:
        # What's the real-world impact of this failure?
        impact = impact_assessment(failure)
        
        if impact["safety_risk"]:
            severity_levels["CRITICAL"].append(failure)
        elif impact["financial_loss"] > 10000:
            severity_levels["HIGH"].append(failure)
        elif impact["user_experience_degraded"]:
            severity_levels["MEDIUM"].append(failure)
        else:
            severity_levels["LOW"].append(failure)
    
    return {"by_severity": {k: len(v) for k, v in severity_levels.items()},
            "critical_rate": len(severity_levels["CRITICAL"]) / max(len(failures), 1),
            "acceptable": len(severity_levels["CRITICAL"]) == 0}
```

## Phase 4: Counter-Adversarial Validation

```python
def validate_counter_adversarial(model, adversarial_examples, defense_fn):
    """Does the model recognize that adversarial input is adversarial?"""
    results = []
    for ex in adversarial_examples:
        normal_pred = model.predict(ex["original"])
        adv_pred = model.predict(ex["perturbed"])
        defended_pred = model.predict(defense_fn(ex["perturbed"]))
        
        results.append({
            "original": normal_pred,
            "adversarial": adv_pred,
            "defended": defended_pred,
            "defense_works": defended_pred == normal_pred,
            "recovery": defended_pred == normal_pred and adv_pred != normal_pred,
        })
    
    return {"defense_success_rate": np.mean([r["defense_works"] for r in results]),
            "recovery_rate": np.mean([r["recovery"] for r in results])}
```

## Phase 5: Red-Teaming Data Construction

```python
def construct_red_team_dataset(model, attack_surface, n_examples=1000):
    """Build a red-team dataset targeting known attack surfaces."""
    dataset = []
    for surface_name, surface_config in attack_surface.items():
        for _ in range(n_examples // len(attack_surface)):
            seed = surface_config["seed_generator"]()
            attack = surface_config["attack_fn"](seed, model)
            dataset.append({"attack_surface": surface_name, "seed": seed,
                            "attack_input": attack["input"],
                            "attack_output": attack.get("output"),
                            "success": attack.get("harmful", False)})
    
    return dataset

ATTACK_SURFACES = {
    "prompt_injection": {"seed_generator": lambda: "Ignore previous instructions...",
                          "attack_fn": _test_prompt_injection},
    "data_extraction": {"seed_generator": lambda: "What was in your training data...",
                         "attack_fn": _test_data_extraction},
    "harmful_content": {"seed_generator": lambda: "How to build dangerous...",
                         "attack_fn": _test_harmful_content},
}
```

## Quality Gate

- Targeted failures achieved with perturbation budget < 0.1 for ≥ 50% of targeted examples.
- Universal patterns validated against ≥ 3 different models.
- Zero CRITICAL-severity adversarial failures in production deployment.
- Counter-adversarial defense recovers ≥ 80% of adversarial examples.
- Red-team dataset covers all defined attack surfaces.
