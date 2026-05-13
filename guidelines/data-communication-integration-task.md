---
name: data-communication-integration-task
description: Bridge data insights to organizational audiences — audience archetype mapping, narrative construction from data, dashboard design principles, alerting architecture, and reporting cadence engineering. Every insight has an audience; every audience has a data literacy profile.
recommended_skills: [data-card-writer, data-visualization-task, data-pipeline-monitoring-task]
recommended_guidelines: [data-decision-integration-task, business-strategy-task, team-operations-data-task]
---

## Overview

Data that isn't communicated is data that doesn't exist. The gap between insight discovery and organizational action is a communication gap. This guideline maps audience archetypes to communication formats, constructs data narratives from analysis outputs, designs dashboards that drive decisions not just display numbers, architects alerting systems with signal-to-noise discipline, and engineers reporting cadences that match decision rhythms.

## Audience Archetype Mapping

```python
AUDIENCE_ARCHETYPES = {
    "executive": {
        "data_literacy": "strategic — understands trends, KPIs, directional signals",
        "time_budget_seconds": 60,
        "format": "one-page with single headline number, 2-3 supporting bullets, 1 chart",
        "what_they_need": "so-what and now-what",
        "depth_tolerance": "zero — do not show methodology",
        "error_tolerance": "show ranges not point estimates, caveat once at bottom",
        "frequency": "weekly_summary + exception_alert",
        "dashboard_type": "executive_dashboard — 4-6 KPIs max, red/amber/green",
        "narrative_style": "Here is the single most important number. Here is what changed. Here is what we are doing about it.",
    },
    "director": {
        "data_literacy": "operational — understands drivers, segments, variance",
        "time_budget_seconds": 300,
        "format": "2-3 page brief with methodology appendix",
        "what_they_need": "what is driving the number and what levers exist",
        "depth_tolerance": "moderate — methodology summary in appendix",
        "error_tolerance": "confidence intervals on key metrics",
        "frequency": "daily_summary + weekly_deep_dive",
        "dashboard_type": "operational_dashboard — drill-down by dimension, time comparison",
        "narrative_style": "The number moved because X. Here are the top 3 drivers. Here are the 2 levers you can pull.",
    },
    "practitioner": {
        "data_literacy": "technical — understands methods, assumptions, edge cases",
        "time_budget_seconds": 1800,
        "format": "full analysis with code, data, and reproducibility artifacts",
        "what_they_need": "exactly how was this computed and what are the edge cases",
        "depth_tolerance": "maximum — show methodology, assumptions, code, raw data access",
        "error_tolerance": "standard errors, confidence intervals, sensitivity analysis",
        "frequency": "on_demand + scheduled_review",
        "dashboard_type": "technical_dashboard — raw data access, custom queries, export",
        "narrative_style": "Here is the method. Here are the assumptions. Here is the sensitivity. Here are the edge cases.",
    },
    "external_regulator": {
        "data_literacy": "compliance — understands standards, audit trails, certification",
        "time_budget_seconds": "unlimited — will read everything",
        "format": "formal report with complete audit trail, attestations, methodology documentation",
        "what_they_need": "prove compliance with standard X clause Y",
        "depth_tolerance": "complete — every step must be documented and reproducible",
        "error_tolerance": "zero — must be exact with documented tolerance bounds",
        "frequency": "quarterly/annual + triggered_by_incident",
        "dashboard_type": "compliance_dashboard — audit trail, lineage, certification status",
        "narrative_style": "Here is the requirement. Here is our compliance. Here is the evidence. Here is the attestation.",
    },
}

def map_insight_to_audience(insight, audience_type):
    template = AUDIENCE_ARCHETYPES.get(audience_type, AUDIENCE_ARCHETYPES["practitioner"])
    return {
        "insight_id": insight["id"],
        "audience": audience_type,
        "format": template["format"],
        "time_budget": template["time_budget_seconds"],
        "depth": template["depth_tolerance"],
        "error_presentation": template["error_tolerance"],
        "narrative_template": template["narrative_style"],
        "dashboard_type": template["dashboard_type"],
        "frequency": template["frequency"],
    }
```

## Narrative Construction

```python
def construct_data_narrative(analysis_result, audience_type, decision_context):
    """Transform analysis output into an audience-matched narrative."""
    template = AUDIENCE_ARCHETYPES.get(audience_type, {})

    narrative = {
        "headline": _headline_from_analysis(analysis_result, audience_type),
        "so_what": _so_what(analysis_result, decision_context),
        "now_what": _now_what(analysis_result, decision_context),
        "supporting_evidence": _select_evidence(analysis_result, audience_type, max_items=3),
        "caveats": _caveats(analysis_result, audience_type),
        "call_to_action": _cta(analysis_result, decision_context),
        "confidence": _format_uncertainty(analysis_result, audience_type),
    }

    # Validate against audience constraints
    if audience_type == "executive":
        assert len(narrative["headline"]) < 140, "Executive headline must fit one screen"
        assert len(narrative["supporting_evidence"]) <= 3, "Max 3 supporting bullets for executives"

    return narrative

def _headline_from_analysis(analysis, audience):
    """The single sentence that captures the insight."""
    headlines = {
        "executive": lambda a: f"{a['metric_name']} is {a['direction']} {a['magnitude']} — {a['implication']}",
        "director": lambda a: f"{a['metric_name']} {a['direction']} by {a['magnitude']}, driven by {a['top_driver']}",
        "practitioner": lambda a: f"{a['metric_name']}: {a['estimate']} ± {a['error']} (n={a['sample_size']}, method={a['method']})",
    }
    fn = headlines.get(audience, headlines["practitioner"])
    return fn(analysis)

def _so_what(analysis, context):
    return f"If this trend continues, {context.get('impact_description', 'business outcome is at risk')}."

def _now_what(analysis, context):
    options = context.get("decision_options", ["investigate further"])
    return f"Recommended action: {options[0]}. Alternatives: {', '.join(options[1:])}."
```

## Dashboard Design

```python
DASHBOARD_PRINCIPLES = {
    "single_decision_per_view": "One dashboard = one decision. Never mix strategic and tactical on one screen.",
    "above_the_fold": "The most important number must be visible without scrolling.",
    "comparison_is_context": "Every number needs a comparison: vs target, vs last period, vs benchmark.",
    "color_is_signal": "Red/amber/green only when thresholds are predefined. Never color for decoration.",
    "drill_path_is_explicit": "Every aggregate must have a drill-down path. No dead-end numbers.",
    "freshness_is_visible": "Show when data was last refreshed. Stale data must look stale.",
    "alert_integration": "Dashboard elements that are in alert state must be visibly distinct.",
}

def audit_dashboard(dashboard_config):
    violations = []
    if dashboard_config.get("decisions_supported", 0) > 1:
        violations.append("multiple_decisions — violates single_decision_per_view")
    if not dashboard_config.get("comparison_column"):
        violations.append("no_comparison — violates comparison_is_context")
    if not dashboard_config.get("last_refresh_visible"):
        violations.append("freshness_hidden — violates freshness_is_visible")
    return {"score": 1 - len(violations) / len(DASHBOARD_PRINCIPLES),
            "violations": violations, "compliant": len(violations) == 0}
```

## Alerting Architecture

```python
ALERTING_SIGNAL_DISCIPLINE = {
    "alert_tiers": {
        "page": {"latency": "seconds", "requires_ack": True, "false_positive_budget": "1 per month",
                 "escalation": "unacked_5min → next_level"},
        "ticket": {"latency": "minutes", "requires_ack": True, "false_positive_budget": "5 per week",
                   "escalation": "unacked_1hour → page"},
        "dashboard": {"latency": "hours", "requires_ack": False, "false_positive_budget": "unlimited",
                      "escalation": "none — reviewed in standup"},
    },
}

def design_alert(metric, audience, severity):
    tier = ALERTING_SIGNAL_DISCIPLINE["alert_tiers"].get(severity, "dashboard")
    return {
        "metric": metric["name"],
        "condition": f"{metric['current_value']} {metric['comparator']} {metric['threshold']}",
        "tier": severity,
        "latency_sla": tier["latency"],
        "requires_acknowledgment": tier["requires_ack"],
        "false_positive_budget": tier["false_positive_budget"],
        "message_template": f"[{severity.upper()}] {metric['name']} is {metric['current_value']} "
                           f"(threshold: {metric['threshold']}). Impact: {metric.get('impact', 'unknown')}. "
                           f"Runbook: {metric.get('runbook_link', 'TBD')}",
    }
```

## Reporting Cadence

```python
REPORTING_ARCHETYPES = {
    "real_time_monitoring": {
        "cadence": "continuous",
        "audience": ["practitioner", "on_call"],
        "format": "dashboard_with_alerts",
        "shelf_life": "hours",
        "decision_type": "operational — immediate action",
    },
    "daily_standup": {
        "cadence": "daily",
        "audience": ["director", "practitioner"],
        "format": "automated_slack_summary + dashboard_link",
        "shelf_life": "1_day",
        "decision_type": "tactical — today's priorities",
    },
    "weekly_business_review": {
        "cadence": "weekly",
        "audience": ["executive", "director"],
        "format": "1_page_brief + 30_min_meeting",
        "shelf_life": "1_week",
        "decision_type": "tactical — resource allocation, course correction",
    },
    "monthly_strategy_review": {
        "cadence": "monthly",
        "audience": ["executive"],
        "format": "3_page_deck + 60_min_meeting",
        "shelf_life": "1_month",
        "decision_type": "strategic — goal progress, investment decisions",
    },
    "quarterly_board_update": {
        "cadence": "quarterly",
        "audience": ["executive", "board"],
        "format": "formal_report + presentation",
        "shelf_life": "1_quarter",
        "decision_type": "strategic — governance, capital allocation",
    },
}

def reporting_cadence_fit(decision_rhythm, current_cadence):
    return {
        "decision_rhythm": decision_rhythm,
        "current_cadence": current_cadence,
        "aligned": decision_rhythm == current_cadence,
        "risk_if_misaligned": "decisions_made_on_stale_data" if current_cadence < decision_rhythm
                              else "reporting_overhead_without_decision_value",
    }
```

## Quality Gate

- Every insight has an audience archetype assigned before delivery.
- Executive communications fit in 60 seconds; practitioner communications are fully reproducible.
- Dashboards support exactly one decision; no multi-purpose dashboards.
- Alert false-positive budgets are enforced; alerts exceeding budget are redesigned or demoted.
- Reporting cadence matches decision cadence — no weekly reports for quarterly decisions.
