---
name: continual-pretraining-data-task
description: Design data strategies for continual pre-training — domain mixing ratios, replay buffer construction, forgetting detection data, capability preservation benchmarks, and progressive domain adaptation schedules. CPT without data strategy is just expensive forgetting.
recommended_skills: [dataset-splitting, embedding-analysis, benchmark-contamination-scan]
recommended_guidelines: [llm-training-data-task, data-strategy-foundation-models-task, scaling-law-data-task]
---

## Overview

Continual pre-training (CPT) adapts a base model to a new domain — medicine, law, finance, code — without losing the general capabilities you paid millions to train. The data strategy is everything: what mix of new domain and replay data prevents catastrophic forgetting, how to sequence domains for progressive adaptation, what data to use for forgetting detection, and how to validate that capabilities survived. This guideline treats CPT as a data engineering problem with measurable forgetting risk.

## Domain Mixing Ratios

```python
CPT_MIXING_STRATEGIES = {
    "domain_dominant": {
        "ratio": "70-80% new domain, 20-30% replay",
        "use_when": "domain_is_highly_specialized_and_distinct_from_pretraining",
        "risk": "forgetting_on_general_capabilities_if_replay_too_low",
        "replay_selection": "use fineweb/c4 stratified by capability_area",
        "example": "medical_cpt: 75% PubMed/clinical, 25% general web",
    },
    "balanced": {
        "ratio": "50% new domain, 50% replay",
        "use_when": "domain_overlaps_with_pretraining_or_general_capability_is_critical",
        "risk": "slower_domain_adaptation",
        "replay_selection": "use pretraining data stratified to match domain distribution",
        "example": "legal_cpt: 50% case law, 50% general with emphasis on reasoning",
    },
    "progressive": {
        "ratio": "gradual shift: 90%→50% new domain over training",
        "use_when": "model_is_far_from_target_domain — warm up first, then stabilize",
        "risk": "complex_schedule_management",
        "replay_selection": "increasing replay buffer toward end to stabilize",
        "example": "code_cpt: start 90% code, end 50% code/50% general for stability",
    },
}

def design_cpt_mix(target_domain, base_model_capabilities, replay_corpus, total_tokens):
    """Design the optimal domain-replay mixing strategy for CPT."""
    
    # Measure domain distance
    domain_distance = _domain_embedding_distance(target_domain, base_model_capabilities["pretraining_domains"])
    
    if domain_distance > 0.7:
        strategy = "domain_dominant"
    elif domain_distance > 0.3:
        strategy = "balanced"
    else:
        strategy = "progressive"
    
    mix_config = CPT_MIXING_STRATEGIES[strategy]
    
    # Allocate tokens
    new_domain_pct, replay_pct = _parse_ratio(mix_config["ratio"])
    domain_tokens = int(total_tokens * new_domain_pct)
    replay_tokens = int(total_tokens * replay_pct)
    
    # Curate replay buffer
    replay_buffer = _curate_replay(replay_corpus, replay_tokens, 
                                    base_model_capabilities["critical_areas"])
    
    return {
        "strategy": strategy,
        "domain_distance": domain_distance,
        "total_tokens": total_tokens,
        "domain_tokens": domain_tokens,
        "replay_tokens": replay_tokens,
        "replay_buffer": replay_buffer,
        "replay_selection_method": mix_config["replay_selection"],
        "risk": mix_config["risk"],
    }
```

## Replay Buffer Construction

```python
REPLAY_BUFFER_PRINCIPLES = {
    "capability_coverage": "Replay must cover ALL general capabilities — not just random web text",
    "stratification": "Sample proportional to capability importance — reasoning > filler text",
    "freshness_rotation": "Rotate replay buffer every N steps to prevent overfitting to replay samples",
    "contamination_free": "Replay buffer must not contain benchmarks — contamination through replay is still contamination",
}

def curate_replay_buffer(general_corpus, replay_token_budget, critical_capabilities):
    """Build a replay buffer that preserves general capabilities during CPT."""
    
    # Stratify general corpus by capability
    stratified = _stratify_by_capability(general_corpus, critical_capabilities)
    
    # Allocate budget proportional to capability importance
    importance_weights = {
        "reasoning": 0.25,
        "factual_knowledge": 0.20,
        "instruction_following": 0.20,
        "multilingual": 0.15,
        "code": 0.10,
        "safety": 0.10,
    }
    
    buffer = {}
    for capability, weight in importance_weights.items():
        capability_data = stratified.get(capability, [])
        token_allocation = int(replay_token_budget * weight)
        buffer[capability] = _sample_to_token_budget(capability_data, token_allocation)
    
    return {
        "buffer": buffer,
        "total_tokens": sum(len(v) for v in buffer.values()),
        "capability_coverage": sum(1 for v in buffer.values() if v) / len(critical_capabilities),
        "rotation_plan": "rotate_25%_every_1000_steps",
    }
```

## Forgetting Detection

```python
FORGETTING_DETECTION = {
    "checkpoint_interval": "every_500_steps — frequent enough to catch forgetting early",
    "benchmark_suite": "pre_cpt_baseline_suite — same benchmarks used pre-CPT",
    "forgetting_threshold": ">2% absolute degradation on any capability → trigger investigation",
    "catastrophic_threshold": ">5% absolute degradation on any capability → halt CPT, increase replay ratio",
}

def detect_forgetting(pre_cpt_baseline, current_checkpoint, capability_tests):
    """Is the model forgetting general capabilities during CPT?"""
    
    forgetting_report = {"capabilities": {}, "overall": {}}
    
    for capability, tests in capability_tests.items():
        pre_score = pre_cpt_baseline.get(capability, 0)
        current_score = _evaluate_capability(current_checkpoint, tests)
        
        degradation = pre_score - current_score
        degradation_pct = (degradation / max(pre_score, 0.01)) * 100
        
        forgetting_report["capabilities"][capability] = {
            "pre_cpt": pre_score,
            "current": current_score,
            "degradation_pct": degradation_pct,
            "status": "CATASTROPHIC" if degradation_pct > 5
                      else "FORGETTING" if degradation_pct > 2
                      else "STABLE" if degradation_pct > -1  # slight improvement OK
                      else "IMPROVING",
            "action": "HALT_CPT_INCREASE_REPLAY" if degradation_pct > 5
                      else "INVESTIGATE_INCREASE_MONITORING" if degradation_pct > 2
                      else "CONTINUE",
        }
    
    # Overall forgetting score
    degradations = [v["degradation_pct"] for v in forgetting_report["capabilities"].values()]
    forgetting_report["overall"] = {
        "mean_degradation": sum(degradations) / max(len(degradations), 1),
        "max_degradation": max(degradations) if degradations else 0,
        "capabilities_stable": sum(1 for v in forgetting_report["capabilities"].values() 
                                   if v["status"] == "STABLE"),
        "capabilities_forgetting": sum(1 for v in forgetting_report["capabilities"].values()
                                       if v["status"] in ["FORGETTING", "CATASTROPHIC"]),
        "cpt_safe": all(v["status"] in ["STABLE", "IMPROVING"] 
                        for v in forgetting_report["capabilities"].values()),
    }
    
    return forgetting_report
```

## Progressive Domain Adaptation

```python
def progressive_domain_schedule(domains, total_tokens, base_model_capabilities):
    """When adapting to multiple domains, sequence them for minimum interference."""
    
    # Compute domain similarity matrix
    similarity = _domain_similarity_matrix(domains, base_model_capabilities)
    
    # Order by: closest to base first, then progressively more distant
    domain_distances = {
        domain: _domain_distance(domain, base_model_capabilities["pretraining_domains"])
        for domain in domains
    }
    ordered = sorted(domain_distances.items(), key=lambda x: x[1])
    
    # Allocate tokens: more to distant domains
    total_distance = sum(v for _, v in ordered)
    schedule = []
    cumulative_tokens = 0
    
    for domain, distance in ordered:
        # Distant domains get proportionally more tokens
        domain_tokens = int(total_tokens * (distance / max(total_distance, 0.01)))
        schedule.append({
            "stage": len(schedule),
            "domain": domain,
            "distance_from_base": distance,
            "tokens": domain_tokens,
            "cumulative_tokens": cumulative_tokens,
            "replay_ratio": min(0.5, 0.2 + distance * 0.3),  # more replay for distant domains
        })
        cumulative_tokens += domain_tokens
    
    return {
        "schedule": schedule,
        "total_stages": len(schedule),
        "principle": "closest_domains_first — minimize interference by gradual drift",
    }
```

## CPT Data Quality Gates

```python
CPT_DATA_CHECKS = {
    "contamination": "Domain data must not contain evaluation benchmarks for target domain",
    "duplication": "Deduplicate domain data against replay buffer — no cross-contamination",
    "format_consistency": "Domain data format must match pretraining format — no distribution shift from formatting",
    "quality_threshold": "Domain data must meet minimum perplexity threshold — no garbage in",
    "license_compliance": "Domain data must be licensed for training — especially critical for medical/legal",
}

def cpt_data_audit(domain_corpus, replay_buffer, benchmarks):
    audit = {}
    
    # Contamination check
    contamination = _scan_for_benchmarks(domain_corpus, benchmarks["excluded"])
    audit["contamination"] = {
        "clean": len(contamination) == 0,
        "contaminated_examples": len(contamination),
        "severity": "BLOCK_TRAINING" if len(contamination) > 0 else "PASS",
    }
    
    # Duplication check
    duplicates = _cross_corpus_duplicates(domain_corpus, replay_buffer)
    audit["duplication"] = {
        "clean": len(duplicates) == 0,
        "overlap_pct": len(duplicates) / max(len(domain_corpus), 1) * 100,
        "severity": "REMOVE_OVERLAP" if len(duplicates) > 0 else "PASS",
    }
    
    # Format consistency
    format_ok = _check_format_consistency(domain_corpus)
    audit["format"] = {
        "consistent": format_ok,
        "issues": "format_mismatch_detected" if not format_ok else None,
    }
    
    all_pass = all(
        a.get("severity") == "PASS" and a.get("consistent", True) != False
        for a in audit.values()
    )
    
    return {"audit": audit, "cpt_ready": all_pass,
            "blockers": [k for k, v in audit.items() if v.get("severity") == "BLOCK_TRAINING"]}
```

## Quality Gate

- Domain mixing ratio selected based on measured domain distance from pretraining.
- Replay buffer covers all critical capabilities with proportional importance weighting.
- Forgetting detection runs every 500 steps — catastrophic forgetting halts CPT immediately.
- Multi-domain schedules ordered by distance from base — closest first.
- CPT data passes contamination, duplication, format, quality, and license checks.
- Post-CPT evaluation confirms domain improvement AND general capability preservation.
