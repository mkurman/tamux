---
name: data-ethics-integration-task
description: Integrate ethical frameworks into data practice beyond compliance checklists — multi-stakeholder harm taxonomy, consent architecture, fairness beyond metrics, algorithmic redress mechanisms, and ethics-by-design gates. Ethics is a data architecture property, not a training module.
recommended_skills: [bias-audit, data-card-writer, label-quality-audit]
recommended_guidelines: [dataset-governance-task, privacy-preserving-data-task, data-culture-integration-task]
---

## Overview

Compliance is the floor, not the ceiling. GDPR checkboxes don't make data ethical any more than building codes make architecture beautiful. This guideline treats ethics as a data architecture property: map stakeholders and their harms, architect consent that works beyond the checkbox, design fairness evaluation that captures what metrics miss, build redress mechanisms that actually work, and embed ethics gates in the data pipeline so they can't be skipped under schedule pressure.

## Multi-Stakeholder Harm Taxonomy

```python
HARM_TAXONOMY = {
    "individual": {
        "privacy": ["re-identification_risk", "inferred_sensitive_attribute", "surveillance_amplification",
                     "data_broker_propagation", "consent_bypass"],
        "autonomy": ["manipulative_personalization", "choice_architecture_exploitation",
                      "information_asymmetry_weaponization", "nudge_dark_patterns"],
        "dignity": ["dehumanizing_categorization", "predictive_stereotyping",
                     "reductive_representation", "identity_erasure"],
        "material": ["discriminatory_pricing", "opportunity_denial", "credit_invisibility",
                      "insurance_redlining", "employment_screening_bias"],
    },
    "group": {
        "representation": ["underrepresentation_in_training_data", "overrepresentation_in_negative_examples",
                           "stereotypical_associations", "cultural_erasure"],
        "distributive": ["benefit_concentration", "harm_concentration", "resource_diversion",
                          "attention_extraction"],
        "procedural": ["excluded_from_design", "excluded_from_evaluation", "excluded_from_redress",
                        "no_meaningful_consent_mechanism"],
    },
    "societal": {
        "epistemic": ["truth_decay", "information_environment_degradation", "synthetic_content_flooding",
                       "evidence_standards_erosion"],
        "power": ["surveillance_capitalism_entrenchment", "asymmetric_power_amplification",
                   "regulatory_capture_enablement", "democratic_process_interference"],
        "ecological": ["compute_carbon_cost_externalization", "e_waste_from_model_churn",
                        "resource_extraction_for_hardware", "energy_inequality"],
    },
}

def stakeholder_harm_analysis(data_product, affected_stakeholders):
    """Map every stakeholder group to potential harms BEFORE data is collected."""
    harm_map = {}
    for stakeholder in affected_stakeholders:
        stakeholder_harms = []
        stype = stakeholder.get("type", "individual")
        harm_categories = HARM_TAXONOMY.get(stype, {})
        
        for category, harms in harm_categories.items():
            for harm in harms:
                if _harm_applies(data_product, harm, stakeholder):
                    stakeholder_harms.append({
                        "category": category,
                        "harm": harm,
                        "likelihood": _estimate_likelihood(data_product, harm),
                        "severity": _estimate_severity(harm, stakeholder),
                        "mitigation": _mitigation_strategy(data_product, harm),
                    })
        
        harm_map[stakeholder["id"]] = {
            "type": stype,
            "harms": stakeholder_harms,
            "total_harms": len(stakeholder_harms),
            "unmitigated": [h for h in stakeholder_harms if not h["mitigation"]],
            "severity_max": max((h["severity"] for h in stakeholder_harms), default=0),
        }
    
    return harm_map
```

## Consent Architecture

```python
CONSENT_LEVELS = {
    "L0_no_consent": "Data should not exist — do not collect, even if legal",
    "L1_blanket": "One-time consent at account creation — insufficient for evolving use cases",
    "L2_purpose_specific": "Consent tied to specific purpose — 'we will use this data for X only'",
    "L3_granular": "Per-use consent — 'do you consent to this specific use of your data?'",
    "L4_dynamic": "Consent can be withdrawn at any time with automatic data deletion and downstream propagation",
    "L5_empowering": "Data subjects control and benefit from their data — data cooperatives, data dividends",
}

def consent_audit(data_product, consent_practices):
    """Where is consent on the L0-L5 spectrum and what's missing?"""
    current_level = consent_practices.get("level", "L1_blanket")
    required = data_product.get("minimum_consent_level", "L2_purpose_specific")
    
    level_value = {"L0": 0, "L1": 1, "L2": 2, "L3": 3, "L4": 4, "L5": 5}
    current_val = level_value.get(current_level.split("_")[0], 0)
    required_val = level_value.get(required.split("_")[0], 0)
    
    return {
        "current_level": current_level,
        "required_level": required,
        "gap": required_val - current_val,
        "sufficient": current_val >= required_val,
        "upgrade_path": [f"L{i}" for i in range(current_val + 1, required_val + 1)]
                        if current_val < required_val else [],
    }
```

## Fairness Beyond Metrics

```python
FAIRNESS_DIMENSIONS = {
    "representation": {
        "metric": "demographic_parity_difference",
        "blind_spot": "Equal representation can hide equal mistreatment — everyone equally poorly served",
        "beyond_metric": "Are groups represented in the DESIGN process, not just the training data?",
    },
    "error_equality": {
        "metric": "equalized_odds_difference",
        "blind_spot": "Equal error rates can hide different ERROR TYPES — one group gets false positives, another false negatives",
        "beyond_metric": "Are error types equally distributed? Is the cost of errors equal across groups?",
    },
    "calibration": {
        "metric": "calibration_by_group",
        "blind_spot": "Well-calibrated models can be perfectly discriminatory — base rate differences encoded honestly",
        "beyond_metric": "Are base rate differences themselves the result of historical discrimination that the model perpetuates?",
    },
    "procedural": {
        "metric": "none — not measurable by output metrics",
        "blind_spot": "Fair outputs from an unfair process are not ethically sound",
        "beyond_metric": "Were affected communities involved in problem definition, metric selection, and deployment decisions?",
    },
    "structural": {
        "metric": "none — not measurable by output metrics",
        "blind_spot": "A fair model in an unfair system is harm laundering",
        "beyond_metric": "Does this model reinforce or challenge existing structural inequalities?",
    },
}

def fairness_audit(model, evaluation_data, process_documentation):
    results = {}
    for dimension, details in FAIRNESS_DIMENSIONS.items():
        if details["metric"] != "none — not measurable by output metrics":
            metric_value = _compute_fairness_metric(model, evaluation_data, details["metric"])
            results[dimension] = {
                "metric": details["metric"],
                "value": metric_value,
                "blind_spot": details["blind_spot"],
                "beyond_metric_question": details["beyond_metric"],
                "beyond_metric_answer": process_documentation.get(dimension, "NOT_DOCUMENTED"),
                "gap": "METRIC_OK_BUT_PROCESS_GAP" if metric_value < 0.05 and not process_documentation.get(dimension)
                       else "PASS" if metric_value < 0.05 and process_documentation.get(dimension)
                       else "FAIL" if metric_value >= 0.05
                       else "UNKNOWN",
            }
        else:
            results[dimension] = {
                "metric": "not_applicable",
                "blind_spot": details["blind_spot"],
                "beyond_metric_question": details["beyond_metric"],
                "answered": bool(process_documentation.get(dimension)),
                "gap": "PASS" if process_documentation.get(dimension) else "PROCEDURAL_GAP",
            }
    
    return {
        "dimensions": results,
        "overall": "ETHICAL_GAPS_PRESENT" if any(
            r.get("gap") in ["FAIL", "PROCEDURAL_GAP", "METRIC_OK_BUT_PROCESS_GAP"]
            for r in results.values()
        ) else "ETHICALLY_SOUND_BY_CURRENT_STANDARDS",
    }
```

## Algorithmic Redress

```python
REDRESS_ARCHITECTURE = {
    "detection": {
        "channels": ["in_product_appeal_button", "customer_support", "automated_disparity_monitoring",
                      "external_watchdog", "regulatory_complaint"],
        "acknowledgment_sla": "24_hours",
        "auto_escalation": "discrimination_claims → human_review",
    },
    "investigation": {
        "data_required": ["model_input_at_decision_time "model_version", "decision_path_or_explanation",
                          "comparable_cases", "policy_applied"],
        "timeline_sla": "5_business_days_for_initial_finding",
        "independent_review": "different_team_from_model_builders",
    },
    "remediation": {
        "types": ["decision_reversal", "model_retraining", "policy_change", "compensation",
                   "process_improvement", "public_acknowledgment"],
        "propagation": "fix should prevent recurrence for similarly situated individuals",
        "feedback_loop": "remediation_changes → model_update → verification → monitoring",
    },
    "transparency": {
        "to_affected": "explanation of what happened, why, and what was done about it",
        "to_organization": "anonymized_pattern_report for systemic learning",
        "to_regulator": "structured_incident_report within mandated timeline",
        "to_public": "transparency_report_published_annually",
    },
}

def redress_readiness(data_product):
    checklist = {
        "appeal_mechanism_exists": data_product.get("has_appeal_button", False),
        "decision_explanation_available": data_product.get("provides_explanation", False),
        "human_review_path": data_product.get("human_review_available", False),
        "data_retained_for_investigation": data_product.get("decision_data_retention_days", 0) >= 90,
        "independent_review_team": data_product.get("independent_review_exists", False),
        "remediation_tracking": data_product.get("remediation_tracker_exists", False),
        "annual_transparency_report": data_product.get("transparency_report_published", False),
    }
    
    ready_count = sum(1 for v in checklist.values() if v)
    return {
        "checklist": checklist,
        "readiness_score": ready_count / len(checklist),
        "ready": ready_count >= 5,
        "critical_gaps": [k for k, v in checklist.items() if not v and k in [
            "appeal_mechanism_exists", "decision_explanation_available", "human_review_path"
        ]],
    }
```

## Ethics-by-Design Pipeline Gates

```python
ETHICS_GATES = {
    "problem_definition": {
        "gate": "stakeholder_harm_analysis_completed",
        "blocks": ["data_collection", "model_design"],
        "cannot_skip": True,
        "evidence_required": "harm_analysis_document with stakeholder sign-off",
    },
    "data_collection": {
        "gate": "consent_architecture_validated + representation_audit_passed",
        "blocks": ["training"],
        "cannot_skip": True,
        "evidence_required": "consent_level >= L2_purpose_specific AND representation_gaps_documented",
    },
    "training": {
        "gate": "contamination_scan_clean + bias_audit_completed",
        "blocks": ["evaluation"],
        "cannot_skip": False,
        "evidence_required": "contamination_report + bias_audit_report",
    },
    "evaluation": {
        "gate": "fairness_audit_completed + robustness_tested + disaggregated_evaluation",
        "blocks": ["deployment"],
        "cannot_skip": True,
        "evidence_required": "fairness_audit_report + robustness_report + per_group_metrics",
    },
    "deployment": {
        "gate": "redress_mechanism_verified + monitoring_configured + rollback_plan_tested",
        "blocks": ["production_traffic"],
        "cannot_skip": True,
        "evidence_required": "redress_readiness >= 0.7 + monitoring_dashboard + rollback_dry_run",
    },
}

def ethics_gate_check(data_product, pipeline_stage):
    gate = ETHICS_GATES.get(pipeline_stage, {})
    passed = data_product.get(gate["gate"], False)
    
    if not passed and gate.get("cannot_skip", False):
        return {
            "stage": pipeline_stage,
            "gate": gate["gate"],
            "passed": False,
            "blocked": True,
            "blocked_stages": gate["blocks"],
            "resolution": f"Complete {gate['gate']} before proceeding. Evidence required: {gate['evidence_required']}",
            "override": "NOT_ALLOWED — this gate cannot be skipped",
        }
    
    return {
        "stage": pipeline_stage,
        "gate": gate["gate"],
        "passed": passed,
        "blocked": not passed,
        "evidence": gate.get("evidence_required", ""),
    }
```

## Quality Gate

- Stakeholder harm analysis completed and signed off before data collection.
- Consent architecture at L2 or above for all personal data products.
- Fairness audit covers all five dimensions, including procedural and structural.
- Redress readiness score ≥ 0.7 — appeal mechanism, explanation, human review all present.
- All five ethics-by-design pipeline gates passed; unskippable gates enforced in CI/CD.
- Annual transparency report published covering all data products.
