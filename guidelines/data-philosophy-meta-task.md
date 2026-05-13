---
name: data-philosophy-meta-task
description: Curate data for data philosophy, meta-dataset science, and data decommissioning — ontology alignment, epistemological consistency, dataset genealogy, cross-dataset dependencies, dataset health scoring, and retirement/destruction validation.
recommended_skills: [data-diff, dataset-versioning, embedding-analysis, data-card-writer]
recommended_guidelines: [data-lifecycle-governance-task, data-archaeology-task, dataset-certification-task]
---

## Data Philosophy & Foundations

```python
def validate_ontology_alignment(dataset_ontologies, reference_ontology):
    """Do data ontologies match across sources?"""
    alignments = {}
    for source, ontology in dataset_ontologies.items():
        overlap = len(set(ontology["terms"]) & set(reference_ontology["terms"]))
        conflict = len([t for t in ontology["terms"] if t in reference_ontology["terms"] and 
                        ontology["definitions"].get(t) != reference_ontology["definitions"].get(t)])
        alignments[source] = {"overlap_ratio": overlap / max(len(reference_ontology["terms"]), 1),
                               "conflicts": conflict, "aligned": conflict == 0}
    return alignments

def assess_measurement_validity(measurements, construct_definition, validation_evidence):
    """Does measurement actually capture the intended construct?"""
    validity = {"content": _content_validity(measurements, construct_definition),
                "convergent": _convergent_validity(measurements, validation_evidence),
                "discriminant": _discriminant_validity(measurements, validation_evidence)}
    return {"validity": validity, "all_valid": all(validity.values()),
            "weakest_dimension": min(validity, key=validity.get)}
```

## Meta-Dataset Science

```python
def reconstruct_dataset_genealogy(dataset, transformation_history):
    """Derivation chains — what transformations produced this dataset?"""
    chain = []
    current = dataset
    for transform in transformation_history:
        chain.append({"step": transform["name"], "input_hash": _hash(current),
                       "output_hash": _hash(transform["output"]),
                       "parameters": transform.get("parameters", {})})
        current = transform["output"]
    return {"genealogy": chain, "depth": len(chain), "reproducible": all(
        "parameters" in t for t in transformation_history)}

def map_cross_dataset_dependencies(datasets):
    """What datasets depend on what upstream sources?"""
    import networkx as nx
    G = nx.DiGraph()
    for ds in datasets:
        G.add_node(ds["id"])
        for upstream in ds.get("source_datasets", []):
            G.add_edge(upstream, ds["id"])
    centrality = nx.betweenness_centrality(G)
    return {"dependency_graph": G, "central_datasets": sorted(centrality, key=centrality.get, reverse=True)[:5],
            "leaf_datasets": [n for n in G.nodes() if G.out_degree(n) == 0]}

def score_dataset_health(dataset, dimensions):
    """Comprehensive quality metric combining all dimensions."""
    weights = {"completeness": 0.2, "accuracy": 0.2, "freshness": 0.15, 
               "consistency": 0.15, "uniqueness": 0.1, "validity": 0.1, "accessibility": 0.1}
    score = sum(dataset.get(dim, 0) * weights.get(dim, 0) for dim in dimensions)
    return {"overall_health": float(score), "per_dimension": {dim: dataset.get(dim, 0) for dim in dimensions},
            "critical_issues": [dim for dim in dimensions if dataset.get(dim, 0) < 0.5]}
```

## Data Decommissioning & Retirement

```python
def validate_data_retirement(dataset, retirement_criteria):
    """When is data truly obsolete?"""
    checks = {"superseded_by_newer": dataset.get("newer_version_available", False),
              "zero_usage_90_days": dataset.get("days_since_last_use", 0) > 90,
              "quality_below_threshold": dataset.get("health_score", 1.0) < 0.3,
              "regulatory_retention_period_elapsed": dataset.get("min_retention_days", 0) <= 0,
              "consent_withdrawn": dataset.get("consent_status") == "withdrawn"}
    can_retire = all(checks[k] for k in ["superseded_by_newer", "zero_usage_90_days"])
    must_retire = checks["consent_withdrawn"]
    cannot_retire = not checks["regulatory_retention_period_elapsed"] and not checks["consent_withdrawn"]
    
    return {"status": "MUST_RETIRE" if must_retire else "CAN_RETIRE" if can_retire else "MUST_KEEP",
            "checks": checks, "blockers": [k for k, v in checks.items() if not v and k in ["regulatory_retention_period_elapsed"]]}

def verify_data_destruction(dataset, destruction_method, verification_audit):
    """Claimed destruction vs actual — is data truly gone?"""
    verification = {"method": destruction_method, "primary_storage": _verify_primary_deletion(dataset),
                    "backups": _verify_backup_deletion(dataset),
                    "derivatives": _verify_derivative_flagging(dataset),
                    "cache": _verify_cache_purging(dataset)}
    return {"verified": all(verification.values()), "verification": verification,
            "certificate_id": _generate_destruction_certificate(dataset, verification)}
```

## Quality Gate

- Ontology alignment: zero conflicts between sources using the same reference ontology.
- Measurement validity: content, convergent, and discriminant validity all pass.
- Dataset genealogy: fully traceable derivation chain.
- Dataset health: overall score > 0.7, zero critical issues.
- Decommissioning: destruction verified across all storage tiers.
