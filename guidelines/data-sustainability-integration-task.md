---
name: data-sustainability-integration-task
description: Integrate sustainability dimensions into data practice — environmental footprint of data operations, social sustainability of data supply chains, economic sustainability of data investments, intergenerational data stewardship, and circular data economies. Sustainable data is data that doesn't borrow from the future.
recommended_skills: [cost-model-task, data-lifecycle-governance-task, data-pipeline-monitoring-task]
recommended_guidelines: [data-ethics-integration-task, data-strategy-foundation-models-task, data-lifecycle-governance-task]
---

## Overview

Data has a physical footprint. Training runs consume energy. Storage requires rare earth minerals. Data supply chains exploit annotation workers. Data investments that look profitable in year 1 create technical debt that cripples year 5. This guideline treats sustainability as a data architecture property: measure the environmental cost of data operations, audit the social sustainability of data supply chains, stress-test the economic sustainability of data investments, design for intergenerational data stewardship, and build circular data economies where data waste becomes data resource.

## Environmental Footprint

```python
ENVIRONMENTAL_FOOTPRINT = {
    "compute": {
        "metrics": ["kwh_per_training_run", "pue_data_center", "carbon_intensity_grid",
                     "compute_utilization_rate", "idle_gpu_hours"],
        "reduction_strategies": ["spot_instance_scheduling", "checkpoint_and_resume",
                                  "model_pruning_before_retraining", "carbon_aware_compute_scheduling",
                                  "smaller_model_architectures_for_same_accuracy"],
    },
    "storage": {
        "metrics": ["tb_stored", "storage_tier_distribution", "data_access_frequency",
                     "cold_storage_ratio", "redundancy_factor"],
        "reduction_strategies": ["lifecycle_based_tiering", "deduplication_before_storage",
                                  "minimum_retention_not_infinite_retention",
                                  "lossless_compression", "derived_data_not_raw_data_for_old_versions"],
    },
    "network": {
        "metrics": ["data_transfer_volume_gb", "cross_region_transfer_pct",
                     "redundant_transfer_volume", "edge_vs_cloud_processing_ratio"],
        "reduction_strategies": ["edge_preprocessing", "incremental_not_full_transfers",
                                  "regional_data_locality", "compression_on_wire"],
    },
}

def environmental_audit(data_infrastructure):
    footprint = {}
    for category, details in ENVIRONMENTAL_FOOTPRINT.items():
        metrics = {}
        for metric in details["metrics"]:
            metrics[metric] = data_infrastructure.get(metric, "NOT_MEASURED")
        
        reduction_applied = sum(
            1 for s in details["reduction_strategies"]
            if data_infrastructure.get(f"strategy_{s}", False)
        )
        
        footprint[category] = {
            "metrics": metrics,
            "reduction_strategies_applied": reduction_applied,
            "reduction_strategies_available": len(details["reduction_strategies"]),
            "optimization_potential": "HIGH" if reduction_applied < len(details["reduction_strategies"]) * 0.5
                                      else "MODERATE" if reduction_applied < len(details["reduction_strategies"])
                                      else "LOW",
        }
    
    # Carbon estimate
    total_kwh = data_infrastructure.get("total_kwh_annual", 0)
    carbon_intensity = data_infrastructure.get("grid_carbon_intensity_g_per_kwh", 400)
    estimated_tco2e = total_kwh * carbon_intensity / 1_000_000  # grams → tonnes
    
    footprint["carbon_estimate"] = {
        "annual_tco2e": estimated_tco2e,
        "equivalent_to": f"{estimated_tco2e / 0.0046:.0f} smartphone charges" if estimated_tco2e > 0 else "unknown",
        "measured": data_infrastructure.get("carbon_measured", False),
        "offset": data_infrastructure.get("carbon_offset_pct", 0),
    }
    
    return footprint
```

## Social Sustainability of Data Supply Chains

```python
SOCIAL_SUPPLY_CHAIN_AUDIT = {
    "annotation_workers": {
        "checks": ["fair_wage_vs_living_wage", "contract_not_precarious", "mental_health_support",
                    "content_moderation_trauma_protocol", "right_to_organize", "career_progression_path",
                    "transparent_pay_structure", "no_surveillance_overreach"],
        "red_flags": ["piece_rate_only", "no_benefits", "ghost_work_classification",
                       "content_moderation_without_psychological_support"],
    },
    "open_source_communities": {
        "checks": ["maintainer_burnout_prevention", "dependency_health_monitoring",
                    "contribution_recognition", "security_vulnerability_responsiveness",
                    "governance_transparency", "community_code_of_conduct"],
        "red_flags": ["single_maintainer_critical_dependency", "unfixed_critical_vulnerabilities_6months",
                       "hostile_community_dynamics", "license_abandonment_risk"],
    },
    "data_subjects": {
        "checks": ["informed_consent_not_just_legal_consent", "benefit_sharing_mechanism",
                    "opt_out_respected_within_30_days", "representation_in_governance",
                    "cultural_data_sovereignty_respected", "vulnerable_population_protection"],
        "red_flags": ["data_collected_without_benefit_to_subjects", "consent_as_checkbox_exercise",
                       "indigenous_data_collected_without_community_consent"],
    },
}

def social_sustainability_audit(data_operations):
    findings = {}
    for stakeholder_group, audit in SOCIAL_SUPPLY_CHAIN_AUDIT.items():
        checks_passed = 0
        red_flags_found = []
        for check in audit["checks"]:
            if data_operations.get(f"{stakeholder_group}_{check}", False):
                checks_passed += 1
        for red_flag in audit["red_flags"]:
            if data_operations.get(f"{stakeholder_group}_{red_flag}", False):
                red_flags_found.append(red_flag)
        
        findings[stakeholder_group] = {
            "checks_passed": checks_passed,
            "checks_total": len(audit["checks"]),
            "red_flags": red_flags_found,
            "risk_level": "CRITICAL" if len(red_flags_found) >= 2
                          else "HIGH" if len(red_flags_found) == 1
                          else "MODERATE" if checks_passed < len(audit["checks"]) * 0.7
                          else "HEALTHY",
        }
    
    return {
        "findings": findings,
        "overall_risk": max((f["risk_level"] for f in findings.values()), key=lambda x: 
                           {"HEALTHY": 0, "MODERATE": 1, "HIGH": 2, "CRITICAL": 3}.get(x, 0)),
    }
```

## Economic Sustainability

```python
def economic_sustainability_stress_test(data_investment):
    """Will this data investment still make sense in 5 years under adverse conditions?"""
    
    scenarios = {
        "baseline": {"growth_rate": 1.0, "storage_cost_decline": 0.1, "compute_cost_decline": 0.15},
        "pessimistic": {"growth_rate": 1.5, "storage_cost_decline": 0.0, "compute_cost_decline": 0.0},
        "explosive_growth": {"growth_rate": 5.0, "storage_cost_decline": 0.05, "compute_cost_decline": 0.05},
    }
    
    results = {}
    for scenario_name, params in scenarios.items():
        year_5_cost = _project_cost_year_n(data_investment, year=5, params=params)
        year_5_value = _project_value_year_n(data_investment, year=5, params=params)
        
        results[scenario_name] = {
            "year_5_cost": year_5_cost,
            "year_5_value": year_5_value,
            "roi": year_5_value / max(year_5_cost, 1),
            "sustainable": year_5_value > year_5_cost,
            "break_even_year": _find_break_even(data_investment, params),
        }
    
    survives_stress = all(r["sustainable"] for r in results.values())
    
    return {
        "scenarios": results,
        "survives_all_scenarios": survives_stress,
        "worst_case_roi": min(r["roi"] for r in results.values()),
        "recommendation": "INVEST" if survives_stress
                          else "CAUTION" if results["baseline"]["sustainable"]
                          else "DO_NOT_INVEST",
    }

def _project_cost_year_n(investment, year, params):
    initial_cost = investment.get("annual_cost_year_0", 0)
    storage_pct = investment.get("storage_cost_pct", 0.3)
    compute_pct = investment.get("compute_cost_pct", 0.4)
    
    volume_growth = params["growth_rate"] ** year
    storage_cost = initial_cost * storage_pct * volume_growth * (1 - params["storage_cost_decline"]) ** year
    compute_cost = initial_cost * compute_pct * volume_growth * (1 - params["compute_cost_decline"]) ** year
    other_cost = initial_cost * (1 - storage_pct - compute_pct) * volume_growth
    
    return storage_cost + compute_cost + other_cost
```

## Intergenerational Data Stewardship

```python
STEWARDSHIP_PRINCIPLES = {
    "future_accessibility": "Data stored today must be readable in 50 years — open formats, no proprietary lock-in",
    "context_preservation": "Data without context is noise — preserve collection methodology, assumptions, limitations",
    "minimal_collection": "Future generations cannot consent — collect only what has clear enduring value",
    "destruction_planning": "Define destruction criteria at collection time — not everything should survive",
    "knowledge_transfer": "Preserve not just data but the capability to interpret it — documentation as stewardship",
}

def stewardship_audit(dataset_catalog):
    results = {}
    for dataset_id, dataset in dataset_catalog.items():
        score = 0
        checks = {
            "open_format": dataset.get("format") in ["csv", "json", "parquet", "avro", "hdf5", "netcdf", "zarr"],
            "context_documented": bool(dataset.get("methodology_document")),
            "destruction_date_set": bool(dataset.get("retention_end_date")),
            "interpretation_guide": bool(dataset.get("data_dictionary")),
            "succession_plan": bool(dataset.get("stewardship_successor")),
        }
        score = sum(1 for v in checks.values() if v)
        
        results[dataset_id] = {
            "checks": checks,
            "score": score,
            "max_score": len(checks),
            "stewardship_grade": "GOLD" if score == len(checks)
                                  else "SILVER" if score >= len(checks) - 1
                                  else "BRONZE" if score >= len(checks) - 2
                                  else "AT_RISK",
        }
    
    return {
        "datasets": results,
        "overall_stewardship": sum(r["score"] for r in results.values()) / max(len(results) * len(checks), 1),
    }
```

## Circular Data Economy

```python
CIRCULAR_DATA_PRINCIPLES = {
    "reduce": "Don't collect what you won't use — storage and compute have environmental cost",
    "reuse": "Existing datasets are capital — reuse before recollecting",
    "recycle": "Failed experiments generate training data for quality models, failure predictors",
    "repurpose": "Operational data becomes training data; training data becomes evaluation data",
    "recover": "Deprecated datasets yield metadata, statistics, and failure patterns",
}

def circularity_audit(data_inventory):
    metrics = {
        "collection_efficiency": _collection_to_use_ratio(data_inventory),
        "reuse_rate": _dataset_reuse_count(data_inventory),
        "recycling_rate": _failed_experiment_data_reuse(data_inventory),
        "repurposing_rate": _cross_domain_data_usage(data_inventory),
        "recovery_rate": _deprecated_dataset_value_extraction(data_inventory),
    }
    
    circularity_score = sum(metrics.values()) / len(metrics)
    
    return {
        "metrics": metrics,
        "circularity_score": circularity_score,
        "linear": circularity_score < 0.3,
        "transitioning": 0.3 <= circularity_score < 0.6,
        "circular": circularity_score >= 0.6,
    }
```

## Quality Gate

- Environmental footprint measured and reduction strategies applied — carbon intensity tracked annually.
- Social supply chain audit passed for annotation workers, open-source communities, and data subjects — zero critical red flags.
- Economic sustainability stress-tested across baseline, pessimistic, and explosive-growth scenarios — survives all.
- Intergenerational stewardship score ≥ 0.6 across all datasets — open formats, context preserved.
- Circular data economy score ≥ 0.5 — reuse, recycling, and repurposing actively practiced.
