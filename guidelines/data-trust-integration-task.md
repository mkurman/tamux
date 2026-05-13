---
name: data-trust-integration-task
description: Engineer verifiable trust into data systems — cryptographic provenance, reputation systems for data sources, attestation protocols, zero-knowledge proofs for data claims, and trust decay models. Trust is not assumed; it is verified, measured, and maintained.
recommended_skills: [dataset-versioning, data-card-writer, benchmark-contamination-scan]
recommended_guidelines: [dataset-governance-task, data-ethics-integration-task, privacy-preserving-data-task]
---

## Overview

"We trust our data" is the most expensive sentence in data engineering. Trust must be verified, not assumed. Cryptographic provenance chains prove data hasn't been tampered with. Reputation systems track which data sources deliver on their promises. Attestation protocols let data consumers verify claims without trusting the provider. Zero-knowledge proofs enable verification without revelation. And trust decay models recognize that trust is not permanent — it erodes without maintenance. This guideline engineers trust as a measurable, verifiable property of data systems.

## Cryptographic Provenance

```python
PROVENANCE_CHAIN = {
    "collection": {"hash": "sha256_of_raw_data", "timestamp": "collection_time",
                   "collector": "agent_or_system_id", "method": "collection_methodology"},
    "transformation": {"input_hash": "hash_of_pre_transformation_data",
                       "output_hash": "hash_of_post_transformation_data",
                       "transform": "transform_identifier_and_version",
                       "parameters": "transform_parameters_hash"},
    "certification": {"dataset_hash": "hash_of_certified_version",
                      "certifier": "certifying_entity_id",
                      "standard": "certification_standard_and_version",
                      "validity_period": "certification_validity_window"},
}

def verify_provenance_chain(dataset, expected_provenance):
    """Cryptographically verify that a dataset matches its provenance claims."""
    verification = {"steps": [], "chain_intact": True}
    
    current_hash = dataset.get("content_hash")
    
    # Walk backward through provenance steps
    for step in reversed(expected_provenance.get("steps", [])):
        step_type = step.get("type")
        if step_type == "transformation":
            expected_output = step.get("output_hash")
            if current_hash != expected_output:
                verification["steps"].append({
                    "step": step["transform"],
                    "status": "FAILED — output hash mismatch",
                    "expected": expected_output,
                    "actual": current_hash,
                })
                verification["chain_intact"] = False
            current_hash = step.get("input_hash")
        elif step_type == "collection":
            if current_hash != step.get("raw_data_hash"):
                verification["steps"].append({
                    "step": "collection",
                    "status": "FAILED — raw data hash mismatch",
                })
                verification["chain_intact"] = False
    
    verification["root_verified"] = verification["chain_intact"]
    return verification
```

## Reputation Systems for Data Sources

```python
REPUTATION_DIMENSIONS = {
    "freshness": {"weight": 0.25, "metrics": ["delivery_on_schedule_ratio", "staleness_days_p95",
                  "update_cadence_adherence"],
                  "decay_rate": "fast — 30 day half-life"},
    "quality": {"weight": 0.30, "metrics": ["schema_compliance_ratio", "null_rate_vs_promise",
                "distribution_match_to_baseline", "contamination_incidents"],
                "decay_rate": "medium — 90 day half-life"},
    "accuracy": {"weight": 0.25, "metrics": ["ground_truth_alignment", "error_rate_trend",
                  "correction_latency", "user_reported_issue_rate"],
                 "decay_rate": "medium — 90 day half-life"},
    "service": {"weight": 0.20, "metrics": ["availability_uptime", "response_time_p95",
                 "incident_resolution_time", "breaking_change_notice_period"],
                "decay_rate": "fast — 30 day half-life"},
}

def compute_source_reputation(source_id, historical_performance, current_weight=1.0):
    reputation = {}
    total_score = 0
    
    for dimension, config in REPUTATION_DIMENSIONS.items():
        dimension_metrics = {}
        for metric in config["metrics"]:
            raw_value = historical_performance.get(f"{source_id}_{metric}", 0)
            decay = _compute_decay(historical_performance, metric, config["decay_rate"])
            dimension_metrics[metric] = {"raw": raw_value, "decayed": raw_value * decay}
        
        dim_score = sum(m["decayed"] for m in dimension_metrics.values()) / max(len(dimension_metrics), 1)
        reputation[dimension] = {
            "score": dim_score,
            "weighted": dim_score * config["weight"],
            "metrics": dimension_metrics,
        }
        total_score += dim_score * config["weight"]
    
    reputation["overall"] = total_score * current_weight
    reputation["trust_tier"] = "GOLD" if total_score > 0.9 \
                                else "SILVER" if total_score > 0.7 \
                                else "BRONZE" if total_score > 0.5 \
                                else "UNTRUSTED"
    
    return reputation

def _compute_decay(history, metric, decay_rate_str):
    last_measurement_days = history.get(f"{metric}_last_measured_days_ago", 0)
    half_life = 30 if "fast" in decay_rate_str else 90 if "medium" in decay_rate_str else 180
    return 0.5 ** (last_measurement_days / half_life)
```

## Attestation Protocols

```python
ATTESTATION_TYPES = {
    "data_quality": {
        "claim": "Dataset meets quality standard X at level Y",
        "evidence": ["quality_report_hash", "audit_log_hash", "certification_body_signature"],
        "verification": "consumer_verifies_attestation_signature_and_recomputes_spot_checks",
        "revocation": "attestation_revoked_if_quality_drops_below_threshold_or_audit_fails",
    },
    "privacy_compliance": {
        "claim": "Dataset processing complies with regulation Z",
        "evidence": ["dpia_hash", "consent_registry_root", "processor_agreement_hashes"],
        "verification": "auditor_attests; consumer_verifies_auditor_credential",
        "revocation": "attestation_revoked_if_breach_or_consent_withdrawal_or_regulatory_finding",
    },
    "bias_audit": {
        "claim": "Model trained on dataset passes fairness audit at threshold T",
        "evidence": ["bias_audit_report_hash", "evaluation_data_hash", "methodology_document_hash"],
        "verification": "consumer_verifies_audit_standards_and_recomputes_on_spot_check",
        "revocation": "attestation_revoked_if_bias_exceeds_threshold_on_new_evaluation",
    },
    "provenance": {
        "claim": "Data lineage from raw source to current version is complete and verified",
        "evidence": ["full_provenance_chain_hashes", "transformation_audit_log"],
        "verification": "consumer_replays_transformations_on_sample_and_verifies_output_hash",
        "revocation": "attestation_revoked_if_chain_break_detected",
    },
}

def attestation_verification(attestation, current_data_hash):
    att_type = ATTESTATION_TYPES.get(attestation.get("type"))
    if att_type is None:
        return {"verified": False, "reason": "unknown_attestation_type"}
    
    if attestation.get("revoked"):
        return {"verified": False, "reason": "attestation_revoked",
                "revocation_reason": attestation.get("revocation_reason")}
    
    if attestation.get("dataset_hash") != current_data_hash:
        return {"verified": False, "reason": "dataset_hash_mismatch",
                "attested_hash": attestation.get("dataset_hash"),
                "current_hash": current_data_hash}
    
    signature_valid = _verify_signature(attestation)
    if not signature_valid:
        return {"verified": False, "reason": "signature_invalid"}
    
    return {"verified": True, "claim": att_type["claim"],
            "attestor": attestation.get("issuer"), "issued": attestation.get("timestamp")}
```

## Zero-Knowledge Proofs for Data Claims

```python
ZK_USE_CASES = {
    "age_verification": {
        "prove": "User is over minimum age",
        "without_revealing": "exact birth date, name, address",
        "zk_system": "zk-SNARK over government ID signature",
        "practical_status": "production_ready",
    },
    "model_training_data_compliance": {
        "prove": "Training data meets regulatory requirements",
        "without_revealing": "training data itself",
        "zk_system": "recursive SNARK over data pipeline",
        "practical_status": "research_to_production",
    },
    "fairness_attribute_absence": {
        "prove": "Model decisions don't use prohibited attributes",
        "without_revealing": "sensitive attributes or full model weights",
        "zk_system": "zk-SNARK over model inference trace",
        "practical_status": "research_phase",
    },
}

def zk_readiness_assessment(use_case, organization_capability):
    zk_case = ZK_USE_CASES.get(use_case)
    if zk_case is None:
        return {"feasible": False, "reason": "no_known_zk_construction"}
    
    return {
        "use_case": use_case,
        "practical_status": zk_case["practical_status"],
        "prove_claim": zk_case["prove"],
        "without_revealing": zk_case["without_revealing"],
        "recommended_system": zk_case["zk_system"],
        "organization_ready": zk_case["practical_status"] == "production_ready",
        "research_investment_needed": zk_case["practical_status"] != "production_ready",
    }
```

## Trust Decay Models

```python
def trust_decay(trust_score, time_since_verification_days, data_criticality):
    """Trust decays without verification. Criticality accelerates perceived risk."""
    base_half_life_days = {"GOLD": 90, "SILVER": 60, "BRONZE": 30, "UNTRUSTED": 0}
    tier = "GOLD" if trust_score > 0.9 else "SILVER" if trust_score > 0.7 \
            else "BRONZE" if trust_score > 0.5 else "UNTRUSTED"
    half_life = base_half_life_days[tier]
    
    criticality_multiplier = 2.0 if data_criticality == "critical" else 1.0
    effective_half_life = half_life / criticality_multiplier
    
    decayed = trust_score * (0.5 ** (time_since_verification_days / effective_half_life))
    
    return {
        "original_score": trust_score,
        "days_since_verification": time_since_verification_days,
        "data_criticality": data_criticality,
        "effective_half_life_days": effective_half_life,
        "decayed_score": decayed,
        "reverification_needed": decayed < 0.5,
        "reverification_urgency": "IMMEDIATE" if decayed < 0.3
                                   else "SCHEDULE" if decayed < 0.5
                                   else "ROUTINE",
    }
```

## Quality Gate

- Every dataset has a cryptographic provenance chain; chain integrity verified before use.
- All data sources have reputation scores across freshness, quality, accuracy, service.
- Critical datasets carry attestations for quality, privacy, bias, and provenance.
- ZK-readiness assessed for privacy-preserving verification use cases.
- Trust decay monitored — re-verification triggered before decayed score drops below 0.5.
- Trust tier (GOLD/SILVER/BRONZE/UNTRUSTED) displayed in data catalog and enforced in pipelines.
