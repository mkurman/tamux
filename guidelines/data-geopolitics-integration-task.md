---
name: data-geopolitics-integration-task
description: Navigate data across geopolitical boundaries — data sovereignty architectures, cross-border transfer compliance, digital trade agreement mapping, data localization requirements, and sanctions-aware data routing. Data doesn't respect borders; regulators do.
recommended_skills: [dataset-governance-task, privacy-preserving-data-task, data-pipeline-monitoring-task]
recommended_guidelines: [dataset-governance-task, data-ecosystem-integration-task, data-resilience-integration-task]
---

## Overview

Data crosses borders in milliseconds. Compliance crosses borders in years. The gap is where fines, blocked mergers, and diplomatic incidents live. This guideline maps data sovereignty requirements by jurisdiction, engineers cross-border transfer architectures, tracks digital trade agreements that create safe harbors, designs for data localization mandates, and builds sanctions-aware data routing. Geopolitical risk is now a data architecture constraint.

## Data Sovereignty Architecture

```python
SOVEREIGNTY_REGIMES = {
    "EU": {
        "regime": "GDPR + EU Data Strategy",
        "transfer_mechanisms": ["adequacy_decision", "standard_contractual_clauses",
                                 "binding_corporate_rules", "approved_certification"],
        "localization_required": False,
        "restrictions": ["law_enforcement_access_by_non_eu_governments_requires_mlat",
                         "schrems_ii_requires_transfer_impact_assessment"],
        "enforcement_risk": "HIGH — up to 4% global annual turnover",
    },
    "China": {
        "regime": "PIPL + DSL + CSL",
        "transfer_mechanisms": ["security_assessment_by_cac", "standard_contract_filing",
                                 "certification_by_accredited_body"],
        "localization_required": True,
        "restrictions": ["critical_information_infrastructure_data_must_stay_in_china",
                         "personal_information_export_requires_separate_consent_or_necessity"],
        "enforcement_risk": "HIGH — criminal liability for serious violations",
    },
    "US": {
        "regime": "sectoral (HIPAA, FCRA, COPPA, state laws) + Executive Orders",
        "transfer_mechanisms": ["eu_us_data_privacy_framework", "uk_us_data_bridge",
                                 "swiss_us_dpf"],
        "localization_required": False,
        "restrictions": ["cfius_review_for_foreign_access_to_sensitive_data",
                         "export_controls_on_ai_training_data_for_adversary_nations"],
        "enforcement_risk": "MODERATE — state AG actions + FTC + sectoral regulators",
    },
    "India": {
        "regime": "DPDP Act 2023",
        "transfer_mechanisms": ["government_notification_of_adequate_countries",
                                 "standard_clauses_awaiting_definition"],
        "localization_required": "partial — certain categories may require localization",
        "restrictions": ["significant_data_fiduciaries_have_additional_obligations",
                         "childrens_data_requires_verifiable_parental_consent"],
        "enforcement_risk": "MODERATE — new regime, enforcement patterns emerging",
    },
    "Russia": {
        "regime": "Federal Law No. 152-FZ + localization amendments",
        "transfer_mechanisms": ["adequacy_decision", "data_subject_consent_for_cross_border"],
        "localization_required": True,
        "restrictions": ["primary_databases_must_be_physically_in_russia",
                         "roscomnadzor_notification_required"],
        "enforcement_risk": "HIGH — website blocking, fines, criminal for repeat violations",
    },
}

def sovereignty_audit(data_flows, jurisdictions_touched):
    violations = []
    for flow in data_flows:
        source_jurisdiction = flow.get("source_jurisdiction")
        target_jurisdiction = flow.get("target_jurisdiction")
        
        source_regime = SOVEREIGNTY_REGIMES.get(source_jurisdiction, {})
        if source_regime.get("localization_required") and flow.get("data_stored_abroad"):
            violations.append({
                "flow": flow["id"],
                "violation": f"{source_jurisdiction}_localization_breach",
                "severity": "CRITICAL",
                "remediation": "store_primary_copy_in_source_jurisdiction",
            })
        
        if not flow.get("transfer_mechanism") and source_regime.get("transfer_mechanisms"):
            violations.append({
                "flow": flow["id"],
                "violation": "no_transfer_mechanism_documented",
                "applicable_mechanisms": source_regime["transfer_mechanisms"],
                "severity": "HIGH",
            })
    
    return {
        "flows_audited": len(data_flows),
        "violations": violations,
        "compliant": len(violations) == 0,
        "critical_violations": [v for v in violations if v["severity"] == "CRITICAL"],
    }
```

## Cross-Border Transfer Architecture

```python
TRANSFER_ARCHITECTURE_PATTERNS = {
    "data_residency": "Data stays in jurisdiction; only aggregated/anon results leave",
    "federated_processing": "Compute travels to data; models/parameters cross borders, raw data doesn't",
    "differential_privacy_export": "ε-differentially private data exported — mathematically bounded privacy loss",
    "synthetic_data_export": "Synthetic data generated locally; statistical properties preserved, individuals unrepresented",
    "trusted_execution_environment": "Data processed in hardware enclave; provider cannot access raw data",
    "homomorphic_encryption": "Computation on encrypted data; results decrypted only in target jurisdiction",
}

def transfer_architecture_recommendation(data_flow, risk_tolerance):
    if risk_tolerance == "zero":
        return ["data_residency", "federated_processing"]
    elif risk_tolerance == "low":
        return ["federated_processing", "differential_privacy_export", "synthetic_data_export"]
    elif risk_tolerance == "moderate":
        return ["trusted_execution_environment", "differential_privacy_export"]
    else:
        return list(TRANSFER_ARCHITECTURE_PATTERNS.keys())
```

## Digital Trade Agreement Mapping

```python
DIGITAL_TRADE_PROVISIONS = {
    "data_free_flow_with_trust": ["DEPAs", "UK-Singapore DEA", "EU-Japan EPA", "US-Japan DTA"],
    "data_localization_prohibition": ["CPTPP", "USMCA", "DEPA", "RCEP"],
    "source_code_protection": ["USMCA", "US-Japan DTA", "CPTPP"],
    "ai_governance_cooperation": ["EU-Singapore DTA", "UK-Singapore DEA"],
}

def trade_agreement_applicability(organization_jurisdictions):
    applicable = []
    for provision, agreements in DIGITAL_TRADE_PROVISIONS.items():
        for agreement in agreements:
            if _covers_jurisdictions(agreement, organization_jurisdictions):
                applicable.append({
                    "agreement": agreement,
                    "provision": provision,
                    "benefit": _provision_benefit(provision),
                    "obligation": _provision_obligation(provision),
                })
    return applicable
```

## Sanctions-Aware Data Routing

```python
SANCTIONS_ROUTING = {
    "comprehensive_embargo": ["Cuba", "Iran", "North Korea", "Syria", "Crimea_region"],
    "sectoral_sanctions": ["Russia_financial_energy_defense", "Venezuela"],
    "entity_list": "OFAC SDN List — specific entities, not countries",
    "technology_export_controls": ["advanced_compute_chips", "ai_training_capability",
                                    "surveillance_technology"],
}

def sanctions_route_check(data_flow, sanctions_regime):
    source = data_flow.get("target_entity_country")
    target = data_flow.get("source_entity_country")
    
    if source in sanctions_regime.get("comprehensive_embargo", []):
        return {"allowed": False, "reason": f"{source}_under_comprehensive_embargo"}
    if target in sanctions_regime.get("comprehensive_embargo", []):
        return {"allowed": False, "reason": f"{target}_under_comprehensive_embargo"}
    
    if data_flow.get("contains_controlled_technology") and \
       target in sanctions_regime.get("technology_export_controls", []):
        return {"allowed": False, "reason": "technology_export_control_restriction"}
    
    return {"allowed": True}
```

## Quality Gate

- All cross-border data flows mapped to source/target jurisdictions with sovereignty assessment.
- Every flow has documented transfer mechanism; localization requirements satisfied.
- Digital trade agreements leveraged where applicable for safe harbor provisions.
- Sanctions-aware routing integrated into data pipeline — no sanctioned destination receives data.
- Geopolitical risk register updated quarterly with regulatory change monitoring.
