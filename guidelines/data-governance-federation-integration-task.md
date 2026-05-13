---
name: data-governance-federation-integration-task
description: Extend data governance across complex organizational structures — subsidiary autonomy vs group control, joint venture data partitioning, franchise data consistency, matrix organization data ownership, and holding company data consolidation. Governance without federation is either tyranny or chaos.
recommended_skills: [dataset-governance-task, data-lifecycle-governance-task, dataset-versioning]
recommended_guidelines: [dataset-governance-task, data-lifecycle-governance-task, data-ecosystem-integration-task]
---

## Overview

Simple organizations have simple governance. Complex organizations — holding companies, matrix structures, joint ventures, franchise networks — need federated governance that balances local autonomy with group consistency. Too much centralization kills local innovation. Too much autonomy creates data chaos and regulatory exposure. This guideline designs governance federation architectures for each organizational topology.

## Topology → Governance Architecture

```python
GOVERNANCE_TOPOLOGIES = {
    "holding_company": {
        "description": "Parent owns subsidiaries; subsidiaries operate independently",
        "governance_model": "minimum_standards_with_local_implementation",
        "central_mandates": ["regulatory_compliance_minimum", "financial_reporting_data",
                              "risk_exposure_aggregation", "data_breach_notification"],
        "local_autonomy": ["operational_data_standards", "tool_selection", "pipeline_architecture",
                           "data_team_structure", "local_regulatory_adaptation"],
        "federation_mechanism": "subsidiary_data_governance_council_with_parent_veto_on_compliance",
        "risk_patterns": ["subsidiary_ignores_minimum_standards", "parent_overreach_kills_innovation"],
    },
    "joint_venture": {
        "description": "Two+ parent orgs create separate entity with shared data",
        "governance_model": "contractual_partition_with_shared_custodian",
        "central_mandates": ["data_purpose_limitation", "intellectual_property_boundaries",
                              "competitive_data_firewalls", "dissolution_data_partition_plan"],
        "local_autonomy": ["operational_execution_within_partition", "tooling_within_partition"],
        "federation_mechanism": "independent_data_custodian_with_parent_audit_rights",
        "risk_patterns": ["data_leakage_between_parents", "custodian_capture_by_one_parent",
                          "dissolution_without_data_partition_plan"],
    },
    "franchise": {
        "description": "Franchisor provides brand/process; franchisees operate locally",
        "governance_model": "standardized_core_with_local_execution_variance",
        "central_mandates": ["customer_data_protection_standard", "brand_consistency_data",
                              "financial_reporting_schema", "minimum_data_quality_threshold"],
        "local_autonomy": ["local_marketing_data", "local_vendor_data", "local_employment_data"],
        "federation_mechanism": "certification_body_audits + shared_platform_with_local_instances",
        "risk_patterns": ["franchisee_data_silos", "inconsistent_customer_experience_data",
                          "brand_reputation_contagion_from_one_franchisee"],
    },
    "matrix_organization": {
        "description": "Employees report to functional AND business unit leaders",
        "governance_model": "dual_ownership_with_tiebreaking_mechanism",
        "central_mandates": ["data_quality_standards", "data_access_policy", "master_data_management"],
        "local_autonomy": ["business_unit_specific_data", "analytic_priorities", "experimentation"],
        "federation_mechanism": "data_stewardship_council_with_functional_and_business_representation",
        "risk_patterns": ["conflicting_data_definitions", "ownership_paralysis",
                          "data_hoarding_by_business_unit"],
    },
}

def topology_governance_fit(organization_structure, current_governance):
    topology = GOVERNANCE_TOPOLOGIES.get(organization_structure.get("type"))
    if topology is None:
        return {"error": "unknown_topology", "type": organization_structure.get("type")}
    
    gaps = []
    for mandate in topology["central_mandates"]:
        if not current_governance.get(mandate):
            gaps.append({"type": "missing_central_mandate", "requirement": mandate,
                        "risk": "REGULATORY_EXPOSURE" if "compliance" in mandate or "regulatory" in mandate
                                else "OPERATIONAL_RISK"})
    
    for autonomy in topology["local_autonomy"]:
        if current_governance.get(f"central_controls_{autonomy}"):
            gaps.append({"type": "overreach", "autonomy_violated": autonomy,
                        "risk": "INNOVATION_SUPPRESSION"})
    
    return {
        "topology": organization_structure.get("type"),
        "recommended_model": topology["governance_model"],
        "federation_mechanism": topology["federation_mechanism"],
        "gaps": gaps,
        "alignment_score": 1 - len(gaps) / max(len(topology["central_mandates"]) + len(topology["local_autonomy"]), 1),
        "risk_patterns_active": [r for r in topology["risk_patterns"]
                                 if current_governance.get(f"risk_{r}", False)],
    }
```

## Federation Mechanism Design

```python
FEDERATION_DECISION_RIGHTS = {
    "exclusive_central": ["regulatory_compliance_interpretation", "data_breach_response_protocol",
                          "financial_data_definition", "cross_entity_data_sharing_approval"],
    "exclusive_local": ["local_data_collection_priorities", "local_analytic_methods",
                         "local_tooling_selection", "local_data_team_hiring"],
    "shared_with_central_tiebreaker": ["data_quality_standards", "data_retention_periods",
                                        "master_data_definitions"],
    "shared_with_local_opt_up": ["security_controls_above_minimum", "privacy_controls_above_minimum"],
    "shared_with_consensus_required": ["federation_governance_charter_amendment",
                                        "new_central_mandate_addition"],
}

def decision_rights_clarity(governance_charter):
    clarity_issues = []
    for category, decisions in FEDERATION_DECISION_RIGHTS.items():
        for decision in decisions:
            assigned = governance_charter.get(decision)
            if assigned is None:
                clarity_issues.append({"decision": decision, "issue": "not_assigned"})
            elif assigned != category:
                clarity_issues.append({"decision": decision, "issue": "misassigned",
                                       "assigned": assigned, "recommended": category})
    
    return {
        "total_decisions": sum(len(v) for v in FEDERATION_DECISION_RIGHTS.values()),
        "issues": clarity_issues,
        "clarity_score": 1 - len(clarity_issues) / max(
            sum(len(v) for v in FEDERATION_DECISION_RIGHTS.values()), 1),
    }
```

## Quality Gate

- Governance topology matches organizational structure — no holding company run as a franchise.
- Central mandates cover compliance and risk; local autonomy covers innovation and execution.
- Decision rights explicitly assigned — no unassigned governance decisions.
- Federation mechanism has working council with clear escalation paths.
- Risk patterns actively monitored with mitigation plans.
