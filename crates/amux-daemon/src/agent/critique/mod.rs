pub(crate) mod advocate;
pub(crate) mod arbiter;
pub(crate) mod critic;
pub(crate) mod types;

#[cfg(test)]
mod tests {
    pub(crate) mod arbiter_calibration;
}

use anyhow::Result;
use serde_json::json;
use uuid::Uuid;

use crate::agent::engine::AgentEngine;
use crate::agent::operator_model::{preferred_tool_fallback_targets, RiskTolerance};

use self::types::{ArgumentPoint, CritiqueSession, Decision, Resolution, SessionStatus};

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub(super) fn sanitize_critique_snippet(value: &str, max_chars: usize) -> String {
    let scrubbed = crate::scrub::scrub_sensitive(value);
    crate::agent::summarize_text(&scrubbed, max_chars)
}

fn sanitize_critique_evidence(value: &str) -> String {
    sanitize_critique_snippet(value, 220)
}

fn sanitize_argument_point(point: &ArgumentPoint) -> ArgumentPoint {
    ArgumentPoint {
        claim: sanitize_critique_snippet(&point.claim, 220),
        weight: point.weight,
        evidence: point
            .evidence
            .iter()
            .map(|value| sanitize_critique_evidence(value))
            .collect(),
    }
}

fn sanitize_resolution(resolution: &Resolution) -> Resolution {
    Resolution {
        decision: resolution.decision,
        synthesis: sanitize_critique_snippet(&resolution.synthesis, 240),
        risk_score: resolution.risk_score,
        confidence: resolution.confidence,
        modifications: resolution
            .modifications
            .iter()
            .map(|value| sanitize_critique_snippet(value, 220))
            .collect(),
        directives: resolution.directives.clone(),
    }
}

fn sanitize_critique_session(mut session: CritiqueSession) -> CritiqueSession {
    session.proposed_action_summary =
        sanitize_critique_snippet(&session.proposed_action_summary, 240);
    session.advocate_argument.points = session
        .advocate_argument
        .points
        .iter()
        .map(sanitize_argument_point)
        .collect();
    session.critic_argument.points = session
        .critic_argument
        .points
        .iter()
        .map(sanitize_argument_point)
        .collect();
    session.resolution = session.resolution.as_ref().map(sanitize_resolution);
    session
}

fn summarize_causal_factor_descriptions(
    factors: &[crate::agent::learning::traces::CausalFactor],
) -> String {
    factors
        .iter()
        .take(2)
        .map(|factor| sanitize_critique_snippet(&factor.description, 100))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn dedupe_argument_points(mut points: Vec<ArgumentPoint>) -> Vec<ArgumentPoint> {
    let mut seen = std::collections::BTreeSet::new();
    points.retain(|point| seen.insert(point.claim.clone()));
    points
}

pub(crate) fn operator_report_summary(session: &CritiqueSession) -> String {
    let decision = session
        .resolution
        .as_ref()
        .map(|resolution| resolution.decision.as_str())
        .unwrap_or(match session.status {
            SessionStatus::InProgress => "in_progress",
            SessionStatus::Resolved => "resolved",
            SessionStatus::Deferred => "deferred",
        });
    let rationale = session
        .resolution
        .as_ref()
        .map(|resolution| resolution.synthesis.as_str())
        .unwrap_or(session.proposed_action_summary.as_str());
    sanitize_critique_snippet(
        &format!(
            "Critiqued {} -> {}: {}",
            session.tool_name, decision, rationale
        ),
        220,
    )
}

fn critique_fallback_argument_points(
    tool_name: &str,
    preferred_fallback_targets: &[String],
) -> Vec<ArgumentPoint> {
    if preferred_fallback_targets.is_empty() {
        return Vec::new();
    }

    let mut points = Vec::new();
    for target in preferred_fallback_targets {
        match (tool_name, target.as_str()) {
            (
                "bash_command" | "run_terminal_command" | "execute_managed_command",
                "apply_patch",
            ) => {
                points.push(ArgumentPoint {
                    claim: "Prefer apply_patch over brittle shell rewrites for this change."
                        .to_string(),
                    weight: 0.78,
                    evidence: vec![
                        "tool_specific:apply_patch:fallback_preference".to_string(),
                        "fallback_match:apply_patch".to_string(),
                    ],
                });
            }
            (
                "bash_command" | "run_terminal_command" | "execute_managed_command",
                "replace_in_file",
            ) => {
                points.push(ArgumentPoint {
                    claim: "Prefer replace_in_file over ad-hoc shell rewrites when a narrow textual edit is enough.".to_string(),
                    weight: 0.74,
                    evidence: vec![
                        "tool_specific:replace_in_file:fallback_preference".to_string(),
                        "fallback_match:replace_in_file".to_string(),
                    ],
                });
            }
            _ => {}
        }
    }

    dedupe_argument_points(points)
}

impl AgentEngine {
    async fn critique_grounded_argument_points(
        &self,
        tool_name: &str,
    ) -> (Vec<ArgumentPoint>, Vec<ArgumentPoint>) {
        let records = match self
            .history
            .list_recent_causal_trace_records(tool_name, 6)
            .await
        {
            Ok(records) => records,
            Err(error) => {
                tracing::warn!(tool = %tool_name, "failed to load recent causal traces for critique grounding: {error}");
                return (Vec::new(), Vec::new());
            }
        };

        let mut advocate_points = Vec::new();
        let mut critic_points = Vec::new();

        for record in records {
            let factors =
                serde_json::from_str::<Vec<crate::agent::learning::traces::CausalFactor>>(
                    &record.causal_factors_json,
                )
                .unwrap_or_default();
            let factor_summary = summarize_causal_factor_descriptions(&factors);
            let factor_evidence = factors
                .iter()
                .take(2)
                .map(|factor| {
                    format!(
                        "causal_factor:{}",
                        sanitize_critique_snippet(&factor.description, 120)
                    )
                })
                .collect::<Vec<_>>();

            let outcome = match serde_json::from_str::<
                crate::agent::learning::traces::CausalTraceOutcome,
            >(&record.outcome_json)
            {
                Ok(outcome) => outcome,
                Err(_) => continue,
            };

            match outcome {
                crate::agent::learning::traces::CausalTraceOutcome::Success => {
                    if advocate_points.is_empty() {
                        let summary = if factor_summary.is_empty() {
                            format!("recent `{tool_name}` executions succeeded")
                        } else {
                            factor_summary.clone()
                        };
                        let mut evidence = vec![format!(
                            "causal_trace:success:{}",
                            sanitize_critique_snippet(&summary, 140)
                        )];
                        evidence.extend(factor_evidence.clone());
                        advocate_points.push(ArgumentPoint {
                            claim: format!(
                                "Recent causal history supports `{tool_name}`: {summary}."
                            ),
                            weight: 0.58,
                            evidence,
                        });
                    }
                }
                crate::agent::learning::traces::CausalTraceOutcome::Failure { reason } => {
                    if critic_points.is_empty() {
                        let safe_reason = sanitize_critique_snippet(&reason, 140);
                        let mut evidence = vec![format!("causal_trace:failure:{safe_reason}")];
                        evidence.extend(factor_evidence.clone());
                        let claim = if factor_summary.is_empty() {
                            format!(
                                "Recent causal history warns against `{tool_name}` without extra caution: {safe_reason}."
                            )
                        } else {
                            format!(
                                "Recent causal history warns against `{tool_name}`: {safe_reason}. Context: {factor_summary}."
                            )
                        };
                        critic_points.push(ArgumentPoint {
                            claim,
                            weight: 0.79,
                            evidence,
                        });
                    }
                }
                crate::agent::learning::traces::CausalTraceOutcome::NearMiss {
                    what_went_wrong,
                    how_recovered,
                } => {
                    if critic_points.is_empty() {
                        let safe_what_went_wrong = sanitize_critique_snippet(&what_went_wrong, 140);
                        let safe_recovery = sanitize_critique_snippet(&how_recovered, 140);
                        let mut evidence =
                            vec![format!("causal_trace:near_miss:{safe_what_went_wrong}")];
                        evidence.extend(factor_evidence.clone());
                        evidence.push(format!("causal_trace:recovery:{safe_recovery}"));
                        let claim = if factor_summary.is_empty() {
                            format!(
                                "Recent causal history shows a near miss for `{tool_name}`: {safe_what_went_wrong}."
                            )
                        } else {
                            format!(
                                "Recent causal history shows a near miss for `{tool_name}`: {safe_what_went_wrong}. Context: {factor_summary}."
                            )
                        };
                        critic_points.push(ArgumentPoint {
                            claim,
                            weight: 0.71,
                            evidence,
                        });
                    }
                    if advocate_points.is_empty() {
                        advocate_points.push(ArgumentPoint {
                            claim: format!(
                                "Recent causal history also shows `{tool_name}` can recover from trouble: {}.",
                                sanitize_critique_snippet(&how_recovered, 140)
                            ),
                            weight: 0.46,
                            evidence: vec![format!(
                                "causal_trace:recovery:{}",
                                sanitize_critique_snippet(&how_recovered, 140)
                            )],
                        });
                    }
                }
                crate::agent::learning::traces::CausalTraceOutcome::Unresolved => {}
            }

            if !advocate_points.is_empty() && !critic_points.is_empty() {
                break;
            }
        }

        (advocate_points, critic_points)
    }

    async fn critique_learned_argument_points(
        &self,
        tool_name: &str,
    ) -> (
        Vec<ArgumentPoint>,
        Vec<ArgumentPoint>,
        Vec<String>,
        Vec<self::types::CritiqueDirective>,
    ) {
        let rows = match self
            .history
            .list_recent_critique_sessions_for_tool(tool_name, 6)
            .await
        {
            Ok(rows) => rows,
            Err(error) => {
                tracing::warn!(tool = %tool_name, "failed to load recent critique history for learning: {error}");
                return (Vec::new(), Vec::new(), Vec::new(), Vec::new());
            }
        };

        let mut advocate_points = Vec::new();
        let mut critic_points = Vec::new();
        let mut learned_modifications = Vec::new();
        let mut learned_directives = Vec::new();

        for row in rows {
            let Ok(session) = serde_json::from_str::<CritiqueSession>(&row.session_json) else {
                continue;
            };
            let Some(resolution) = session.resolution else {
                continue;
            };

            match resolution.decision {
                Decision::ProceedWithModifications => {
                    for modification in resolution.modifications {
                        let safe_modification = sanitize_critique_snippet(&modification, 180);
                        let normalized = safe_modification.trim().to_ascii_lowercase();
                        if !normalized.is_empty()
                            && !learned_modifications.iter().any(|existing: &String| {
                                existing.eq_ignore_ascii_case(&safe_modification)
                            })
                        {
                            learned_modifications.push(safe_modification.clone());
                            critic_points.push(ArgumentPoint {
                                claim: format!(
                                    "Learned from previous critique sessions for `{tool_name}`: {safe_modification}"
                                ),
                                weight: 0.61,
                                evidence: vec![format!(
                                    "critique_history:modification:{normalized}"
                                )],
                            });
                        }
                    }
                    for directive in resolution.directives {
                        if !learned_directives.contains(&directive) {
                            learned_directives.push(directive);
                        }
                    }
                }
                Decision::Proceed => {
                    if advocate_points.is_empty() {
                        advocate_points.push(ArgumentPoint {
                            claim: format!(
                                "Previous critique sessions for `{tool_name}` resolved cleanly without extra blocking."
                            ),
                            weight: 0.41,
                            evidence: vec!["critique_history:proceed".to_string()],
                        });
                    }
                }
                Decision::Defer | Decision::Reject => {
                    if critic_points.is_empty() {
                        critic_points.push(ArgumentPoint {
                            claim: format!(
                                "Previous critique sessions for `{tool_name}` escalated to defer/reject outcomes, so extra caution is warranted."
                            ),
                            weight: 0.66,
                            evidence: vec![format!(
                                "critique_history:{}",
                                resolution.decision.as_str()
                            )],
                        });
                    }
                }
            }
        }

        (
            dedupe_argument_points(advocate_points),
            dedupe_argument_points(critic_points),
            learned_modifications,
            learned_directives,
        )
    }

    pub(crate) async fn run_critique_preflight(
        &self,
        action_id: &str,
        tool_name: &str,
        action_summary: &str,
        reasons: &[String],
        thread_id: Option<&str>,
        task_id: Option<&str>,
    ) -> Result<CritiqueSession> {
        let risk_tolerance = self
            .operator_model
            .read()
            .await
            .risk_fingerprint
            .risk_tolerance;
        let satisfaction_label = self
            .operator_model
            .read()
            .await
            .operator_satisfaction
            .label
            .clone();
        let preferred_fallback_targets = self
            .operator_model
            .read()
            .await
            .implicit_feedback
            .top_tool_fallbacks
            .clone();
        let preferred_fallback_targets =
            preferred_tool_fallback_targets(&preferred_fallback_targets, 3);
        let fallback_points =
            critique_fallback_argument_points(tool_name, &preferred_fallback_targets);
        let (grounded_advocate_points, grounded_critic_points) =
            self.critique_grounded_argument_points(tool_name).await;
        let (
            learned_advocate_points,
            learned_critic_points,
            learned_modifications,
            learned_directives,
        ) = self.critique_learned_argument_points(tool_name).await;
        let advocate_argument = advocate::build_argument(
            tool_name,
            action_summary,
            reasons,
            [grounded_advocate_points, learned_advocate_points].concat(),
        );
        let critic_argument = critic::build_argument(
            tool_name,
            action_summary,
            reasons,
            [
                grounded_critic_points,
                learned_critic_points,
                fallback_points,
            ]
            .concat(),
        );
        let mut resolution = arbiter::resolve_with_satisfaction_label(
            &advocate_argument,
            &critic_argument,
            risk_tolerance,
            Some(&satisfaction_label),
        );
        if matches!(resolution.decision, Decision::ProceedWithModifications) {
            for modification in &learned_modifications {
                if !resolution
                    .modifications
                    .iter()
                    .any(|existing| existing.eq_ignore_ascii_case(modification))
                {
                    resolution.modifications.push(modification.clone());
                }
            }
            for directive in &learned_directives {
                if !resolution.directives.contains(directive) {
                    resolution.directives.push(*directive);
                }
            }
        }
        if let Some(forced_decision) = self
            .config
            .read()
            .await
            .extra
            .get("test_force_critique_decision")
            .and_then(|value| value.as_str())
        {
            resolution.decision = match forced_decision {
                "proceed" => Decision::Proceed,
                "proceed_with_modifications" => Decision::ProceedWithModifications,
                "defer" => Decision::Defer,
                "reject" => Decision::Reject,
                _ => resolution.decision,
            };
            if matches!(resolution.decision, Decision::ProceedWithModifications)
                && resolution.modifications.is_empty()
            {
                let mut critic_guidance = arbiter::recommended_modifications_with_fallback_targets(
                    &critic_argument,
                    &preferred_fallback_targets,
                    2,
                );
                if critic_guidance.is_empty() {
                    critic_guidance
                        .push("Apply the critic's safer constraints before execution.".to_string());
                }
                resolution.modifications = critic_guidance;
                resolution.directives =
                    arbiter::directives_for_modifications(&resolution.modifications);
                resolution.synthesis = format!(
                    "Proceed with modifications. Keep the action, but incorporate: {}.",
                    resolution.modifications.join(" | ")
                );
            }
        }
        if let Some(forced_modifications) = self
            .config
            .read()
            .await
            .extra
            .get("test_force_critique_modifications")
            .and_then(|value| value.as_array())
        {
            let forced_modifications = forced_modifications
                .iter()
                .filter_map(|value| value.as_str())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();
            if !forced_modifications.is_empty() {
                resolution.modifications = forced_modifications;
                resolution.directives =
                    arbiter::directives_for_modifications(&resolution.modifications);
                if matches!(resolution.decision, Decision::ProceedWithModifications) {
                    resolution.synthesis = format!(
                        "Proceed with modifications. Keep the action, but incorporate: {}.",
                        resolution.modifications.join(" | ")
                    );
                }
            }
        }
        if let Some(forced_directives) = self
            .config
            .read()
            .await
            .extra
            .get("test_force_critique_directives")
            .and_then(|value| value.as_array())
        {
            let forced_directives = forced_directives
                .iter()
                .filter_map(|value| value.as_str())
                .filter_map(|value| {
                    serde_json::from_str::<self::types::CritiqueDirective>(&format!("\"{value}\""))
                        .ok()
                })
                .collect::<Vec<_>>();
            if !forced_directives.is_empty() {
                resolution.directives = forced_directives;
            }
        }

        if matches!(resolution.decision, Decision::ProceedWithModifications) {
            for modification in &learned_modifications {
                if !resolution
                    .modifications
                    .iter()
                    .any(|existing| existing.eq_ignore_ascii_case(modification))
                {
                    resolution.modifications.push(modification.clone());
                }
            }
            for directive in &learned_directives {
                if !resolution.directives.contains(directive) {
                    resolution.directives.push(*directive);
                }
            }
        }
        let created_at_ms = now_millis();
        let resolved_at_ms = Some(created_at_ms);
        let status = if matches!(resolution.decision, Decision::Defer) {
            SessionStatus::Deferred
        } else {
            SessionStatus::Resolved
        };
        let session = sanitize_critique_session(CritiqueSession {
            id: format!("critique_{}", Uuid::new_v4()),
            action_id: action_id.to_string(),
            tool_name: tool_name.to_string(),
            proposed_action_summary: action_summary.to_string(),
            thread_id: thread_id.map(str::to_string),
            task_id: task_id.map(str::to_string),
            advocate_id: "advocate".to_string(),
            critic_id: "critic".to_string(),
            arbiter_id: "arbiter".to_string(),
            status,
            advocate_argument,
            critic_argument,
            resolution: Some(resolution),
            created_at_ms,
            resolved_at_ms,
        });
        self.persist_critique_session(&session).await?;
        Ok(session)
    }

    pub(crate) async fn persist_critique_session(&self, session: &CritiqueSession) -> Result<()> {
        let session = sanitize_critique_session(session.clone());
        let session_json = serde_json::to_string(&session)?;
        self.history
            .upsert_critique_session(&session.id, &session_json, session.created_at_ms)
            .await?;

        for point in &session.advocate_argument.points {
            self.history
                .insert_critique_argument(
                    &session.id,
                    "advocate",
                    &point.claim,
                    point.weight,
                    &serde_json::to_string(&point.evidence)?,
                    session.created_at_ms,
                )
                .await?;
        }
        for point in &session.critic_argument.points {
            self.history
                .insert_critique_argument(
                    &session.id,
                    "critic",
                    &point.claim,
                    point.weight,
                    &serde_json::to_string(&point.evidence)?,
                    session.created_at_ms,
                )
                .await?;
        }
        if let Some(resolution) = session.resolution.as_ref() {
            self.history
                .upsert_critique_resolution(
                    &session.id,
                    resolution.decision.as_str(),
                    &serde_json::to_string(resolution)?,
                    resolution.risk_score,
                    resolution.confidence,
                    session.resolved_at_ms.unwrap_or(session.created_at_ms),
                )
                .await?;
        }
        Ok(())
    }

    pub(crate) async fn get_persisted_critique_session(
        &self,
        session_id: &str,
    ) -> Result<Option<CritiqueSession>> {
        let row = self.history.get_critique_session(session_id).await?;
        let Some(row) = row else {
            return Ok(None);
        };
        Ok(Some(serde_json::from_str(&row.session_json)?))
    }

    pub(crate) async fn get_critique_session_payload(
        &self,
        session_id: &str,
    ) -> Result<serde_json::Value> {
        let session = sanitize_critique_session(
            self.get_persisted_critique_session(session_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("unknown critique session: {session_id}"))?,
        );
        Ok(json!({
            "session_id": session.id,
            "status": session.status,
            "action_id": session.action_id,
            "tool_name": session.tool_name,
            "proposed_action_summary": session.proposed_action_summary,
            "report_summary": operator_report_summary(&session),
            "advocate_argument": session.advocate_argument,
            "critic_argument": session.critic_argument,
            "resolution": session.resolution,
            "created_at_ms": session.created_at_ms,
            "resolved_at_ms": session.resolved_at_ms,
        }))
    }

    pub(crate) async fn should_run_critique_preflight(
        &self,
        tool_name: &str,
        classification: &crate::agent::weles_governance::WelesToolClassification,
    ) -> bool {
        let cfg = self.config.read().await;
        if !cfg.critique.enabled {
            return false;
        }
        if matches!(
            cfg.critique.mode,
            crate::agent::types::CritiqueMode::Disabled
        ) {
            return false;
        }
        if cfg.critique.guard_suspicious_tool_calls_only
            && classification.reasons.is_empty()
            && !matches!(
                classification.class,
                crate::agent::weles_governance::WelesGovernanceClass::GuardAlways
            )
        {
            return false;
        }
        let explicitly_supported = matches!(
            tool_name,
            "bash_command"
                | "execute_managed_command"
                | "run_terminal_command"
                | "write_file"
                | "create_file"
                | "append_to_file"
                | "replace_in_file"
                | "apply_file_patch"
                | "apply_patch"
                | "send_slack_message"
                | "send_discord_message"
                | "send_telegram_message"
                | "send_whatsapp_message"
                | "spawn_subagent"
                | "enqueue_task"
        );
        explicitly_supported
            || matches!(
                classification.class,
                crate::agent::weles_governance::WelesGovernanceClass::GuardAlways
                    | crate::agent::weles_governance::WelesGovernanceClass::RejectBypass
            )
            || (!classification.reasons.is_empty()
                && !matches!(
                    classification.class,
                    crate::agent::weles_governance::WelesGovernanceClass::AllowDirect
                ))
    }

    pub(crate) fn critique_requires_blocking_review(
        &self,
        resolution: &Resolution,
        risk_tolerance: RiskTolerance,
    ) -> bool {
        let satisfaction_label = self
            .operator_model
            .try_read()
            .map(|model| model.operator_satisfaction.label.clone())
            .unwrap_or_default();
        match resolution.decision {
            Decision::Reject => true,
            Decision::Defer => true,
            Decision::ProceedWithModifications => {
                if satisfaction_label == "strained" {
                    false
                } else if satisfaction_label == "fragile" {
                    matches!(risk_tolerance, RiskTolerance::Conservative)
                } else {
                    !matches!(risk_tolerance, RiskTolerance::Aggressive)
                }
            }
            Decision::Proceed => false,
        }
    }
}
