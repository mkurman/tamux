---
name: social-science-causal-data-task
description: Curate data for computational social science and causal discovery — social network validation, community detection ground truth, causal graph validation, confounder detection, mediator identification, and counterfactual generation.
recommended_skills: [graph-data-task, embedding-analysis, bias-audit, evaluation-dataset-design-task]
recommended_guidelines: [experimental-methodology-data-task, data-contamination-task, intersectional-evaluation-task]
---

## Computational Social Science

```python
def validate_social_network(network_edges, validation_method, ground_truth=None):
    """Do edges represent real relationships?"""
    result = {"n_nodes": len(set(e[0] for e in network_edges) | set(e[1] for e in network_edges)),
              "n_edges": len(network_edges),
              "density": len(network_edges) / max(len(set(e[0] for e in network_edges) | set(e[1] for e in network_edges))**2, 1)}
    
    if ground_truth:
        tp = len(set(network_edges) & set(ground_truth))
        fp = len(set(network_edges) - set(ground_truth))
        fn = len(set(ground_truth) - set(network_edges))
        result["precision"] = tp / max(tp + fp, 1)
        result["recall"] = tp / max(tp + fn, 1)
        result["f1"] = 2 * result["precision"] * result["recall"] / max(result["precision"] + result["recall"], 1)
    
    return result

def validate_community_detection(detected_communities, ground_truth_communities):
    """Do detected communities match real groups?"""
    from sklearn.metrics import adjusted_rand_score, normalized_mutual_info_score
    detected_labels = _flatten_communities(detected_communities)
    truth_labels = _flatten_communities(ground_truth_communities)
    return {"ari": float(adjusted_rand_score(truth_labels, detected_labels)),
            "nmi": float(normalized_mutual_info_score(truth_labels, detected_labels)),
            "valid": adjusted_rand_score(truth_labels, detected_labels) > 0.5}

def trace_information_cascade(cascade_model, observed_spread, network_structure):
    """Can you reconstruct how information actually spread?"""
    predicted_paths = cascade_model(network_structure)
    observed_paths = observed_spread
    path_overlap = len(set(predicted_paths) & set(observed_paths)) / max(len(set(observed_paths)), 1)
    return {"path_reconstruction_accuracy": float(path_overlap),
            "cascade_traceable": path_overlap > 0.7}
```

## Causal Discovery

```python
def validate_causal_graph(discovered_graph, true_graph):
    """Does discovered causal structure match truth?"""
    discovered_edges = set(discovered_graph.edges())
    true_edges = set(true_graph.edges())
    tp = len(discovered_edges & true_edges)
    fp = len(discovered_edges - true_edges)
    fn = len(true_edges - discovered_edges)
    return {"edge_precision": tp / max(tp + fp, 1), "edge_recall": tp / max(tp + fn, 1),
            "structural_hamming_distance": fp + fn,
            "causal_direction_accuracy": _edge_direction_accuracy(discovered_graph, true_graph)}

def detect_confounders(variables, causal_effect_estimates, domain_knowledge):
    """What variables are actually confounders?"""
    confounders = []
    for var in variables:
        adjusted = causal_effect_estimates.get(f"adjusted_{var}")
        unadjusted = causal_effect_estimates.get(f"unadjusted_{var}")
        if adjusted and unadjusted and abs(adjusted - unadjusted) / max(abs(unadjusted), 1e-6) > 0.2:
            confounders.append({"variable": var, "confounding_strength": abs(adjusted-unadjusted)/max(abs(unadjusted), 1e-6)})
    return {"confounders": confounders, "n_confounders": len(confounders)}

def distinguish_mediator_moderator(variable, outcome, mechanism_test):
    """Mediation: X → M → Y. Moderation: X → Y depends on M."""
    direct_effect = mechanism_test.get("direct", 0)
    indirect_effect = mechanism_test.get("indirect", 0)
    interaction_effect = mechanism_test.get("interaction", 0)
    
    if abs(indirect_effect) > abs(interaction_effect) * 2:
        return {"role": "MEDIATOR", "indirect_effect": indirect_effect, 
                "mediation_pct": indirect_effect / max(direct_effect + indirect_effect, 1e-6)}
    elif abs(interaction_effect) > abs(indirect_effect) * 2:
        return {"role": "MODERATOR", "interaction_effect": interaction_effect}
    else:
        return {"role": "MIXED", "indirect": indirect_effect, "interaction": interaction_effect}

def validate_counterfactuals(generated_counterfactuals, true_counterfactuals):
    """Are generated counterfactuals plausible?"""
    plausibility_scores = []
    for gen, true in zip(generated_counterfactuals, true_counterfactuals):
        if true: plausibility_scores.append(_plausibility_score(gen, true))
    return {"mean_plausibility": float(np.mean(plausibility_scores)) if plausibility_scores else 0,
            "plausible": np.mean(plausibility_scores) > 0.7 if plausibility_scores else False}
```

## Quality Gate

- Social network: F1 > 0.7 for edge prediction; community detection ARI > 0.5.
- Information cascade: path reconstruction > 70%.
- Causal graph: edge F1 > 0.7; structural Hamming distance < |true_edges| * 0.3.
- Confounders: identified and adjusted where confounding strength > 0.2.
