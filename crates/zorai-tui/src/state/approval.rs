#![allow(dead_code)]


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    /// Parse from string, defaulting to Medium for unknown values.
    pub fn from_str_lossy(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => Self::Low,
            "medium" => Self::Medium,
            "high" => Self::High,
            "critical" => Self::Critical,
            _ => Self::Medium,
        }
    }

    /// Heuristic: classify risk from command text.
    pub fn classify_command(command: &str) -> Self {
        let cmd = command.to_lowercase();

        if cmd.contains("rm -rf /")
            || cmd.contains("rm -rf ~/")
            || cmd.contains("mkfs")
            || cmd.contains("dd if=")
        {
            return Self::Critical;
        }

        if cmd.contains("git push --force")
            || cmd.contains("git push -f")
            || (cmd.contains("curl") && cmd.contains("| bash"))
            || cmd.contains("docker system prune")
            || cmd.contains("kubectl delete")
            || cmd.contains("systemctl")
            || cmd.contains("npm publish")
            || cmd.contains("cargo publish")
        {
            return Self::High;
        }

        if cmd.contains("rm -rf")
            || cmd.contains("git reset --hard")
            || cmd.contains("drop table")
            || cmd.contains("drop database")
        {
            return Self::Medium;
        }

        Self::Low
    }
}


#[derive(Debug, Clone)]
pub struct PendingApproval {
    pub approval_id: String,
    pub task_id: String,
    pub task_title: Option<String>,
    pub thread_id: Option<String>,
    pub thread_title: Option<String>,
    pub workspace_id: Option<String>,
    pub rationale: Option<String>,
    pub reasons: Vec<String>,
    /// The command text extracted (heuristically) from blocked_reason.
    pub command: String,
    pub risk_level: RiskLevel,
    pub blast_radius: String,
    pub received_at: u64,
    pub seen_at: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedApprovalRule {
    pub id: String,
    pub command: String,
    pub created_at: u64,
    pub last_used_at: Option<u64>,
    pub use_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalFilter {
    AllPending,
    CurrentThread,
    CurrentWorkspace,
    SavedRules,
}


#[derive(Debug, Clone)]
pub enum ApprovalAction {
    ApprovalRequired(PendingApproval),
    SelectApproval(String),
    SelectRule(String),
    SetFilter(ApprovalFilter),
    Resolve {
        approval_id: String,
        decision: String,
    },
    SetRules(Vec<SavedApprovalRule>),
    RemoveRule(String),
    ClearResolved(String),
}


pub struct ApprovalState {
    pending_approvals: Vec<PendingApproval>,
    saved_rules: Vec<SavedApprovalRule>,
    selected_approval_id: Option<String>,
    selected_rule_id: Option<String>,
    filter: ApprovalFilter,
}

impl ApprovalState {
    pub fn new() -> Self {
        Self {
            pending_approvals: Vec::new(),
            saved_rules: Vec::new(),
            selected_approval_id: None,
            selected_rule_id: None,
            filter: ApprovalFilter::AllPending,
        }
    }

    pub fn pending_approvals(&self) -> &[PendingApproval] {
        &self.pending_approvals
    }

    pub fn selected_approval_id(&self) -> Option<&str> {
        self.selected_approval_id.as_deref()
    }

    pub fn filter(&self) -> ApprovalFilter {
        self.filter
    }

    pub fn saved_rules(&self) -> &[SavedApprovalRule] {
        &self.saved_rules
    }

    pub fn selected_approval(&self) -> Option<&PendingApproval> {
        self.selected_approval_id
            .as_deref()
            .and_then(|approval_id| self.approval_by_id(approval_id))
            .or_else(|| self.current_approval())
    }

    pub fn selected_visible_approval<'a>(
        &'a self,
        current_thread_id: Option<&str>,
        current_workspace_id: Option<&str>,
    ) -> Option<&'a PendingApproval> {
        let visible = self.visible_approvals(current_thread_id, current_workspace_id);
        self.selected_approval_id
            .as_deref()
            .and_then(|approval_id| {
                visible
                    .iter()
                    .find(|approval| approval.approval_id == approval_id)
                    .copied()
            })
            .or_else(|| visible.first().copied())
    }

    pub fn approval_by_id(&self, approval_id: &str) -> Option<&PendingApproval> {
        self.pending_approvals
            .iter()
            .find(|approval| approval.approval_id == approval_id)
    }

    pub fn visible_approvals<'a>(
        &'a self,
        current_thread_id: Option<&str>,
        current_workspace_id: Option<&str>,
    ) -> Vec<&'a PendingApproval> {
        self.pending_approvals
            .iter()
            .filter(|approval| match self.filter {
                ApprovalFilter::AllPending => true,
                ApprovalFilter::CurrentThread => approval.thread_id.as_deref() == current_thread_id,
                ApprovalFilter::CurrentWorkspace => {
                    approval.workspace_id.as_deref() == current_workspace_id
                }
                ApprovalFilter::SavedRules => false,
            })
            .collect()
    }

    /// The first pending approval (the one currently shown to the user).
    pub fn current_approval(&self) -> Option<&PendingApproval> {
        self.pending_approvals.first()
    }

    pub fn selected_rule_id(&self) -> Option<&str> {
        self.selected_rule_id.as_deref()
    }

    pub fn selected_rule(&self) -> Option<&SavedApprovalRule> {
        self.selected_rule_id
            .as_deref()
            .and_then(|rule_id| self.saved_rules.iter().find(|rule| rule.id == rule_id))
            .or_else(|| self.saved_rules.first())
    }

    pub fn reduce(&mut self, action: ApprovalAction) {
        match action {
            ApprovalAction::ApprovalRequired(approval) => {
                if let Some(existing) = self
                    .pending_approvals
                    .iter_mut()
                    .find(|existing| existing.approval_id == approval.approval_id)
                {
                    *existing = approval;
                } else {
                    if self.selected_approval_id.is_none() {
                        self.selected_approval_id = Some(approval.approval_id.clone());
                    }
                    self.pending_approvals.push(approval);
                }
            }

            ApprovalAction::SelectApproval(approval_id) => {
                if self
                    .pending_approvals
                    .iter()
                    .any(|approval| approval.approval_id == approval_id)
                {
                    self.selected_approval_id = Some(approval_id);
                    self.selected_rule_id = None;
                }
            }

            ApprovalAction::SelectRule(rule_id) => {
                if self.saved_rules.iter().any(|rule| rule.id == rule_id) {
                    self.selected_rule_id = Some(rule_id);
                    self.selected_approval_id = None;
                }
            }

            ApprovalAction::SetFilter(filter) => {
                self.filter = filter;
                match filter {
                    ApprovalFilter::SavedRules => {
                        self.selected_approval_id = None;
                    }
                    _ => {
                        self.selected_rule_id = None;
                        self.selected_approval_id = None;
                    }
                }
            }

            ApprovalAction::Resolve {
                approval_id,
                decision: _,
            } => {
                self.pending_approvals
                    .retain(|a| a.approval_id != approval_id);
                if self.selected_approval_id.as_deref() == Some(approval_id.as_str()) {
                    self.selected_approval_id = self
                        .pending_approvals
                        .first()
                        .map(|approval| approval.approval_id.clone());
                }
            }

            ApprovalAction::SetRules(mut rules) => {
                rules.sort_by(|left, right| left.command.cmp(&right.command));
                let selected_rule_id = self.selected_rule_id.clone();
                self.saved_rules = rules;
                self.selected_rule_id = selected_rule_id
                    .filter(|rule_id| self.saved_rules.iter().any(|rule| &rule.id == rule_id))
                    .or_else(|| self.saved_rules.first().map(|rule| rule.id.clone()));
            }

            ApprovalAction::RemoveRule(rule_id) => {
                self.saved_rules.retain(|rule| rule.id != rule_id);
                if self.selected_rule_id.as_deref() == Some(rule_id.as_str()) {
                    self.selected_rule_id = self.saved_rules.first().map(|rule| rule.id.clone());
                }
            }

            ApprovalAction::ClearResolved(approval_id) => {
                self.pending_approvals
                    .retain(|a| a.approval_id != approval_id);
                if self.selected_approval_id.as_deref() == Some(approval_id.as_str()) {
                    self.selected_approval_id = self
                        .pending_approvals
                        .first()
                        .map(|approval| approval.approval_id.clone());
                }
            }
        }
    }
}

impl Default for ApprovalState {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
#[path = "approval_tests_parts"]
mod tests {
    

    mod approval_required_adds_to_pending_to_from_str_lossy_defaults_to_medium;
}
