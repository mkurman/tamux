---
name: disciplinary-bridges-data-task
description: Curate data across disciplinary boundaries — data anthropology (cultural context), data archaeology (digital heritage preservation), data psychology (emotional/behavioral annotation), data ethics beyond compliance, and data futures/speculation.
recommended_skills: [bias-audit, embedding-analysis, data-card-writer]
recommended_guidelines: [cultural-linguistic-data-task, dataset-governance-task, data-archaeology-task]
---

## Data Anthropology

```python
def validate_cultural_context(dataset, cultural_dimensions):
    """Does data capture cultural meaning, not just surface features?"""
    coverage = {}
    for dim_name, dim_values in cultural_dimensions.items():
        present = set(dataset.get(dim_name, []))
        expected = set(dim_values)
        coverage[dim_name] = {"present": len(present), "expected": len(expected),
                               "coverage": len(present & expected) / max(len(expected), 1),
                               "missing": list(expected - present)}
    return {"cultural_coverage": coverage,
            "ethnocentric_risk": any(c["coverage"] < 0.5 for c in coverage.values())}

COMMUNITY_CONSENT_PROTOCOL = {
    "individual": "Each person consents individually",
    "collective": "Community representatives consent for the group",
    "tiered": "Individual + community, with opt-out mechanisms",
    "dynamic": "Consent can be updated/renegotiated over time",
}
```

## Data Archaeology (Digital Heritage)

```python
def audit_digital_preservation(artifacts, current_date):
    """Will this data survive?"""
    risks = []
    for artifact in artifacts:
        if artifact["format"] in OBSOLETE_FORMATS:
            risks.append({"artifact": artifact["id"], "risk": "FORMAT_OBSOLESCENCE"})
        if artifact.get("last_migration_date"):
            years_since_migration = (current_date - artifact["last_migration_date"]).days / 365
            if years_since_migration > 10:
                risks.append({"artifact": artifact["id"], "risk": "MIGRATION_OVERDUE", "years": years_since_migration})
        if not artifact.get("checksum"):
            risks.append({"artifact": artifact["id"], "risk": "NO_INTEGRITY_CHECK"})
    return {"preservation_risks": risks, "at_risk_count": len(risks),
            "preservation_score": 1 - len(risks) / max(len(artifacts) * 3, 1)}

OBSOLETE_FORMATS = ["WordPerfect", "Lotus 1-2-3", "Flash", "RealPlayer", "Shockwave"]
```

## Data Psychology

```python
def validate_emotional_annotation(annotations, n_annotators=3):
    """Can emotions be reliably labeled?"""
    from sklearn.metrics import cohen_kappa_score
    if len(annotations) < 2: return {"reliable": False, "reason": "need_multiple_annotators"}
    kappas = [cohen_kappa_score(annotations[i], annotations[j]) 
              for i in range(len(annotations)) for j in range(i+1, len(annotations))]
    mean_kappa = np.mean(kappas)
    return {"mean_kappa": float(mean_kappa), "reliable": mean_kappa > 0.6,
            "emotions_reliable": mean_kappa > 0.6,
            "recommendation": "USE_LABELS" if mean_kappa > 0.6 else "REFINE_ANNOTATION_GUIDE"}

COGNITIVE_LOAD_INDICATORS = {
    "response_time_increase": "Slower responses = higher load",
    "error_rate_increase": "More errors = overloaded",
    "simplification": "Choosing simpler strategies = conserving resources",
    "omission": "Skipping steps = overload coping mechanism",
}
```

## Data Ethics Beyond Compliance

```python
def audit_ethical_debt(dataset, decisions_log):
    """Track accumulated ethical compromises — compliance is the floor, not the ceiling."""
    debt_items = []
    for decision in decisions_log:
        if decision.get("tradeoff_made"):
            debt_items.append({"decision": decision["id"], "tradeoff": decision["tradeoff_made"],
                                "stakeholders_affected": decision.get("affected_groups", []),
                                "mitigation": decision.get("mitigation", "none"),
                                "review_date": decision.get("next_review_date")})
    unresolved = [d for d in debt_items if d["mitigation"] == "none"]
    return {"ethical_debt_items": len(debt_items), "unresolved": len(unresolved),
            "debt_severity": "HIGH" if len(unresolved) > 5 else "MODERATE" if unresolved else "LOW"}

def stakeholder_harm_assessment(dataset, stakeholder_groups):
    """Who is harmed by these data practices?"""
    harms = {}
    for group in stakeholder_groups:
        harms[group] = {"privacy_risk": _assess_privacy_risk(dataset, group),
                         "representation_risk": _assess_representation(dataset, group),
                         "misuse_risk": _assess_misuse_potential(dataset, group)}
    return harms
```

## Data Futures

```python
def construct_scenario_data(current_data, scenario_parameters, time_horizon_years=10):
    """Build datasets for imagined futures — what would data look like if X happens?"""
    scenarios = {}
    for scenario_name, params in scenario_parameters.items():
        # Project current patterns forward with modified assumptions
        projected = _project_temporal_patterns(current_data, time_horizon_years, params)
        scenarios[scenario_name] = {"data": projected, "assumptions": params,
                                     "validity_window_years": _estimate_projection_validity(params)}
    return scenarios

def assess_legacy_responsibility(dataset, future_impact_assessment):
    """What responsibility do we have to future data subjects?"""
    impacts = {}
    for impact_area, assessment in future_impact_assessment.items():
        impacts[impact_area] = {"current_mitigation": assessment.get("mitigation", "none"),
                                 "future_harm_risk": assessment.get("risk_level", "unknown"),
                                 "reversibility": assessment.get("reversible", False)}
    irreversible = [k for k, v in impacts.items() if not v["reversibility"] and v["future_harm_risk"] == "high"]
    return {"irreversible_high_risk_impacts": irreversible,
            "recommendation": "HALT_AND_MITIGATE" if irreversible else "MONITOR"}
```

## Quality Gate

- Cultural coverage > 50% for all relevant dimensions.
- Digital preservation score > 0.7, zero format obsolescence risks.
- Emotional annotation reliability: κ > 0.6 across ≥ 2 annotators.
- Unresolved ethical debt < 5 items.
- Irreversible high-risk future impacts documented with mitigation plan.
