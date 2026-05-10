use super::*;

#[path = "mouse_helpers.rs"]
mod helpers;

use helpers::contains_mouse;

impl TuiModel {
    pub fn handle_resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.show_sidebar_override = None;
        self.clear_chat_drag_selection();
        self.clear_work_context_drag_selection();
        self.clear_task_view_drag_selection();
        self.clear_workspace_drag();
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        if matches!(
            mouse.kind,
            MouseEventKind::Down(_) | MouseEventKind::Up(_) | MouseEventKind::Drag(_)
        ) {
            self.clear_dismissable_input_notice();
        }

        if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
            && self.should_show_operator_profile_onboarding()
        {
            let overlay_area = if let Some((modal::ModalKind::OperatorProfileOnboarding, area)) =
                self.current_modal_area()
            {
                area
            } else {
                self.open_operator_profile_onboarding_modal();
                self.current_modal_area()
                    .map(|(_, area)| area)
                    .unwrap_or_else(|| Rect::new(0, 0, self.width, self.height))
            };
            let index = {
                let view = self.operator_profile_onboarding_view();
                widgets::operator_profile_onboarding::index_at_position(
                    overlay_area,
                    &view,
                    Position::new(mouse.column, mouse.row),
                )
            };
            if let Some(index) = index {
                self.modal_navigate_to(index);
                self.handle_modal_enter(modal::ModalKind::OperatorProfileOnboarding);
                self.input.set_mode(input::InputMode::Insert);
                return;
            }
        }

        if self.modal.top().is_some() {
            self.handle_modal_mouse(mouse);
            self.input.set_mode(input::InputMode::Insert);
            return;
        }

        let header_area = Rect::new(0, 0, self.width, 3);
        if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            if let Some(widgets::header::HeaderHitTarget::NotificationBell) =
                widgets::header::hit_test(
                    header_area,
                    self.approval.pending_approvals().len(),
                    self.notifications.unread_count(),
                    Position::new(mouse.column, mouse.row),
                )
            {
                self.toggle_notifications_modal();
                self.input.set_mode(input::InputMode::Insert);
                return;
            }
            if let Some(widgets::header::HeaderHitTarget::ApprovalBadge) = widgets::header::hit_test(
                header_area,
                self.approval.pending_approvals().len(),
                self.notifications.unread_count(),
                Position::new(mouse.column, mouse.row),
            ) {
                self.toggle_approval_center();
                self.input.set_mode(input::InputMode::Insert);
                return;
            }
        }

        let status_area = Rect::new(0, self.height.saturating_sub(1), self.width, 1);
        if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            if widgets::footer::status_bar_hit_test(
                status_area,
                self.connected,
                self.last_error.is_some(),
                self.voice_recording,
                self.voice_player.is_some(),
                self.queued_prompts.len(),
                Position::new(mouse.column, mouse.row),
            ) == Some(widgets::footer::StatusBarHitTarget::QueuedPrompts)
            {
                self.open_queued_prompts_modal();
                self.input.set_mode(input::InputMode::Insert);
                return;
            }
        }

        let layout = self.pane_layout();
        let chat_area = layout.chat;
        let conversation_chat_area = self.conversation_content_area().unwrap_or(chat_area);
        let sidebar_area = layout.sidebar.unwrap_or_default();
        let cursor_in_concierge =
            layout.concierge.height > 0 && contains_mouse(layout.concierge, mouse);
        let cursor_in_sidebar = layout
            .sidebar
            .is_some_and(|rect| contains_mouse(rect, mouse));
        let cursor_in_chat = contains_mouse(chat_area, mouse);
        let cursor_in_input = contains_mouse(layout.input, mouse);
        let cursor_in_scroll_surface = cursor_in_chat
            || (cursor_in_concierge && matches!(self.main_pane_view, MainPaneView::Conversation));
        let concierge_area = layout.concierge;

        match mouse.kind {
            MouseEventKind::ScrollUp => {
                if cursor_in_scroll_surface {
                    if matches!(self.main_pane_view, MainPaneView::Workspace) {
                        self.step_workspace_board_scroll_at(
                            Position::new(mouse.column, mouse.row),
                            -1,
                        );
                        self.input.set_mode(input::InputMode::Insert);
                        return;
                    }
                    if matches!(
                        self.main_pane_view,
                        MainPaneView::Task(_)
                            | MainPaneView::WorkContext
                            | MainPaneView::FilePreview(_)
                    ) {
                        if matches!(
                            self.main_pane_view,
                            MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
                        ) {
                            let pane = widgets::goal_workspace::pane_at(
                                chat_area,
                                Position::new(mouse.column, mouse.row),
                            )
                            .unwrap_or(self.goal_workspace.focused_pane());
                            self.step_goal_workspace_pane_scroll(pane, -3);
                        } else {
                            self.step_detail_view_scroll(-3);
                        }
                        if self.work_context_drag_anchor.is_some()
                            && matches!(self.main_pane_view, MainPaneView::WorkContext)
                        {
                            let pos = Position::new(mouse.column, mouse.row);
                            self.work_context_drag_current = Some(pos);
                            self.work_context_drag_current_point =
                                widgets::work_context_view::selection_point_from_mouse(
                                    chat_area,
                                    &self.tasks,
                                    self.chat.active_thread_id(),
                                    self.sidebar.active_tab(),
                                    self.sidebar.selected_item(),
                                    &self.theme,
                                    self.task_view_scroll,
                                    pos,
                                );
                        } else if self.task_view_drag_anchor.is_some()
                            && matches!(
                                self.main_pane_view,
                                MainPaneView::Task(_) | MainPaneView::FilePreview(_)
                            )
                        {
                            let pos = Position::new(mouse.column, mouse.row);
                            self.task_view_drag_current = Some(pos);
                            self.task_view_drag_current_point = match &self.main_pane_view {
                                MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun {
                                    goal_run_id,
                                    ..
                                }) => widgets::goal_workspace::selection_point_from_mouse(
                                    chat_area,
                                    &self.tasks,
                                    goal_run_id,
                                    &self.goal_workspace,
                                    pos,
                                ),
                                MainPaneView::Task(target) => {
                                    widgets::task_view::selection_point_from_mouse(
                                        chat_area,
                                        &self.tasks,
                                        target,
                                        &self.theme,
                                        self.task_view_scroll,
                                        self.task_show_live_todos,
                                        self.task_show_timeline,
                                        self.task_show_files,
                                        pos,
                                    )
                                }
                                MainPaneView::FilePreview(target) => {
                                    widgets::file_preview::selection_point_from_mouse(
                                        chat_area,
                                        &self.tasks,
                                        target,
                                        &self.theme,
                                        self.task_view_scroll,
                                        pos,
                                    )
                                }
                                _ => None,
                            };
                        }
                    } else {
                        self.chat.reduce(chat::ChatAction::ScrollChat(3));
                        if self.chat_drag_anchor.is_some() {
                            let selection_area =
                                if matches!(self.main_pane_view, MainPaneView::Conversation) {
                                    conversation_chat_area
                                } else {
                                    chat_area
                                };
                            self.chat_selection_snapshot = widgets::chat::build_selection_snapshot(
                                selection_area,
                                &self.chat,
                                &self.theme,
                                self.tick_counter,
                                self.retry_wait_start_selected,
                            );
                            let pos = Position::new(mouse.column, mouse.row);
                            self.chat_drag_current = Some(pos);
                            self.chat_drag_current_point =
                                self.chat_selection_snapshot.as_ref().and_then(|snapshot| {
                                    widgets::chat::selection_point_from_cached_snapshot(
                                        snapshot, pos,
                                    )
                                });
                        }
                    }
                } else if cursor_in_sidebar {
                    if self.sidebar_uses_goal_sidebar() {
                        self.navigate_goal_sidebar(-3);
                    } else {
                        self.sidebar.navigate(-3, self.sidebar_item_count());
                    }
                } else if cursor_in_input {
                    for _ in 0..3 {
                        self.input.reduce(input::InputAction::MoveCursorUp);
                    }
                }
            }
            MouseEventKind::ScrollDown => {
                if cursor_in_scroll_surface {
                    if matches!(self.main_pane_view, MainPaneView::Workspace) {
                        self.step_workspace_board_scroll_at(
                            Position::new(mouse.column, mouse.row),
                            1,
                        );
                        self.input.set_mode(input::InputMode::Insert);
                        return;
                    }
                    if matches!(
                        self.main_pane_view,
                        MainPaneView::Task(_)
                            | MainPaneView::WorkContext
                            | MainPaneView::FilePreview(_)
                    ) {
                        if matches!(
                            self.main_pane_view,
                            MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
                        ) {
                            let pane = widgets::goal_workspace::pane_at(
                                chat_area,
                                Position::new(mouse.column, mouse.row),
                            )
                            .unwrap_or(self.goal_workspace.focused_pane());
                            self.step_goal_workspace_pane_scroll(pane, 3);
                        } else {
                            self.step_detail_view_scroll(3);
                        }
                        if self.work_context_drag_anchor.is_some()
                            && matches!(self.main_pane_view, MainPaneView::WorkContext)
                        {
                            let pos = Position::new(mouse.column, mouse.row);
                            self.work_context_drag_current = Some(pos);
                            self.work_context_drag_current_point =
                                widgets::work_context_view::selection_point_from_mouse(
                                    chat_area,
                                    &self.tasks,
                                    self.chat.active_thread_id(),
                                    self.sidebar.active_tab(),
                                    self.sidebar.selected_item(),
                                    &self.theme,
                                    self.task_view_scroll,
                                    pos,
                                );
                        } else if self.task_view_drag_anchor.is_some()
                            && matches!(
                                self.main_pane_view,
                                MainPaneView::Task(_) | MainPaneView::FilePreview(_)
                            )
                        {
                            let pos = Position::new(mouse.column, mouse.row);
                            self.task_view_drag_current = Some(pos);
                            self.task_view_drag_current_point = match &self.main_pane_view {
                                MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun {
                                    goal_run_id,
                                    ..
                                }) => widgets::goal_workspace::selection_point_from_mouse(
                                    chat_area,
                                    &self.tasks,
                                    goal_run_id,
                                    &self.goal_workspace,
                                    pos,
                                ),
                                MainPaneView::Task(target) => {
                                    widgets::task_view::selection_point_from_mouse(
                                        chat_area,
                                        &self.tasks,
                                        target,
                                        &self.theme,
                                        self.task_view_scroll,
                                        self.task_show_live_todos,
                                        self.task_show_timeline,
                                        self.task_show_files,
                                        pos,
                                    )
                                }
                                MainPaneView::FilePreview(target) => {
                                    widgets::file_preview::selection_point_from_mouse(
                                        chat_area,
                                        &self.tasks,
                                        target,
                                        &self.theme,
                                        self.task_view_scroll,
                                        pos,
                                    )
                                }
                                _ => None,
                            };
                        }
                    } else {
                        self.chat.reduce(chat::ChatAction::ScrollChat(-3));
                        if self.chat_drag_anchor.is_some() {
                            let selection_area =
                                if matches!(self.main_pane_view, MainPaneView::Conversation) {
                                    conversation_chat_area
                                } else {
                                    chat_area
                                };
                            self.chat_selection_snapshot = widgets::chat::build_selection_snapshot(
                                selection_area,
                                &self.chat,
                                &self.theme,
                                self.tick_counter,
                                self.retry_wait_start_selected,
                            );
                            let pos = Position::new(mouse.column, mouse.row);
                            self.chat_drag_current = Some(pos);
                            self.chat_drag_current_point =
                                self.chat_selection_snapshot.as_ref().and_then(|snapshot| {
                                    widgets::chat::selection_point_from_cached_snapshot(
                                        snapshot, pos,
                                    )
                                });
                        }
                    }
                } else if cursor_in_sidebar {
                    if self.sidebar_uses_goal_sidebar() {
                        self.navigate_goal_sidebar(3);
                    } else {
                        self.sidebar.navigate(3, self.sidebar_item_count());
                    }
                } else if cursor_in_input {
                    for _ in 0..3 {
                        self.input.reduce(input::InputAction::MoveCursorDown);
                    }
                }
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if cursor_in_concierge {
                    if let Some(widgets::concierge::ConciergeHitTarget::Action(index)) =
                        widgets::concierge::hit_test(
                            concierge_area,
                            self.chat.active_actions(),
                            self.concierge.selected_action,
                            Some(
                                self.current_conversation_agent_profile()
                                    .agent_label
                                    .as_str(),
                            ),
                            Position::new(mouse.column, mouse.row),
                        )
                    {
                        self.select_visible_concierge_action(index);
                        self.execute_concierge_action(index);
                    } else if self.chat.active_thread_id() == Some("concierge") {
                        self.focus = FocusArea::Chat;
                    }
                } else if cursor_in_chat {
                    self.focus = FocusArea::Chat;
                    if matches!(self.main_pane_view, MainPaneView::Workspace) {
                        let pos = Position::new(mouse.column, mouse.row);
                        match widgets::workspace_board::hit_test_with_scroll(
                            chat_area,
                            &self.workspace,
                            &self.workspace_expanded_task_ids,
                            &self.workspace_board_scroll,
                            pos,
                        ) {
                            Some(widgets::workspace_board::WorkspaceBoardHitTarget::Toolbar(
                                action,
                            )) => {
                                self.clear_workspace_drag();
                                self.workspace_board_selection = Some(
                                    widgets::workspace_board::WorkspaceBoardHitTarget::Toolbar(
                                        action.clone(),
                                    ),
                                );
                                self.activate_workspace_toolbar_action(action);
                            }
                            Some(widgets::workspace_board::WorkspaceBoardHitTarget::Action {
                                task_id,
                                status,
                                action,
                            }) => {
                                let target =
                                    widgets::workspace_board::WorkspaceBoardHitTarget::Action {
                                        task_id: task_id.clone(),
                                        status: status.clone(),
                                        action: action.clone(),
                                    };
                                self.workspace_board_selection = Some(target.clone());
                                if action
                                    == widgets::workspace_board::WorkspaceBoardAction::ToggleActions
                                {
                                    self.workspace_drag_task = Some(task_id);
                                    self.workspace_drag_status = Some(status);
                                    self.workspace_drag_start_target = Some(target);
                                    self.status_line =
                                        "Drag workspace task to another column".to_string();
                                } else {
                                    self.clear_workspace_drag();
                                    self.activate_workspace_task_action(task_id, status, action);
                                }
                            }
                            Some(widgets::workspace_board::WorkspaceBoardHitTarget::Task {
                                task_id,
                                status,
                            }) => {
                                let target =
                                    widgets::workspace_board::WorkspaceBoardHitTarget::Task {
                                        task_id: task_id.clone(),
                                        status: status.clone(),
                                    };
                                self.workspace_board_selection = Some(target.clone());
                                self.workspace_drag_task = Some(task_id);
                                self.workspace_drag_status = Some(status);
                                self.workspace_drag_start_target = Some(target);
                                self.status_line =
                                    "Drag workspace task to another column".to_string();
                            }
                            _ => {
                                self.clear_workspace_drag();
                            }
                        }
                        self.input.set_mode(input::InputMode::Insert);
                        return;
                    }
                    if matches!(self.main_pane_view, MainPaneView::GoalComposer) {
                        if matches!(
                            widgets::goal_mission_control::hit_test(
                                chat_area,
                                Position::new(mouse.column, mouse.row),
                                self.mission_control_has_thread_target(),
                            ),
                            Some(
                                widgets::goal_mission_control::GoalMissionControlHitTarget::OpenActiveThread
                            )
                        ) {
                            let _ = self.open_mission_control_goal_thread();
                            self.input.set_mode(input::InputMode::Insert);
                            return;
                        }
                    } else if matches!(self.main_pane_view, MainPaneView::Conversation) {
                        if self
                            .conversation_return_to_goal_button_area()
                            .is_some_and(|rect| contains_mouse(rect, mouse))
                        {
                            let _ = self.return_from_mission_control_navigation();
                            self.input.set_mode(input::InputMode::Insert);
                            return;
                        }
                    } else if matches!(self.main_pane_view, MainPaneView::Task(_))
                        && self
                            .task_return_to_workspace_button_area()
                            .is_some_and(|rect| contains_mouse(rect, mouse))
                    {
                        let _ = self.return_from_mission_control_navigation();
                        self.input.set_mode(input::InputMode::Insert);
                        return;
                    }
                    if matches!(self.main_pane_view, MainPaneView::Collaboration) {
                        if let Some(hit) = widgets::collaboration_view::hit_test(
                            chat_area,
                            &self.collaboration,
                            Position::new(mouse.column, mouse.row),
                        ) {
                            match hit {
                                widgets::collaboration_view::CollaborationHitTarget::Row(index) => {
                                    self.collaboration
                                        .reduce(CollaborationAction::SelectRow(index));
                                    self.collaboration.reduce(CollaborationAction::SetFocus(
                                        CollaborationPaneFocus::Navigator,
                                    ));
                                }
                                widgets::collaboration_view::CollaborationHitTarget::DetailAction(index) => {
                                    self.collaboration.reduce(CollaborationAction::SetFocus(
                                        CollaborationPaneFocus::Detail,
                                    ));
                                    let current = self.collaboration.selected_detail_action_index() as i32;
                                    self.collaboration.reduce(CollaborationAction::StepDetailAction(
                                        index as i32 - current,
                                    ));
                                    self.submit_selected_collaboration_vote();
                                }
                            }
                            self.input.set_mode(input::InputMode::Insert);
                            return;
                        }
                    } else if matches!(self.main_pane_view, MainPaneView::Conversation) {
                        if self
                            .conversation_participant_summary_area()
                            .is_some_and(|rect| contains_mouse(rect, mouse))
                        {
                            if let Some((yes_rect, no_rect, always_rect)) = self
                                .active_auto_response_countdown_secs()
                                .and_then(|countdown_secs| {
                                    render_helpers::auto_response_button_bounds(
                                        self.conversation_participant_summary_area()?,
                                        countdown_secs,
                                    )
                                })
                            {
                                let pos = Position::new(mouse.column, mouse.row);
                                let in_yes = pos.y == yes_rect.y
                                    && pos.x >= yes_rect.x
                                    && pos.x < yes_rect.x.saturating_add(yes_rect.width);
                                let in_no = pos.y == no_rect.y
                                    && pos.x >= no_rect.x
                                    && pos.x < no_rect.x.saturating_add(no_rect.width);
                                let in_always = pos.y == always_rect.y
                                    && pos.x >= always_rect.x
                                    && pos.x < always_rect.x.saturating_add(always_rect.width);
                                if in_yes {
                                    self.auto_response_selection = AutoResponseActionSelection::Yes;
                                    let _ = self.execute_active_auto_response_action(
                                        AutoResponseActionSelection::Yes,
                                    );
                                    self.input.set_mode(input::InputMode::Insert);
                                    return;
                                }
                                if in_no {
                                    self.auto_response_selection = AutoResponseActionSelection::No;
                                    let _ = self.execute_active_auto_response_action(
                                        AutoResponseActionSelection::No,
                                    );
                                    self.input.set_mode(input::InputMode::Insert);
                                    return;
                                }
                                if in_always {
                                    self.auto_response_selection =
                                        AutoResponseActionSelection::Always;
                                    let _ = self.execute_active_auto_response_action(
                                        AutoResponseActionSelection::Always,
                                    );
                                    self.input.set_mode(input::InputMode::Insert);
                                    return;
                                }
                            }
                            self.open_thread_participants_modal();
                            self.input.set_mode(input::InputMode::Insert);
                            return;
                        }
                        let pos = Position::new(mouse.column, mouse.row);
                        if pos.x
                            == conversation_chat_area
                                .x
                                .saturating_add(conversation_chat_area.width)
                                .saturating_sub(1)
                        {
                            if let Some(layout) =
                                self.chat_scrollbar_layout_for_area(conversation_chat_area)
                            {
                                let in_scrollbar = pos.y >= layout.scrollbar.y
                                    && pos.y
                                        < layout
                                            .scrollbar
                                            .y
                                            .saturating_add(layout.scrollbar.height);
                                if in_scrollbar {
                                    self.clear_chat_drag_selection();
                                    let default_grab_offset = layout.thumb.height / 2;
                                    let grab_offset = if pos.y >= layout.thumb.y
                                        && pos.y
                                            < layout.thumb.y.saturating_add(layout.thumb.height)
                                    {
                                        pos.y.saturating_sub(layout.thumb.y)
                                    } else {
                                        default_grab_offset
                                            .min(layout.thumb.height.saturating_sub(1))
                                    };
                                    if let Some(target) = self
                                        .chat_scrollbar_scroll_offset_for_pointer(
                                            conversation_chat_area,
                                            pos.y,
                                            grab_offset,
                                        )
                                    {
                                        self.set_chat_scroll_offset(target);
                                    }
                                    self.chat_scrollbar_drag_grab_offset = Some(grab_offset);
                                    self.input.set_mode(input::InputMode::Insert);
                                    return;
                                }
                            }
                        }
                        if let Some(hit) = self.cached_chat_hit_test(conversation_chat_area, pos) {
                            if !matches!(hit, chat::ChatHitTarget::Message(_)) {
                                self.clear_chat_drag_selection();
                                self.handle_chat_click(conversation_chat_area, pos);
                                self.input.set_mode(input::InputMode::Insert);
                                return;
                            }
                        }
                        if self
                            .chat_selection_snapshot
                            .as_ref()
                            .is_none_or(|snapshot| {
                                !widgets::chat::cached_snapshot_matches_area(
                                    snapshot,
                                    conversation_chat_area,
                                )
                            })
                        {
                            self.chat_selection_snapshot = widgets::chat::build_selection_snapshot(
                                conversation_chat_area,
                                &self.chat,
                                &self.theme,
                                self.tick_counter,
                                self.retry_wait_start_selected,
                            );
                        }
                        self.chat_drag_anchor = Some(pos);
                        self.chat_drag_current = Some(pos);
                        let point = self.chat_selection_snapshot.as_ref().and_then(|snapshot| {
                            widgets::chat::selection_point_from_cached_snapshot(snapshot, pos)
                        });
                        self.chat_drag_anchor_point = point;
                        self.chat_drag_current_point = point;
                    } else if let MainPaneView::FilePreview(target) = self.main_pane_view.clone() {
                        let pos = Position::new(mouse.column, mouse.row);
                        if pos.x
                            == chat_area
                                .x
                                .saturating_add(chat_area.width)
                                .saturating_sub(1)
                        {
                            if let Some(layout) = widgets::file_preview::scrollbar_layout(
                                chat_area,
                                &self.tasks,
                                &target,
                                &self.theme,
                                self.task_view_scroll,
                            ) {
                                let in_scrollbar = pos.y >= layout.scrollbar.y
                                    && pos.y
                                        < layout
                                            .scrollbar
                                            .y
                                            .saturating_add(layout.scrollbar.height);
                                if in_scrollbar {
                                    self.clear_task_view_drag_selection();
                                    let default_grab_offset = layout.thumb.height / 2;
                                    let grab_offset = if pos.y >= layout.thumb.y
                                        && pos.y
                                            < layout.thumb.y.saturating_add(layout.thumb.height)
                                    {
                                        pos.y.saturating_sub(layout.thumb.y)
                                    } else {
                                        default_grab_offset
                                            .min(layout.thumb.height.saturating_sub(1))
                                    };
                                    if let Some(target_scroll) =
                                        widgets::file_preview::scrollbar_scroll_offset_for_pointer(
                                            chat_area,
                                            &self.tasks,
                                            &target,
                                            &self.theme,
                                            self.task_view_scroll,
                                            pos.y,
                                            grab_offset,
                                        )
                                    {
                                        self.task_view_scroll = target_scroll
                                            .min(self.current_detail_view_max_scroll());
                                    }
                                    self.file_preview_scrollbar_drag_grab_offset =
                                        Some(grab_offset);
                                    self.input.set_mode(input::InputMode::Insert);
                                    return;
                                }
                            }
                        }
                        if let Some(widgets::file_preview::FilePreviewHitTarget::ClosePreview) =
                            widgets::file_preview::hit_test(
                                chat_area,
                                &self.tasks,
                                &target,
                                self.task_view_scroll,
                                pos,
                                &self.theme,
                            )
                        {
                            let _ = self.dismiss_active_main_pane(FocusArea::Chat);
                            self.status_line = "Closed preview".to_string();
                            return;
                        }
                        self.task_view_drag_anchor = Some(pos);
                        self.task_view_drag_current = Some(pos);
                        let point = widgets::file_preview::selection_point_from_mouse(
                            chat_area,
                            &self.tasks,
                            &target,
                            &self.theme,
                            self.task_view_scroll,
                            pos,
                        );
                        self.task_view_drag_anchor_point = point;
                        self.task_view_drag_current_point = point;
                    } else if matches!(self.main_pane_view, MainPaneView::WorkContext) {
                        if let Some(
                            widgets::work_context_view::WorkContextHitTarget::ClosePreview,
                        ) = widgets::work_context_view::hit_test(
                            chat_area,
                            &self.tasks,
                            self.chat.active_thread_id(),
                            self.sidebar.active_tab(),
                            self.sidebar.selected_item(),
                            self.task_view_scroll,
                            Position::new(mouse.column, mouse.row),
                            &self.theme,
                        ) {
                            let _ = self.dismiss_active_main_pane(FocusArea::Chat);
                            self.status_line = "Closed preview".to_string();
                            return;
                        }
                        let pos = Position::new(mouse.column, mouse.row);
                        self.work_context_drag_anchor = Some(pos);
                        self.work_context_drag_current = Some(pos);
                        let point = widgets::work_context_view::selection_point_from_mouse(
                            chat_area,
                            &self.tasks,
                            self.chat.active_thread_id(),
                            self.sidebar.active_tab(),
                            self.sidebar.selected_item(),
                            &self.theme,
                            self.task_view_scroll,
                            pos,
                        );
                        self.work_context_drag_anchor_point = point;
                        self.work_context_drag_current_point = point;
                    } else if let MainPaneView::Task(target) = self.main_pane_view.clone() {
                        let pos = Position::new(mouse.column, mouse.row);
                        let task_area = self.task_content_area().unwrap_or(chat_area);
                        if matches!(target, sidebar::SidebarItemTarget::GoalRun { .. }) {
                            self.focus = FocusArea::Chat;
                            self.clear_chat_drag_selection();
                            self.clear_work_context_drag_selection();
                            self.clear_task_view_drag_selection();
                            self.task_view_drag_anchor = Some(pos);
                            self.task_view_drag_current = Some(pos);
                            let point = match &target {
                                sidebar::SidebarItemTarget::GoalRun { goal_run_id, .. } => {
                                    widgets::goal_workspace::selection_point_from_mouse(
                                        task_area,
                                        &self.tasks,
                                        goal_run_id,
                                        &self.goal_workspace,
                                        pos,
                                    )
                                }
                                _ => None,
                            };
                            self.task_view_drag_anchor_point = point;
                            self.task_view_drag_current_point = point;
                            return;
                        }
                        self.task_view_drag_anchor = Some(pos);
                        self.task_view_drag_current = Some(pos);
                        let point = widgets::task_view::selection_point_from_mouse(
                            task_area,
                            &self.tasks,
                            &target,
                            &self.theme,
                            self.task_view_scroll,
                            self.task_show_live_todos,
                            self.task_show_timeline,
                            self.task_show_files,
                            pos,
                        );
                        self.task_view_drag_anchor_point = point;
                        self.task_view_drag_current_point = point;
                    }
                } else if cursor_in_sidebar {
                    self.clear_chat_drag_selection();
                    self.clear_work_context_drag_selection();
                    self.clear_task_view_drag_selection();
                    self.focus = FocusArea::Sidebar;
                    if self.sidebar_uses_goal_sidebar() {
                        match self.goal_sidebar_hit_test(sidebar_area, mouse) {
                            Some(widgets::goal_sidebar::GoalSidebarHitTarget::Tab(tab)) => {
                                self.activate_goal_sidebar_tab(tab);
                            }
                            Some(widgets::goal_sidebar::GoalSidebarHitTarget::Step(index)) => {
                                self.select_goal_sidebar_row(index);
                                let _ = self.handle_goal_sidebar_enter();
                            }
                            Some(widgets::goal_sidebar::GoalSidebarHitTarget::Checkpoint(
                                index,
                            )) => {
                                self.select_goal_sidebar_row(index);
                                let _ = self.handle_goal_sidebar_enter();
                            }
                            Some(widgets::goal_sidebar::GoalSidebarHitTarget::Task(index)) => {
                                self.select_goal_sidebar_row(index);
                                let _ = self.handle_goal_sidebar_enter();
                            }
                            Some(widgets::goal_sidebar::GoalSidebarHitTarget::File(index)) => {
                                self.select_goal_sidebar_row(index);
                                let _ = self.handle_goal_sidebar_enter();
                            }
                            None => {}
                        }
                    } else {
                        let sidebar_snapshot =
                            self.current_sidebar_snapshot().cloned().unwrap_or_else(|| {
                                let snapshot = widgets::sidebar::build_cached_snapshot(
                                    sidebar_area,
                                    &self.chat,
                                    &self.sidebar,
                                    &self.tasks,
                                    self.chat.active_thread_id(),
                                );
                                self.sidebar_snapshot = Some(snapshot.clone());
                                snapshot
                            });
                        match widgets::sidebar::hit_test_cached(
                            sidebar_area,
                            &self.sidebar,
                            &sidebar_snapshot,
                            Position::new(mouse.column, mouse.row),
                        ) {
                            Some(widgets::sidebar::SidebarHitTarget::Tab(tab)) => {
                                self.activate_sidebar_tab(tab);
                            }
                            Some(widgets::sidebar::SidebarHitTarget::File(path)) => {
                                if self.chat.active_thread_id().is_some() {
                                    let index =
                                        self.filtered_sidebar_file_index(&path).unwrap_or(0);
                                    self.sidebar.select(index, self.sidebar_item_count());
                                    self.handle_sidebar_enter();
                                }
                            }
                            Some(widgets::sidebar::SidebarHitTarget::Todo(index)) => {
                                self.sidebar.select(index, self.sidebar_item_count());
                                self.handle_sidebar_enter();
                            }
                            Some(widgets::sidebar::SidebarHitTarget::Spawned(index)) => {
                                self.sidebar.select(index, self.sidebar_item_count());
                                self.handle_sidebar_enter();
                            }
                            Some(widgets::sidebar::SidebarHitTarget::Pinned(index)) => {
                                self.sidebar.select(index, self.sidebar_item_count());
                                self.handle_sidebar_enter();
                            }
                            None => {}
                        }
                    }
                } else if cursor_in_input {
                    self.clear_chat_drag_selection();
                    self.clear_work_context_drag_selection();
                    self.clear_task_view_drag_selection();
                    self.focus = FocusArea::Input;
                    let inner_row =
                        mouse.row.saturating_sub(layout.input.y + 1) as usize;
                    if inner_row < self.attachments.len() {
                        let att = self.attachments.remove(inner_row);
                        self.status_line = format!("Removed: {}", att.filename);
                    } else if let Some(offset) =
                        self.input_offset_from_mouse(layout.input.y, mouse)
                    {
                        self.input
                            .reduce(input::InputAction::MoveCursorToPos(offset));
                    }
                }
                self.input.set_mode(input::InputMode::Insert);
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if let Some(grab_offset) = self.chat_scrollbar_drag_grab_offset {
                    if matches!(self.main_pane_view, MainPaneView::Conversation) {
                        if let Some(target) = self.chat_scrollbar_scroll_offset_for_pointer(
                            conversation_chat_area,
                            mouse.row,
                            grab_offset,
                        ) {
                            self.set_chat_scroll_offset(target);
                        }
                    }
                } else if let Some(grab_offset) = self.file_preview_scrollbar_drag_grab_offset {
                    if let MainPaneView::FilePreview(target) = &self.main_pane_view {
                        if let Some(target_scroll) =
                            widgets::file_preview::scrollbar_scroll_offset_for_pointer(
                                chat_area,
                                &self.tasks,
                                target,
                                &self.theme,
                                self.task_view_scroll,
                                mouse.row,
                                grab_offset,
                            )
                        {
                            self.task_view_scroll =
                                target_scroll.min(self.current_detail_view_max_scroll());
                        }
                    }
                } else if self.chat_drag_anchor.is_some()
                    && matches!(self.main_pane_view, MainPaneView::Conversation)
                {
                    let mut scrolled = false;
                    if mouse.row <= conversation_chat_area.y.saturating_add(1) {
                        self.chat.reduce(chat::ChatAction::ScrollChat(1));
                        scrolled = true;
                    } else if mouse.row
                        >= conversation_chat_area
                            .y
                            .saturating_add(conversation_chat_area.height)
                            .saturating_sub(2)
                    {
                        self.chat.reduce(chat::ChatAction::ScrollChat(-1));
                        scrolled = true;
                    }
                    if scrolled || self.chat_selection_snapshot.is_none() {
                        self.chat_selection_snapshot = widgets::chat::build_selection_snapshot(
                            conversation_chat_area,
                            &self.chat,
                            &self.theme,
                            self.tick_counter,
                            self.retry_wait_start_selected,
                        );
                    }
                    let pos = Position::new(mouse.column, mouse.row);
                    self.chat_drag_current = Some(pos);
                    self.chat_drag_current_point =
                        self.chat_selection_snapshot.as_ref().and_then(|snapshot| {
                            widgets::chat::selection_point_from_cached_snapshot(snapshot, pos)
                        });
                } else if self.work_context_drag_anchor.is_some()
                    && matches!(self.main_pane_view, MainPaneView::WorkContext)
                {
                    if mouse.row <= chat_area.y.saturating_add(1) {
                        self.step_detail_view_scroll(-1);
                    } else if mouse.row
                        >= chat_area
                            .y
                            .saturating_add(chat_area.height)
                            .saturating_sub(2)
                    {
                        self.step_detail_view_scroll(1);
                    }
                    let pos = Position::new(mouse.column, mouse.row);
                    self.work_context_drag_current = Some(pos);
                    self.work_context_drag_current_point =
                        widgets::work_context_view::selection_point_from_mouse(
                            chat_area,
                            &self.tasks,
                            self.chat.active_thread_id(),
                            self.sidebar.active_tab(),
                            self.sidebar.selected_item(),
                            &self.theme,
                            self.task_view_scroll,
                            pos,
                        );
                } else if self.task_view_drag_anchor.is_some()
                    && matches!(
                        self.main_pane_view,
                        MainPaneView::Task(_) | MainPaneView::FilePreview(_)
                    )
                {
                    if mouse.row <= chat_area.y.saturating_add(1) {
                        self.step_detail_view_scroll(-1);
                    } else if mouse.row
                        >= chat_area
                            .y
                            .saturating_add(chat_area.height)
                            .saturating_sub(2)
                    {
                        self.step_detail_view_scroll(1);
                    }
                    let pos = Position::new(mouse.column, mouse.row);
                    let task_area = self.task_content_area().unwrap_or(chat_area);
                    self.task_view_drag_current = Some(pos);
                    self.task_view_drag_current_point = match &self.main_pane_view {
                        MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun {
                            goal_run_id,
                            ..
                        }) => widgets::goal_workspace::selection_point_from_mouse(
                            task_area,
                            &self.tasks,
                            goal_run_id,
                            &self.goal_workspace,
                            pos,
                        ),
                        MainPaneView::Task(target) => {
                            widgets::task_view::selection_point_from_mouse(
                                task_area,
                                &self.tasks,
                                target,
                                &self.theme,
                                self.task_view_scroll,
                                self.task_show_live_todos,
                                self.task_show_timeline,
                                self.task_show_files,
                                pos,
                            )
                        }
                        MainPaneView::FilePreview(target) => {
                            widgets::file_preview::selection_point_from_mouse(
                                chat_area,
                                &self.tasks,
                                target,
                                &self.theme,
                                self.task_view_scroll,
                                pos,
                            )
                        }
                        _ => None,
                    };
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                if self.chat_scrollbar_drag_grab_offset.take().is_some() {
                    return;
                }
                if self
                    .file_preview_scrollbar_drag_grab_offset
                    .take()
                    .is_some()
                {
                    return;
                }
                if let Some(task_id) = self.workspace_drag_task.take() {
                    let start_status = self.workspace_drag_status.take();
                    let start_target = self.workspace_drag_start_target.take();
                    if cursor_in_chat && matches!(self.main_pane_view, MainPaneView::Workspace) {
                        let pos = Position::new(mouse.column, mouse.row);
                        let drop_target = widgets::workspace_board::hit_test_with_scroll(
                            chat_area,
                            &self.workspace,
                            &self.workspace_expanded_task_ids,
                            &self.workspace_board_scroll,
                            pos,
                        );
                        if start_target == drop_target {
                            if let Some(
                                widgets::workspace_board::WorkspaceBoardHitTarget::Action {
                                    task_id,
                                    status,
                                    action:
                                        widgets::workspace_board::WorkspaceBoardAction::ToggleActions,
                                },
                            ) = drop_target
                            {
                                self.activate_workspace_task_action(
                                    task_id,
                                    status,
                                    widgets::workspace_board::WorkspaceBoardAction::ToggleActions,
                                );
                            }
                            return;
                        }
                        let (drop_status, target_task_id) = match drop_target {
                            Some(widgets::workspace_board::WorkspaceBoardHitTarget::Column {
                                status,
                            }) => (Some(status), None),
                            Some(widgets::workspace_board::WorkspaceBoardHitTarget::Task {
                                status,
                                task_id,
                            }) => (Some(status), Some(task_id)),
                            Some(widgets::workspace_board::WorkspaceBoardHitTarget::Action {
                                status,
                                task_id,
                                ..
                            }) => (Some(status), Some(task_id)),
                            Some(widgets::workspace_board::WorkspaceBoardHitTarget::Toolbar(_)) => {
                                (None, None)
                            }
                            None => (None, None),
                        };
                        if let Some(status) = drop_status {
                            let target_task_id = target_task_id.as_deref();
                            if Some(&status) == start_status.as_ref()
                                && (target_task_id.is_none()
                                    || self.workspace.drop_targets_same_task(
                                        &task_id,
                                        &status,
                                        target_task_id,
                                    ))
                            {
                                return;
                            }
                            let should_auto_run = start_status.as_ref()
                                == Some(&zorai_protocol::WorkspaceTaskStatus::Todo)
                                && status == zorai_protocol::WorkspaceTaskStatus::InProgress
                                && !self.workspace.drop_to_in_progress_run_blocked(
                                    &task_id,
                                    start_status.as_ref(),
                                    &status,
                                );
                            let moved_without_auto_run = start_status.as_ref()
                                == Some(&zorai_protocol::WorkspaceTaskStatus::Todo)
                                && status == zorai_protocol::WorkspaceTaskStatus::InProgress
                                && !should_auto_run;
                            if moved_without_auto_run {
                                self.status_line =
                                    "Moving workspace task without running; assign before running"
                                        .to_string();
                            }
                            let sort_order = self.workspace.sort_order_for_drop(
                                &task_id,
                                &status,
                                target_task_id,
                            );
                            self.send_daemon_command(DaemonCommand::MoveWorkspaceTask(
                                zorai_protocol::WorkspaceTaskMove {
                                    task_id: task_id.clone(),
                                    status: status.clone(),
                                    sort_order: Some(sort_order),
                                },
                            ));
                            if should_auto_run {
                                self.send_daemon_command(DaemonCommand::RunWorkspaceTask(task_id));
                            }
                            if should_auto_run {
                                self.status_line = "Moving workspace task...".to_string();
                            } else if !moved_without_auto_run {
                                self.status_line = "Moving workspace task...".to_string();
                            }
                        }
                    }
                    return;
                }
                if let Some(anchor) = self.chat_drag_anchor.take() {
                    let current = self.chat_drag_current.take().unwrap_or(anchor);
                    let snapshot = self.chat_selection_snapshot.take();
                    let anchor_point = self.chat_drag_anchor_point.take().or_else(|| {
                        snapshot.as_ref().and_then(|snapshot| {
                            widgets::chat::selection_point_from_cached_snapshot(snapshot, anchor)
                        })
                    });
                    let current_point = self.chat_drag_current_point.take().or_else(|| {
                        snapshot.as_ref().and_then(|snapshot| {
                            widgets::chat::selection_point_from_cached_snapshot(snapshot, current)
                        })
                    });
                    let Some((anchor_point, current_point)) = anchor_point.zip(current_point)
                    else {
                        if cursor_in_chat {
                            self.chat_selection_snapshot = snapshot;
                            self.handle_chat_click(
                                conversation_chat_area,
                                Position::new(mouse.column, mouse.row),
                            );
                        }
                        return;
                    };

                    if anchor_point != current_point {
                        if let Some(text) = snapshot.as_ref().and_then(|snapshot| {
                            widgets::chat::selected_text_from_cached_snapshot(
                                snapshot,
                                anchor_point,
                                current_point,
                            )
                        }) {
                            conversion::copy_to_clipboard(&text);
                            self.status_line = "Copied selection to clipboard".to_string();
                        }
                    } else if cursor_in_chat {
                        self.chat_selection_snapshot = snapshot;
                        self.handle_chat_click(conversation_chat_area, anchor);
                    }
                } else if let Some(anchor) = self.task_view_drag_anchor.take() {
                    let task_area = self.task_content_area().unwrap_or(chat_area);
                    let current = self
                        .task_view_drag_current
                        .take()
                        .unwrap_or(Position::new(mouse.column, mouse.row));
                    let anchor_point = self.task_view_drag_anchor_point.take().or_else(|| {
                        match &self.main_pane_view {
                            MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun {
                                goal_run_id,
                                ..
                            }) => widgets::goal_workspace::selection_point_from_mouse(
                                task_area,
                                &self.tasks,
                                goal_run_id,
                                &self.goal_workspace,
                                anchor,
                            ),
                            MainPaneView::Task(target) => {
                                widgets::task_view::selection_point_from_mouse(
                                    task_area,
                                    &self.tasks,
                                    target,
                                    &self.theme,
                                    self.task_view_scroll,
                                    self.task_show_live_todos,
                                    self.task_show_timeline,
                                    self.task_show_files,
                                    anchor,
                                )
                            }
                            MainPaneView::FilePreview(target) => {
                                widgets::file_preview::selection_point_from_mouse(
                                    chat_area,
                                    &self.tasks,
                                    target,
                                    &self.theme,
                                    self.task_view_scroll,
                                    anchor,
                                )
                            }
                            _ => None,
                        }
                    });
                    let current_point = self.task_view_drag_current_point.take().or_else(|| {
                        match &self.main_pane_view {
                            MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun {
                                goal_run_id,
                                ..
                            }) => widgets::goal_workspace::selection_point_from_mouse(
                                task_area,
                                &self.tasks,
                                goal_run_id,
                                &self.goal_workspace,
                                current,
                            ),
                            MainPaneView::Task(target) => {
                                widgets::task_view::selection_point_from_mouse(
                                    task_area,
                                    &self.tasks,
                                    target,
                                    &self.theme,
                                    self.task_view_scroll,
                                    self.task_show_live_todos,
                                    self.task_show_timeline,
                                    self.task_show_files,
                                    current,
                                )
                            }
                            MainPaneView::FilePreview(target) => {
                                widgets::file_preview::selection_point_from_mouse(
                                    chat_area,
                                    &self.tasks,
                                    target,
                                    &self.theme,
                                    self.task_view_scroll,
                                    current,
                                )
                            }
                            _ => None,
                        }
                    });
                    let Some((anchor_point, current_point)) = anchor_point.zip(current_point)
                    else {
                        if cursor_in_chat {
                            self.handle_task_view_click(task_area, anchor);
                        }
                        return;
                    };

                    if anchor_point != current_point {
                        match &self.main_pane_view {
                            MainPaneView::Task(target) => {
                                let text = match target {
                                    sidebar::SidebarItemTarget::GoalRun { goal_run_id, .. } => {
                                        widgets::goal_workspace::selected_text(
                                            task_area,
                                            &self.tasks,
                                            goal_run_id,
                                            &self.goal_workspace,
                                            anchor_point,
                                            current_point,
                                        )
                                    }
                                    _ => widgets::task_view::selected_text(
                                        task_area,
                                        &self.tasks,
                                        target,
                                        &self.theme,
                                        self.task_view_scroll,
                                        self.task_show_live_todos,
                                        self.task_show_timeline,
                                        self.task_show_files,
                                        anchor_point,
                                        current_point,
                                    ),
                                };
                                if let Some(text) = text {
                                    conversion::copy_to_clipboard(&text);
                                    self.status_line = "Copied selection to clipboard".to_string();
                                }
                            }
                            MainPaneView::FilePreview(target) => {
                                if let Some(text) = widgets::file_preview::selected_text(
                                    chat_area,
                                    &self.tasks,
                                    target,
                                    &self.theme,
                                    self.task_view_scroll,
                                    anchor_point,
                                    current_point,
                                ) {
                                    conversion::copy_to_clipboard(&text);
                                    self.status_line = "Copied selection to clipboard".to_string();
                                }
                            }
                            _ => {}
                        }
                    } else if cursor_in_chat {
                        self.handle_task_view_click(task_area, anchor);
                    }
                } else if let Some(anchor) = self.work_context_drag_anchor.take() {
                    let current = self
                        .work_context_drag_current
                        .take()
                        .unwrap_or(Position::new(mouse.column, mouse.row));
                    let anchor_point = self.work_context_drag_anchor_point.take().or_else(|| {
                        widgets::work_context_view::selection_point_from_mouse(
                            chat_area,
                            &self.tasks,
                            self.chat.active_thread_id(),
                            self.sidebar.active_tab(),
                            self.sidebar.selected_item(),
                            &self.theme,
                            self.task_view_scroll,
                            anchor,
                        )
                    });
                    let current_point = self.work_context_drag_current_point.take().or_else(|| {
                        widgets::work_context_view::selection_point_from_mouse(
                            chat_area,
                            &self.tasks,
                            self.chat.active_thread_id(),
                            self.sidebar.active_tab(),
                            self.sidebar.selected_item(),
                            &self.theme,
                            self.task_view_scroll,
                            current,
                        )
                    });
                    let Some((anchor_point, current_point)) = anchor_point.zip(current_point)
                    else {
                        return;
                    };

                    if anchor_point != current_point {
                        if let Some(text) = widgets::work_context_view::selected_text(
                            chat_area,
                            &self.tasks,
                            self.chat.active_thread_id(),
                            self.sidebar.active_tab(),
                            self.sidebar.selected_item(),
                            &self.theme,
                            self.task_view_scroll,
                            anchor_point,
                            current_point,
                        ) {
                            conversion::copy_to_clipboard(&text);
                            self.status_line = "Copied selection to clipboard".to_string();
                        }
                    }
                }
            }
            MouseEventKind::Down(MouseButton::Right) => {
                if let Ok(text) = arboard::Clipboard::new().and_then(|mut cb| cb.get_text()) {
                    if !text.is_empty() {
                        self.handle_paste(text);
                    }
                }
            }
            _ => {}
        }
    }
}
