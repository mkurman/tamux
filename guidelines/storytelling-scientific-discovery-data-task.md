---
name: storytelling-scientific-discovery-data-task
description: Curate data for data storytelling/journalism, scientific discovery pipelines, and creative/artistic domains — narrative validity, visualization deception detection, hypothesis trace quality, discovery claim verification, and copyright/plagiarism detection.
recommended_skills: [data-visualization-task, bias-audit, reproducibility-science-task, benchmark-contamination-scan]
recommended_guidelines: [experimental-methodology-data-task, data-contamination-task, cultural-linguistic-data-task]
---

## Data Storytelling / Journalism

```python
def validate_narrative(narrative_claims, source_data):
    """Does the data story accurately represent the data?"""
    validated = []
    for claim in narrative_claims:
        evidence = _find_evidence(claim["statement"], source_data)
        validated.append({"claim": claim["statement"][:100],
                          "supported": evidence is not None,
                          "evidence_snippet": str(evidence)[:200] if evidence else None})
    return {"claims_validated": len(validated), "accuracy": np.mean([v["supported"] for v in validated]),
            "unsupported_claims": [v["claim"] for v in validated if not v["supported"]]}

def detect_visualization_deception(viz_spec, underlying_data):
    """Is the visualization misleading?"""
    checks = {
        "truncated_axis": _detect_axis_truncation(viz_spec),
        "area_encoding_error": _detect_area_vs_length(viz_spec),
        "cherry_picked_range": _detect_range_selection(viz_spec, underlying_data),
        "dual_axis_deception": _detect_dual_axis_misalignment(viz_spec),
        "missing_baseline": _detect_missing_zero(viz_spec),
    }
    return {"checks": checks, "deceptive": any(checks.values()),
            "issues": [k for k, v in checks.items() if v],
            "severity": "MISLEADING" if sum(checks.values()) >= 3 else "QUESTIONABLE" if any(checks.values()) else "ACCURATE"}
```

## Scientific Discovery Pipeline

```python
def trace_hypothesis_evolution(hypotheses, experiment_results, temporal_order):
    """What hypotheses were proposed, tested, rejected, pursued?"""
    trajectory = []
    for hyp_id in temporal_order:
        hyp = hypotheses[hyp_id]
        result = experiment_results.get(hyp_id)
        trajectory.append({"hypothesis": hyp["statement"][:100],
                           "result": "SUPPORTED" if result and result["p_value"] < 0.05 else "REJECTED",
                           "pursued": hyp.get("follow_up_conducted", False)})
    return {"trajectory": trajectory, "n_tested": len(trajectory),
            "rejection_rate": np.mean([t["result"]=="REJECTED" for t in trajectory]),
            "dead_ends_avoided": sum(1 for t in trajectory if t["result"]=="REJECTED" and not t["pursued"])}

def verify_discovery_claim(claim, supporting_evidence, independent_replications):
    """Does evidence support the claimed discovery?"""
    evidence_check = {"original": claim["statistical_significance"],
                      "effect_size": claim["effect_size"],
                      "power": claim.get("statistical_power", 0)}
    replications = {"n_replications": len(independent_replications),
                    "replicated": np.mean([r["significant"] for r in independent_replications]) if independent_replications else 0}
    return {"original_evidence": evidence_check, "replication_rate": replications["replicated"],
            "credible": evidence_check["original"] and replications["replicated"] > 0.5,
            "replication_crisis_flag": evidence_check["original"] and replications["replicated"] < 0.3}

def archive_negative_result(experiment_id, hypothesis, method, result, controls_passed):
    """Failed experiments are valuable data. Archive them properly."""
    return {"experiment": experiment_id, "hypothesis": hypothesis,
            "result": "NEGATIVE", "validity": "VALID" if controls_passed else "INCONCLUSIVE",
            "value": "Prevents others from repeating this dead end",
            "publication_recommendation": "PUBLISH" if controls_passed else "INCONCLUSIVE_DO_NOT_PUBLISH"}
```

## Creative / Artistic

```python
def detect_training_data_originality(generated_output, training_data, similarity_threshold=0.8):
    """Is output novel or derivative of training data?"""
    train_embeddings = embed(training_data)
    output_embedding = embed([generated_output])[0]
    similarities = cosine_similarity([output_embedding], train_embeddings)[0]
    max_sim = similarities.max()
    closest_idx = similarities.argmax()
    return {"max_similarity": float(max_sim), "closest_training_example": str(training_data[closest_idx])[:200],
            "original": max_sim < similarity_threshold,
            "risk": "DERIVATIVE" if max_sim > similarity_threshold else "NOVEL"}

def audit_copyright_risk(generated_content, protected_works, legal_threshold=0.7):
    """Does generated content risk infringing protected works?"""
    risks = []
    for work_id, work_content in protected_works.items():
        sim = _content_similarity(generated_content, work_content)
        if sim > legal_threshold:
            risks.append({"work": work_id, "similarity": float(sim), "risk_level": "HIGH"})
        elif sim > legal_threshold * 0.7:
            risks.append({"work": work_id, "similarity": float(sim), "risk_level": "MODERATE"})
    return {"risks": risks, "safe": len(risks) == 0,
            "recommendation": "CLEAR" if not risks else "LEGAL_REVIEW_REQUIRED"}
```

## Quality Gate

- Journalism: 100% of claims traceable to source data; zero deceptive visualizations.
- Scientific discovery: negative results archived for ALL experiments (not just positive).
- Discovery claims: replication rate > 50% for credible claims.
- Creative: all outputs pass originality check; copyright risks flagged for legal review.
