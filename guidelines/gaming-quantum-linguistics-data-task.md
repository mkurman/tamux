---
name: gaming-quantum-linguistics-data-task
description: Curate data for gaming/simulation, quantum computing, linguistics beyond NLP, and veterinary/animal health — simulation fidelity validation, quantum circuit correctness, phonetic annotation quality, and cross-species diagnostic validation.
recommended_skills: [embedding-analysis, evaluation-dataset-design-task, data-contamination-task]
recommended_guidelines: [sim-to-real-bridge-task, synthetic-data-generation-task, specialized-modality-data-task]
---

## Gaming / Simulation

```python
def validate_simulation_fidelity(simulated_states, real_states, fidelity_metrics):
    """Does simulation match reality on key dimensions?"""
    results = {}
    for metric_name, metric_fn in fidelity_metrics.items():
        sim_vals = [metric_fn(s) for s in simulated_states]
        real_vals = [metric_fn(s) for s in real_states]
        mape = np.mean(np.abs(np.array(sim_vals) - np.array(real_vals)) / np.maximum(np.abs(np.array(real_vals)), 1))
        results[metric_name] = {"mape": float(mape), "fidelity": "HIGH" if mape < 0.1 else "MEDIUM" if mape < 0.3 else "LOW"}
    return results

def detect_simulation_exploits(agent_behaviors, simulation_rules, reward_function):
    """Is the agent learning to exploit sim bugs rather than solve the task?"""
    exploits = []
    for behavior in agent_behaviors:
        if _violates_physics(behavior, simulation_rules):
            exploits.append({"behavior": behavior["description"][:100], "exploit_type": "PHYSICS_VIOLATION"})
        elif _exploits_reward(behavior, reward_function):
            exploits.append({"behavior": behavior["description"][:100], "exploit_type": "REWARD_HACKING"})
    return {"exploits": exploits, "exploit_rate": len(exploits) / max(len(agent_behaviors), 1),
            "sim_needs_patching": len(exploits) > 0}
```

## Quantum Computing

```python
def validate_quantum_circuit(predicted_output, measured_output, n_qubits, n_shots=1000):
    """Does predicted circuit output match measured quantum results?"""
    fidelity = _compute_state_fidelity(predicted_output, measured_output)
    statistical_error = 1 / np.sqrt(n_shots)
    return {"state_fidelity": float(fidelity), "statistical_error_bound": float(statistical_error),
            "validated": fidelity > 1 - statistical_error,
            "recommendation": "ACCEPT" if fidelity > 1 - statistical_error else "RECALIBRATE_OR_INCREASE_SHOTS"}

def detect_decoherence(density_matrices, expected_purity, time_evolution):
    """How fast does quantum state lose coherence?"""
    purities = [np.trace(dm @ dm).real for dm in density_matrices]
    decoherence_rate = np.polyfit(time_evolution, np.log(purities), 1)[0] if len(purities) >= 3 else 0
    return {"purity_decay_rate": float(decoherence_rate), "t1_estimate": float(-1/decoherence_rate) if decoherence_rate < 0 else float('inf'),
            "coherence_time_us": float(-1e6/decoherence_rate) if decoherence_rate < 0 else float('inf')}
```

## Linguistics Beyond NLP

```python
def validate_phonetic_annotation(audio, phonetic_transcription, phoneme_inventory, annotator_agreement):
    """Do phonetic transcriptions match actual speech sounds?"""
    predicted_phonemes = _extract_phonemes(audio, phoneme_inventory)
    agreement_scores = []
    for annotator_id, annotation in annotator_agreement.items():
        match_rate = np.mean([p == a for p, a in zip(predicted_phonemes, annotation)])
        agreement_scores.append(match_rate)
    return {"phoneme_accuracy": float(np.mean(agreement_scores)),
            "inter_annotator_agreement": float(np.mean(agreement_scores)) if len(agreement_scores) > 1 else None,
            "acceptable": np.mean(agreement_scores) > 0.85}

def detect_language_contact(parallel_corpora, borrowing_patterns):
    """Code-switching, loanwords, calques — language mixing patterns."""
    borrowings = []
    for corpus_pair in parallel_corpora:
        source_words = set(corpus_pair["source"].split())
        target_words = set(corpus_pair["target"].split())
        shared = source_words & target_words
        if len(shared) / max(len(target_words), 1) > 0.05:
            borrowings.append({"pair": f"{corpus_pair['source_lang']}-{corpus_pair['target_lang']}",
                                "borrowing_rate": float(len(shared) / max(len(target_words), 1))})
    return borrowings
```

## Veterinary / Animal Health

```python
CROSS_SPECIES_LAB_MAPPING = {
    "canine": {"ALT_range": (10, 125), "CREAT_range": (0.5, 1.8), "GLUC_range": (70, 140)},
    "feline": {"ALT_range": (10, 100), "CREAT_range": (0.8, 2.4), "GLUC_range": (70, 150)},
    "equine": {"ALT_range": (5, 25), "CREAT_range": (0.9, 1.9), "GLUC_range": (60, 120)},
}

def validate_cross_species_diagnosis(model_predictions, confirmed_diagnoses, species):
    """Does model transfer across species correctly?"""
    per_species_results = {}
    for sp in set(species):
        mask = np.array(species) == sp
        if mask.sum() < 10: continue
        accuracy = np.mean(model_predictions[mask] == confirmed_diagnoses[mask])
        per_species_results[sp] = {"accuracy": float(accuracy), "n_samples": int(mask.sum())}
    return {"per_species": per_species_results,
            "cross_species_gap": max(r["accuracy"] for r in per_species_results.values()) - min(r["accuracy"] for r in per_species_results.values())}

def validate_lab_reference_ranges(measured_values, species, test_type, reference_ranges):
    """Are lab values within expected reference ranges for each species?"""
    outliers = []
    for i, (value, sp, test) in enumerate(zip(measured_values, species, test_type)):
        sp_range = reference_ranges.get(sp, {}).get(f"{test}_range")
        if sp_range and not (sp_range[0] <= value <= sp_range[1]):
            outliers.append({"index": i, "species": sp, "test": test, "value": value, "expected_range": sp_range})
    return {"outlier_rate": len(outliers) / max(len(measured_values), 1),
            "outliers": outliers[:20], "acceptable": len(outliers) / max(len(measured_values), 1) < 0.05}
```

## Quality Gate

- Gaming: simulation fidelity > 90% on key metrics; zero reward-hacking exploits.
- Quantum: state fidelity within statistical error bounds.
- Linguistics: phoneme accuracy > 85%; borrowing patterns documented.
- Veterinary: cross-species accuracy gap < 10pp; lab outliers < 5%.
