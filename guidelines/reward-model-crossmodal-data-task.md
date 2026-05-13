---
name: reward-model-crossmodal-data-task
description: Curate data for reward models, cross-modal grounding, and world model dynamics — reward hacking detection, reward generalization, cross-modal referent alignment, contradiction detection, state transition coherence, and physics-grounding validation.
recommended_skills: [embedding-analysis, llm-assisted-curation, evaluation-dataset-design-task, robustness-engineering-task]
recommended_guidelines: [rl-alignment-data-task, synthetic-data-generation-task, data-contamination-task]
---

## Reward Model Quality

```python
def detect_reward_hacking(model, reward_model, base_prompts, n_optimization_steps=100):
    """Does the model optimize the proxy, not the intent?"""
    initial_rewards = [reward_model.score(model.generate(p)) for p in base_prompts]
    initial_quality = [_human_quality_score(model.generate(p)) for p in base_prompts]
    
    optimized_rewards = initial_rewards[:]
    optimized_quality = initial_quality[:]
    
    for step in range(n_optimization_steps):
        model.optimize_for_reward(reward_model)  # one step of RL
        optimized_rewards = [reward_model.score(model.generate(p)) for p in base_prompts]
        optimized_quality = [_human_quality_score(model.generate(p)) for p in base_prompts]
        
        reward_quality_gap = np.mean(optimized_rewards) - np.mean(optimized_quality)
        if reward_quality_gap > 1.0:  # rewarded but not actually better
            return {"reward_hacked": True, "step": step, "reward_gap": float(reward_quality_gap),
                    "reward_trend": "UP", "quality_trend": "DOWN" if optimized_quality[-1] < initial_quality[-1] else "FLAT"}
    
    return {"reward_hacked": False, "final_reward_gap": float(np.mean(optimized_rewards) - np.mean(optimized_quality))}

def test_reward_generalization(reward_model, seen_domains, unseen_domains, n_samples_per_domain=100):
    """Does reward model transfer to new situations?"""
    seen_scores = {d: [reward_model.score(_sample(d)) for _ in range(n_samples_per_domain)] for d in seen_domains}
    unseen_scores = {d: [reward_model.score(_sample(d)) for _ in range(n_samples_per_domain)] for d in unseen_domains}
    
    seen_variance = np.mean([np.var(v) for v in seen_scores.values()])
    unseen_variance = np.mean([np.var(v) for v in unseen_scores.values()])
    
    return {"generalization_ratio": seen_variance / max(unseen_variance, 1e-6),
            "overconfident_on_unseen": unseen_variance < seen_variance * 0.5,
            "calibrated": 0.7 < seen_variance / max(unseen_variance, 1e-6) < 1.3}
```

## Cross-Modal Grounding

```python
def validate_crossmodal_alignment(text_descriptions, images, reference_model):
    """Do text and image refer to the same entity?"""
    alignments = []
    for text, image in zip(text_descriptions, images):
        text_emb = reference_model.encode_text(text)
        image_emb = reference_model.encode_image(image)
        cos_sim = np.dot(text_emb, image_emb) / (np.linalg.norm(text_emb) * np.linalg.norm(image_emb))
        alignments.append({"text": text[:100], "cosine_similarity": float(cos_sim),
                           "aligned": cos_sim > 0.25, "misaligned": cos_sim < 0.15})
    return {"alignment_rate": np.mean([a["aligned"] for a in alignments]),
            "misalignment_rate": np.mean([a["misaligned"] for a in alignments]),
            "mean_similarity": float(np.mean([a["cosine_similarity"] for a in alignments]))}

def detect_crossmodal_contradictions(modality_pairs, contradiction_model):
    """When modalities say different things about the same thing."""
    contradictions = []
    for pair in modality_pairs:
        text_claim = pair["text"]
        image_content = contradiction_model.extract_content(pair["image"])
        if _contradicts(text_claim, image_content):
            contradictions.append({"id": pair["id"], "text_claim": text_claim[:100],
                                    "image_content": str(image_content)[:100],
                                    "contradiction_type": _classify_contradiction(text_claim, image_content)})
    return {"contradictions": len(contradictions), "contradiction_rate": len(contradictions) / max(len(modality_pairs), 1),
            "modal_trust_crisis": len(contradictions) / max(len(modality_pairs), 1) > 0.1}

MODAL_TRUST_HIERARCHY = {
    "medical_diagnosis": ["pathology_report", "radiology_report", "clinical_notes"],
    "autonomous_driving": ["lidar", "camera", "radar"],
    "content_moderation": ["human_review", "text_classifier", "image_classifier"],
}
```

## World Model / Environment Dynamics

```python
def validate_physics_grounding(model, physics_scenarios):
    """Does model understand real-world dynamics?"""
    results = {}
    for scenario_type, scenarios in physics_scenarios.items():
        predictions = [model.predict(s["observation"]) for s in scenarios]
        correct = [p == s["expected_outcome"] for p, s in zip(predictions, scenarios)]
        results[scenario_type] = {"accuracy": float(np.mean(correct)),
                                   "understands": np.mean(correct) > 0.8}
    return {"physics_understanding": results,
            "physics_aware": np.mean([r["understands"] for r in results.values()]) > 0.7}

def validate_state_transition_coherence(transitions, expected_constraints):
    """Are predicted future states consistent with known constraints?"""
    violations = []
    for i, transition in enumerate(transitions):
        for constraint_name, constraint_fn in expected_constraints.items():
            if not constraint_fn(transition["from_state"], transition["to_state"]):
                violations.append({"transition": i, "constraint": constraint_name,
                                   "violation": "State transition violates known constraint"})
    return {"coherence_score": 1 - len(violations) / max(len(transitions) * len(expected_constraints), 1),
            "violations": violations, "coherent": len(violations) == 0}

def construct_causal_intervention_test(environment, intervention_points, expected_effects):
    """Does intervention A produce expected effect B?"""
    results = []
    for intervention, expected in zip(intervention_points, expected_effects):
        actual = environment.intervene(intervention)
        effect_match = _compare_effects(actual, expected)
        results.append({"intervention": intervention["name"], "expected": expected["name"],
                        "actual": str(actual)[:100], "match": effect_match})
    return {"causal_accuracy": np.mean([r["match"] for r in results]),
            "world_understood": np.mean([r["match"] for r in results]) > 0.7}
```

## Quality Gate

- Reward model: no hacking detected; generalization ratio between 0.7-1.3.
- Cross-modal: alignment rate > 80%; contradiction rate < 10%.
- World model: physics understanding > 80%; state transitions coherent; causal accuracy > 70%.
