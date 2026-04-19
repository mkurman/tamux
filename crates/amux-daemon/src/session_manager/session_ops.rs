use super::*;

use crate::governance::{
    apply_constraints_to_request, can_honor_constraints, effective_constraints,
    evaluate_governance, governance_input_for_managed_command, ConstraintKind, GovernanceInput,
    GovernanceVerdict, RiskClass, TransitionKind, VerdictClass,
};
use crate::history::{ApprovalRecordRow, GovernanceEvaluationRow};
use amux_protocol::ApprovalPayload;
use serde_json::json;

fn transition_kind_str(kind: &TransitionKind) -> &'static str {
    match kind {
        TransitionKind::RunAdmission => "run_admission",
        TransitionKind::LaneAdmission => "lane_admission",
        TransitionKind::StageAdvance => "stage_advance",
        TransitionKind::LaneRetry => "lane_retry",
        TransitionKind::ResumeFromBlocked => "resume_from_blocked",
        TransitionKind::CompensationEntry => "compensation_entry",
        TransitionKind::FinalDisposition => "final_disposition",
        TransitionKind::ManagedCommandDispatch => "managed_command_dispatch",
        TransitionKind::ApprovalReuseCheck => "approval_reuse_check",
    }
}

fn risk_class_str(risk_class: &RiskClass) -> &'static str {
    match risk_class {
        RiskClass::Low => "low",
        RiskClass::Medium => "medium",
        RiskClass::High => "high",
        RiskClass::Critical => "critical",
    }
}

fn constraint_kind_str(kind: &ConstraintKind) -> &'static str {
    match kind {
        ConstraintKind::SandboxRequired => "sandbox_required",
        ConstraintKind::NetworkDenied => "network_denied",
        ConstraintKind::NetworkRestricted => "network_restricted",
        ConstraintKind::FilesystemScopeNarrowed => "filesystem_scope_narrowed",
        ConstraintKind::TargetScopeCapped => "target_scope_capped",
        ConstraintKind::SerialOnlyExecution => "serial_only_execution",
        ConstraintKind::RetriesDisabled => "retries_disabled",
        ConstraintKind::RetriesRequireFreshCheckpoint => "retries_require_fresh_checkpoint",
        ConstraintKind::ArtifactRetentionElevated => "artifact_retention_elevated",
        ConstraintKind::ManualResumeRequiredAfterCompletion => {
            "manual_resume_required_after_completion"
        }
    }
}

fn approval_resolution_str(decision: ApprovalDecision) -> &'static str {
    match decision {
        ApprovalDecision::ApproveOnce => "approved_once",
        ApprovalDecision::ApproveSession => "approved_session",
        ApprovalDecision::Deny => "denied",
    }
}

fn blast_radius_summary(input: &GovernanceInput) -> String {
    format!(
        "{} (lane: {}, stage: {})",
        input.blast_radius.run_scope, input.blast_radius.lane_scope, input.blast_radius.stage_scope
    )
}

fn approval_payload_from_verdict(
    execution_id: &str,
    request: &ManagedCommandRequest,
    workspace_id: Option<String>,
    input: &GovernanceInput,
    verdict: &GovernanceVerdict,
    expires_at: Option<u64>,
) -> ApprovalPayload {
    let constraints = effective_constraints(verdict);
    let mut reasons = verdict.rationale.clone();
    reasons.extend(constraints.iter().filter_map(|constraint| {
        constraint.rationale.as_ref().map(|rationale| {
            format!(
                "constraint {}: {}",
                constraint_kind_str(&constraint.kind),
                rationale
            )
        })
    }));

    if reasons.is_empty() {
        reasons.push(format!(
            "governance classified this transition as {} risk",
            risk_class_str(&verdict.risk_class)
        ));
    }

    ApprovalPayload {
        approval_id: format!("apr_{}", Uuid::new_v4()),
        execution_id: execution_id.to_string(),
        command: request.command.clone(),
        rationale: request.rationale.clone(),
        risk_level: risk_class_str(&verdict.risk_class).to_string(),
        blast_radius: blast_radius_summary(input),
        reasons,
        workspace_id,
        allow_network: request.allow_network,
        transition_kind: Some(transition_kind_str(&input.transition_kind).to_string()),
        policy_fingerprint: Some(verdict.policy_fingerprint.clone()),
        expires_at,
        constraints: constraints
            .iter()
            .map(|constraint| constraint_kind_str(&constraint.kind).to_string())
            .collect(),
        scope_summary: verdict
            .approval_requirement
            .as_ref()
            .map(|requirement| requirement.scope_summary.clone())
            .or_else(|| Some(blast_radius_summary(input))),
    }
}

fn governance_rejection_message(verdict: &GovernanceVerdict) -> String {
    let verdict_label = match verdict.verdict_class {
        VerdictClass::Allow => "allow",
        VerdictClass::AllowWithConstraints => "allow_with_constraints",
        VerdictClass::RequireApproval => "require_approval",
        VerdictClass::Defer => "defer",
        VerdictClass::Deny => "deny",
        VerdictClass::HaltAndIsolate => "halt_and_isolate",
        VerdictClass::AllowOnlyWithCompensationPlan => "compensation_plan_required",
    };

    let reason = if verdict.rationale.is_empty() {
        format!("governance returned {verdict_label} for this transition")
    } else {
        verdict.rationale.join("; ")
    };

    format!(
        "managed command blocked by governance ({verdict_label}, {} risk): {reason}",
        risk_class_str(&verdict.risk_class),
    )
}

async fn queue_with_snapshot(
    snapshots: &crate::snapshot::SnapshotStore,
    session: &Arc<Mutex<PtySession>>,
    workspace_id: Option<String>,
    session_id: SessionId,
    execution_id: String,
    request: ManagedCommandRequest,
    snapshot_reason: &str,
) -> Result<(usize, Option<amux_protocol::SnapshotInfo>)> {
    let snapshot_cwd = if let Some(cwd) = request.cwd.as_deref() {
        Some(cwd.to_owned())
    } else {
        session.lock().await.cwd().map(ToOwned::to_owned)
    };
    let snapshot = snapshots
        .create_snapshot(
            workspace_id,
            Some(session_id),
            snapshot_cwd.as_deref(),
            Some(&request.command),
            snapshot_reason,
        )
        .await?;
    let position =
        session
            .lock()
            .await
            .queue_managed_command(execution_id, request, snapshot.clone())?;
    Ok((position, snapshot))
}

impl SessionManager {
    async fn prune_expired_session_grants(&self, session_id: SessionId) {
        let now = crate::history::now_ts();
        let mut grants = self.session_approval_grants.write().await;
        if let Some(entries) = grants.get_mut(&session_id) {
            entries.retain(|grant| match grant.expires_at {
                Some(expires_at) => expires_at > now,
                None => true,
            });
            if entries.is_empty() {
                grants.remove(&session_id);
            }
        }
    }

    async fn has_valid_session_grant(
        &self,
        session_id: SessionId,
        policy_fingerprint: &str,
    ) -> bool {
        self.prune_expired_session_grants(session_id).await;
        self.session_approval_grants
            .read()
            .await
            .get(&session_id)
            .map(|grants| {
                grants
                    .iter()
                    .any(|grant| grant.policy_fingerprint == policy_fingerprint)
            })
            .unwrap_or(false)
    }

    async fn store_session_grant(
        &self,
        session_id: SessionId,
        _approval_id: &str,
        policy_fingerprint: &str,
        expires_at: Option<u64>,
    ) {
        self.prune_expired_session_grants(session_id).await;
        let mut grants = self.session_approval_grants.write().await;
        let entries = grants.entry(session_id).or_default();
        entries.retain(|grant| grant.policy_fingerprint != policy_fingerprint);
        entries.push(SessionApprovalGrant {
            approval_id: _approval_id.to_string(),
            policy_fingerprint: policy_fingerprint.to_string(),
            expires_at,
        });
    }

    pub async fn spawn(
        &self,
        shell: Option<String>,
        cwd: Option<String>,
        workspace_id: Option<String>,
        env: Option<Vec<(String, String)>>,
        cols: u16,
        rows: u16,
    ) -> Result<(SessionId, broadcast::Receiver<DaemonMessage>)> {
        let id = Uuid::new_v4();
        let session = PtySession::spawn(
            id,
            shell,
            cwd,
            workspace_id,
            env,
            cols,
            rows,
            (*self.history).clone(),
            self.pty_channel_capacity,
        )?;
        let rx = session.subscribe();

        self.sessions
            .write()
            .await
            .insert(id, Arc::new(Mutex::new(session)));

        self.persist_state().await;

        tracing::info!(%id, "session spawned");
        Ok((id, rx))
    }

    pub async fn clone_session(
        &self,
        source_id: SessionId,
        workspace_id: Option<String>,
        cols: Option<u16>,
        rows: Option<u16>,
        replay_scrollback: bool,
        cwd_override: Option<String>,
    ) -> Result<(
        SessionId,
        broadcast::Receiver<DaemonMessage>,
        Option<String>,
    )> {
        let source = self
            .sessions
            .read()
            .await
            .get(&source_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("session not found: {source_id}"))?;

        let (
            shell,
            cwd,
            source_workspace_id,
            source_cols,
            source_rows,
            replay_bytes,
            src_active_cmd,
        ) = {
            let source = source.lock().await;
            (
                source.shell().map(ToOwned::to_owned),
                source.resolved_cwd().or(cwd_override),
                source.workspace_id().map(ToOwned::to_owned),
                source.cols(),
                source.rows(),
                if replay_scrollback {
                    source.scrollback(None)
                } else {
                    Vec::new()
                },
                source.active_command(),
            )
        };

        let target_workspace_id = workspace_id.or(source_workspace_id);
        let (id, rx) = self
            .spawn(
                shell,
                cwd,
                target_workspace_id,
                None,
                cols.unwrap_or(source_cols),
                rows.unwrap_or(source_rows),
            )
            .await?;

        if replay_scrollback && !replay_bytes.is_empty() {
            let sanitized = crate::pty_session::sanitize_scrollback_for_replay(&replay_bytes);
            if let Some(cloned_session) = self.sessions.read().await.get(&id).cloned() {
                cloned_session.lock().await.preload_output(&sanitized);
            }
        }

        tracing::info!(%source_id, %id, active_command = ?src_active_cmd, "session cloned");
        Ok((id, rx, src_active_cmd))
    }

    pub async fn write_input(&self, id: SessionId, data: &[u8]) -> Result<()> {
        let session = self
            .sessions
            .read()
            .await
            .get(&id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("session not found: {id}"))?;
        session.lock().await.write(data)?;
        Ok(())
    }

    pub async fn resize(&self, id: SessionId, cols: u16, rows: u16) -> Result<()> {
        let session = self
            .sessions
            .read()
            .await
            .get(&id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("session not found: {id}"))?;
        session.lock().await.resize(cols, rows)?;
        Ok(())
    }

    pub async fn kill(&self, id: SessionId) -> Result<()> {
        let session = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(&id)
        };

        if let Some(session) = session {
            session.lock().await.kill()?;
            tracing::info!(%id, "session killed");
        }

        self.persist_state().await;
        Ok(())
    }

    pub async fn subscribe(
        &self,
        id: SessionId,
    ) -> Result<(broadcast::Receiver<DaemonMessage>, bool)> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("session not found: {id}"))?
            .clone();
        let s = session.lock().await;
        let rx = s.subscribe();
        let alive = !s.is_dead();
        Ok((rx, alive))
    }

    pub fn read_workspace_topology(&self) -> Option<WorkspaceTopology> {
        let path = match amux_protocol::ensure_amux_data_dir() {
            Ok(dir) => dir.join("workspace-topology.json"),
            Err(_) => return None,
        };
        let data = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&data).ok()
    }

    pub async fn list(&self) -> Vec<SessionInfo> {
        self.list_filtered(None).await
    }

    pub async fn list_workspace(&self, workspace_id: &str) -> Vec<SessionInfo> {
        self.list_filtered(Some(workspace_id)).await
    }

    async fn list_filtered(&self, workspace_filter: Option<&str>) -> Vec<SessionInfo> {
        let sessions: Vec<(SessionId, Arc<Mutex<PtySession>>)> = self
            .sessions
            .read()
            .await
            .iter()
            .map(|(id, session)| (*id, session.clone()))
            .collect();

        let mut infos = Vec::with_capacity(sessions.len());
        let mut live_ids = std::collections::HashSet::with_capacity(sessions.len());
        for (id, session) in sessions {
            let s = session.lock().await;
            if s.is_dead() {
                continue;
            }
            let workspace_id = s.workspace_id().map(ToOwned::to_owned);

            if let Some(filter) = workspace_filter {
                if workspace_id.as_deref() != Some(filter) {
                    continue;
                }
            }

            infos.push(SessionInfo {
                id,
                title: s.title().map(|t| t.to_owned()),
                cwd: s.resolved_cwd().or_else(|| s.cwd().map(|c| c.to_owned())),
                cols: s.cols(),
                rows: s.rows(),
                created_at: s.created_at(),
                workspace_id,
                exit_code: None,
                is_alive: !s.is_dead(),
                active_command: s.active_command(),
            });
            live_ids.insert(id);
        }

        let restored_sessions = self.restored_sessions.read().await.clone();
        for session in restored_sessions {
            let Ok(id) = session.id.parse() else {
                tracing::warn!(session_id = %session.id, "skipping restored session with invalid id");
                continue;
            };

            if live_ids.contains(&id) {
                continue;
            }

            if let Some(filter) = workspace_filter {
                if session.workspace_id.as_deref() != Some(filter) {
                    continue;
                }
            }

            infos.push(SessionInfo {
                id,
                title: None,
                cwd: session.cwd,
                cols: session.cols,
                rows: session.rows,
                created_at: 0,
                workspace_id: session.workspace_id,
                exit_code: None,
                is_alive: false,
                active_command: None,
            });
        }
        infos
    }

    pub async fn get_scrollback(&self, id: SessionId, max_lines: Option<usize>) -> Result<Vec<u8>> {
        let session = self
            .sessions
            .read()
            .await
            .get(&id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("session not found: {id}"))?;
        let data = session.lock().await.scrollback(max_lines);
        Ok(data)
    }

    pub async fn get_analysis_text(
        &self,
        id: SessionId,
        max_lines: Option<usize>,
    ) -> Result<String> {
        let raw = self.get_scrollback(id, max_lines).await?;
        let stripped = strip_ansi_escapes::strip(&raw);
        Ok(String::from_utf8_lossy(&stripped).into_owned())
    }

    pub async fn get_background_task_status(
        &self,
        background_task_id: &str,
    ) -> Result<Option<BackgroundTaskStatus>> {
        let sessions: Vec<(SessionId, Arc<Mutex<PtySession>>)> = self
            .sessions
            .read()
            .await
            .iter()
            .map(|(id, session)| (*id, session.clone()))
            .collect();

        for (session_id, session) in sessions {
            let live_status = session
                .lock()
                .await
                .managed_command_status(background_task_id);
            if let Some(live_status) = live_status {
                let state = match live_status.state {
                    crate::pty_session::ManagedCommandLiveState::Queued => {
                        BackgroundTaskState::Queued
                    }
                    crate::pty_session::ManagedCommandLiveState::Running => {
                        BackgroundTaskState::Running
                    }
                };
                return Ok(Some(BackgroundTaskStatus {
                    background_task_id: background_task_id.to_string(),
                    kind: "managed_command".to_string(),
                    state,
                    session_id: Some(session_id.to_string()),
                    position: Some(live_status.position),
                    command: Some(live_status.command),
                    exit_code: None,
                    duration_ms: None,
                    snapshot_path: live_status.snapshot_path,
                }));
            }
        }

        if let Some(finished) = self.history.get_managed_finish(background_task_id).await? {
            let state = if finished.exit_code == Some(0) {
                BackgroundTaskState::Completed
            } else {
                BackgroundTaskState::Failed
            };
            return Ok(Some(BackgroundTaskStatus {
                background_task_id: background_task_id.to_string(),
                kind: "managed_command".to_string(),
                state,
                session_id: None,
                position: None,
                command: Some(finished.command),
                exit_code: finished.exit_code,
                duration_ms: finished.duration_ms,
                snapshot_path: finished.snapshot_path,
            }));
        }

        Ok(None)
    }

    pub async fn reap_dead(&self) {
        let mut sessions = self.sessions.write().await;
        let mut dead: Vec<SessionId> = Vec::new();
        for (id, session) in sessions.iter() {
            if session.lock().await.is_dead() {
                dead.push(*id);
            }
        }
        for id in &dead {
            sessions.remove(id);
            tracing::info!(%id, "reaped dead session");
        }

        drop(sessions);
        self.persist_state().await;
    }

    pub async fn execute_managed_command(
        &self,
        id: SessionId,
        request: ManagedCommandRequest,
    ) -> Result<DaemonMessage> {
        validate_command(&request.command, request.language_hint.as_deref())?;

        let session = self
            .sessions
            .read()
            .await
            .get(&id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("session not found: {id}"))?;
        if session.lock().await.is_dead() {
            anyhow::bail!("session is not alive: {id}");
        }

        let workspace_id = session.lock().await.workspace_id().map(ToOwned::to_owned);
        let execution_id = format!("exec_{}", Uuid::new_v4());
        let governance_input = governance_input_for_managed_command(
            &execution_id,
            &request,
            workspace_id.clone(),
            Some(id.to_string()),
        );
        let verdict = evaluate_governance(&governance_input);
        let constraints = effective_constraints(&verdict);

        self.history
            .insert_governance_evaluation(&GovernanceEvaluationRow {
                id: format!("gov_{}", Uuid::new_v4()),
                run_id: governance_input.run_id.clone(),
                task_id: governance_input.task_id.clone(),
                goal_run_id: governance_input.goal_run_id.clone(),
                thread_id: governance_input.thread_id.clone(),
                transition_kind: transition_kind_str(&governance_input.transition_kind).to_string(),
                input_json: serde_json::to_string(&governance_input)?,
                verdict_json: serde_json::to_string(&verdict)?,
                policy_fingerprint: verdict.policy_fingerprint.clone(),
                created_at: crate::history::now_ts(),
            })
            .await?;

        if !can_honor_constraints(&constraints, &request) {
            return Ok(DaemonMessage::ManagedCommandRejected {
                id,
                execution_id: Some(execution_id),
                message: format!(
                    "managed command blocked because current runtime cannot honor governance constraints: {}",
                    constraints
                        .iter()
                        .map(|constraint| constraint_kind_str(&constraint.kind))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            });
        }
        let mut constrained_request = request.clone();
        apply_constraints_to_request(&mut constrained_request, &constraints);

        match verdict.verdict_class {
            VerdictClass::Allow | VerdictClass::AllowWithConstraints => {
                let (position, snapshot) = queue_with_snapshot(
                    &self.snapshots,
                    &session,
                    workspace_id.clone(),
                    id,
                    execution_id.clone(),
                    constrained_request,
                    "pre-execution checkpoint",
                )
                .await?;
                Ok(DaemonMessage::ManagedCommandQueued {
                    id,
                    execution_id,
                    position,
                    snapshot,
                })
            }
            VerdictClass::RequireApproval => {
                let requested_at = crate::history::now_ts();
                let expires_at = verdict
                    .approval_requirement
                    .as_ref()
                    .and_then(|requirement| requirement.expires_at)
                    .or_else(|| {
                        verdict
                            .freshness_window_secs
                            .map(|window| requested_at + window)
                    });

                if self
                    .has_valid_session_grant(id, &verdict.policy_fingerprint)
                    .await
                {
                    let (position, snapshot) = queue_with_snapshot(
                        &self.snapshots,
                        &session,
                        workspace_id.clone(),
                        id,
                        execution_id.clone(),
                        constrained_request,
                        "session-approved pre-execution checkpoint",
                    )
                    .await?;
                    return Ok(DaemonMessage::ManagedCommandQueued {
                        id,
                        execution_id,
                        position,
                        snapshot,
                    });
                }

                let approval = approval_payload_from_verdict(
                    &execution_id,
                    &request,
                    workspace_id.clone(),
                    &governance_input,
                    &verdict,
                    expires_at,
                );

                self.history
                    .insert_approval_record(&ApprovalRecordRow {
                        approval_id: approval.approval_id.clone(),
                        run_id: governance_input.run_id.clone(),
                        task_id: governance_input.task_id.clone(),
                        goal_run_id: governance_input.goal_run_id.clone(),
                        thread_id: governance_input.thread_id.clone(),
                        transition_kind: transition_kind_str(&governance_input.transition_kind)
                            .to_string(),
                        stage_id: governance_input.stage_id.clone(),
                        scope_summary: verdict
                            .approval_requirement
                            .as_ref()
                            .map(|requirement| requirement.scope_summary.clone())
                            .or_else(|| Some(blast_radius_summary(&governance_input))),
                        target_scope_json: json!({
                            "lane_ids": &governance_input.lane_ids,
                            "target_ids": &governance_input.target_ids,
                        })
                        .to_string(),
                        constraints_json: serde_json::to_string(&constraints)?,
                        risk_class: risk_class_str(&verdict.risk_class).to_string(),
                        rationale_json: serde_json::to_string(&verdict.rationale)?,
                        policy_fingerprint: verdict.policy_fingerprint.clone(),
                        requested_at,
                        resolved_at: None,
                        expires_at,
                        resolution: None,
                        invalidated_at: None,
                        invalidation_reason: None,
                    })
                    .await?;

                self.pending_approvals.write().await.insert(
                    approval.approval_id.clone(),
                    PendingApproval {
                        session_id: id,
                        workspace_id,
                        execution_id,
                        request,
                        policy_fingerprint: verdict.policy_fingerprint.clone(),
                        constraints: constraints.clone(),
                        transition_kind: governance_input.transition_kind.clone(),
                        expires_at,
                    },
                );
                Ok(DaemonMessage::ApprovalRequired { id, approval })
            }
            VerdictClass::Defer
            | VerdictClass::Deny
            | VerdictClass::HaltAndIsolate
            | VerdictClass::AllowOnlyWithCompensationPlan => {
                Ok(DaemonMessage::ManagedCommandRejected {
                    id,
                    execution_id: Some(execution_id),
                    message: governance_rejection_message(&verdict),
                })
            }
        }
    }

    pub async fn resolve_approval(
        &self,
        id: SessionId,
        approval_id: &str,
        decision: ApprovalDecision,
    ) -> Result<Vec<DaemonMessage>> {
        let pending = self
            .pending_approvals
            .read()
            .await
            .get(approval_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("approval not found: {approval_id}"))?;

        let now = crate::history::now_ts();
        if pending
            .expires_at
            .map(|expires_at| now > expires_at)
            .unwrap_or(false)
        {
            self.pending_approvals.write().await.remove(approval_id);
            self.history
                .invalidate_approval_record(approval_id, "approval expired before resolution", now)
                .await?;
            anyhow::bail!("approval is stale: approval window expired before resolution");
        }

        let mut fresh_input = governance_input_for_managed_command(
            &pending.execution_id,
            &pending.request,
            pending.workspace_id.clone(),
            Some(pending.session_id.to_string()),
        );
        fresh_input.transition_kind = pending.transition_kind.clone();
        let fresh_verdict = evaluate_governance(&fresh_input);
        if fresh_verdict.policy_fingerprint != pending.policy_fingerprint {
            self.pending_approvals.write().await.remove(approval_id);
            self.history
                .invalidate_approval_record(
                    approval_id,
                    "approval invalidated because governance conditions changed",
                    now,
                )
                .await?;
            anyhow::bail!("approval is stale: governance conditions changed since it was issued");
        }

        let pending = self
            .pending_approvals
            .write()
            .await
            .remove(approval_id)
            .ok_or_else(|| anyhow::anyhow!("approval not found: {approval_id}"))?;

        self.history
            .resolve_approval_record(approval_id, approval_resolution_str(decision), now)
            .await?;

        let mut responses = vec![DaemonMessage::ApprovalResolved {
            id,
            approval_id: approval_id.to_string(),
            decision,
        }];

        if matches!(decision, ApprovalDecision::Deny) {
            responses.push(DaemonMessage::ManagedCommandRejected {
                id,
                execution_id: Some(pending.execution_id),
                message: "execution denied by operator".to_string(),
            });
            return Ok(responses);
        }

        let session = self
            .sessions
            .read()
            .await
            .get(&pending.session_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("session not found: {}", pending.session_id))?;

        let mut request = pending.request.clone();
        if !can_honor_constraints(&pending.constraints, &request) {
            anyhow::bail!(
                "managed command blocked because current runtime can no longer honor governance constraints"
            );
        }
        apply_constraints_to_request(&mut request, &pending.constraints);

        if matches!(decision, ApprovalDecision::ApproveSession) {
            self.store_session_grant(
                pending.session_id,
                approval_id,
                &pending.policy_fingerprint,
                pending.expires_at,
            )
            .await;
        }

        let (position, snapshot) = queue_with_snapshot(
            &self.snapshots,
            &session,
            pending.workspace_id.clone(),
            pending.session_id,
            pending.execution_id.clone(),
            request,
            "approved pre-execution checkpoint",
        )
        .await?;
        responses.push(DaemonMessage::ManagedCommandQueued {
            id,
            execution_id: pending.execution_id,
            position,
            snapshot,
        });
        Ok(responses)
    }

    pub fn verify_telemetry_integrity(&self) -> Result<Vec<TelemetryLedgerStatus>> {
        let results = self.history.verify_worm_integrity()?;
        Ok(results
            .into_iter()
            .map(|r| TelemetryLedgerStatus {
                kind: r.kind,
                total_entries: r.total_entries,
                valid: r.valid,
                first_invalid_seq: r.first_invalid_seq,
                message: r.message,
            })
            .collect())
    }

    async fn snapshot_saved_sessions(&self) -> Vec<SavedSession> {
        let sessions: Vec<Arc<Mutex<PtySession>>> =
            self.sessions.read().await.values().cloned().collect();

        let mut saved = Vec::with_capacity(sessions.len());
        for session in sessions {
            let session = session.lock().await;
            saved.push(SavedSession {
                id: session.id().to_string(),
                shell: session.shell().map(ToOwned::to_owned),
                cwd: session.cwd().map(ToOwned::to_owned),
                workspace_id: session.workspace_id().map(ToOwned::to_owned),
                cols: session.cols(),
                rows: session.rows(),
            });
        }

        saved
    }

    async fn persist_state(&self) {
        let previous_sessions = self.snapshot_saved_sessions().await;
        let state = DaemonState { previous_sessions };
        if let Err(error) = save_state(&self.state_path, &state) {
            tracing::error!(error = %error, path = %self.state_path.display(), "failed to persist daemon state");
        }
    }
}
