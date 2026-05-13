---
name: data-resilience-integration-task
description: Design data architectures that survive organizational disruption — M&A data integration, leadership change continuity, bankruptcy data preservation, technology obsolescence migration, and chaos engineering for data pipelines. Data must outlast the organization that created it.
recommended_skills: [data-pipeline-monitoring-task, dataset-versioning, data-lifecycle-governance-task]
recommended_guidelines: [data-lifecycle-governance-task, data-ecosystem-integration-task, data-sustainability-integration-task]
---

## Overview

Organizations die. Mergers happen. Leadership changes. Technology stacks become obsolete. But data must survive. This guideline treats organizational resilience as a data architecture property: design M&A data integration protocols, build leadership change continuity mechanisms, preserve data through bankruptcy, plan technology migration paths, and chaos-engineer data pipelines to find failure modes before they find you.

## M&A Data Integration

```python
MA_DATA_INTEGRATION_PHASES = {
    "pre_diligence": {
        "actions": ["data_inventory_anonymized_for_sharing", "data_quality_baseline_established",
                     "regulatory_red_flags_identified", "open_source_license_audit",
                     "data_subject_consent_coverage_assessed"],
        "duration_weeks": 2,
        "output": "data_due_diligence_report with risk assessment",
    },
    "integration_planning": {
        "actions": ["ontology_reconciliation", "schema_mapping_matrix", "data_quality_compatibility_check",
                     "privacy_posture_alignment", "retention_policy_harmonization",
                     "access_control_model_merge", "pipeline_redundancy_resolution"],
        "duration_weeks": 4,
        "output": "integration_architecture with cost estimates and timeline",
    },
    "execution": {
        "actions": ["low_risk_data_first", "pipeline_parallel_run", "quality_comparison",
                     "stakeholder_validation_at_each_milestone", "rollback_plan_active"],
        "duration_weeks": 8,
        "output": "unified_data_platform with reconciled data",
    },
    "post_merge_validation": {
        "actions": ["business_metric_reconciliation", "regulatory_compliance_re_verification",
                     "data_subject_rights_portability", "redundancy_retirement"],
        "duration_weeks": 4,
        "output": "validated_unified_state with decommissioning plan for legacy",
    },
}

def ma_data_readiness(organization_data_catalog):
    checks = {
        "inventory_complete": bool(organization_data_catalog),
        "quality_baseline": all(d.get("quality_score") is not None for d in organization_data_catalog.values()),
        "consent_audit": all(d.get("consent_status") != "unknown" for d in organization_data_catalog.values()),
        "license_audit": all(d.get("license") != "unknown" for d in organization_data_catalog.values()),
        "retention_documented": all(d.get("retention_policy") is not None for d in organization_data_catalog.values()),
        "access_control_documented": bool(organization_data_catalog.get("access_control_model")),
        "pipeline_architecture_documented": bool(organization_data_catalog.get("pipeline_topology")),
    }
    
    ready_count = sum(1 for v in checks.values() if v)
    return {
        "checks": checks,
        "readiness_score": ready_count / len(checks),
        "ready_for_diligence": ready_count >= 5,
        "critical_gaps": [k for k, v in checks.items() if not v and k in [
            "inventory_complete", "quality_baseline", "consent_audit"
        ]],
    }
```

## Leadership Change Continuity

```python
LEADERSHIP_CONTINUITY_ARTIFACTS = {
    "data_strategy_document": {
        "contents": ["current_data_architecture", "strategic_rationale_for_choices",
                      "investment_commitments", "vendor_relationships", "team_structure_and_rationale"],
        "shelf_life": "2_years_or_until_next_major_strategy_update",
        "succession_value": "Prevents new leadership from undoing data investments they don't understand",
    },
    "data_decision_log": {
        "contents": ["decision_description", "context_at_time", "alternatives_considered",
                      "who_decided", "when_revisit", "reversal_cost_estimate"],
        "shelf_life": "permanent",
        "succession_value": "New leadership can understand WHY decisions were made, not just WHAT was done",
    },
    "data_vendor_relationship_map": {
        "contents": ["vendor", "contract_value", "contract_term", "renewal_deadline",
                      "integration_depth", "replacement_difficulty", "relationship_owner"],
        "shelf_life": "updated_quarterly",
        "succession_value": "Prevents vendor relationship disruption during transition",
    },
    "data_team_capability_matrix": {
        "contents": ["team_member", "critical_skills", "bus_factor_per_skill",
                      "succession_candidate", "irreplaceable_knowledge"],
        "shelf_life": "updated_quarterly",
        "succession_value": "Prevents capability loss when key people leave with the old leader",
    },
}

def leadership_change_readiness(organization):
    artifacts_present = []
    artifacts_missing = []
    for artifact_id, artifact in LEADERSHIP_CONTINUITY_ARTIFACTS.items():
        if organization.get(f"has_{artifact_id}", False):
            artifacts_present.append(artifact_id)
        else:
            artifacts_missing.append({
                "artifact": artifact_id,
                "contents": artifact["contents"],
                "risk": "NEW_LEADERSHIP_MAY_REVERSE_UNINFORMED" if artifact_id == "data_strategy_document"
                        else "DECISIONS_MADE_WITHOUT_CONTEXT" if artifact_id == "data_decision_log"
                        else "VENDOR_DISRUPTION" if artifact_id == "data_vendor_relationship_map"
                        else "CAPABILITY_LOSS",
            })
    
    return {
        "artifacts_present": artifacts_present,
        "artifacts_missing": artifacts_missing,
        "readiness_score": len(artifacts_present) / len(LEADERSHIP_CONTINUITY_ARTIFACTS),
        "ready": len(artifacts_missing) == 0,
    }
```

## Bankruptcy Data Preservation

```python
BANKRUPTCY_DATA_SCENARIOS = {
    "restructuring": {
        "data_priority": "preserve_core_operational_data — everything needed to run the restructured business",
        "retention_strategy": "identify_minimum_viable_data, tier_rest_to_cold_storage, document_what_was_discarded",
        "regulatory_requirements": "preserve data subject to litigation hold, regulatory retention, audit requirements",
    },
    "acquisition": {
        "data_priority": "preserve_data_with_transferable_value — customer data, IP, models, training data",
        "retention_strategy": "attach_data_assets_to_acquirable_entities, document_transfer_restrictions",
        "regulatory_requirements": "data_transfer_subject_to_consent, cross_border_restrictions, antitrust_review",
    },
    "liquidation": {
        "data_priority": "secure_deletion_first — customer data, employee data; preserve public_value_data",
        "retention_strategy": "delete_personal_data_per_consent_and_law, release_public_datasets_to_archive",
        "regulatory_requirements": "certify_deletion, notify_data_subjects, transfer_to_archival_institution",
    },
}

def bankruptcy_data_plan(organization_data, bankruptcy_scenario):
    scenario = BANKRUPTCY_DATA_SCENARIOS.get(bankruptcy_scenario, BANKRUPTCY_DATA_SCENARIOS["liquidation"])
    
    data_triage = {"preserve": [], "cold_store": [], "delete": [], "transfer": []}
    
    for dataset_id, dataset in organization_data.items():
        if dataset.get("regulatory_hold") or dataset.get("litigation_hold"):
            data_triage["preserve"].append(dataset_id)
        elif dataset.get("personal_data") and bankruptcy_scenario == "liquidation":
            data_triage["delete"].append(dataset_id)
        elif dataset.get("transferable_asset"):
            data_triage["transfer"].append(dataset_id)
        elif dataset.get("operational_critical"):
            data_triage["preserve"].append(dataset_id)
        else:
            data_triage["cold_store"].append(dataset_id)
    
    return {
        "scenario": bankruptcy_scenario,
        "triage": data_triage,
        "summary": {
            "preserve": len(data_triage["preserve"]),
            "cold_store": len(data_triage["cold_store"]),
            "delete": len(data_triage["delete"]),
            "transfer": len(data_triage["transfer"]),
        },
        "regulatory_checklist": scenario["regulatory_requirements"],
    }
```

## Technology Obsolescence Migration

```python
OBSOLESCENCE_RISK_FACTORS = {
    "format": {"risk_window_years": 10, "indicators": ["proprietary_format", "single_vendor_support",
                 "no_open_source_reader", "declining_ecosystem"],
               "mitigation": "migrate_to_open_format_while_tools_still_exist"},
    "database": {"risk_window_years": 7, "indicators": ["vendor_acquired", "cloud_only_migration_path",
                  "license_change", "end_of_life_announced"],
                 "mitigation": "export_to_open_format, validate_export_fidelity"},
    "pipeline_framework": {"risk_window_years": 5, "indicators": ["community_abandonment",
                            "no_new_releases_12_months", "security_vulnerabilities_unfixed"],
                           "mitigation": "port_pipeline_logic_to_successor_framework"},
    "cloud_provider": {"risk_window_years": 3, "indicators": ["price_increase_trajectory",
                        "service_deprecation_pattern", "market_position_decline"],
                       "mitigation": "multi_cloud_abstraction, avoid_proprietary_service_lock_in"},
}

def obsolescence_audit(data_infrastructure):
    at_risk = []
    for component_id, component in data_infrastructure.items():
        component_type = component.get("type")
        risk_profile = OBSOLESCENCE_RISK_FACTORS.get(component_type)
        if risk_profile is None:
            continue
        
        risk_indicators = []
        for indicator in risk_profile["indicators"]:
            if component.get(indicator, False):
                risk_indicators.append(indicator)
        
        if risk_indicators:
            at_risk.append({
                "component": component_id,
                "type": component_type,
                "risk_window_years": risk_profile["risk_window_years"],
                "indicators_active": risk_indicators,
                "urgency": "IMMEDIATE" if len(risk_indicators) >= 3
                           else "HIGH" if len(risk_indicators) >= 2
                           else "PLAN",
                "mitigation": risk_profile["mitigation"],
            })
    
    return {
        "components_at_risk": at_risk,
        "total_at_risk": len(at_risk),
        "critical_migrations": [a for a in at_risk if a["urgency"] == "IMMEDIATE"],
        "migration_backlog_cost": sum(
            _estimate_migration_cost(a) for a in at_risk
        ),
    }
```

## Chaos Engineering for Data Pipelines

```python
CHAOS_EXPERIMENTS = {
    "schema_change": {
        "injection": "Add/remove/rename column in source system without notice",
        "expected_behavior": "Pipeline fails gracefully with clear error — not silent data corruption",
        "recovery_check": "Pipeline resumes correctly after schema fix — no data loss, no duplicates",
    },
    "network_partition": {
        "injection": "Block network between pipeline and one data source for 30 minutes",
        "expected_behavior": "Pipeline retries with backoff, alerts after threshold, no partial data committed",
        "recovery_check": "All missed data backfilled correctly when partition heals",
    },
    "disk_pressure": {
        "injection": "Fill disk to 95% during pipeline execution",
        "expected_behavior": "Pipeline detects low disk, pauses gracefully, alerts",
        "recovery_check": "Pipeline resumes without data corruption when disk freed",
    },
    "clock_skew": {
        "injection": "Shift system clock by 2 hours during pipeline execution",
        "expected_behavior": "Pipeline uses monotonic clock for ordering, wall clock for display only",
        "recovery_check": "No data reordered, no duplicates, partitioning correct",
    },
    "data_volume_spike": {
        "injection": "10x normal data volume in one ingestion window",
        "expected_behavior": "Pipeline degrades gracefully: backpressure, not crash; alert triggered",
        "recovery_check": "All data processed correctly, no silent drops",
    },
}

def chaos_experiment_runner(pipeline, experiment_type, blast_radius_controls):
    experiment = CHAOS_EXPERIMENTS.get(experiment_type)
    if experiment is None:
        return {"error": "unknown_experiment_type"}
    
    result = {
        "experiment": experiment_type,
        "injection": experiment["injection"],
        "blast_radius": blast_radius_controls,
        "expected": experiment["expected_behavior"],
        "actual": None,
        "passed": False,
        "recovery": None,
    }
    
    # Execute with blast radius controls
    if blast_radius_controls.get("production", False):
        result["skipped"] = "chaos_in_production_requires_separate_approval"
        return result
    
    outcome = _run_chaos_experiment(pipeline, experiment)
    result["actual"] = outcome["behavior"]
    result["passed"] = outcome["matched_expected"]
    result["recovery"] = outcome["recovery_behavior"]
    result["data_integrity_check"] = outcome["data_integrity_verified"]
    
    return result
```

## Quality Gate

- M&A data readiness score ≥ 0.7 — inventory complete, quality baselined, consent audited.
- Leadership change artifacts all present — data strategy, decision log, vendor map, capability matrix.
- Bankruptcy data plan triaged for all datasets — preservation, deletion, and transfer explicit.
- Technology obsolescence audit completed — critical migrations scheduled before risk window closes.
- Chaos experiments run quarterly on staging — all pipelines pass schema, partition, disk, clock, volume tests.
