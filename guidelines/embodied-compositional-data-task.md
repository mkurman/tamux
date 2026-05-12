---
name: embodied-compositional-data-task
description: Curate data for embodied AI and compositional understanding — robot manipulation trajectory validation, physical dynamics learning, compositionality test construction, primitive operation validation, and cross-domain generalization.
recommended_skills: [embedding-analysis, dataset-splitting, robustness-engineering-task]
recommended_guidelines: [synthetic-data-generation-task, sim-to-real-bridge-task, agentic-training-data-task]
---

## Embodied / Physical Interaction

### Trajectory Validation

```python
def validate_trajectories(trajectories, task_specs):
    """Distinguish task success from trajectory quality."""
    results = []
    for traj in trajectories:
        task_success = traj["final_state"] in task_specs["success_states"]
        path_efficiency = _optimal_path_length(traj["task"]) / max(len(traj["actions"]), 1)
        smoothness = -np.mean(np.abs(np.diff(traj["joint_positions"], axis=0)))
        results.append({"success": task_success, "efficiency": path_efficiency, "smoothness": smoothness})
    
    # Higher efficiency + smoothness but same success rate = better trajectory quality
    return {"success_rate": np.mean([r["success"] for r in results]),
            "mean_efficiency": np.mean([r["efficiency"] for r in results]),
            "smoothness": np.mean([r["smoothness"] for r in results])}
```

### Physical Dynamics Learning

```python
def validate_physics_understanding(model, physics_scenarios):
    """Does model understand object permanence, gravity, collision?"""
    scenarios = {"object_permanence": [], "gravity": [], "collision": [], "containment": []}
    for scenario in physics_scenarios:
        prediction = model.predict(scenario["observation"])
        scenarios[scenario["type"]].append(prediction == scenario["expected"])
    
    return {k: {"accuracy": np.mean(v)} for k, v in scenarios.items()}
```

## Compositional Understanding

### Primitive Operation Validation

```python
def construct_compositional_test(primitives, compositions, n_holdout=20):
    """Test generalization to novel compositions of known primitives."""
    train_compositions, test_compositions = compositions[:-n_holdout], compositions[-n_holdout:]
    
    def is_novel(test_comp, train_comps):
        return test_comp not in train_comps
    
    novel = [c for c in test_compositions if is_novel(c, train_compositions)]
    return {"train": train_compositions, "test_novel": novel,
            "compositional_gap": "Evaluate: does model succeed on novel compositions?"}

def test_recursive_depth(model, base_operation, max_depth=10):
    """Can model handle arbitrarily deep recursive structures?"""
    results = []
    for depth in range(1, max_depth + 1):
        example = _build_recursive_example(base_operation, depth)
        correct = model.predict(example) == _expected_recursive_output(base_operation, depth)
        results.append({"depth": depth, "correct": correct})
    return {"max_depth_solved": max([r["depth"] for r in results if r["correct"]], default=0),
            "recursive_failure_depth": next((r["depth"] for r in results if not r["correct"]), None)}
```

### Hierarchical Composition

```python
def test_hierarchical_composition(model, hierarchical_tasks):
    """Can model compose operations across abstraction levels?"""
    levels = ["primitive", "subtask", "task", "mission"]
    results = {}
    for level in levels:
        tasks_at_level = [t for t in hierarchical_tasks if t["level"] == level]
        results[level] = np.mean([evaluate(model, t) for t in tasks_at_level])
    return {"by_level": results, "hierarchical_transfer": 
            results.get("mission", 0) / max(results.get("primitive", 1e-6), 1e-6)}
```

## Quality Gate

- Trajectories validated for both success AND quality.
- Physics understanding tested on ≥ 4 physical concepts.
- Compositional generalization measured: novel compositions ≠ train compositions.
- Recursive depth tested to failure point.
