---
name: streaming-edge-mesh-data-task
description: Curate data for streaming, edge, and distributed systems — real-time quality validation, edge-device constraints, federated learning coordination, multi-cloud consistency, and data mesh governance.
recommended_skills: [data-pipeline-monitoring-task, embedding-analysis, data-diff]
recommended_guidelines: [privacy-preserving-data-task, data-lifecycle-governance-task]
---

## Streaming / Real-Time

### Continuous Quality Validation

```python
class StreamingQualityMonitor:
    def __init__(self, baseline_stats, window_size=1000, drift_threshold=0.1):
        self.baseline = baseline_stats
        self.window = []
        self.window_size = window_size
    
    def ingest(self, batch):
        self.window.extend(batch)
        if len(self.window) > self.window_size * 2:
            self.window = self.window[-self.window_size:]
        
        if len(self.window) < self.window_size:
            return {"status": "warming_up", "n": len(self.window)}
        
        recent = self.window[-self.window_size:]
        checks = {
            "volume": len(batch) > 0,
            "null_rate": np.mean([x is None for x in recent]) < 0.1,
            "drift": _ks_test(recent, self.baseline["distribution"]),
        }
        return {"status": "OK" if all(checks.values()) else "ALERT", "checks": checks}

STREAMING_METRICS = {
    "throughput": "items/second — alert if drops below 50% of baseline",
    "latency": "p95 processing time — alert if > 2x baseline",
    "schema_drift": "new/missing fields — alert on ANY change",
    "value_range": "min/max per numeric field — alert if outside historical bounds",
}
```

## Edge Deployment

### Device Constraints

```python
EDGE_TIERS = {
    "tier_1_mobile": {"ram_mb": 512, "storage_mb": 200, "inference_ms_budget": 50},
    "tier_2_embedded": {"ram_mb": 128, "storage_mb": 50, "inference_ms_budget": 10},
    "tier_3_micro": {"ram_mb": 32, "storage_mb": 10, "inference_ms_budget": 5},
}

def validate_edge_deployability(model, dataset_sample, tier):
    constraints = EDGE_TIERS[tier]
    model_size = estimate_model_size_mb(model)
    inference_time = measure_inference_ms(model, dataset_sample)
    return {"model_fits": model_size < constraints["storage_mb"],
            "inference_fast_enough": inference_time < constraints["inference_ms_budget"],
            "ram_ok": model_size * 3 < constraints["ram_mb"]}
```

## Federated Learning

```python
def audit_federated_client_data(client_data, global_distribution):
    """Validate per-client data quality and distribution."""
    client_audits = {}
    for client_id, data in client_data.items():
        client_audits[client_id] = {
            "n_samples": len(data),
            "distribution_shift": _js_divergence(data, global_distribution),
            "label_quality": _estimate_label_quality(data),
            "participation_score": min(len(data) / 100, 1.0),
            "flagged": _js_divergence(data, global_distribution) > 0.3 or len(data) < 10,
        }
    
    skewness = np.std([a["distribution_shift"] for a in client_audits.values()])
    return {"clients": client_audits, "skewness": skewness,
            "iid_level": "IID" if skewness < 0.1 else "NON_IID_MILD" if skewness < 0.3 else "NON_IID_SEVERE"}
```

## Multi-Cloud Consistency

```python
def validate_cross_cloud_consistency(datasets_by_cloud, tolerance=0.01):
    """Same data across clouds should produce same results."""
    clouds = list(datasets_by_cloud.keys())
    inconsistencies = []
    for i in range(len(clouds)):
        for j in range(i+1, len(clouds)):
            d1, d2 = datasets_by_cloud[clouds[i]], datasets_by_cloud[clouds[j]]
            if len(d1) != len(d2):
                inconsistencies.append({"clouds": (clouds[i], clouds[j]), "issue": "row_count_mismatch"})
                continue
            for col in d1.columns:
                if not np.allclose(d1[col].dropna(), d2[col].dropna(), rtol=tolerance):
                    inconsistencies.append({"clouds": (clouds[i], clouds[j]), "issue": f"value_mismatch:{col}"})
    return {"consistent": len(inconsistencies) == 0, "inconsistencies": inconsistencies[:20]}
```

## Data Mesh

```python
DATA_MESH_PRINCIPLES = {
    "domain_ownership": "Each domain team owns their data end-to-end",
    "data_as_product": "Data is a product with SLAs, not a byproduct",
    "self_service": "Domain teams self-serve infrastructure via platform",
    "federated_governance": "Global standards, local implementation",
}

def validate_mesh_readiness(domain_datasets):
    """Check if domain datasets meet mesh governance standards."""
    checks = {}
    for domain, dataset in domain_datasets.items():
        checks[domain] = {
            "has_data_card": dataset.data_card is not None,
            "has_sla": dataset.freshness_sla is not None,
            "has_schema_doc": dataset.schema_doc is not None,
            "has_owner": dataset.owner_team is not None,
            "mesh_ready": all([
                dataset.data_card, dataset.freshness_sla, dataset.schema_doc, dataset.owner_team
            ]),
        }
    return checks
```

## Quality Gate

- Streaming: drift < 0.1, null rate < 10%, volume > 0.
- Edge: model fits, inference meets latency budget.
- Federated: per-client distribution shift < 0.3, skewness < 0.3.
- Multi-cloud: zero inconsistencies across clouds.
- Mesh: all domains pass mesh readiness with owner, SLA, and data card.
