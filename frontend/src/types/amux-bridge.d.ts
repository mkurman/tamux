declare global {
    type AmuxOpenAICodexAuthStatus = {
        available: boolean;
        status?: string;
        authMode?: string;
        accountId?: string;
        expiresAt?: number;
        source?: string;
        error?: string;
    };

    type AmuxOpenAICodexAuthLogin = AmuxOpenAICodexAuthStatus & {
        authUrl?: string;
    };

    type AmuxCodingAgentCheck = {
        label: string;
        path: string;
        exists: boolean;
    };

    type AmuxCodingAgentDiscoveryResult = {
        id: string;
        available: boolean;
        executable: string | null;
        path: string | null;
        version: string | null;
        error?: string | null;
        readiness?: "ready" | "needs-setup" | "missing";
        checks?: AmuxCodingAgentCheck[];
        runtimeNotes?: string[];
        gatewayLabel?: string | null;
        gatewayReachable?: boolean | null;
    };

    type AmuxAITrainingCheck = {
        label: string;
        path: string;
        exists: boolean;
        scope: "system" | "workspace";
    };

    type AmuxAITrainingDiscoveryResult = {
        id: string;
        available: boolean;
        executable: string | null;
        path: string | null;
        version: string | null;
        readiness: "ready" | "needs-setup" | "missing";
        checks: AmuxAITrainingCheck[];
        runtimeNotes?: string[];
        workspacePath?: string | null;
        error?: string | null;
    };

    type AmuxSetupDependency = {
        name: string;
        label: string;
        command: string;
        found: boolean;
        path: string | null;
        installHints: string[];
    };

    type AmuxSetupPrereqReport = {
        profile: "source" | "desktop";
        platform: string;
        required: AmuxSetupDependency[];
        optional: AmuxSetupDependency[];
        missingRequired: string[];
        daemonPath: string;
        cliPath: string;
        installRoot: string;
        dataDir: string;
        gettingStartedPath: string;
        whatIsTamux: string;
    };

    type AmuxInstalledPluginRecord = {
        packageName: string;
        packageVersion: string;
        pluginName: string;
        entryPath: string;
        format: string;
        installedAt: number;
    };

    type AmuxInstalledPluginLoadResult = {
        packageName: string;
        pluginName: string;
        status: "loaded" | "already-loaded" | "skipped" | "error";
        error?: string;
    };

    type AmuxGoalRunStatus =
        | "queued"
        | "planning"
        | "running"
        | "awaiting_approval"
        | "paused"
        | "completed"
        | "failed"
        | "cancelled";

    type AmuxGoalRunControlAction = "pause" | "resume" | "cancel" | "retry_step" | "rerun_from_step";

    type AmuxTodoItem = {
        id: string;
        content: string;
        status: "pending" | "in_progress" | "completed" | "blocked";
        position: number;
        step_index?: number | null;
        created_at?: number | null;
        updated_at?: number | null;
    };

    type AmuxWorkContextEntry = {
        path: string;
        previous_path?: string | null;
        kind?: "repo_change" | "artifact" | "generated_skill" | null;
        source: string;
        change_kind?: string | null;
        repo_root?: string | null;
        goal_run_id?: string | null;
        step_index?: number | null;
        session_id?: string | null;
        is_text?: boolean;
        updated_at: number;
    };

    type AmuxThreadWorkContext = {
        thread_id: string;
        entries: AmuxWorkContextEntry[];
    };

    type AmuxThreadMessagePinResult = {
        ok: boolean;
        thread_id: string;
        message_id: string;
        error?: string | null;
        current_pinned_chars: number;
        pinned_budget_chars: number;
        candidate_pinned_chars?: number | null;
    };

    type AmuxGoalRunStep = {
        id: string;
        title: string;
        kind: string;
        status?: string | null;
        success_condition?: string | null;
        session_id?: string | null;
    };

    type AmuxGoalRun = {
        id: string;
        title: string;
        goal: string;
        client_request_id?: string | null;
        status: AmuxGoalRunStatus;
        priority?: string | null;
        created_at: number;
        started_at?: number | null;
        completed_at?: number | null;
        thread_id?: string | null;
        current_step_index?: number | null;
        current_step_title?: string | null;
        current_step_kind?: string | null;
        replan_count?: number | null;
        plan_summary?: string | null;
        reflection_summary?: string | null;
        result?: string | null;
        error?: string | null;
        last_error?: string | null;
        failure_cause?: string | null;
        memory_updates?: string[];
        generated_skill_path?: string | null;
        child_task_ids?: string[];
        child_task_count?: number | null;
        approval_count?: number | null;
        duration_ms?: number | null;
        session_id?: string | null;
        awaiting_approval_id?: string | null;
        active_task_id?: string | null;
        steps?: AmuxGoalRunStep[];
    };

    type AmuxAgentRunStatus =
        | "queued"
        | "in_progress"
        | "awaiting_approval"
        | "blocked"
        | "failed_analyzing"
        | "budget_exceeded"
        | "completed"
        | "failed"
        | "cancelled";

    type AmuxAgentRunRuntimeStatusKind =
        | "queued"
        | "running"
        | "awaiting_approval"
        | "waiting_for_dependencies"
        | "waiting_for_subagents"
        | "scheduled"
        | "waiting_for_resources"
        | "blocked"
        | "retrying"
        | "failed_analyzing"
        | "budget_exceeded"
        | "completed"
        | "failed"
        | "cancelled";

    type AmuxAgentRunRuntimeStatus = {
        kind: AmuxAgentRunRuntimeStatusKind;
        reason?: string | null;
        awaiting_approval_id?: string | null;
        next_retry_at?: number | null;
        scheduled_at?: number | null;
    };

    type AmuxAgentRunPriority = "low" | "normal" | "high" | "urgent";

    type AmuxAgentRun = {
        id: string;
        task_id: string;
        kind: "task" | "subagent";
        classification: "coding" | "research" | "ops" | "browser" | "messaging" | "mixed" | string;
        title: string;
        description: string;
        status: AmuxAgentRunStatus;
        runtime_status?: AmuxAgentRunRuntimeStatus | null;
        priority: AmuxAgentRunPriority;
        progress: number;
        created_at: number;
        started_at?: number | null;
        completed_at?: number | null;
        thread_id?: string | null;
        session_id?: string | null;
        workspace_id?: string | null;
        source: string;
        runtime?: string | null;
        goal_run_id?: string | null;
        goal_run_title?: string | null;
        goal_step_id?: string | null;
        goal_step_title?: string | null;
        parent_run_id?: string | null;
        parent_task_id?: string | null;
        parent_thread_id?: string | null;
        parent_title?: string | null;
        blocked_reason?: string | null;
        error?: string | null;
        result?: string | null;
        last_error?: string | null;
    };

    type AmuxStatisticsWindow = "today" | "7d" | "30d" | "all";

    type AmuxAgentStatisticsTotals = {
        input_tokens: number;
        output_tokens: number;
        total_tokens: number;
        cost_usd: number;
        provider_count: number;
        model_count: number;
    };

    type AmuxProviderStatisticsRow = {
        provider: string;
        input_tokens: number;
        output_tokens: number;
        total_tokens: number;
        cost_usd: number;
    };

    type AmuxModelStatisticsRow = {
        provider: string;
        model: string;
        input_tokens: number;
        output_tokens: number;
        total_tokens: number;
        cost_usd: number;
    };

    type AmuxAgentStatisticsSnapshot = {
        window: AmuxStatisticsWindow;
        generated_at: number;
        has_incomplete_cost_history: boolean;
        totals: AmuxAgentStatisticsTotals;
        providers: AmuxProviderStatisticsRow[];
        models: AmuxModelStatisticsRow[];
        top_models_by_tokens: AmuxModelStatisticsRow[];
        top_models_by_cost: AmuxModelStatisticsRow[];
    };

    type AmuxBridge = {
        checkSetupPrereqs?: (profile?: "source" | "desktop") => Promise<AmuxSetupPrereqReport>;
        discoverCodingAgents?: () => Promise<AmuxCodingAgentDiscoveryResult[]>;
        discoverAITraining?: (workspacePath?: string | null) => Promise<AmuxAITrainingDiscoveryResult[]>;
        getDataDir?: () => Promise<string>;
        readJsonFile?: (relativePath: string) => Promise<unknown>;
        writeJsonFile?: (relativePath: string, data: unknown) => Promise<boolean>;
        readTextFile?: (relativePath: string) => Promise<string | null>;
        writeTextFile?: (relativePath: string, content: string) => Promise<boolean>;
        deleteDataPath?: (relativePath: string) => Promise<boolean>;
        listDataDir?: (relativeDir?: string) => Promise<Array<{ name: string; path: string; isDirectory: boolean }>>;
        openDataPath?: (relativePath: string) => Promise<string>;
        revealDataPath?: (relativePath: string) => Promise<boolean>;
        listFsDir?: (targetDir: string) => Promise<Array<{ name: string; path: string; isDirectory: boolean; sizeBytes?: number | null; modifiedAt?: number | null }>>;
        copyFsPath?: (sourcePath: string, destinationPath: string) => Promise<boolean>;
        moveFsPath?: (sourcePath: string, destinationPath: string) => Promise<boolean>;
        deleteFsPath?: (targetPath: string) => Promise<boolean>;
        createFsDirectory?: (targetDirPath: string) => Promise<boolean>;
        openFsPath?: (targetPath: string) => Promise<string>;
        revealFsPath?: (targetPath: string) => Promise<boolean>;
        readFsText?: (targetPath: string) => Promise<string | null>;
        writeFsText?: (targetPath: string, content: string) => Promise<boolean>;
        getFsPathInfo?: (targetPath: string) => Promise<{ path: string; isDirectory: boolean; sizeBytes: number; modifiedAt: number; createdAt: number } | null>;
        gitStatus?: (targetPath: string) => Promise<string>;
        gitDiff?: (targetPath: string, filePath?: string | null) => Promise<string>;
        dbAppendCommandLog?: (entry: unknown) => Promise<boolean>;
        dbCompleteCommandLog?: (id: string, exitCode?: number | null, durationMs?: number | null) => Promise<boolean>;
        dbQueryCommandLog?: (opts?: { workspaceId?: string | null; paneId?: string | null; limit?: number | null }) => Promise<unknown[]>;
        dbClearCommandLog?: () => Promise<boolean>;
        dbCreateThread?: (thread: unknown) => Promise<boolean>;
        dbDeleteThread?: (id: string) => Promise<boolean>;
        dbListThreads?: () => Promise<unknown[]>;
        dbGetThread?: (id: string) => Promise<{ thread: unknown; messages: unknown[] }>;
        dbAddMessage?: (message: unknown) => Promise<boolean>;
        dbDeleteMessage?: (threadId: string, messageId: string) => Promise<boolean>;
        dbListMessages?: (threadId: string, limit?: number | null) => Promise<unknown[]>;
        agentAddTask?: (payload: { title: string; description: string; priority?: string; command?: string | null; sessionId?: string | null; scheduledAt?: number | null; dependencies?: string[] }) => Promise<unknown>;
        agentListRuns?: () => Promise<AmuxAgentRun[] | unknown>;
        agentGetRun?: (runId: string) => Promise<AmuxAgentRun | null | unknown>;
        agentListTodos?: () => Promise<Record<string, AmuxTodoItem[]> | unknown>;
        agentGetTodos?: (threadId: string) => Promise<{ thread_id: string; items: AmuxTodoItem[] } | AmuxTodoItem[] | unknown>;
        agentGetWorkContext?: (threadId: string) => Promise<{ thread_id: string; context: AmuxThreadWorkContext } | AmuxThreadWorkContext | null | unknown>;
        agentGetGitDiff?: (repoPath: string, filePath?: string | null) => Promise<{ repo_path: string; file_path?: string | null; diff: string } | string | unknown>;
        agentGetFilePreview?: (path: string, maxBytes?: number | null) => Promise<{ path: string; content: string; truncated: boolean; is_text: boolean } | null | unknown>;
        agentStartGoalRun?: (payload: { goal: string; title?: string | null; sessionId?: string | null; priority?: string | null; threadId?: string | null; clientRequestId?: string | null }) => Promise<AmuxGoalRun | unknown>;
        agentListGoalRuns?: () => Promise<AmuxGoalRun[] | unknown>;
        agentGetGoalRun?: (goalRunId: string) => Promise<AmuxGoalRun | unknown>;
        agentControlGoalRun?: (goalRunId: string, action: AmuxGoalRunControlAction, stepIndex?: number | null) => Promise<boolean | { ok?: boolean; success?: boolean } | unknown>;
        agentExplainAction?: (actionId: string, stepIndex?: number | null) => Promise<unknown>;
        agentStartDivergentSession?: (payload: {
            problemStatement: string;
            threadId: string;
            goalRunId?: string | null;
            customFramingsJson?: string | null;
        }) => Promise<unknown>;
        agentGetDivergentSession?: (sessionId: string) => Promise<unknown>;
        dbUpsertTranscriptIndex?: (entry: unknown) => Promise<boolean>;
        dbListTranscriptIndex?: (workspaceId?: string | null) => Promise<unknown[]>;
        dbUpsertSnapshotIndex?: (entry: unknown) => Promise<boolean>;
        dbListSnapshotIndex?: (workspaceId?: string | null) => Promise<unknown[]>;
        dbUpsertAgentEvent?: (eventRow: unknown) => Promise<boolean>;
        dbListAgentEvents?: (opts?: { category?: string | null; paneId?: string | null; limit?: number | null }) => Promise<unknown[]>;
        startTerminalSession?: (options: {
            paneId: string;
            sessionId?: string | null;
            shell?: string | null;
            cwd?: string | null;
            workspaceId?: string | null;
            cols?: number;
            rows?: number;
            sourcePaneId?: string | null;
        }) => Promise<{ sessionId?: string | null; initialOutput?: string[]; state?: string }>;
        onTerminalEvent?: (cb: (event: any) => void) => (() => void) | void;
        sendTerminalInput?: (paneId: string | null, data: string) => Promise<boolean>;
        cloneTerminalSession?: (payload: {
            sourcePaneId?: string;
            sourceSessionId?: string | null;
            workspaceId?: string | null;
            cwd?: string | null;
            cols?: number;
            rows?: number;
        }) => Promise<{ sessionId: string; activeCommand?: string }>;
        stopTerminalSession?: (paneId: string, killSession?: boolean) => Promise<boolean>;
        executeManagedCommand?: (paneId: string | null, payload: unknown) => Promise<boolean | { output?: string }>;
        agentSendMessage?: (threadId: string | null, content: string, sessionId?: string | null, contextMessages?: unknown[]) => Promise<{ ok?: boolean; error?: string } | unknown>;
        agentInternalDelegate?: (threadId: string | null, targetAgentId: string, content: string, sessionId?: string | null) => Promise<{ ok?: boolean; error?: string } | unknown>;
        agentThreadParticipantCommand?: (payload: {
            threadId: string;
            targetAgentId: string;
            action: string;
            instruction?: string | null;
            sessionId?: string | null;
        }) => Promise<{ ok?: boolean; error?: string } | unknown>;
        agentSendParticipantSuggestion?: (payload: {
            threadId: string;
            suggestionId: string;
            sessionId?: string | null;
        }) => Promise<{ ok?: boolean; error?: string } | unknown>;
        agentDismissParticipantSuggestion?: (payload: {
            threadId: string;
            suggestionId: string;
            sessionId?: string | null;
        }) => Promise<{ ok?: boolean; error?: string } | unknown>;
        agentListThreads?: () => Promise<unknown[]>;
        agentGetThread?: (
            threadId: string,
            options?: {
                messageLimit?: number | null;
                messageOffset?: number | null;
            },
        ) => Promise<unknown | null>;
        agentPinThreadMessageForCompaction?: (threadId: string, messageId: string) => Promise<AmuxThreadMessagePinResult | unknown>;
        agentUnpinThreadMessageForCompaction?: (threadId: string, messageId: string) => Promise<AmuxThreadMessagePinResult | unknown>;
        openAICodexAuthStatus?: (options?: { refresh?: boolean }) => Promise<AmuxOpenAICodexAuthStatus>;
        openAICodexAuthLogin?: () => Promise<AmuxOpenAICodexAuthLogin>;
        openAICodexAuthLogout?: () => Promise<{ ok: boolean }>;
        agentFetchModels?: (providerId: string, base_url: string, api_key: string) => Promise<{ models?: Array<{ id: string; name?: string; context_window?: number; pricing?: { prompt?: string; completion?: string; image?: string; request?: string; web_search?: string; internal_reasoning?: string; input_cache_read?: string; input_cache_write?: string; audio?: string }; metadata?: Record<string, unknown> }>; error?: string } | unknown>;
        agentGetProviderAuthStates?: () => Promise<unknown[]>;
        agentLoginProvider?: (providerId: string, api_key: string, base_url?: string) => Promise<unknown[] | { error?: string }>;
        agentLogoutProvider?: (providerId: string) => Promise<unknown[] | { error?: string }>;
        agentValidateProvider?: (providerId: string, base_url: string, api_key: string, auth_source: string) => Promise<{ valid: boolean; error?: string; models?: unknown[] }>;
        agentGetConfig?: () => Promise<unknown>;
        agentGetStatus?: () => Promise<{
            tier: string;
            feature_flags: unknown;
            activity: string;
            active_thread_id: string | null;
            active_goal_run_id: string | null;
            active_goal_run_title: string | null;
            provider_health: Record<string, { can_execute: boolean; trip_count: number }>;
            gateway_statuses: Record<string, { status: string; consecutive_failures: number }>;
            recent_actions: Array<{ id: number; timestamp: number; action_type: string; summary: string }>;
            diagnostics?: {
                operator_profile_sync_state?: string;
                operator_profile_sync_dirty?: boolean;
                operator_profile_scheduler_fallback?: boolean;
            };
        } | null>;
        agentGetStatistics?: (window?: AmuxStatisticsWindow) => Promise<AmuxAgentStatisticsSnapshot | null | unknown>;
        agentInspectPrompt?: (agentId?: string | null) => Promise<{
            agent_id: string;
            agent_name: string;
            provider_id: string;
            model: string;
            sections: Array<{ id: string; title: string; content: string }>;
            final_prompt: string;
        } | null>;
        agentQueryAudits?: (actionTypes?: string[] | null, since?: number | null, limit?: number | null) => Promise<unknown>;
        agentGetProvenanceReport?: (limit?: number | null) => Promise<unknown>;
        agentGetMemoryProvenanceReport?: (target?: string | null, limit?: number | null) => Promise<unknown>;
        agentConfirmMemoryProvenanceEntry?: (entryId: string) => Promise<unknown>;
        agentRetractMemoryProvenanceEntry?: (entryId: string) => Promise<unknown>;
        agentGetCollaborationSessions?: (parentTaskId?: string | null) => Promise<unknown>;
        agentVoteOnCollaborationDisagreement?: (parentTaskId: string, disagreementId: string, taskId: string, position: string, confidence?: number | null) => Promise<unknown>;
        agentListGeneratedTools?: () => Promise<unknown>;
        agentRunGeneratedTool?: (toolName: string, argsJson: string) => Promise<unknown>;
        agentActivateGeneratedTool?: (toolName: string) => Promise<unknown>;
        agentPromoteGeneratedTool?: (toolName: string) => Promise<unknown>;
        agentRetireGeneratedTool?: (toolName: string) => Promise<unknown>;
        agentGetOperatorModel?: () => Promise<unknown>;
        agentResetOperatorModel?: () => Promise<{ ok?: boolean; error?: string } | unknown>;
        agentSetConfigItem?: (keyPath: string, value: unknown) => Promise<unknown>;
        agentSetProviderModel?: (providerId: string, model: string) => Promise<{ ok?: boolean; error?: string }>;
        agentSetTargetAgentProviderModel?: (targetAgentId: string, providerId: string, model: string) => Promise<{ ok?: boolean; error?: string }>;
        agentSetTierOverride?: (tier: string | null) => Promise<unknown>;
        agentSetSubAgent?: (subAgentJson: string) => Promise<{ ok?: boolean; error?: string }>;
        agentRemoveSubAgent?: (subAgentId: string) => Promise<{ ok?: boolean }>;
        agentListSubAgents?: () => Promise<unknown[]>;
        agentGetConciergeConfig?: () => Promise<unknown>;
        agentSetConciergeConfig?: (config: unknown) => Promise<unknown>;
        agentDismissConciergeWelcome?: () => Promise<{ ok?: boolean }>;
        agentRequestConciergeWelcome?: () => Promise<{ ok?: boolean }>;
        listInstalledPlugins?: () => Promise<AmuxInstalledPluginRecord[]>;
        loadInstalledPlugins?: () => Promise<AmuxInstalledPluginLoadResult[]>;
        pluginDaemonList?: () => Promise<{ plugins: Array<{ name: string; version: string; description?: string; author?: string; enabled: boolean; install_source: string; has_api: boolean; has_auth: boolean; has_commands: boolean; has_skills: boolean; endpoint_count: number; settings_count: number; installed_at: string; updated_at: string; auth_status?: string }> }>;
        pluginDaemonGet?: (name: string) => Promise<{ plugin: { name: string; version: string; description?: string; author?: string; enabled: boolean; install_source: string; has_api: boolean; has_auth: boolean; has_commands: boolean; has_skills: boolean; endpoint_count: number; settings_count: number; installed_at: string; updated_at: string; auth_status?: string } | null; settings_schema: string | null }>;
        pluginOAuthStart?: (name: string) => Promise<{ name: string; url: string }>;
        onPluginOAuthComplete?: (callback: (data: { name: string; success: boolean; error?: string }) => void) => (() => void) | void;
        pluginDaemonEnable?: (name: string) => Promise<{ ok?: boolean; error?: string }>;
        pluginDaemonDisable?: (name: string) => Promise<{ ok?: boolean; error?: string }>;
        pluginGetSettings?: (name: string) => Promise<{ plugin_name: string; settings: Array<{ key: string; value: string; is_secret: boolean }> }>;
        pluginUpdateSettings?: (pluginName: string, key: string, value: string, isSecret: boolean) => Promise<{ ok?: boolean; error?: string }>;
        pluginTestConnection?: (name: string) => Promise<{ plugin_name: string; success: boolean; message: string }>;
        readClipboardText?: () => Promise<string | null>;
        writeClipboardText?: (text: string) => Promise<void>;
        agentResolveTaskApproval?: (approvalId: string, decision: string) => Promise<unknown>;
        agentListTasks?: () => Promise<unknown[]>;
        sendDiscordMessage?: (payload: { token: string; channelId?: string; userId?: string; message: string }) => Promise<{ ok?: boolean; error?: string; channelId?: string; userId?: string; destination?: string }>;
        whatsappSend?: (jid: string, message: string) => Promise<{ ok?: boolean; error?: string }>;
        whatsappConnect?: () => Promise<{ ok?: boolean; error?: string }>;
        whatsappDisconnect?: () => Promise<{ ok?: boolean; error?: string }>;
        whatsappStatus?: () => Promise<{ status?: string; phone?: string; phoneNumber?: string; lastError?: string | null }>;
        onWhatsAppQR?: (cb: (dataUrl: string | null) => void) => (() => void) | void;
        onWhatsAppConnected?: (cb: (info: { phone: string }) => void) => (() => void) | void;
        onWhatsAppDisconnected?: (cb: (info?: { reason?: string | null } | null) => void) => (() => void) | void;
        onWhatsAppError?: (cb: (message: string) => void) => (() => void) | void;
        saveVisionScreenshot?: (payload: { dataUrl: string }) => Promise<{ ok?: boolean; error?: string; path?: string; expiresAt?: number }>;
        getSystemMonitorSnapshot?: (opts?: { processLimit?: number }) => Promise<any>;
        getSystemFonts?: () => Promise<string[]>;
        getAvailableShells?: () => Promise<Array<{ name: string; path: string; args?: string }>>;
        checkDaemon?: () => Promise<boolean>;
        checkLspHealth?: () => Promise<unknown>;
        checkMcpHealth?: (servers: unknown) => Promise<unknown>;
        onAppCommand?: (cb: (command: string) => void) => (() => void) | void;
        verifyIntegrity?: () => Promise<unknown>;
        resolveManagedApproval?: (paneId: string, approvalId: string, decision: string) => Promise<unknown>;
        onAgentEvent?: (cb: (event: any) => void) => (() => void) | void;
        agentStopStream?: (threadId: string) => Promise<unknown>;
        agentCancelTask?: (taskId: string) => Promise<unknown>;
        agentHeartbeatGetItems?: () => Promise<unknown[]>;
        setWindowOpacity?: (opacity: number) => Promise<void>;
        windowMinimize?: () => void;
        windowMaximize?: () => void;
        windowClose?: () => void;
        windowIsMaximized?: () => Promise<boolean>;
        onWindowState?: (cb: (state: any) => void) => (() => void) | void;
        getPlatform?: () => Promise<string>;
        resizeTerminalSession?: (paneId: string | null, cols: number, rows: number) => Promise<boolean>;
        listSnapshots?: (paneId: string, workspaceId?: string | null) => Promise<unknown[]>;
        restoreSnapshot?: (paneId: string, snapshotId: string) => Promise<unknown>;
        searchManagedHistory?: (paneId: string, query: string, limit?: number) => Promise<unknown>;
        findManagedSymbol?: (paneId: string, workspaceRoot: string, symbol: string, limit?: number) => Promise<unknown>;
        generateManagedSkill?: (paneId: string, query?: string | null, title?: string | null) => Promise<unknown>;
        onWhatsAppMessage?: (cb: (event: any) => void) => (() => void) | void;
        agentStartOperatorProfileSession?: (kind: string) => Promise<{ session_id: string; kind: string } | { error?: string }>;
        agentNextOperatorProfileQuestion?: (sessionId: string) => Promise<{ session_id: string; question_id: string; field_key: string; prompt: string; input_kind: string; optional: boolean } | { error?: string }>;
        agentSubmitOperatorProfileAnswer?: (sessionId: string, questionId: string, answerJson: string) => Promise<{ session_id: string; answered: number; remaining: number; completion_ratio: number } | { error?: string }>;
        agentSkipOperatorProfileQuestion?: (sessionId: string, questionId: string, reason?: string | null) => Promise<{ session_id: string; answered: number; remaining: number; completion_ratio: number } | { error?: string }>;
        agentDeferOperatorProfileQuestion?: (sessionId: string, questionId: string, deferUntilUnixMs?: number | null) => Promise<{ session_id: string; answered: number; remaining: number; completion_ratio: number } | { error?: string }>;
        agentGetOperatorProfileSummary?: () => Promise<unknown>;
        agentSetOperatorProfileConsent?: (consentKey: string, granted: boolean) => Promise<{ ok?: boolean; error?: string }>;
        agentAnswerQuestion?: (questionId: string, answer: string) => Promise<{ ok?: boolean; error?: string }>;
    };

    interface Window {
        tamux?: AmuxBridge;
        amux?: AmuxBridge;
    }
}

export { };
