---
name: data-talent-integration-task
description: Bridge talent systems to data capability needs — map data roles to skill requirements, engineer hiring signals for data literacy, design data career lattices, measure data talent retention, and orchestrate upskilling at scale. Talent is a data dependency.
recommended_skills: [team-operations-data-task, cost-model-task, annotation-management-task]
recommended_guidelines: [data-culture-integration-task, organizational-implementation-data-task, data-learning-organization-task]
---

## Overview

Data capability is a function of talent, not tools. The most sophisticated data infrastructure is worthless without people who can question methodology, interpret distributions, and kill bad ideas with evidence. This guideline maps data roles to required skills, engineers hiring signals that predict data literacy, designs career lattices that retain data talent, measures retention risk before it becomes attrition, and orchestrates upskilling programs that close literacy gaps at scale.

## Data Role → Capability Mapping

```python
DATA_ROLES = {
    "data_curator": {
        "description": "Ensures datasets are clean, versioned, documented, and certification-ready",
        "required_skills": ["dataset_cleaning", "dataset_splitting", "dataset_versioning", 
                           "data_card_writing", "contamination_detection", "bias_audit"],
        "data_literacy_minimum": "L3_data_fluent",
        "certification_path": "Data Lattice Bronze → Silver → Gold → Platinum curator",
        "career_adjacencies": ["data_engineer", "ml_engineer", "data_product_manager"],
        "retention_risk_factors": ["underappreciated_role", "no_certification_path",
                                    "treated_as_janitorial", "no_visibility_to_impact"],
    },
    "data_engineer": {
        "description": "Builds and maintains data pipelines, storage, and serving infrastructure",
        "required_skills": ["pipeline_orchestration", "schema_design", "data_modeling",
                           "streaming_architectures", "cost_optimization", "monitoring"],
        "data_literacy_minimum": "L2_data_literate",
        "certification_path": "Pipeline Bronze → Silver → Gold certification",
        "career_adjacencies": ["data_curator", "ml_engineer", "infrastructure_engineer"],
        "retention_risk_factors": ["on_call_burnout", "legacy_system_maintenance_only",
                                    "no_greenfield_work", "pipeline_firefighting_constant"],
    },
    "data_analyst": {
        "description": "Translates data into business insights through analysis and reporting",
        "required_skills": ["statistical_reasoning", "data_visualization", "sql_mastery",
                           "business_acumen", "narrative_construction", "experiment_design"],
        "data_literacy_minimum": "L3_data_fluent",
        "certification_path": "Analyst → Senior → Staff → Principal",
        "career_adjacencies": ["data_scientist", "product_manager", "business_operations"],
        "retention_risk_factors": ["report_factory_work", "no_decision_influence",
                                    "analysis_ignored", "dashboard_monkey_role"],
    },
    "data_product_manager": {
        "description": "Owns data products — datasets, features, models — as products with users, SLAs, and roadmaps",
        "required_skills": ["product_strategy", "stakeholder_management", "data_literacy_L2",
                           "cost_modeling", "roadmap_planning", "metric_design"],
        "data_literacy_minimum": "L2_data_literate",
        "certification_path": "PM → Senior PM → Director of Data Product",
        "career_adjacencies": ["product_manager", "data_analyst", "engineering_manager"],
        "retention_risk_factors": ["data_product_not_resourced_as_product",
                                    "no_dedicated_engineering", "treated_as_project_manager"],
    },
    "ml_researcher": {
        "description": "Advances model capabilities through novel architectures, training methods, and data strategies",
        "required_skills": ["deep_learning", "experiment_design", "data_curation_awareness",
                           "research_communication", "reproducibility", "failure_analysis"],
        "data_literacy_minimum": "L4_data_leader",
        "certification_path": "Researcher → Senior → Staff → Principal → Fellow",
        "career_adjacencies": ["ml_engineer", "research_scientist", "professor"],
        "retention_risk_factors": ["compute_constrained", "data_access_limited",
                                    "publication_pressure_over_impact", "no_long_term_research"],
    },
}

def map_talent_to_need(organizational_need, available_talent):
    gaps = []
    for role_id, need in organizational_need.items():
        role_profile = DATA_ROLES.get(role_id, {})
        required = set(role_profile.get("required_skills", []))
        available = set(available_talent.get(role_id, {}).get("skills_present", []))
        missing = required - available
        if missing:
            gaps.append({"role": role_id, "missing_skills": list(missing),
                        "severity": "CRITICAL" if len(missing) > len(required) * 0.5
                                    else "MODERATE" if missing else "COVERED"})
    return {"gaps": gaps, "total_gaps": len(gaps),
            "fully_staffed": len([g for g in gaps if g["severity"] == "COVERED"])}
```

## Hiring Signal Engineering

```python
HIRING_SIGNALS = {
    "data_literacy": {
        "interview_prompt": "Here is a chart showing metric X over time with an intervention at week 12. What questions do you have?",
        "signal": "Asks about baseline, seasonality, sample size, confounders — not just 'what happened next'",
        "anti_signal": "Takes chart at face value; asks no questions; jumps to causal conclusion",
    },
    "curation_mindset": {
        "interview_prompt": "You receive a dataset from a partner team. Walk us through what you check before using it.",
        "signal": "Checks provenance, missingness, distributions, outliers, duplicates, documentation, version",
        "anti_signal": "Loads and starts modeling immediately; 'the data looks fine'",
    },
    "uncertainty_comfort": {
        "interview_prompt": "Your analysis shows a 5% lift with p=0.06. Your VP wants to launch. What do you say?",
        "signal": "Explains uncertainty, confidence intervals, false positive risk, recommends additional data or A/B test",
        "anti_signal": "Says 'it's not statistically significant so we shouldn't launch' with no nuance; or says '5% is great let's ship'",
    },
    "methodology_defense": {
        "interview_prompt": "A stakeholder says 'why did you use this method instead of that one?' — walk us through your reasoning.",
        "signal": "Articulates trade-offs, assumptions, alternatives considered, why chosen method fits the problem",
        "anti_signal": "Defensive or dismissive; 'this is just how you do it'; cannot name alternatives",
    },
}

def score_candidate(candidate, interview_responses):
    signals_detected = 0
    for signal_name, signal in HIRING_SIGNALS.items():
        response = interview_responses.get(signal_name, "")
        if signal["signal"] in response.lower():
            signals_detected += 1
        if signal["anti_signal"] in response.lower():
            signals_detected -= 0.5
    
    return {
        "candidate": candidate["id"],
        "signals_detected": signals_detected,
        "max_possible": len(HIRING_SIGNALS),
        "data_readiness_score": signals_detected / len(HIRING_SIGNALS),
        "verdict": "STRONG_HIRE" if signals_detected >= 3.5
                   else "HIRE" if signals_detected >= 2.5
                   else "BORDERLINE" if signals_detected >= 1.5
                   else "NO_HIRE",
    }
```

## Career Lattice Design

```python
CAREER_LATTICE = {
    "vertical_growth": "Deepen expertise in current role — Staff Curator, Principal Data Engineer",
    "horizontal_growth": "Move to adjacent role — Curator → Data Engineer, Analyst → Data Product Manager",
    "diagonal_growth": "Move to different domain with same skills — Healthcare Data Curator → Finance Data Curator",
    "impact_growth": "Same role, larger scope — from team-level to department-level to organization-level",
}

def career_path_viability(employee, organization_roles):
    current_role = DATA_ROLES.get(employee["role"], {})
    options = []
    
    for adj in current_role.get("career_adjacencies", []):
        if adj in organization_roles and organization_roles[adj].get("open", False):
            skill_gap = set(DATA_ROLES[adj]["required_skills"]) - set(employee["skills"])
            options.append({
                "target_role": adj,
                "type": "horizontal_growth",
                "skill_gap": list(skill_gap),
                "viability": "READY" if len(skill_gap) <= 1 else "UP_SKILL_NEEDED",
            })
    
    return {
        "employee": employee["id"],
        "current_role": employee["role"],
        "career_options": options,
        "viable_paths": len([o for o in options if o["viability"] == "READY"]),
    }
```

## Retention Risk Detection

```python
RETENTION_RISK_MODEL = {
    "leading_indicators": [
        "declining_code_commit_frequency",        # disengaging
        "increased_sick_days",                      # burnout
        "decreased_meeting_participation",          # checking out
        "stopped_mentoring_juniors",                # stopped investing
        "no_longer_proposing_improvements",         # gave up
        "linkedin_activity_increased",              # looking
        "external_conference_speaking_increased",   # marketing themselves
    ],
    "lagging_indicators": [
        "comp_below_market_20pct",
        "no_promotion_in_24_months",
        "no_new_skills_in_12_months",
        "manager_relationship_strained",
        "team_culture_declining",
    ],
}

def retention_risk_score(employee, time_series_signals):
    risk = 0
    risk_factors = []
    
    for indicator in RETENTION_RISK_MODEL["leading_indicators"]:
        if time_series_signals.get(indicator, False):
            risk += 1
            risk_factors.append(indicator)
    
    for indicator in RETENTION_RISK_MODEL["lagging_indicators"]:
        if employee.get(indicator, False):
            risk += 0.5
            risk_factors.append(indicator)
    
    max_risk = len(RETENTION_RISK_MODEL["leading_indicators"]) + \
               len(RETENTION_RISK_MODEL["lagging_indicators"]) * 0.5
    
    normalized = risk / max_risk if max_risk > 0 else 0
    
    return {
        "employee": employee["id"],
        "risk_score": normalized,
        "risk_level": "CRITICAL" if normalized > 0.6 
                      else "HIGH" if normalized > 0.4
                      else "MODERATE" if normalized > 0.2
                      else "LOW",
        "risk_factors": risk_factors,
        "intervention": "IMMEDIATE_RETENTION_CONVERSATION" if normalized > 0.6
                        else "CAREER_PATH_AND_GROWTH_PLAN" if normalized > 0.4
                        else "MONITOR" if normalized > 0.2
                        else "NURTURE",
    }
```

## Upskilling Orchestration

```python
UPSKILLING_PATHS = {
    "L0_to_L1": {"program": "Data Foundations", "duration_weeks": 4,
                 "modules": ["reading_charts", "averages_vs_distributions", "correlation_is_not_causation"],
                 "format": "self_paced + weekly_cohort_discussion",
                 "success_metric": "can_critique_a_chart_with_3_questions"},
    "L1_to_L2": {"program": "Data Literacy", "duration_weeks": 8,
                 "modules": ["distributions_and_uncertainty", "sampling_and_bias", "experiment_design",
                            "methodology_critique", "data_narrative_construction"],
                 "format": "cohort_based + project + peer_review",
                 "success_metric": "can_design_a_basic_experiment_and_interpret_results"},
    "L2_to_L3": {"program": "Data Fluency", "duration_weeks": 12,
                 "modules": ["advanced_statistics", "data_curation_practice", "evaluation_design",
                            "bias_detection", "data_product_thinking"],
                 "format": "apprenticeship_with_senior_analyst + capstone_project",
                 "success_metric": "independently_delivers_end_to_end_analysis_with_confidence_intervals"},
}

def upskilling_plan(organization_literacy_assessment, target_distribution):
    cohorts = []
    current = organization_literacy_assessment["distribution"]
    target = target_distribution
    
    for level in ["L0", "L1", "L2"]:
        current_pct = current.get(level, 0)
        target_pct = target.get(level, 0)
        if current_pct > target_pct:
            gap = int((current_pct - target_pct) * organization_literacy_assessment.get("total_headcount", 0))
            next_level = f"L{int(level[1]) + 1}"
            path = UPSKILLING_PATHS.get(f"{level}_to_{next_level}")
            if path:
                cohorts.append({"from_level": level, "to_level": next_level,
                               "count": gap, "program": path["program"],
                               "duration_weeks": path["duration_weeks"],
                               "start_by": "immediate" if level == "L0" else "next_quarter"})
    
    return {"cohorts": cohorts, "total_upskilling": sum(c["count"] for c in cohorts),
            "estimated_budget": sum(c["count"] * 2000 for c in cohorts)}
```

## Quality Gate

- Every data role has explicit required skills mapped to organizational need.
- Hiring process includes at least 3 of 4 data literacy signals.
- Every employee has at least one visible career path (vertical, horizontal, diagonal, or impact).
- Retention risk monitored monthly with leading indicators — intervention before resignation.
- Organization-wide data literacy at target distribution with funded upskilling cohorts.
