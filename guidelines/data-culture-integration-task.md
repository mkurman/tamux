---
name: data-culture-integration-task
description: Build a data culture that sustains — diagnose culture type, measure data literacy, engineer psychological safety for data honesty, align incentives with data quality, and institutionalize data rituals. Culture eats data strategy for breakfast.
recommended_skills: [bias-audit, data-card-writer, team-operations-data-task]
recommended_guidelines: [data-learning-organization-task, organizational-implementation-data-task, data-communication-integration-task]
---

## Overview

The best data strategy collapses on contact with a culture that punishes bad news, rewards heroics over process, or treats data as ammunition for pre-existing opinions. This guideline treats culture as a data engineering substrate: diagnose the culture type, measure the gap to a healthy data culture, engineer safety for honest data interpretation, align incentives with data quality behaviors, and institutionalize data rituals that survive leadership changes.

## Culture Type → Data Practice Diagnosis

```python
CULTURE_TYPES = {
    "blame": {
        "symptoms": ["metrics_used_to_punish", "bad_news_hidden", "heroics_rewarded_over_process",
                      "data_requests_defensive", "reports_positive_only"],
        "data_impact": "Data quality deteriorates silently — nobody flags issues, metrics are gamed",
        "intervention": "psychological_safety_first — celebrate finding problems before celebration of hitting targets",
        "transition_path": "blame → fear_reduction → safety → accountability → learning",
    },
    "bureaucratic": {
        "symptoms": ["process_over_insight", "reports_never_read", "data_collection_as_checkbox",
                      "no_questions_about_numbers", "reports_filed_not_used"],
        "data_impact": "Data is produced but never consumed — cost without value",
        "intervention": "demand_signal_first — stop producing reports nobody reads, build pull not push",
        "transition_path": "bureaucratic → demand_discovery → insight_consumption → action → value",
    },
    "political": {
        "symptoms": ["data_weaponized", "analysis_shopping", "methodology_attacked_when_inconvenient",
                      "numbers_negotiated", "competing_versions_of_truth"],
        "data_impact": "Multiple versions of truth — data loses authority, decisions made on power not evidence",
        "intervention": "single_source_of_truth_first — one version, transparent methodology, independent audit",
        "transition_path": "political → truth_consolidation → methodology_trust → evidence_basis → data_authority",
    },
    "learning": {
        "symptoms": ["bad_news_surfaced_quickly", "decisions_revisited_with_data", "experiments_encouraged",
                      "failures_analyzed", "data_questions_welcomed"],
        "data_impact": "Data quality improves continuously — feedback loops close, assumptions tested",
        "intervention": "sustain_and_deepen — guard against regression to blame or political under pressure",
        "transition_path": "sustain — cultural regression detection → protection mechanisms → deepening",
    },
}

def diagnose_culture(organization_signals):
    """Which culture type is dominant?"""
    scores = {}
    for culture_type, profile in CULTURE_TYPES.items():
        symptom_count = len(profile["symptoms"])
        present = sum(1 for s in profile["symptoms"] 
                     if organization_signals.get(s, False))
        scores[culture_type] = present / symptom_count
    
    dominant = max(scores, key=scores.get)
    return {
        "dominant_culture": dominant,
        "profile": CULTURE_TYPES[dominant],
        "scores": scores,
        "healthy": dominant == "learning",
        "intervention": CULTURE_TYPES[dominant]["intervention"],
    }
```

## Data Literacy Assessment

```python
DATA_LITERACY_LEVELS = {
    "L0_data_blind": "Cannot interpret a chart. Does not understand averages vs distributions.",
    "L1_data_aware": "Can read a simple chart. Understands averages. Confuses correlation with causation.",
    "L2_data_literate": "Can interpret distributions. Understands correlation vs causation. Can question methodology.",
    "L3_data_fluent": "Can design analyses. Understands sampling, bias, confidence. Can critique methods.",
    "L4_data_leader": "Can set data strategy. Understands limits of data. Champions data culture.",
}

def assess_organization_literacy(population):
    levels = {"L0": 0, "L1": 0, "L2": 0, "L3": 0, "L4": 0}
    for person in population:
        level = person.get("data_literacy", "L0")
        levels[level] = levels.get(level, 0) + 1
    
    total = sum(levels.values())
    return {
        "distribution": {k: v / max(total, 1) for k, v in levels.items()},
        "literate_and_above": sum(levels.get(f"L{i}", 0) for i in range(2, 5)) / max(total, 1),
        "literacy_gap": "SIGNIFICANT" if levels.get("L0", 0) + levels.get("L1", 0) > total * 0.4
                        else "MODERATE" if levels.get("L0", 0) + levels.get("L1", 0) > total * 0.15
                        else "HEALTHY",
    }
```

## Psychological Safety for Data Honesty

```python
DATA_SAFETY_INDICATORS = {
    "can_challenge_metric": "People openly question whether a metric measures what it claims",
    "can_report_bad_news": "Bad news surfaces quickly and is received without retribution",
    "can_disagree_with_analysis": "Methodological disagreements are welcomed as rigor, not seen as obstruction",
    "can_admit_data_error": "Data errors are flagged immediately, not hidden hoping nobody notices",
    "can_kill_project_with_data": "Projects can be killed based on data without career penalty for proponents",
}

def measure_data_safety(organization):
    safety_scores = {}
    for indicator, description in DATA_SAFETY_INDICATORS.items():
        safety_scores[indicator] = organization.get(indicator, 0)  # 0-1 scale
    
    overall = sum(safety_scores.values()) / max(len(safety_scores), 1)
    
    return {
        "overall_safety_score": overall,
        "indicators": safety_scores,
        "weakest_link": min(safety_scores, key=safety_scores.get) if safety_scores else None,
        "safe_enough": overall >= 0.7,
        "critical_risk": overall < 0.4,
    }
```

## Incentive Alignment

```python
INCENTIVE_ALIGNMENT_CHECK = {
    "data_quality": {
        "good_incentive": "Data quality metrics in performance review; quality incidents treated as system failures not individual failures",
        "bad_incentive": "Speed rewarded over quality; 'just ship it' culture; data errors punished individually",
    },
    "metric_definition": {
        "good_incentive": "Ownership of metric definition separated from ownership of metric outcome",
        "bad_incentive": "Same person defines the metric AND is evaluated on it — Goodhart's law guaranteed",
    },
    "experimentation": {
        "good_incentive": "Well-designed negative results celebrated; learning documented and rewarded",
        "bad_incentive": "Only positive results rewarded; experimentation seen as risky to career",
    },
    "data_sharing": {
        "good_incentive": "Data sharing recognized in performance; data hoarding flagged as organizational risk",
        "bad_incentive": "Data hoarded as power source; 'my data' mentality rewarded",
    },
    "methodology": {
        "good_incentive": "Rigorous methods rewarded even when results are inconvenient; methodology challenges welcomed",
        "bad_incentive": "Methodology criticized only when results are inconvenient; 'analysis shopping' tolerated",
    },
}

def audit_incentives(org_practices):
    misalignments = []
    for dimension, check in INCENTIVE_ALIGNMENT_CHECK.items():
        practice = org_practices.get(dimension, {})
        if practice.get("type") == "bad_incentive" or practice.get("alignment_score", 0) < 0.5:
            misalignments.append({
                "dimension": dimension,
                "current_practice": practice.get("description", "unknown"),
                "good_practice": check["good_incentive"],
                "risk": practice.get("risk", "Goodhart's law likely active"),
            })
    
    return {
        "total_dimensions": len(INCENTIVE_ALIGNMENT_CHECK),
        "misaligned_count": len(misalignments),
        "misalignments": misalignments,
        "alignment_score": 1 - len(misalignments) / len(INCENTIVE_ALIGNMENT_CHECK),
        "healthy": len(misalignments) == 0,
    }
```

## Data Rituals

```python
DATA_RITUALS = {
    "metric_review": {
        "frequency": "weekly",
        "participants": "cross-functional — engineering, product, data, business",
        "agenda": ["What moved?", "Why did it move?", "What are we doing about it?", "What are we NOT doing?"],
        "anti_pattern": "Reviewing metrics that nobody can act on — ritual without agency is theater",
    },
    "decision_retrospective": {
        "frequency": "monthly",
        "participants": "decision-makers + data providers",
        "agenda": ["What decisions did we make?", "What data did we use?", 
                   "Was the data correct in hindsight?", "What would we do differently?"],
        "anti_pattern": "Only reviewing decisions that turned out well",
    },
    "data_quality_standup": {
        "frequency": "daily for critical pipelines, weekly otherwise",
        "participants": "data engineers + data consumers",
        "agenda": ["Any data quality incidents in last 24h?", "Any near misses?", 
                   "What's the freshest data gap?"],
        "anti_pattern": "Standup becomes status report — no action items generated",
    },
    "assumption_busting": {
        "frequency": "quarterly",
        "participants": "leadership + data leaders",
        "agenda": ["What assumptions are we making?", "What data would falsify them?",
                   "Let's look at that data now."],
        "anti_pattern": "Assumptions listed but never tested — becomes another checkbox",
    },
    "failure_celebration": {
        "frequency": "monthly",
        "participants": "anyone who ran a rigorous experiment with negative result",
        "agenda": ["What did you test?", "What did you expect?", "What actually happened?",
                   "What did we learn?", "What should we stop doing based on this?"],
        "anti_pattern": "Only celebrating 'interesting' failures — boring negative results also valuable",
    },
}

def ritual_health(rituals, organization):
    health = {}
    for ritual_name, ritual in DATA_RITUALS.items():
        implementation = rituals.get(ritual_name, {})
        health[ritual_name] = {
            "implemented": bool(implementation),
            "frequency_match": implementation.get("frequency") == ritual["frequency"],
            "participants_correct": set(implementation.get("participants", [])) == set(ritual.get("participants", [])),
            "anti_pattern_active": implementation.get("anti_pattern_detected", False),
            "healthy": bool(implementation) and not implementation.get("anti_pattern_detected", False),
        }
    return health
```

## Quality Gate

- Culture type diagnosed and intervention path selected.
- Data literacy measured — >60% of organization at L2 or above.
- Psychological safety for data honesty scored — overall > 0.7.
- All five incentive dimensions aligned with data quality behaviors.
- Five data rituals institutionalized with anti-pattern detection.
