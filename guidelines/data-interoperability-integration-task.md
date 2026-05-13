---
name: data-interoperability-integration-task
description: Achieve data interoperability across organizational boundaries — semantic interoperability through shared ontologies, syntactic interoperability through format standardization, organizational interoperability through governance alignment, and temporal interoperability through schema evolution compatibility. Data that cannot be joined is data that cannot be used.
recommended_skills: [dataset-cleaning, dataset-splitting, dataset-versioning]
recommended_guidelines: [data-ecosystem-integration-task, data-governance-federation-integration-task, data-lifecycle-governance-task]
---

## Overview

The most expensive data is the data you already have but cannot use because it doesn't interoperate with the data you just acquired. Interoperability is not a technology problem — it's an architecture problem with semantic, syntactic, organizational, and temporal dimensions. This guideline designs interoperability across all four layers so data from different teams, organizations, and eras can be meaningfully joined.

## Semantic Interoperability

```python
SEMANTIC_INTEROPERABILITY = {
    "problem": "Two datasets use different words for the same thing or the same word for different things",
    "approaches": {
        "shared_ontology": "One canonical definition per concept — all systems map to it",
        "crosswalk": "Explicit mapping between ontologies — 'our X = your Y under condition Z'",
        "entity_resolution": "Probabilistic matching of entities across datasets with no common identifier",
    },
}

def ontology_alignment(source_ontology, target_ontology, concept_mappings):
    alignment = {"exact_matches": [], "partial_matches": [], "unmatched": []}
    
    for source_concept in source_ontology:
        source_id = source_concept["id"]
        if source_id in concept_mappings:
            target_concept = concept_mappings[source_id]
            if _semantic_equivalence(source_concept, target_concept, target_ontology):
                alignment["exact_matches"].append({
                    "source": source_id,
                    "target": target_concept["id"],
                    "confidence": 1.0,
                })
            else:
                alignment["partial_matches"].append({
                    "source": source_id,
                    "target": target_concept["id"],
                    "confidence": _semantic_overlap(source_concept, target_concept),
                    "differences": _concept_diff(source_concept, target_concept),
                })
        else:
            alignment["unmatched"].append({"source": source_id, "recommendation": "find_or_create_mapping"})
    
    return {
        "alignment": alignment,
        "coverage": len(alignment["exact_matches"]) + len(alignment["partial_matches"]),
        "total_concepts": len(source_ontology),
        "interoperability_score": (len(alignment["exact_matches"]) + len(alignment["partial_matches"]) * 0.5) 
                                   / max(len(source_ontology), 1),
    }
```

## Syntactic Interoperability

```python
SYNTACTIC_LAYERS = {
    "format": ["csv", "json", "parquet", "avro", "protobuf", "arrow"],
    "encoding": ["utf-8", "utf-16", "latin-1", "ascii"],
    "schema": ["explicit_schema", "schema_on_read", "schemaless"],
    "compression": ["none", "gzip", "snappy", "zstd", "lz4"],
    "transport": ["rest", "grpc", "s3_select", "sql", "streaming"],
}

def syntactic_compatibility(dataset_spec_a, dataset_spec_b):
    checks = {}
    for layer, options in SYNTACTIC_LAYERS.items():
        a_value = dataset_spec_a.get(layer)
        b_value = dataset_spec_b.get(layer)
        checks[layer] = {
            "source": a_value,
            "target": b_value,
            "compatible": a_value == b_value,
            "translation_needed": a_value != b_value,
            "translation_cost": "LOW" if a_value in ["csv", "json", "utf-8"] and b_value in ["csv", "json", "utf-8"]
                                else "MODERATE" if layer == "format"
                                else "HIGH" if layer == "schema"
                                else "LOW",
        }
    
    compatible = all(c["compatible"] for c in checks.values())
    return {"checks": checks, "fully_compatible": compatible,
            "translation_layers": [k for k, v in checks.items() if not v["compatible"]],
            "estimated_integration_complexity": "TRIVIAL" if sum(1 for c in checks.values() if not c["compatible"]) == 0
                                               else "LOW" if sum(1 for c in checks.values() if not c["compatible"]) <= 1
                                               else "MODERATE" if sum(1 for c in checks.values() if not c["compatible"]) <= 2
                                               else "HIGH"}
```

## Organizational Interoperability

```python
ORGANIZATIONAL_INTEROP_ALIGNMENT = {
    "governance": {
        "check": "data_access_policies_compatible",
        "conflict_example": "org_A_open_by_default vs org_B_closed_by_default",
        "resolution": "negotiated_data_sharing_agreement_with_least_privilege_default",
    },
    "quality_standards": {
        "check": "data_quality_thresholds_compatible",
        "conflict_example": "org_A_99%_completeness vs org_B_best_effort",
        "resolution": "minimum_viable_quality_standard_agreed_up_front_with_remediation_plan",
    },
    "retention": {
        "check": "data_retention_policies_aligned",
        "conflict_example": "org_A_retains_7_years vs org_B_retains_1_year",
        "resolution": "retention_harmonization_or_derived_data_destruction_upon_partnership_end",
    },
    "stewardship": {
        "check": "data_stewardship_responsibilities_clear",
        "conflict_example": "both_orgs_claim_stewardship_or_neither_does",
        "resolution": "explicit_stewardship_assignment_in_data_sharing_agreement",
    },
    "incident_response": {
        "check": "breach_notification_timelines_compatible",
        "conflict_example": "org_A_notifies_in_24h vs org_B_notifies_in_72h",
        "resolution": "agreed_notification_timeline_with_automatic_escalation",
    },
}

def organizational_interop_assessment(org_a_practices, org_b_practices):
    conflicts = []
    for dimension, config in ORGANIZATIONAL_INTEROP_ALIGNMENT.items():
        a_value = org_a_practices.get(dimension)
        b_value = org_b_practices.get(dimension)
        
        if not _are_compatible(a_value, b_value):
            conflicts.append({
                "dimension": dimension,
                "org_a": a_value,
                "org_b": b_value,
                "conflict_pattern": config["conflict_example"],
                "resolution": config["resolution"],
                "blocker": dimension in ["governance", "incident_response"],
            })
    
    return {
        "conflicts": conflicts,
        "blocker_count": sum(1 for c in conflicts if c["blocker"]),
        "total_dimensions": len(ORGANIZATIONAL_INTEROP_ALIGNMENT),
        "interop_ready": sum(1 for c in conflicts if c["blocker"]) == 0,
        "interop_score": 1 - len(conflicts) / len(ORGANIZATIONAL_INTEROP_ALIGNMENT),
    }
```

## Temporal Interoperability

```python
SCHEMA_EVOLUTION_PATTERNS = {
    "add_column": {"compatibility": "FORWARD", "risk": "LOW", "mitigation": "default_value_for_old_data"},
    "remove_column": {"compatibility": "BACKWARD", "risk": "MODERATE", "mitigation": "grace_period_before_removal"},
    "rename_column": {"compatibility": "BREAKING", "risk": "HIGH", "mitigation": "dual_write_during_transition"},
    "change_type": {"compatibility": "BREAKING", "risk": "HIGH", "mitigation": "new_column_with_new_type_and_migration"},
    "change_semantics": {"compatibility": "SILENT_CORRUPTION", "risk": "CRITICAL",
                          "mitigation": "NEVER CHANGE SEMANTICS IN PLACE — always new column"},
    "split_table": {"compatibility": "BREAKING", "risk": "MODERATE", "mitigation": "view_over_old_and_new_structures"},
}

def temporal_compatibility(dataset_v1_schema, dataset_v2_schema, time_gap_days):
    evolution = _detect_schema_evolution(dataset_v1_schema, dataset_v2_schema)
    
    breaking_changes = [e for e in evolution 
                        if SCHEMA_EVOLUTION_PATTERNS.get(e["type"], {}).get("compatibility") in 
                        ["BREAKING", "SILENT_CORRUPTION"]]
    
    return {
        "evolution_detected": evolution,
        "breaking_changes": breaking_changes,
        "compatible": len(breaking_changes) == 0,
        "time_gap_days": time_gap_days,
        "risk": "HIGH" if any(
            SCHEMA_EVOLUTION_PATTERNS.get(e["type"], {}).get("compatibility") == "SILENT_CORRUPTION"
            for e in breaking_changes
        ) else "MODERATE" if breaking_changes else "LOW",
    }
```

## Interoperability Maturity Model

```python
INTEROP_MATURITY = {
    1: {"name": "Ad Hoc", "description": "Manual CSV export/import; no shared definitions",
        "semantic": "none", "syntactic": "csv_export", "organizational": "ad_hoc_emails",
        "temporal": "no_schema_evolution_management"},
    2: {"name": "Defined", "description": "Shared schemas documented; crosswalks maintained",
        "semantic": "documented_crosswalks", "syntactic": "shared_json_schema",
        "organizational": "data_sharing_agreements", "temporal": "versioned_schemas"},
    3: {"name": "Automated", "description": "Automated validation; schema registry; semantic mappings active",
        "semantic": "active_ontology_with_mappings", "syntactic": "schema_registry_with_compatibility_checks",
        "organizational": "federated_governance", "temporal": "evolvable_schemas_with_migration_tooling"},
    4: {"name": "Optimized", "description": "Plug-and-play data integration; semantic layer abstracts sources",
        "semantic": "semantic_layer_queryable", "syntactic": "multi_format_unified_query_engine",
        "organizational": "ecosystem_governance_federation", "temporal": "temporally_versioned_data_lakehouse"},
}

def interop_maturity_assessment(organization_interop_practices):
    current_level = 1
    for level in range(1, 5):
        level_reqs = INTEROP_MATURITY[level]
        if all(
            organization_interop_practices.get(dim) == level_reqs[dim]
            for dim in ["semantic", "syntactic", "organizational", "temporal"]
        ):
            current_level = level
        else:
            break
    
    return {"current_level": current_level, "level_name": INTEROP_MATURITY[current_level]["name"],
            "next_level": current_level + 1 if current_level < 4 else None,
            "gaps_to_next": _maturity_gaps(organization_interop_practices, current_level + 1)
                            if current_level < 4 else []}
```

## Quality Gate

- Semantic alignment covers all shared concepts — unmatched concepts have explicit mapping plans.
- Syntactic compatibility checked at all five layers; translation automation built for incompatible layers.
- Organizational interoperability blockers resolved before data sharing begins.
- Schema evolution respects compatibility rules — no silent semantic changes, breaking changes dual-written.
- Interoperability maturity assessed; roadmap to next level defined.
