use super::*;
use crate::agent::AgentEngine;
use anyhow::Result;
use std::sync::Arc;
use zorai_protocol::{ClientMessage, DaemonMessage};

pub(crate) async fn dispatch_part8(
    msg: &ClientMessage,
    agent: &Arc<AgentEngine>,
    framed: &mut ConnectionWriter,
    background_daemon_queues: &mut BackgroundSubsystemQueues,
    background_daemon_pending: &mut BackgroundPendingCounts,
) -> Result<bool>
{
    if !matches!(
        msg,
        ClientMessage::SkillInspect { .. }
            | ClientMessage::SkillReject { .. }
            | ClientMessage::SkillPromote { .. }
            | ClientMessage::SkillSearch { .. }
            | ClientMessage::SkillImport { .. }
            | ClientMessage::SkillExport { .. }
    ) {
        return Ok(false);
    }
    let msg = msg.clone();

    match msg {
        ClientMessage::SkillInspect { identifier } => {
            let variant = match agent.history.get_skill_variant(&identifier).await {
                Ok(Some(v)) => Some(v),
                _ => {
                    match agent
                        .history
                        .list_skill_variants(Some(&identifier), 1)
                        .await
                    {
                        Ok(variants) => variants.into_iter().next(),
                        Err(_) => None,
                    }
                }
            };

            let (public, content) = if let Some(ref v) = variant {
                let skill_path = agent
                    .data_dir
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join("skills")
                    .join(&v.relative_path);
                let raw_content = tokio::fs::read_to_string(&skill_path).await.ok();
                let inspection_note = agent
                            .history
                            .inspect_skill_variants(&v.skill_name, &v.context_tags)
                            .await
                            .ok()
                            .and_then(|items| {
                                items.into_iter()
                                    .find(|item| item.record.variant_id == v.variant_id)
                            })
                            .map(|item| {
                                format!(
                                    "## Lifecycle Inspection\n- Status rationale: {}\n- Selection rationale: {}\n- Selected for current context: {}\n\n",
                                    item.lifecycle_summary,
                                    item.selection_summary,
                                    if item.selected_for_context { "yes" } else { "no" }
                                )
                            });
                let content = match (inspection_note, raw_content) {
                    (Some(note), Some(body)) => Some(format!("{note}{body}")),
                    (Some(note), None) => Some(note),
                    (None, body) => body,
                };
                let public = zorai_protocol::SkillVariantPublic {
                    variant_id: v.variant_id.clone(),
                    skill_name: v.skill_name.clone(),
                    variant_name: v.variant_name.clone(),
                    relative_path: v.relative_path.clone(),
                    status: v.status.clone(),
                    use_count: v.use_count,
                    success_count: v.success_count,
                    failure_count: v.failure_count,
                    context_tags: v.context_tags.clone(),
                    created_at: v.created_at,
                    updated_at: v.updated_at,
                };
                (Some(public), content)
            } else {
                (None, None)
            };

            framed
                .send(DaemonMessage::SkillInspectResult {
                    variant: public,
                    content,
                })
                .await?;
        }

        ClientMessage::SkillReject { identifier } => {
            let variant = match agent.history.get_skill_variant(&identifier).await {
                Ok(Some(v)) => Some(v),
                _ => {
                    match agent
                        .history
                        .list_skill_variants(Some(&identifier), 1)
                        .await
                    {
                        Ok(variants) => variants.into_iter().next(),
                        Err(_) => None,
                    }
                }
            };

            let msg = if let Some(v) = variant {
                if v.status != "draft" && v.status != "testing" {
                    DaemonMessage::SkillActionResult {
                                success: false,
                                message: format!(
                                    "Cannot reject skill '{}' with status '{}' -- only draft/testing skills can be rejected.",
                                    v.skill_name, v.status
                                ),
                            }
                } else {
                    let skill_path = agent
                        .data_dir
                        .parent()
                        .unwrap_or(std::path::Path::new("."))
                        .join("skills")
                        .join(&v.relative_path);
                    let _ = tokio::fs::remove_file(&skill_path).await;

                    match agent
                        .history
                        .update_skill_variant_status(&v.variant_id, "archived")
                        .await
                    {
                        Ok(()) => DaemonMessage::SkillActionResult {
                            success: true,
                            message: format!("Rejected and archived skill '{}'.", v.skill_name),
                        },
                        Err(e) => DaemonMessage::SkillActionResult {
                            success: false,
                            message: format!("Failed to archive skill: {e}"),
                        },
                    }
                }
            } else {
                DaemonMessage::SkillActionResult {
                    success: false,
                    message: format!("Skill not found: {identifier}"),
                }
            };
            framed.send(msg).await?;
        }

        ClientMessage::SkillPromote {
            identifier,
            target_status,
        } => {
            let valid_statuses = [
                "draft",
                "testing",
                "active",
                "proven",
                "promoted_to_canonical",
            ];
            if !valid_statuses.contains(&target_status.as_str()) {
                framed
                    .send(DaemonMessage::SkillActionResult {
                        success: false,
                        message: format!(
                            "Invalid target status '{}'. Valid: {}",
                            target_status,
                            valid_statuses.join(", ")
                        ),
                    })
                    .await?;
            } else {
                let variant = match agent.history.get_skill_variant(&identifier).await {
                    Ok(Some(v)) => Some(v),
                    _ => {
                        match agent
                            .history
                            .list_skill_variants(Some(&identifier), 1)
                            .await
                        {
                            Ok(variants) => variants.into_iter().next(),
                            Err(_) => None,
                        }
                    }
                };

                let msg = if let Some(v) = variant {
                    match agent
                        .history
                        .update_skill_variant_status(&v.variant_id, &target_status)
                        .await
                    {
                        Ok(()) => {
                            agent
                                .record_provenance_event(
                                    "skill_lifecycle_promotion",
                                    &format!(
                                        "Skill '{}' fast-promoted {} -> {} via CLI",
                                        v.skill_name, v.status, target_status
                                    ),
                                    serde_json::json!({
                                        "variant_id": v.variant_id,
                                        "skill_name": v.skill_name,
                                        "from_status": v.status,
                                        "to_status": target_status,
                                        "trigger": "cli_promote",
                                    }),
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                )
                                .await;

                            DaemonMessage::SkillActionResult {
                                success: true,
                                message: format!(
                                    "Skill '{}' promoted from {} to {}.",
                                    v.skill_name, v.status, target_status
                                ),
                            }
                        }
                        Err(e) => DaemonMessage::SkillActionResult {
                            success: false,
                            message: format!("Failed to promote skill: {e}"),
                        },
                    }
                } else {
                    DaemonMessage::SkillActionResult {
                        success: false,
                        message: format!("Skill not found: {identifier}"),
                    }
                };
                framed.send(msg).await?;
            }
        }

        ClientMessage::SkillSearch { query } => {
            let config = agent.config.read().await;
            let registry_url = config
                .extra
                .get("registry_url")
                .and_then(|value| value.as_str())
                .unwrap_or("https://registry.zorai.dev")
                .to_string();
            drop(config);

            let registry_root = agent.data_dir.parent().unwrap_or(agent.data_dir.as_path());
            let client = RegistryClient::new(registry_url, registry_root);
            let entries: Vec<zorai_protocol::CommunitySkillEntry> =
                match client.search(&query).await {
                    Ok(entries) => entries
                        .into_iter()
                        .map(|entry| to_community_entry(&entry))
                        .collect(),
                    Err(_) => Vec::new(),
                };
            framed
                .send(DaemonMessage::SkillSearchResult { entries })
                .await?;
        }

        ClientMessage::SkillImport {
            source,
            force,
            publisher_verified,
        } => {
            let config = agent.config.read().await;
            let registry_url = config
                .extra
                .get("registry_url")
                .and_then(|value| value.as_str())
                .unwrap_or("https://registry.zorai.dev")
                .to_string();
            drop(config);

            let whitelist = vec![
                zorai_protocol::tool_names::READ_FILE.to_string(),
                zorai_protocol::tool_names::WRITE_FILE.to_string(),
                zorai_protocol::tool_names::LIST_FILES.to_string(),
                "create_directory".to_string(),
                zorai_protocol::tool_names::SEARCH_HISTORY.to_string(),
            ];
            if !background_daemon_pending.has_capacity(BackgroundSubsystem::PluginIo) {
                background_daemon_pending.note_rejection(BackgroundSubsystem::PluginIo);
                framed
                    .send(DaemonMessage::Error {
                        message: "plugin_io background queue is full".to_string(),
                    })
                    .await?;
                return Ok(true);
            }

            let operation = operation_registry().accept_operation(
                OPERATION_KIND_SKILL_IMPORT,
                Some(skill_import_dedup_key(
                    &agent,
                    &source,
                    force,
                    publisher_verified,
                )),
            );

            framed
                .send(DaemonMessage::OperationAccepted {
                    operation_id: operation.operation_id.clone(),
                    kind: operation.kind.clone(),
                    dedup: operation.dedup.clone(),
                    revision: operation.revision,
                })
                .await?;

            let operation_id = Some(operation.operation_id.clone());
            let result_operation_id = operation_id.clone();
            let history = agent.history.clone();
            let skills_root = agent.history.data_dir().to_path_buf();
            let registry_root = agent
                .data_dir
                .parent()
                .unwrap_or(agent.data_dir.as_path())
                .to_path_buf();
            let background_daemon_tx =
                background_daemon_queues.sender(BackgroundSubsystem::PluginIo);
            spawn_background_operation(
                BackgroundSubsystem::PluginIo,
                operation_id,
                background_daemon_tx,
                background_daemon_pending,
                async move {
                    let client = RegistryClient::new(registry_url, &registry_root);
                    let import_result: Result<(String, String), anyhow::Error> = async {
                        if source.starts_with("http://") || source.starts_with("https://") {
                            let archive_name = source
                                .rsplit('/')
                                .next()
                                .unwrap_or("community-skill.tar.gz")
                                .trim_end_matches(".tar.gz")
                                .to_string();
                            let archive_path = client.fetch_skill(&archive_name).await?;
                            let extract_dir = std::env::temp_dir().join(format!(
                                "zorai-community-import-{}-{}",
                                archive_name,
                                uuid::Uuid::new_v4()
                            ));
                            if extract_dir.exists() {
                                let _ = tokio::fs::remove_dir_all(&extract_dir).await;
                            }
                            tokio::fs::create_dir_all(&extract_dir).await?;
                            unpack_skill(&archive_path, &extract_dir)?;
                            let skill_path = extract_dir.join("SKILL.md");
                            let content = tokio::fs::read_to_string(&skill_path).await?;
                            Ok((archive_name, content))
                        } else {
                            let archive_path = client.fetch_skill(&source).await?;
                            let extract_dir = std::env::temp_dir().join(format!(
                                "zorai-community-import-{}-{}",
                                source,
                                uuid::Uuid::new_v4()
                            ));
                            if extract_dir.exists() {
                                let _ = tokio::fs::remove_dir_all(&extract_dir).await;
                            }
                            tokio::fs::create_dir_all(&extract_dir).await?;
                            unpack_skill(&archive_path, &extract_dir)?;
                            let skill_path = extract_dir.join("SKILL.md");
                            let content = tokio::fs::read_to_string(&skill_path).await?;
                            Ok((source.clone(), content))
                        }
                    }
                    .await;

                    match import_result {
                        Ok((skill_name, content)) => match import_community_skill(
                            &history,
                            &content,
                            &skill_name,
                            &source,
                            &whitelist,
                            force,
                            publisher_verified,
                            &skills_root,
                        )
                        .await
                        {
                            Ok(ImportResult::Success {
                                variant_id,
                                scan_verdict,
                            }) => BackgroundOperationOutput::Completed(
                                DaemonMessage::SkillImportResult {
                                    operation_id: result_operation_id.clone(),
                                    success: true,
                                    message: format!(
                                        "Imported community skill '{skill_name}' as draft."
                                    ),
                                    variant_id: Some(variant_id),
                                    scan_verdict: Some(scan_verdict),
                                    findings_count: 0,
                                },
                            ),
                            Ok(ImportResult::Blocked {
                                report_summary,
                                findings_count,
                            }) => BackgroundOperationOutput::Failed(
                                DaemonMessage::SkillImportResult {
                                    operation_id: result_operation_id.clone(),
                                    success: false,
                                    message: report_summary,
                                    variant_id: None,
                                    scan_verdict: Some("block".to_string()),
                                    findings_count,
                                },
                            ),
                            Ok(ImportResult::NeedsForce {
                                report_summary,
                                findings_count,
                            }) => BackgroundOperationOutput::Failed(
                                DaemonMessage::SkillImportResult {
                                    operation_id: result_operation_id.clone(),
                                    success: false,
                                    message: report_summary,
                                    variant_id: None,
                                    scan_verdict: Some("warn".to_string()),
                                    findings_count,
                                },
                            ),
                            Err(e) => BackgroundOperationOutput::Failed(
                                DaemonMessage::SkillImportResult {
                                    operation_id: result_operation_id.clone(),
                                    success: false,
                                    message: format!("community skill import failed: {e}"),
                                    variant_id: None,
                                    scan_verdict: None,
                                    findings_count: 0,
                                },
                            ),
                        },
                        Err(e) => {
                            BackgroundOperationOutput::Failed(DaemonMessage::SkillImportResult {
                                operation_id: result_operation_id.clone(),
                                success: false,
                                message: format!("community skill fetch failed: {e}"),
                                variant_id: None,
                                scan_verdict: None,
                                findings_count: 0,
                            })
                        }
                    }
                },
            );
        }

        ClientMessage::SkillExport {
            identifier,
            format,
            output_dir,
        } => {
            let variant = match agent.history.get_skill_variant(&identifier).await {
                Ok(Some(v)) => Some(v),
                _ => match agent
                    .history
                    .list_skill_variants(Some(&identifier), 1)
                    .await
                {
                    Ok(variants) => variants.into_iter().next(),
                    Err(_) => None,
                },
            };

            let msg = if let Some(v) = variant {
                let skill_path = agent
                    .history
                    .data_dir()
                    .join("skills")
                    .join(&v.relative_path);
                match tokio::fs::read_to_string(&skill_path).await {
                    Ok(content) => {
                        match export_skill(&content, &format, Path::new(&output_dir), &v.skill_name)
                        {
                            Ok(path) => DaemonMessage::SkillExportResult {
                                success: true,
                                message: format!("Exported skill '{}' to {}.", v.skill_name, path),
                                output_path: Some(path),
                            },
                            Err(e) => DaemonMessage::SkillExportResult {
                                success: false,
                                message: format!("community skill export failed: {e}"),
                                output_path: None,
                            },
                        }
                    }
                    Err(e) => DaemonMessage::SkillExportResult {
                        success: false,
                        message: format!("failed to read skill for export: {e}"),
                        output_path: None,
                    },
                }
            } else {
                DaemonMessage::SkillExportResult {
                    success: false,
                    message: format!("Skill not found: {identifier}"),
                    output_path: None,
                }
            };
            framed.send(msg).await?;
        }

        _ => unreachable!("message chunk should be exhaustive"),
    }
    Ok(true)
}
