use super::*;
use crate::agent::AgentEngine;
use anyhow::Result;
use std::sync::Arc;
use zorai_protocol::{ClientMessage, DaemonMessage};

pub(crate) async fn dispatch_part5(
    msg: &ClientMessage,
    agent: &Arc<AgentEngine>,
    framed: &mut ConnectionWriter,
) -> Result<bool>
{
    if !matches!(
        msg,
        ClientMessage::AgentListHealthLog { .. }
            | ClientMessage::AgentStartOperatorProfileSession { .. }
            | ClientMessage::AgentNextOperatorProfileQuestion { .. }
            | ClientMessage::AgentSubmitOperatorProfileAnswer { .. }
            | ClientMessage::AgentSkipOperatorProfileQuestion { .. }
            | ClientMessage::AgentDeferOperatorProfileQuestion { .. }
            | ClientMessage::AgentGetOperatorProfileSummary
            | ClientMessage::AgentAskQuestion { .. }
            | ClientMessage::AgentAnswerQuestion { .. }
    ) {
        return Ok(false);
    }
    let msg = msg.clone();

    match msg {
        ClientMessage::AgentListHealthLog { limit } => {
            let entries_json = match agent.health_log_entries(limit.unwrap_or(50).max(1)).await {
                Ok(entries) => serde_json::to_string(&entries).unwrap_or_else(|_| "[]".into()),
                Err(e) => {
                    framed
                        .send(DaemonMessage::AgentError {
                            message: format!("failed to list health log: {e}"),
                        })
                        .await
                        .ok();
                    return Ok(true);
                }
            };
            framed
                .send(DaemonMessage::AgentHealthLog { entries_json })
                .await
                .ok();
        }

        ClientMessage::AgentStartOperatorProfileSession { kind } => {
            match agent.start_operator_profile_session(&kind).await {
                Ok(started) => {
                    framed
                        .send(DaemonMessage::AgentOperatorProfileSessionStarted {
                            session_id: started.session_id.clone(),
                            kind: started.kind.clone(),
                        })
                        .await
                        .ok();
                    match agent
                        .next_operator_profile_question_for_session(&started.session_id)
                        .await
                    {
                        Ok((question, progress)) => {
                            if let Some(question) = question {
                                framed
                                    .send(DaemonMessage::AgentOperatorProfileQuestion {
                                        session_id: question.session_id,
                                        question_id: question.question_id,
                                        field_key: question.field_key,
                                        prompt: question.prompt,
                                        input_kind: question.input_kind,
                                        optional: question.optional,
                                    })
                                    .await
                                    .ok();
                                framed
                                    .send(DaemonMessage::AgentOperatorProfileProgress {
                                        session_id: progress.session_id,
                                        answered: progress.answered,
                                        remaining: progress.remaining,
                                        completion_ratio: progress.completion_ratio,
                                    })
                                    .await
                                    .ok();
                            } else {
                                match agent
                                    .complete_operator_profile_session(&started.session_id)
                                    .await
                                {
                                    Ok(done) => {
                                        framed
                                                    .send(DaemonMessage::AgentOperatorProfileSessionCompleted {
                                                        session_id: done.session_id,
                                                        updated_fields: done.updated_fields,
                                                    })
                                                    .await
                                                    .ok();
                                    }
                                    Err(error) => {
                                        framed
                                                    .send(DaemonMessage::AgentError {
                                                        message: format!(
                                                            "failed to complete operator profile session: {error}"
                                                        ),
                                                    })
                                                    .await
                                                    .ok();
                                    }
                                }
                            }
                        }
                        Err(error) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: format!(
                                        "failed to fetch operator profile question: {error}"
                                    ),
                                })
                                .await
                                .ok();
                        }
                    }
                }
                Err(error) => {
                    framed
                        .send(DaemonMessage::AgentError {
                            message: format!("failed to start operator profile session: {error}"),
                        })
                        .await
                        .ok();
                }
            }
        }

        ClientMessage::AgentNextOperatorProfileQuestion { session_id } => {
            match agent
                .next_operator_profile_question_for_session(&session_id)
                .await
            {
                Ok((question, progress)) => {
                    if let Some(question) = question {
                        framed
                            .send(DaemonMessage::AgentOperatorProfileQuestion {
                                session_id: question.session_id,
                                question_id: question.question_id,
                                field_key: question.field_key,
                                prompt: question.prompt,
                                input_kind: question.input_kind,
                                optional: question.optional,
                            })
                            .await
                            .ok();
                        framed
                            .send(DaemonMessage::AgentOperatorProfileProgress {
                                session_id: progress.session_id,
                                answered: progress.answered,
                                remaining: progress.remaining,
                                completion_ratio: progress.completion_ratio,
                            })
                            .await
                            .ok();
                    } else if progress.remaining > 0 {
                        framed
                            .send(DaemonMessage::AgentOperatorProfileProgress {
                                session_id: progress.session_id,
                                answered: progress.answered,
                                remaining: progress.remaining,
                                completion_ratio: progress.completion_ratio,
                            })
                            .await
                            .ok();
                    } else {
                        match agent.complete_operator_profile_session(&session_id).await {
                            Ok(done) => {
                                framed
                                    .send(DaemonMessage::AgentOperatorProfileSessionCompleted {
                                        session_id: done.session_id,
                                        updated_fields: done.updated_fields,
                                    })
                                    .await
                                    .ok();
                            }
                            Err(error) => {
                                framed
                                    .send(DaemonMessage::AgentError {
                                        message: format!(
                                            "failed to complete operator profile session: {error}"
                                        ),
                                    })
                                    .await
                                    .ok();
                            }
                        }
                    }
                }
                Err(error) => {
                    if is_unknown_operator_profile_session_error(&error) {
                        tracing::debug!(
                            session_id = %session_id,
                            "ignored stale operator profile next-question request"
                        );
                        return Ok(true);
                    }
                    framed
                        .send(DaemonMessage::AgentError {
                            message: format!("failed to fetch operator profile question: {error}"),
                        })
                        .await
                        .ok();
                }
            }
        }

        ClientMessage::AgentSubmitOperatorProfileAnswer {
            session_id,
            question_id,
            answer_json,
        } => {
            match agent
                .submit_operator_profile_answer(&session_id, &question_id, &answer_json)
                .await
            {
                Ok((question, progress)) => {
                    if let Some(question) = question {
                        framed
                            .send(DaemonMessage::AgentOperatorProfileQuestion {
                                session_id: question.session_id,
                                question_id: question.question_id,
                                field_key: question.field_key,
                                prompt: question.prompt,
                                input_kind: question.input_kind,
                                optional: question.optional,
                            })
                            .await
                            .ok();
                        framed
                            .send(DaemonMessage::AgentOperatorProfileProgress {
                                session_id: progress.session_id,
                                answered: progress.answered,
                                remaining: progress.remaining,
                                completion_ratio: progress.completion_ratio,
                            })
                            .await
                            .ok();
                    } else if progress.remaining > 0 {
                        framed
                            .send(DaemonMessage::AgentOperatorProfileProgress {
                                session_id: progress.session_id,
                                answered: progress.answered,
                                remaining: progress.remaining,
                                completion_ratio: progress.completion_ratio,
                            })
                            .await
                            .ok();
                    } else {
                        match agent.complete_operator_profile_session(&session_id).await {
                            Ok(done) => {
                                framed
                                    .send(DaemonMessage::AgentOperatorProfileSessionCompleted {
                                        session_id: done.session_id,
                                        updated_fields: done.updated_fields,
                                    })
                                    .await
                                    .ok();
                            }
                            Err(error) => {
                                framed
                                    .send(DaemonMessage::AgentError {
                                        message: format!(
                                            "failed to complete operator profile session: {error}"
                                        ),
                                    })
                                    .await
                                    .ok();
                            }
                        }
                    }
                }
                Err(error) => {
                    framed
                        .send(DaemonMessage::AgentError {
                            message: format!("failed to submit operator profile answer: {error}"),
                        })
                        .await
                        .ok();
                }
            }
        }

        ClientMessage::AgentSkipOperatorProfileQuestion {
            session_id,
            question_id,
            reason,
        } => {
            match agent
                .skip_operator_profile_question(&session_id, &question_id, reason.as_deref())
                .await
            {
                Ok((question, progress)) => {
                    if let Some(question) = question {
                        framed
                            .send(DaemonMessage::AgentOperatorProfileQuestion {
                                session_id: question.session_id,
                                question_id: question.question_id,
                                field_key: question.field_key,
                                prompt: question.prompt,
                                input_kind: question.input_kind,
                                optional: question.optional,
                            })
                            .await
                            .ok();
                        framed
                            .send(DaemonMessage::AgentOperatorProfileProgress {
                                session_id: progress.session_id,
                                answered: progress.answered,
                                remaining: progress.remaining,
                                completion_ratio: progress.completion_ratio,
                            })
                            .await
                            .ok();
                    } else {
                        match agent.complete_operator_profile_session(&session_id).await {
                            Ok(done) => {
                                framed
                                    .send(DaemonMessage::AgentOperatorProfileSessionCompleted {
                                        session_id: done.session_id,
                                        updated_fields: done.updated_fields,
                                    })
                                    .await
                                    .ok();
                            }
                            Err(error) => {
                                framed
                                    .send(DaemonMessage::AgentError {
                                        message: format!(
                                            "failed to complete operator profile session: {error}"
                                        ),
                                    })
                                    .await
                                    .ok();
                            }
                        }
                    }
                }
                Err(error) => {
                    framed
                        .send(DaemonMessage::AgentError {
                            message: format!("failed to skip operator profile question: {error}"),
                        })
                        .await
                        .ok();
                }
            }
        }

        ClientMessage::AgentDeferOperatorProfileQuestion {
            session_id,
            question_id,
            defer_until_unix_ms,
        } => {
            match agent
                .defer_operator_profile_question(&session_id, &question_id, defer_until_unix_ms)
                .await
            {
                Ok((question, progress)) => {
                    if let Some(question) = question {
                        framed
                            .send(DaemonMessage::AgentOperatorProfileQuestion {
                                session_id: question.session_id,
                                question_id: question.question_id,
                                field_key: question.field_key,
                                prompt: question.prompt,
                                input_kind: question.input_kind,
                                optional: question.optional,
                            })
                            .await
                            .ok();
                        framed
                            .send(DaemonMessage::AgentOperatorProfileProgress {
                                session_id: progress.session_id,
                                answered: progress.answered,
                                remaining: progress.remaining,
                                completion_ratio: progress.completion_ratio,
                            })
                            .await
                            .ok();
                    } else if progress.remaining > 0 {
                        framed
                            .send(DaemonMessage::AgentOperatorProfileProgress {
                                session_id: progress.session_id,
                                answered: progress.answered,
                                remaining: progress.remaining,
                                completion_ratio: progress.completion_ratio,
                            })
                            .await
                            .ok();
                    } else {
                        match agent.complete_operator_profile_session(&session_id).await {
                            Ok(done) => {
                                framed
                                    .send(DaemonMessage::AgentOperatorProfileSessionCompleted {
                                        session_id: done.session_id,
                                        updated_fields: done.updated_fields,
                                    })
                                    .await
                                    .ok();
                            }
                            Err(error) => {
                                framed
                                    .send(DaemonMessage::AgentError {
                                        message: format!(
                                            "failed to complete operator profile session: {error}"
                                        ),
                                    })
                                    .await
                                    .ok();
                            }
                        }
                    }
                }
                Err(error) => {
                    framed
                        .send(DaemonMessage::AgentError {
                            message: format!("failed to defer operator profile question: {error}"),
                        })
                        .await
                        .ok();
                }
            }
        }

        ClientMessage::AgentGetOperatorProfileSummary => {
            match agent.get_operator_profile_summary_json().await {
                Ok(summary_json) => {
                    framed
                        .send(DaemonMessage::AgentOperatorProfileSummary { summary_json })
                        .await
                        .ok();
                }
                Err(error) => {
                    framed
                        .send(DaemonMessage::AgentError {
                            message: format!("failed to build operator profile summary: {error}"),
                        })
                        .await
                        .ok();
                }
            }
        }

        ClientMessage::AgentAskQuestion {
            content,
            options,
            session_id,
        } => match agent
            .ask_operator_question(&content, options.clone(), session_id.clone(), None)
            .await
        {
            Ok((question_id, answer)) => {
                framed
                    .send(DaemonMessage::AgentQuestionAnswered {
                        question_id: question_id.clone(),
                        answer: answer.clone(),
                    })
                    .await
                    .ok();
            }
            Err(error) => {
                framed
                    .send(DaemonMessage::AgentError {
                        message: format!("failed to ask operator question: {error}"),
                    })
                    .await
                    .ok();
            }
        },

        ClientMessage::AgentAnswerQuestion {
            question_id,
            answer,
        } => match agent.answer_operator_question(&question_id, &answer).await {
            Ok(()) => {
                framed
                    .send(DaemonMessage::AgentQuestionAnswered {
                        question_id: question_id.clone(),
                        answer: answer.clone(),
                    })
                    .await
                    .ok();
            }
            Err(error) => {
                framed
                    .send(DaemonMessage::AgentError {
                        message: format!("failed to answer operator question: {error}"),
                    })
                    .await
                    .ok();
            }
        },

        _ => unreachable!("message chunk should be exhaustive"),
    }
    Ok(true)
}
