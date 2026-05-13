---
name: data-ecosystem-integration-task
description: Integrate data across organizational boundaries — partner data sharing agreements, open-source data community health, regulatory ecosystem navigation, standards body participation, and academic-industry data collaboration. An organization's data doesn't end at its firewall.
recommended_skills: [dataset-versioning, data-card-writer, data-pipeline-monitoring-task]
recommended_guidelines: [dataset-governance-task, data-ethics-integration-task, privacy-preserving-data-task]
---

## Overview

No organization's data is an island. Your models depend on open-source datasets maintained by burned-out PhD students. Your compliance depends on regulatory interpretations you didn't shape. Your competitive advantage depends on data partnerships with organizations that don't share your incentives. This guideline treats the data ecosystem as an architectural layer: engineer data partnerships with aligned incentives, monitor open-source dependency health, navigate regulatory ecosystems proactively, participate in standards bodies strategically, and build academic collaborations that produce real data value.

## Partner Data Sharing

```python
DATA_PARTNERSHIP_ARCHETYPES = {
    "bilateral_exchange": {
        "description": "Two organizations exchange data directly",
        "agreement_requirements": ["data_usage_purpose_binding", "derived_data_rights", 
                                    "termination_and_deletion", "liability_for_data_quality",
                                    "audit_rights", "breach_notification_timeline"],
        "technical_requirements": ["schema_compatibility_validation", "secure_transfer_protocol",
                                     "access_audit_logging", "revocation_mechanism"],
        "risk_factors": ["data_leakage_across_boundary", "purpose_creep", "competitive_intelligence_exposure"],
    },
    "data_consortium": {
        "description": "Multiple organizations pool data under shared governance",
        "agreement_requirements": ["governance_model", "voting_rights_by_contribution",
                                    "data_quality_minimum_standard", "exit_terms_and_data_rights",
                                    "antitrust_compliance", "intellectual_property_framework"],
        "technical_requirements": ["federated_access_control", "contribution_attribution",
                                     "shared_ontology", "privacy_preserving_aggregation"],
        "risk_factors": ["free_riding", "data_quality_heterogeneity", "competitive_collusion_concern"],
    },
    "data_marketplace": {
        "description": "Buy and sell data through a market mechanism",
        "agreement_requirements": ["pricing_model", "exclusivity_terms", "quality_guarantee",
                                    "derivative_works_rights", "indemnification"],
        "technical_requirements": ["data_quality_verification_at_purchase", "subscription_management",
                                     "usage_metering", "revocation_on_non_payment"],
        "risk_factors": ["information_asymmetry", "data_quality_misrepresentation", "vendor_lock_in"],
    },
}

def partnership_readiness(internal_data_capability, partner_profile, partnership_type):
    template = DATA_PARTNERSHIP_ARCHETYPES.get(partnership_type, {})
    
    internal_ready = all(
        internal_data_capability.get(f"capability_{req}", False)
        for req in template.get("technical_requirements", [])
    )
    
    agreement_gaps = [
        req for req in template.get("agreement_requirements", [])
        if req not in partner_profile.get("agreed_terms", [])
    ]
    
    return {
        "partnership_type": partnership_type,
        "internal_readiness": internal_ready,
        "agreement_gaps": agreement_gaps,
        "ready_to_execute": internal_ready and len(agreement_gaps) == 0,
        "risk_factors": template.get("risk_factors", []),
    }
```

## Open-Source Data Dependency Health

```python
OPEN_SOURCE_DEPENDENCY_HEALTH = {
    "criticality": {
        "assessment": ["download_count", "dependent_packages_count", "pipeline_criticality",
                        "replacement_cost_estimate", "fallback_exists"],
        "health_indicators": ["maintainer_count", "commit_frequency", "issue_response_time_days",
                               "open_issues_count", "open_critical_issues", "last_release_date"],
        "bus_factor": "number_of_maintainers_who_could_be_hit_by_a_bus_before_project_dies",
    },
}

def open_source_dependency_audit(dependencies):
    findings = []
    for dep in dependencies:
        health = {
            "name": dep["name"],
            "criticality": "CRITICAL" if dep.get("pipeline_criticality") and not dep.get("fallback_exists")
                           else "HIGH" if dep.get("pipeline_criticality")
                           else "MODERATE" if dep.get("download_count", 0) > 1000
                           else "LOW",
            "health_score": _compute_health(dep),
            "bus_factor": dep.get("maintainer_count", 1),
            "risk": "IMMEDIATE_ACTION" if dep.get("maintainer_count", 1) <= 1 and dep.get("pipeline_criticality")
                    else "MONITOR" if dep.get("maintainer_count", 1) <= 1
                    else "HEALTHY",
        }
        
        if health["risk"] == "IMMEDIATE_ACTION":
            health["recommendation"] = "Fork, fund maintainer, or find alternative immediately"
        elif health["risk"] == "MONITOR":
            health["recommendation"] = "Establish relationship with maintainer, contribute, consider funding"
        
        findings.append(health)
    
    return {
        "dependencies": findings,
        "critical_at_risk": [f for f in findings if f["criticality"] == "CRITICAL" and f["risk"] != "HEALTHY"],
        "overall_ecosystem_risk": "HIGH" if len([f for f in findings if f["risk"] == "IMMEDIATE_ACTION"]) > 0
                                  else "MODERATE" if len([f for f in findings if f["risk"] == "MONITOR"]) > 3
                                  else "LOW",
    }

def _compute_health(dep):
    indicators = []
    if dep.get("maintainer_count", 0) >= 3: indicators.append(1)
    if dep.get("commit_frequency") == "weekly_or_better": indicators.append(1)
    if dep.get("issue_response_time_days", 999) < 30: indicators.append(1)
    if dep.get("last_release_date_days_ago", 999) < 180: indicators.append(1)
    return sum(indicators) / 4
```

## Regulatory Ecosystem Navigation

```python
REGULATORY_LANDSCAPE = {
    "monitoring": ["draft_regulations", "enforcement_actions", "regulatory_guidance",
                    "court_decisions", "industry_self_regulation", "international_harmonization"],
    "participation": ["comment_on_draft_regulations", "participate_in_regulatory_sandboxes",
                       "join_industry_working_groups", "proactive_standards_proposal"],
    "implementation": ["gap_analysis_against_regulation", "technical_controls_mapping",
                        "evidence_generation_for_compliance", "audit_trail_automation"],
}

def regulatory_readiness(applicable_regulations, current_controls):
    gaps = []
    for regulation in applicable_regulations:
        for requirement in regulation.get("requirements", []):
            control = current_controls.get(requirement["id"])
            if not control:
                gaps.append({
                    "regulation": regulation["name"],
                    "requirement": requirement["description"],
                    "deadline": regulation.get("effective_date", "unknown"),
                    "risk": "NON_COMPLIANT_AT_DEADLINE" if regulation.get("effective_date")
                            else "UNKNOWN_DEADLINE_RISK",
                })
    
    return {
        "total_requirements": sum(len(r.get("requirements", [])) for r in applicable_regulations),
        "gaps": gaps,
        "gap_count": len(gaps),
        "compliance_readiness": 1 - len(gaps) / max(
            sum(len(r.get("requirements", [])) for r in applicable_regulations), 1),
        "critical_gaps": [g for g in gaps if g["risk"] == "NON_COMPLIANT_AT_DEADLINE"],
    }
```

## Standards Body Participation

```python
STANDARDS_BODIES = {
    "data_formats": ["W3C", "ISO", "IETF"],
    "ml_evaluation": ["MLCommons", "NIST", "ISO/IEC JTC 1/SC 42"],
    "privacy": ["IAPP", "ISO/IEC 27701", "NIST Privacy Framework"],
    "healthcare": ["HL7 FHIR", "DICOM", "OHDSI OMOP"],
    "finance": ["ISO 20022", "FIX Protocol", "XBRL"],
}

def standards_participation_strategy(organization_domain, current_participation):
    relevant_bodies = []
    for domain, bodies in STANDARDS_BODIES.items():
        if domain in organization_domain:
            for body in bodies:
                participation = current_participation.get(body, {})
                relevant_bodies.append({
                    "body": body,
                    "domain": domain,
                    "current_level": participation.get("level", "none"),
                    "recommended_level": "leadership" if organization_domain.get("market_position") == "leader"
                                        else "active_participation" if organization_domain.get("market_position") == "challenger"
                                        else "monitoring",
                    "gap": _participation_gap(participation.get("level", "none"),
                                              "leadership" if organization_domain.get("market_position") == "leader"
                                              else "active_participation"),
                })
    
    return relevant_bodies
```

## Academic-Industry Data Collaboration

```python
ACADEMIC_COLLABORATION_MODELS = {
    "data_donation": "Industry donates data to academic lab — no strings, maximum impact, minimum control",
    "funded_research": "Industry funds academic research with data access — joint IP, publication review rights",
    "embedded_researcher": "Academic researcher embedded in industry team — deep access, NDAs, publication constraints",
    "open_challenge": "Industry releases data as competition — broad participation, benchmark creation, talent identification",
    "joint_lab": "Dedicated joint facility with shared data infrastructure — long-term, strategic",
}

def collaboration_fit(organization_goals, academic_partner, data_sensitivity):
    models = []
    for model_name, description in ACADEMIC_COLLABORATION_MODELS.items():
        if data_sensitivity == "high" and model_name in ["data_donation ", "open_challenge"]:
            continue
        if organization_goals.get("speed") == "critical" and model_name == "joint_lab":
            continue
        models.append({"model": model_name, "description": description,
                       "fit": "STRONG" if _model_matches_goals(model_name, organization_goals)
                              else "POSSIBLE"})
    return models
```

## Quality Gate

- All data partnerships have signed agreements covering purpose binding, derived data, termination, audit.
- Open-source dependencies with bus factor = 1 and pipeline criticality have immediate mitigation plans.
- Regulatory gap analysis completed for all applicable regulations with compliance readiness > 0.9.
- Organization participates at recommended level in relevant standards bodies.
- Academic collaborations match data sensitivity and organizational goals — models selected explicitly.
