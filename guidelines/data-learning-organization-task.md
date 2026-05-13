---
name: data-learning-organization-task
description: Build a learning organization through data — design data capture for single/double/deutero-learning loops, extract structured knowledge from lessons learned, distribute learning via shared data mechanisms, preserve learning history in data archives, and track whether lessons are actually applied.
recommended_skills: [dataset-versioning, data-pipeline-monitoring-task, data-card-writer]
recommended_guidelines: [data-lifecycle-governance-task, team-operations-data-task, organizational-implementation-data-task]
---

## Overview

Organizations that don't learn from data are organizations that don't learn. This guideline treats organizational learning as a data engineering problem: what data does each learning type need, how do you capture lessons as structured data, how do you distribute learning across teams, how do you preserve learning history, and how do you prove that lessons were actually applied? A learning organization is a data organization.

## Learning Type → Data Learning Design

```python
LEARNING_ARCHITECTURE = {
    "single_loop": {
        "description": "Correct errors within existing frameworks — 'are we doing things right?'",
        "data_needed": ["performance_metrics", "error_logs", "deviation_from_target"],
        "feedback_latency": "immediate_to_hours",
        "correction_mechanism": "alert_triggered → root_cause_identified → fix_applied → verified",
        "data_output": "corrective_action_record with before/after metrics",
        "example": "Pipeline failure → root cause data → fix → verify pipeline health restored",
    },
    "double_loop": {
        "description": "Question underlying assumptions and frameworks — 'are we doing the right things?'",
        "data_needed": ["assumption_registry", "outcome_vs_expectation", "counterfactual_evidence",
                        "external_benchmark", "mental_model_documentation"],
        "feedback_latency": "weeks_to_months",
        "correction_mechanism": "assumption_surfaced → challenged_with_data → framework_updated → propagated",
        "data_output": "assumption_revision_log + updated_framework_document",
        "example": "Model accuracy target challenged → data shows accuracy ≠ user value → target changed to user outcome",
    },
    "deutero_learning": {
        "description": "Learn how to learn — improve the learning system itself",
        "data_needed": ["learning_loop_metadata", "correction_effectiveness", "time_to_learn",
                        "learning_transfer_rate", "meta_failure_patterns"],
        "feedback_latency": "months_to_years",
        "correction_mechanism": "learning_system_audited → bottlenecks_identified → learning_process_redesigned",
        "data_output": "learning_system_improvement_log with meta-metrics",
        "example": "Postmortem process audited → 60% of lessons never applied → process redesigned with application tracking",
    },
}

def classify_learning_need(incident_or_opportunity):
    """What type of learning does this situation demand?"""
    if incident_or_opportunity.get("framework_challenge", False):
        return "double_loop"
    if incident_or_opportunity.get("learning_system_concern", False):
        return "deutero_learning"
    return "single_loop"

def design_learning_data_capture(learning_event, learning_type):
    architecture = LEARNING_ARCHITECTURE.get(learning_type, LEARNING_ARCHITECTURE["single_loop"])
    return {
        "event": learning_event["id"],
        "learning_type": learning_type,
        "data_capture_plan": architecture["data_needed"],
        "expected_latency": architecture["feedback_latency"],
        "correction_workflow": architecture["correction_mechanism"],
        "output_artifact": architecture["data_output"],
    }
```

## Learning Capture → Knowledge Extraction

```python
LESSON_SCHEMA = {
    "lesson_id": "string",
    "source": "incident_id | experiment_id | observation_id",
    "learning_type": "single_loop | double_loop | deutero_learning",
    "what_happened": "objective description",
    "what_we_expected": "prior assumption or prediction",
    "what_we_learned": "the actual lesson — falsifiable statement",
    "data_evidence": ["evidence_sources with timestamps and versions"],
    "confidence": "float 0-1 based on evidence strength",
    "applicability_scope": "team | department | organization | industry",
    "recommended_action": "concrete, testable action",
    "action_validation_metric": "how we will know the action was applied",
}

def extract_lessons(events, existing_knowledge_base):
    """Extract structured lessons from events, deduplicate against existing knowledge."""
    lessons = []
    for event in events:
        lesson = {
            "lesson_id": _generate_lesson_id(event),
            "source": event["id"],
            "learning_type": classify_learning_need(event),
            "what_happened": event.get("description", ""),
            "what_we_expected": event.get("expected_outcome", ""),
            "what_we_learned": event.get("learning", ""),
            "data_evidence": event.get("evidence", []),
            "confidence": _assess_lesson_confidence(event),
            "applicability_scope": event.get("scope", "team"),
            "recommended_action": event.get("action", ""),
            "action_validation_metric": event.get("validation_metric", ""),
        }
        
        # Deduplicate against existing knowledge
        if _is_duplicate(lesson, existing_knowledge_base):
            lesson["status"] = "duplicate — merged with existing lesson XYZ"
        else:
            lesson["status"] = "new"
            existing_knowledge_base.append(lesson["lesson_id"])
        
        lessons.append(lesson)
    
    return {
        "lessons_extracted": len(lessons),
        "new_lessons": sum(1 for l in lessons if l["status"] == "new"),
        "duplicates": sum(1 for l in lessons if "duplicate" in l.get("status", "")),
        "by_type": {
            "single_loop": sum(1 for l in lessons if l["learning_type"] == "single_loop"),
            "double_loop": sum(1 for l in lessons if l["learning_type"] == "double_loop"),
            "deutero_learning": sum(1 for l in lessons if l["learning_type"] == "deutero_learning"),
        },
    }
```

## Learning Distribution → Data Distribution Mechanisms

```python
DISTRIBUTION_MECHANISMS = {
    "push": {
        "methods": ["automated_digest", "learning_newsletter", "slack_channel_summary",
                     "executive_brief", "on_call_handoff"],
        "cadence": {"single_loop": "real_time_to_daily", "double_loop": "weekly_to_monthly",
                    "deutero_learning": "quarterly_to_annual"},
        "audience_targeting": "role_based + relevance_scored",
        "effectiveness_metric": "open_rate, read_rate, action_rate",
    },
    "pull": {
        "methods": ["searchable_knowledge_base", "tagged_incident_archive", "failure_registry",
                     "decision_log", "assumption_log"],
        "cadence": "always_available",
        "audience_targeting": "self_service",
        "effectiveness_metric": "search_success_rate, time_to_answer, reuse_count",
    },
    "embedded": {
        "methods": ["pipeline_gate_check", "code_review_checklist", "design_review_template",
                     "onboarding_curriculum", "runbook_integration"],
        "cadence": "triggered_by_workflow",
        "audience_targeting": "contextual — right lesson at right moment",
        "effectiveness_metric": "gate_prevention_rate, checklist_compliance",
    },
}

def distribute_lesson(lesson, organization_graph):
    """Match lesson to distribution mechanisms based on type, scope, and urgency."""
    distribution_plan = []
    
    learning_type = lesson.get("learning_type", "single_loop")
    scope = lesson.get("applicability_scope", "team")
    
    # Push: notify relevant audiences
    if scope in ["organization", "industry"] or learning_type == "deutero_learning":
        distribution_plan.append({"mechanism": "push", "method": "executive_brief",
                                  "audience": "leadership", "cadence": "weekly"})
    
    if scope in ["team", "department", "organization"]:
        distribution_plan.append({"mechanism": "push", "method": "automated_digest",
                                  "audience": scope, "cadence": "daily"})
    
    # Pull: index in knowledge base
    distribution_plan.append({"mechanism": "pull", "method": "searchable_knowledge_base",
                              "tags": [learning_type, scope] + lesson.get("tags", [])})
    
    # Embedded: if action is specific and automatable
    if lesson.get("automatable", False):
        distribution_plan.append({"mechanism": "embedded", "method": "pipeline_gate_check",
                                  "integration_point": lesson.get("gate_location")})
    
    return distribution_plan
```

## Learning Retention → Data Knowledge Preservation

```python
RETENTION_POLICY = {
    "lessons_learned": {"retention_years": "permanent", "format": "versioned_knowledge_base",
                         "indexing": "full_text_search + semantic_embedding + tag_hierarchy",
                         "decay_model": "relevance_decay_not_deletion — flag for review, never delete"},
    "incident_postmortems": {"retention_years": 5, "format": "structured_postmortem_template",
                              "indexing": "by_root_cause + by_affected_component + by_severity",
                              "decay_model": "archive after 5 years, keep aggregated learnings"},
    "decision_logs": {"retention_years": 10, "format": "decision_record_with_context_snapshot",
                       "indexing": "by_decision_type + by_outcome + by_reversibility",
                       "decay_model": "preserve context snapshot — decisions make sense only in context"},
    "assumption_logs": {"retention_years": "permanent", "format": "assumption_with_validation_status",
                         "indexing": "by_domain + by_validation_state + by_impact",
                         "decay_model": "marked as validated/invalidated but never deleted"},
}

def preserve_learning(learning_asset, retention_policy):
    policy = RETENTION_POLICY.get(learning_asset.get("type"), RETENTION_POLICY["lessons_learned"])
    return {
        "asset_id": learning_asset["id"],
        "retention_years": policy["retention_years"],
        "storage_format": policy["format"],
        "indexing_strategy": policy["indexing"],
        "archive_trigger": learning_asset.get("age_years", 0) >= policy.get("retention_years", 5),
        "decay_action": policy["decay_model"],
    }
```

## Learning Application → Data Application Tracking

```python
def track_application(lesson, organization_actions, time_window_days=180):
    """Prove that lessons are actually applied, not just recorded."""
    
    # Find actions that match the lesson's recommended action
    matching_actions = []
    for action in organization_actions:
        if action.get("timestamp", 0) < lesson.get("created_at", 0):
            continue
        if action.get("description", "").find(lesson["recommended_action"][:30]) != -1:
            matching_actions.append(action)
    
    applied = len(matching_actions) > 0
    
    # Measure impact
    if applied:
        before_metric = _get_metric_at_time(lesson["action_validation_metric"], lesson["created_at"])
        after_metric = _get_metric_at_time(lesson["action_validation_metric"], 
                                            matching_actions[-1].get("timestamp"))
        impact = after_metric - before_metric if before_metric is not None else None
    else:
        before_metric = None
        after_metric = None
        impact = None
    
    return {
        "lesson_id": lesson["lesson_id"],
        "recommended_action": lesson["recommended_action"],
        "applied": applied,
        "application_count": len(matching_actions),
        "time_to_apply_days": (matching_actions[0].get("timestamp", 0) - lesson.get("created_at", 0)) 
                              / 86400 if applied else None,
        "before_metric": before_metric,
        "after_metric": after_metric,
        "impact": impact,
        "verdict": "APPLIED_AND_MEASURED" if applied and impact is not None
                   else "APPLIED_NOT_MEASURED" if applied
                   else "NOT_APPLIED",
    }
```

## Learning Organization Health Dashboard

```python
def learning_health_dashboard(knowledge_base, actions_log, time_window_days=365):
    """Is the organization actually learning? Measure it."""
    lessons = [l for l in knowledge_base if l["created_at"] >= time_window_days]
    applications = [track_application(l, actions_log) for l in lessons]
    
    applied_count = sum(1 for a in applications if a["applied"])
    measured_count = sum(1 for a in applications if a["verdict"] == "APPLIED_AND_MEASURED")
    
    return {
        "total_lessons_captured": len(lessons),
        "lessons_by_type": {
            "single_loop": sum(1 for l in lessons if l.get("learning_type") == "single_loop"),
            "double_loop": sum(1 for l in lessons if l.get("learning_type") == "double_loop"),
            "deutero_learning": sum(1 for l in lessons if l.get("learning_type") == "deutero_learning"),
        },
        "application_rate": applied_count / max(len(lessons), 1),
        "measured_impact_rate": measured_count / max(len(lessons), 1),
        "avg_time_to_apply_days": sum(a.get("time_to_apply_days", 0) for a in applications 
                                      if a["applied"] and a.get("time_to_apply_days")) 
                                  / max(applied_count, 1),
        "learning_health": "HEALTHY" if applied_count / max(len(lessons), 1) > 0.7
                           else "ADEQUATE" if applied_count / max(len(lessons), 1) > 0.4
                           else "LEARNING_GAP — lessons captured but not applied",
    }
```

## Quality Gate

- Every incident/experiment/observation that generates a lesson has a learning type classified.
- Lessons are structured per LESSON_SCHEMA and deduplicated against the knowledge base.
- Distribution uses push (targeted), pull (searchable), and embedded (contextual) mechanisms.
- Learning assets have explicit retention policies — assumptions and lessons are permanent.
- Application tracking proves lessons are applied — application rate > 70% is healthy.
- Meta-learning (deutero-learning) audits are conducted annually.
