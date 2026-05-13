---
name: annotation-economics-task
description: Optimize annotation economics — fatigue modeling, quality decay curves per session, task-specialization matching, cost-quality tradeoff curves, and disagreement value quantification.
recommended_skills:
  - label-quality-audit
  - bias-audit
  - embedding-analysis
recommended_guidelines:
  - annotation-management-task
  - cost-model-task
  - training-data-design-principles
---

## Overview

Annotation economics treats annotators as a scarce resource with cognitive limits, skill profiles, and quality decay. This guideline models where money stops helping, who should label what, and why annotator disagreement is information, not noise.

## Phase 1: Annotator Fatigue Modeling

### The Quality Decay Curve

```python
def model_fatigue_decay(annotator_sessions, task_type):
    """
    Each session: (duration_minutes, time_of_day, accuracy_trajectory)
    accuracy_trajectory: per-10-min accuracy on calibration examples
    """
    all_trajectories = []
    
    for session in annotator_sessions:
        times = np.arange(0, session["duration"], 10)
        accuracies = session["accuracy_trajectory"]
        
        # Fit exponential decay: acc(t) = a * exp(-λ*t) + c
        # Where c = asymptotic accuracy (fatigue floor)
        # λ = decay rate (higher = faster fatigue)
        p0 = [1.0, 0.01, 0.5]
        try:
            params, _ = curve_fit(_exp_decay, times, accuracies, p0=p0, maxfev=5000)
            fatigue = {
                "initial_accuracy": params[0] + params[2],
                "asymptotic_accuracy": params[2],
                "decay_rate": params[1],
                "time_to_90pct_floor": -np.log(0.1) / params[1],
                "total_quality_loss": (params[0] + params[2]) - params[2],
            }
        except:
            fatigue = {"fit_failed": True}
        
        fatigue["duration"] = session["duration"]
        fatigue["time_of_day"] = session.get("time_of_day")
        fatigue["task_type"] = task_type
        
        all_trajectories.append(fatigue)
    
    # Population summary
    decay_rates = [t["decay_rate"] for t in all_trajectories if "decay_rate" in t]
    
    return {
        "individual_curves": all_trajectories,
        "mean_decay_rate": np.mean(decay_rates) if decay_rates else None,
        "median_time_to_floor": np.median([t.get("time_to_90pct_floor", 0) for t in all_trajectories]),
        "recommended_session_limit": int(np.median([t.get("time_to_90pct_floor", 60) for t in all_trajectories]) * 0.8),
    }

def _exp_decay(t, a, lam, c):
    return a * np.exp(-lam * t) + c
```

### Fatigue Mitigation Strategies

| Strategy | When | Impact |
|-------|-------|-------|
| **Session length limit** | Always | Cap at `recommended_session_limit` minutes |
| **Mandatory breaks** | Sessions > 60 min | 5 min break every 25 min (Pomodoro) |
| **Task rotation** | Repetitive tasks | Switch between easy/hard, different modalities |
| **Time-of-day matching** | When possible | Let annotators work in their peak hours |
| **Calibration interleaving** | All sessions | Inject gold-standard examples every 20 items |
| **Recovery tracking** | After long breaks | Check if accuracy returns to initial level |

## Phase 2: Task-Specialization Matching

### Who Should Label What

```python
def assign_annotators_to_tasks(annotators, tasks, history):
    """
    Match annotators to tasks based on:
    - Historical accuracy on similar tasks
    - Calibration (how well their confidence matches correctness)
    - Speed
    - Self-reported domain expertise
    """
    assignments = {}
    
    for task_type, task_examples in tasks.items():
        # Score each annotator for this task type
        scores = {}
        for annotator_id, profile in annotators.items():
            # Historical accuracy on this task type
            past_accuracy = history.get((annotator_id, task_type), 0.5)
            
            # Calibration: do they know when they're right?
            calibration = profile.get("calibration_score", 0.5)
            # High calibration = annotator's confidence predicts their accuracy
            
            # Domain match: self-reported expertise
            domain_match = profile.get("expertise", {}).get(task_type, 0.0)
            
            # Speed bonus (efficiency, not carelessness)
            speed = min(profile.get("tokens_per_minute", 0) / 30.0, 1.0)
            # Penalize if speed correlates with errors
            speed_quality_corr = profile.get("speed_quality_correlation", 0)
            if speed_quality_corr < -0.3:  # faster = worse
                speed *= 0.5
            
            scores[annotator_id] = (
                0.4 * past_accuracy +
                0.2 * calibration +
                0.25 * domain_match +
                0.15 * speed
            )
        
        # Assign top N annotators per task
        sorted_annotators = sorted(scores.items(), key=lambda x: -x[1])
        assignments[task_type] = {
            "primary": sorted_annotators[:3],
            "reviewer": sorted_annotators[3:5],
        }
    
    return assignments
```

## Phase 3: Cost-Quality Tradeoff Curves

### Where Does More Money Stop Helping?

```python
def cost_quality_curve(cost_per_example_levels, quality_at_each_level):
    """
    cost_per_example_levels: [$0.10, $0.50, $1.00, $2.00, $5.00, $10.00]
    quality_at_each_level: corresponding inter-annotator F1 scores
    
    Find the knee: where marginal quality gain / marginal cost < threshold.
    """
    costs = np.array(cost_per_example_levels)
    quality = np.array(quality_at_each_level)
    
    # Marginal gain / marginal cost
    marginal_gain = np.diff(quality) / np.diff(costs)
    
    # Knee: where marginal gain drops below threshold
    knee_threshold = 0.01  # less than 1% quality per dollar
    knee_idx = np.where(marginal_gain < knee_threshold)[0]
    
    optimal_cost = costs[knee_idx[0] + 1] if len(knee_idx) > 0 else costs[-1]
    
    return {
        "costs": costs.tolist(),
        "quality": quality.tolist(),
        "marginal_gain_per_dollar": marginal_gain.tolist(),
        "knee_cost": float(optimal_cost),
        "knee_quality": float(quality[knee_idx[0] + 1]) if len(knee_idx) > 0 else float(quality[-1]),
        "diminishing_returns": len(knee_idx) > 0,
        "recommendation": f"Spend ${optimal_cost:.2f}/example — beyond this, marginal gain < 1% per dollar",
    }
```

### Annotation Budget Allocation

| Budget Tier | Strategy | Expected Quality |
|-------|-------|-------|
| **$0.01-0.10/ex** | Single annotator, simple binary | 85-90% accuracy |
| **$0.10-0.50/ex** | Single annotator + spot check | 90-93% accuracy |
| **$0.50-2.00/ex** | Dual annotation, adjudicate disagreements | 93-96% accuracy |
| **$2.00-10.00/ex** | Expert + reviewer + calibration QC | 96-98% accuracy |
| **$10.00+/ex** | Multi-expert panel, consensus-driven | 98%+ accuracy |
| **> $50/ex** | Diminishing returns — usually not worth it | Marginal gain < 0.5% |

## Phase 4: Disagreement Value Quantification

### Conflict Is Information

```python
def quantify_disagreement_value(annotations, examples, model=None):
    """
    Where annotators disagree, there's signal about:
    - Example difficulty (hard examples produce more disagreement)
    - Label ambiguity (the label definition needs refinement)
    - Annotator bias (systematic disagreement reveals perspective)
    - Model failure prediction (model will fail where humans disagree)
    """
    # Compute per-example disagreement
    n_annotators = annotations.shape[1]
    per_example_agreement = []
    
    for i, example_anns in enumerate(annotations):
        labels, counts = np.unique(example_anns, return_counts=True)
        agreement = counts.max() / n_annotators
        
        per_example_agreement.append({
            "example_id": examples[i]["id"],
            "agreement": agreement,
            "disagreement": 1 - agreement,
            "n_unique_labels": len(labels),
            "entropy": -np.sum((counts / n_annotators) * np.log(counts / n_annotators + 1e-10)),
        })
    
    # Disagreement correlates with model failure
    if model is not None:
        model_proba = model.predict_proba([ex["text"] for ex in examples])
        model_confidence = model_proba.max(axis=1)
        
        disagreement_scores = [d["disagreement"] for d in per_example_agreement]
        corr = np.corrcoef(disagreement_scores, 1 - model_confidence)[0, 1]
        
        # High disagreement → low model confidence means:
        # "Where humans disagree, models are uncertain"
        # This is GOOD — it means the model captures genuine ambiguity
        pass
    
    # Value of disagreement:
    # 1. Hard examples identified → use for active learning
    # 2. Label definition issues found → refine annotation guidelines
    # 3. Annotator bias detected → systematic disagreement patterns
    # 4. Model failure predicted → high-disagreement examples will break models
    
    return {
        "per_example": per_example_agreement,
        "mean_agreement": np.mean([d["agreement"] for d in per_example_agreement]),
        "high_disagreement_examples": [d for d in per_example_agreement if d["disagreement"] > 0.5],
        "model_disagreement_correlation": corr if model else None,
        "disagreement_is_information": True,  # always
    }
```

## Quality Gate

- Fatigue curves modeled per annotator; session limits enforced.
- Task-assignment based on historical accuracy + calibration + domain expertise.
- Cost-quality knee identified; budget allocated accordingly.
- Disagreement quantified and fed back to: active learning, guideline refinement, bias detection.
- No annotator scheduled beyond their fatigue floor duration.
