---
name: strategy-integration-data-task
description: Integrate data strategy with organizational strategy — data-model-organization triad alignment, cross-functional coordination, capability mapping, strategy-data translation, and technology roadmapping.
recommended_skills: [cost-model-task, team-operations-data-task, organizational-implementation-data-task]
recommended_guidelines: [data-strategy-foundation-models-task, data-portfolio-theory-task, dataset-certification-task]
---

## Data-Model-Organization Triad

```python
def audit_org_model_alignment(org_structure, model_architecture, data_pipeline_topology):
    """Does org structure support how the model needs data?"""
    gaps = []
    
    if org_structure.get("data_team_type") == "centralized" and model_architecture.get("requires_domain_expertise"):
        gaps.append("centralized_team_cannot_provide_domain_expertise")
    
    if data_pipeline_topology.get("real_time") and not org_structure.get("on_call_support"):
        gaps.append("real_time_pipeline_without_on_call")
    
    if model_architecture.get("multimodal") and len(org_structure.get("data_sources_owned", [])) < 2:
        gaps.append("multimodal_model_with_single_data_owner")
    
    return {"alignment_score": 1 - len(gaps) / max(len(org_structure) + len(model_architecture), 1),
            "gaps": gaps, "aligned": len(gaps) == 0}

def map_capability_to_data(organizational_capabilities, data_assets):
    """What data enables which capabilities?"""
    mapping = {}
    for cap_id, cap_info in organizational_capabilities.items():
        enablers = []
        for data_id, data_info in data_assets.items():
            if _data_supports_capability(data_info, cap_info):
                enablers.append(data_id)
        mapping[cap_id] = {"enabled_by": enablers, "data_dependency_risk": "HIGH" if not enablers else "LOW"}
    return mapping
```

## Cross-Functional Coordination

```python
CROSS_FUNCTIONAL_CONTRACTS = {
    "engineering_to_research": {"schema_version": "required", "sample_interval": "specified", 
                                  "quality_baseline": "documented"},
    "product_to_ml": {"success_metric": "defined", "latency_budget": "specified", 
                        "fairness_requirements": "documented"},
    "business_to_analytics": {"kpi_definition": "agreed", "refresh_frequency": "specified",
                                "data_retention_policy": "documented"},
}

def validate_data_contract(contract_type, contract_terms):
    required = CROSS_FUNCTIONAL_CONTRACTS.get(contract_type, {})
    met = {k: v in contract_terms for k, v in required.items()}
    return {"contract": contract_type, "terms_met": met, "complete": all(met.values()),
            "missing_terms": [k for k, v in met.items() if not v]}
```

## Strategy Translation

```python
def translate_business_strategy(business_objectives, current_data_assets, investment_budget):
    """What data investments does the strategy demand?"""
    data_investments = []
    for obj in business_objectives:
        required_data = _infer_data_requirements(obj)
        gaps = [d for d in required_data if d not in current_data_assets]
        if gaps:
            priority = obj.get("priority", 3)
            data_investments.append({"objective": obj["description"][:100], "gaps": gaps,
                                      "priority": priority, "estimated_cost": _estimate_acquisition_cost(gaps)})
    
    data_investments.sort(key=lambda x: -x["priority"])
    affordable = []
    remaining_budget = investment_budget
    for inv in data_investments:
        if inv["estimated_cost"] <= remaining_budget:
            affordable.append(inv)
            remaining_budget -= inv["estimated_cost"]
    
    return {"total_investments_needed": len(data_investments),
            "affordable": len(affordable), "budget_sufficient": len(affordable) == len(data_investments),
            "unfunded_priorities": [i for i in data_investments if i not in affordable]}
```

## Quality Gate

- Org-model alignment: zero gaps for critical model requirements.
- All cross-functional contracts complete.
- Strategy-driven data investments prioritized by business objective priority.
- Unfunded data gaps escalated to leadership with cost estimates.
