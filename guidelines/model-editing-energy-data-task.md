---
name: model-editing-energy-data-task
description: Curate data for model editing/patching and energy-aware ML — post-hoc behavior modification, knowledge insertion validation, behavior deletion verification, training carbon estimation, energy-efficient data selection, and green ML dataset design.
recommended_skills: [embedding-analysis, data-diff, benchmark-contamination-scan]
recommended_guidelines: [data-attribution-task, synthetic-data-generation-task, cost-model-task]
---

## Model Editing / Patching

### Post-Hoc Behavior Modification

```python
def validate_edit(model, edit_examples, retain_examples, edit_fn):
    """Did the edit work without breaking unrelated capabilities?"""
    pre_edit = evaluate(model, edit_examples)
    pre_retain = evaluate(model, retain_examples)
    
    edited_model = edit_fn(model, edit_examples)
    post_edit = evaluate(edited_model, edit_examples)
    post_retain = evaluate(edited_model, retain_examples)
    
    return {"edit_success": post_edit > pre_edit + 0.1,
            "retention": post_retain / max(pre_retain, 1e-6),
            "catastrophic_forgetting": post_retain < pre_retain * 0.7,
            "patch_quality": "GOOD" if post_edit > pre_edit and post_retain > pre_retain * 0.9 else "NEEDS_WORK"}

def verify_knowledge_insertion(model, inserted_facts, probe_questions):
    """Does inserted knowledge integrate correctly?"""
    results = []
    for fact, probe in zip(inserted_facts, probe_questions):
        correct = model.answer(probe) == fact["expected_answer"]
        generalized = model.answer(probe["paraphrase"]) == fact["expected_answer"]
        results.append({"correct": correct, "generalized": generalized,
                        "integration": "FULL" if correct and generalized else "FRAGILE"})
    return {"full_integration": np.mean([r["integration"]=="FULL" for r in results]),
            "fragile": np.mean([r["integration"]=="FRAGILE" for r in results])}

def verify_behavior_deletion(model, deleted_behavior, related_behavior, test_prompts):
    """Did unwanted behavior get removed without affecting related behavior?"""
    pre_delete = evaluate_behavior(model, deleted_behavior, test_prompts)
    pre_related = evaluate_behavior(model, related_behavior, test_prompts)
    
    edited = delete_behavior(model, deleted_behavior)
    
    post_delete = evaluate_behavior(edited, deleted_behavior, test_prompts)
    post_related = evaluate_behavior(edited, related_behavior, test_prompts)
    
    return {"deletion_success": post_delete < 0.1,  # nearly zero
            "specificity": post_related / max(pre_related, 1e-6),
            "collateral_damage": pre_related - post_related}
```

## Energy / Carbon-Aware

### Training Carbon Estimation

```python
CARBON_INTENSITY = {"us-east": 400, "us-west": 200, "eu-west": 300, "eu-north": 100}  # gCO2/kWh

def estimate_training_carbon(gpu_hours, gpu_type="A100", region="us-east", pue=1.1):
    GPU_POWER = {"A100": 0.4, "H100": 0.7, "V100": 0.3}  # kW
    energy_kwh = GPU_POWER[gpu_type] * gpu_hours * pue
    carbon_kg = energy_kwh * CARBON_INTENSITY.get(region, 400) / 1000
    return {"energy_kwh": energy_kwh, "carbon_kg": carbon_kg,
            "equivalent_miles_driven": carbon_kg * 2.5,  # ~2.5 miles per kg CO2
            "recommendation": "SCHEDULE_IN_LOW_CARBON_REGION" if CARBON_INTENSITY.get(region, 400) > 300 else "ACCEPTABLE"}
```

### Energy-Efficient Data Selection

```python
def select_energy_efficient_subset(dataset, model, energy_budget_kwh, eval_task):
    """Maximize performance per kWh of training energy."""
    subset_sizes = [int(len(dataset) * pct) for pct in [0.01, 0.05, 0.1, 0.25, 0.5]]
    results = []
    for size in subset_sizes:
        subset = dataset.sample(size)
        energy = estimate_training_energy(model, subset)
        if energy > energy_budget_kwh: continue
        perf = evaluate(model_trained_on(subset), eval_task)
        results.append({"size": size, "perf": perf, "energy_kwh": energy, "efficiency": perf / energy})
    return max(results, key=lambda r: r["efficiency"]) if results else None
```

## Quality Gate

- Model edits validated: success on target, > 90% retention on unrelated.
- Behavior deletion verified: deleted behavior < 10%, related behavior retained.
- Training carbon estimated before every large run.
- Energy-efficient subset identified where applicable.
