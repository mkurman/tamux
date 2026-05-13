---
name: model-compression-data-task
description: Curate data for model compression — pruning calibration sets, distillation teacher-target corpora, quantization calibration data, and post-compression validation benchmarks. Incorporates SlimQwen (arXiv:2605.08738, 2026), DistilQwen2.5 (ACL 2025 Industry), and REAP MoE pruning. Compression is a data problem.
recommended_skills: [dataset-splitting, embedding-analysis, benchmark-contamination-scan]
recommended_guidelines: [data-compression-learning-task, specialized-training-data-task, scaling-law-data-task]
---

## Overview

Pruning, distillation, and quantization are not just architecture operations — they are data curation problems. What data you use for importance scoring determines which weights survive. What data the teacher generates determines what the student learns. What data you use for calibration determines quantization accuracy. This guideline treats model compression as a data pipeline: curate pruning calibration sets, design teacher-student distillation corpora, select quantization calibration data, and validate that compressed models actually preserved capability.

## 2026 Research Context

| Paper | Venue | Contribution |
|-------|-------|-------------|
| **SlimQwen** (Tang et al.) | arXiv:2605.08738, May 2026 | Pruning+distillation for MoE pre-training; progressive pruning > one-shot; MTP distillation; partial-preservation expert merging; 80A3B→23A2B competitive |
| **DistilQwen2.5** (Wang et al.) | ACL 2025 Industry | Multi-agent teachers select/rewrite instruction data; model fusion for hidden knowledge transfer |
| **REAP** | OpenMOSE, 2026 | Router-weighted Expert Activation Pruning for MoE models |

## Phase 1: Pruning Calibration Data

```python
PRUNING_STRATEGIES = {
    "magnitude": {
        "description": "Remove weights with smallest absolute values",
        "data_requirement": "none — weight values alone",
        "risk": "removes_rare_but_critical_capability_weights",
        "slimqwen_finding": "Effective as initialization baseline; one-shot works but progressive better",
    },
    "activation_based": {
        "description": "Remove weights/experts with lowest activation on calibration data",
        "data_requirement": "representative_calibration_corpus — must match deployment distribution",
        "risk": "calibration_data_bias → pruned_model_bias",
        "slimqwen_finding": "Expert compression methods converge to similar final performance after continued training",
    },
    "gradient_based": {
        "description": "Remove weights with smallest gradient × weight product",
        "data_requirement": "training_data_sample — must match training distribution",
        "risk": "gradient_noise_on_small_batch → wrong_pruning_decisions",
        "slimqwen_finding": "Pruning pretrained MoE consistently outperforms training target architecture from scratch",
    },
}

def curate_pruning_calibration_data(training_corpus, pruning_strategy, calibration_size=10000):
    """What data should the pruner see to decide which weights matter?"""
    
    if pruning_strategy == "activation_based":
        # Must match deployment distribution — sample from intended use cases
        corpus = _stratified_sample(training_corpus, 
                                     strata=["task_type", "domain", "language"],
                                     target_total=calibration_size)
        quality_checks = ["no_contamination_with_benchmarks", "covers_all_intended_capabilities",
                          "includes_edge_cases", "distribution_matches_deployment"]
    
    elif pruning_strategy == "gradient_based":
        # Must match training distribution — sample from pretraining data
        corpus = _random_sample(training_corpus, calibration_size)
        quality_checks = ["same_distribution_as_train", "sufficient_batch_size_for_stable_gradients",
                          "includes_rare_patterns", "no_deduplication_bias"]
    
    else:  # magnitude — no data needed
        corpus = None
        quality_checks = ["n/a"]
    
    return {
        "strategy": pruning_strategy,
        "calibration_corpus": corpus,
        "corpus_size": len(corpus) if corpus else 0,
        "quality_checks": quality_checks,
        "validation": "verify_pruned_model_on_held_out_capability_tests",
    }
```

### MoE Expert Preservation (SlimQwen Partial-Preservation Merging)

```python
def partial_preservation_expert_merge(expert_activations, preservation_ratio=0.2):
    """
    SlimQwen finding: Instead of dropping all non-top experts, preserve a fraction
    of secondary experts by merging their weights into the primary experts.
    This improves downstream performance across most benchmarks.
    """
    num_experts = len(expert_activations)
    num_preserve = max(1, int(num_experts * preservation_ratio))
    
    # Sort experts by activation frequency on calibration data
    ranked = sorted(expert_activations.items(), 
                   key=lambda x: x[1]["mean_activation"], reverse=True)
    
    primary = [e for e, _ in ranked[:num_experts - num_preserve]]
    secondary = [e for e, _ in ranked[num_experts - num_preserve:]]
    
    # Merge secondary into nearest primary by activation correlation
    merge_map = {}
    for sec_expert in secondary:
        correlations = {
            prim: _activation_correlation(sec_expert, prim, expert_activations)
            for prim in primary
        }
        best_primary = max(correlations, key=correlations.get)
        merge_map[sec_expert] = {
            "merge_into": best_primary,
            "correlation": correlations[best_primary],
            "method": "weighted_average_by_activation_frequency",
        }
    
    return {
        "primary_experts": primary,
        "secondary_merged": merge_map,
        "preservation_ratio": preservation_ratio,
        "slimqwen_note": "Partial preservation outperforms full expert dropping",
    }
```

## Phase 2: Distillation Data Construction

```python
DISTILLATION_DATA_STRATEGIES = {
    "logit_distillation": {
        "description": "Student learns to match teacher's output probabilities",
        "data_requirement": "teacher_logits_on_diverse_corpus",
        "corpus_curation": "same_as_student_training_but_with_teacher_forward_pass",
        "slimqwen_finding": "Combining KD loss with LM loss outperforms KD alone, especially on knowledge-intensive tasks",
        "temperature": "soften_logits — higher T (3-5) reveals teacher uncertainty structure",
    },
    "multi_agent_teacher": {
        "description": "Multiple teacher LLMs select/rewrite/refine instruction data (DistilQwen2.5)",
        "data_requirement": "instruction_dataset_curated_by_teacher_agents",
        "corpus_curation": "strong_teacher_judges_quality → medium_teacher_rewrites → weak_teacher_validates_format",
        "slimqwen_finding": "Multi-agent selection produces more suitable instruction pairs for student learning",
    },
    "mtp_distillation": {
        "description": "Multi-token prediction distillation — student learns to predict N future tokens (SlimQwen)",
        "data_requirement": "teacher_mtp_logits — multi-token future predictions from teacher",
        "corpus_curation": "diverse_next_N_tokens — requires longer sequences with coherent futures",
        "slimqwen_finding": "MTP distillation yields consistent gains; novel contribution beyond standard KD",
    },
    "feature_distillation": {
        "description": "Student learns to match teacher's hidden representations",
        "data_requirement": "teacher_hidden_states_on_diverse_corpus",
        "corpus_curation": "layer_aligned_teacher_representations — requires same dimensionality or projection",
        "slimqwen_finding": "Model fusion for fine-grained hidden knowledge transfer (DistilQwen2.5 finding)",
    },
}

def curate_distillation_corpus(teacher_model, student_architecture, distillation_type, raw_corpus):
    strategies = DISTILLATION_DATA_STRATEGIES
    
    if distillation_type == "logit_distillation":
        # Generate teacher logits on diverse corpus
        corpus = _diversity_sample(raw_corpus, n=50000, 
                                    dimensions=["topic", "difficulty", "length"])
        teacher_outputs = _generate_teacher_logits(teacher_model, corpus)
        return {
            "corpus": corpus,
            "teacher_logits": teacher_outputs,
            "temperature": strategies["logit_distillation"]["temperature"],
            "slimqwen_tip": "Combine KD loss (0.3 weight) with LM loss (0.7 weight) for best results",
        }
    
    elif distillation_type == "mtp_distillation":
        # Require longer sequences for multi-token prediction
        long_corpus = [ex for ex in raw_corpus if len(ex["tokens"]) >= 512]
        teacher_mtp = _generate_teacher_mtp_logits(teacher_model, long_corpus, n_future=4)
        return {
            "corpus": long_corpus,
            "teacher_mtp_logits": teacher_mtp,
            "n_future_tokens": 4,
            "slimqwen_tip": "MTP distillation provides consistent gains; use depth=4 for compute-quality tradeoff",
        }
    
    elif distillation_type == "multi_agent_teacher":
        # DistilQwen2.5 pipeline: select → rewrite → refine
        selected = _teacher_agent_select(raw_corpus, teacher_model, quality_threshold=0.7)
        rewritten = _teacher_agent_rewrite(selected, teacher_model)
        refined = _teacher_agent_refine(rewritten, teacher_model)
        return {
            "corpus": refined,
            "selection_ratio": len(refined) / max(len(raw_corpus), 1),
            "distilqwen_note": "Multi-agent pipeline: select → rewrite → refine",
        }
    
    else:
        return {"error": f"unknown_distillation_type: {distillation_type}"}
```

### Progressive Pruning Schedule (SlimQwen Finding)

```python
def progressive_pruning_schedule(initial_architecture, target_architecture, 
                                  total_training_tokens, pruning_stages=5):
    """
    SlimQwen key finding: Progressive pruning schedules outperform one-shot compression.
    Gradual architecture transitions lead to better optimization trajectories.
    """
    schedule = []
    tokens_per_stage = total_training_tokens // pruning_stages
    
    for stage in range(pruning_stages + 1):
        progress = stage / pruning_stages
        current_experts = int(initial_architecture["num_experts"] - 
                             (initial_architecture["num_experts"] - target_architecture["num_experts"]) * progress)
        current_active = int(initial_architecture["active_experts"] -
                            (initial_architecture["active_experts"] - target_architecture["active_experts"]) * progress)
        
        schedule.append({
            "stage": stage,
            "progress": f"{progress:.0%}",
            "num_experts": current_experts,
            "active_experts": current_active,
            "tokens_this_stage": tokens_per_stage,
            "cumulative_tokens": tokens_per_stage * stage,
            "action": "TRAIN" if stage == 0 else "PRUNE_AND_TRAIN",
            "slimqwen_note": "Gradual architecture transitions → better optimization" if stage > 0 else None,
        })
    
    return {
        "schedule": schedule,
        "total_stages": pruning_stages + 1,
        "total_tokens": total_training_tokens,
        "compression_ratio": f"{initial_architecture['num_experts']}A{initial_architecture['active_experts']}B → "
                            f"{target_architecture['num_experts']}A{target_architecture['active_experts']}B",
        "slimqwen_reference": "80A3B → 23A2B achieved with progressive schedule",
    }
```

## Phase 3: Quantization Calibration Data

```python
QUANTIZATION_CALIBRATION = {
    "static_quantization": {
        "data_need": "representative_sample_for_activation_range_estimation",
        "sample_size": "100-1000 examples — enough to capture activation distribution",
        "curation_rule": "cover_full_value_range — include edge cases, not just typical cases",
    },
    "dynamic_quantization": {
        "data_need": "none_at_calibration_time — ranges computed per-input",
        "sample_size": "n/a",
        "curation_rule": "runtime-dependent — latency impact varies by input shape",
    },
    "quantization_aware_training": {
        "data_need": "full_training_corpus_with_fake_quantization_nodes",
        "sample_size": "full_training_set",
        "curation_rule": "same_as_original_training — distribution must match",
    },
}

def calibration_data_quality(calibration_corpus, model_activations):
    """Is calibration data capturing the full activation range?"""
    activation_stats = {}
    for layer_name, activations in model_activations.items():
        calibration_range = (activations.min(), activations.max())
        expected_range = _deployment_activation_range(layer_name)
        
        coverage = {
            "min_covered": calibration_range[0] <= expected_range[0],
            "max_covered": calibration_range[1] >= expected_range[1],
            "outlier_coverage": _outliers_represented(activations, expected_range),
        }
        activation_stats[layer_name] = {
            "calibration_range": calibration_range,
            "expected_range": expected_range,
            "coverage": coverage,
            "sufficient": all(coverage.values()),
        }
    
    return {
        "layers_checked": len(activation_stats),
        "layers_sufficient": sum(1 for s in activation_stats.values() if s["sufficient"]),
        "calibration_quality": "GOOD" if all(s["sufficient"] for s in activation_stats.values())
                               else "INSUFFICIENT — add edge cases to calibration corpus",
    }
```

## Phase 4: Post-Compression Validation

```python
POST_COMPRESSION_TESTS = {
    "capability_preservation": {
        "description": "Does the compressed model retain the same capabilities?",
        "benchmarks": ["production_eval_suite", "capability_specific_tests"],
        "threshold": "no_more_than_2%_degradation_on_any_capability",
        "slimqwen_finding": "Competitive performance after 80A3B→23A2B compression",
    },
    "knowledge_retention": {
        "description": "Does the compressed model retain factual knowledge?",
        "benchmarks": ["knowledge_intensive_benchmarks", "closed_book_qa"],
        "threshold": "no_more_than_5%_degradation_on_knowledge_tasks",
        "slimqwen_finding": "KD+LM loss combination especially important for knowledge tasks",
    },
    "regression_detection": {
        "description": "Find specific examples where compressed model degrades",
        "benchmarks": ["large_diff_subset — examples where teacher vs student diverge most"],
        "threshold": "investigate_top_1%_largest_regressions",
    },
    "fairness_preservation": {
        "description": "Does compression preserve fairness properties?",
        "benchmarks": ["bias_audit_on_compressed_model"],
        "threshold": "no_new_disparities_introduced_by_compression",
    },
}

def validate_compression(original_model, compressed_model, test_suites):
    results = {}
    for test_name, test_config in POST_COMPRESSION_TESTS.items():
        original_score = evaluate(original_model, test_suites.get(test_name, []))
        compressed_score = evaluate(compressed_model, test_suites.get(test_name, []))
        degradation = (original_score - compressed_score) / max(original_score, 0.01)
        
        passed = degradation <= _threshold_for_test(test_name)
        results[test_name] = {
            "original": original_score,
            "compressed": compressed_score,
            "degradation_pct": degradation * 100,
            "passed": passed,
            "threshold": test_config["threshold"],
        }
    
    all_passed = all(r["passed"] for r in results.values())
    return {
        "tests": results,
        "overall_pass": all_passed,
        "compression_safe": all_passed,
        "investigate": [k for k, v in results.items() if not v["passed"]],
    }
```

## MoE-Specific Considerations

```python
MOE_COMPRESSION_CHECKLIST = {
    "expert_diversity": "Did pruning preserve expert specialization diversity?",
    "routing_balance": "Did pruning preserve balanced routing across remaining experts?",
    "load_imbalance_risk": "Fewer experts → higher load per expert → potential bottleneck",
    "slimqwen_progressive": "Use progressive pruning, not one-shot, for MoE compression",
    "slimqwen_partial_preservation": "Preserve 20% of secondary experts via merging, not dropping",
    "slimqwen_mtp_distillation": "Multi-token prediction distillation provides consistent gains",
    "distilqwen_multi_agent": "Multi-agent teacher selection/rewriting improves instruction data quality",
}

def moe_compression_audit(original_moe, compressed_moe, calibration_data):
    results = {}
    
    # Expert diversity
    orig_specialization = _expert_specialization_score(original_moe, calibration_data)
    comp_specialization = _expert_specialization_score(compressed_moe, calibration_data)
    results["expert_diversity"] = {
        "original": orig_specialization,
        "compressed": comp_specialization,
        "preserved": comp_specialization >= orig_specialization * 0.9,
    }
    
    # Routing balance
    orig_routing = _routing_entropy(original_moe, calibration_data)
    comp_routing = _routing_entropy(compressed_moe, calibration_data)
    results["routing_balance"] = {
        "original_entropy": orig_routing,
        "compressed_entropy": comp_routing,
        "balanced": comp_routing >= orig_routing * 0.85,
    }
    
    return results
```

## Quality Gate

- Pruning calibration data matches deployment distribution (activation) or training distribution (gradient).
- Distillation corpus covers all intended capabilities with teacher logits at appropriate temperature.
- Quantization calibration data covers full activation range including edge cases.
- Post-compression validation passes all four test categories (capability, knowledge, regression, fairness).
- MoE-specific audit: expert diversity preserved, routing balanced, SlimQwen progressive schedule used.
- Progressive pruning schedule documented; compression ratio and token budget explicit.
