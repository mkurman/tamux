# tamux MCP Cheatsheet

> Quick reference for all tamux MCP tools. For detailed usage, see the per-domain skill files linked from [README.md](README.md).

## Agent Rules (Read This First)

1. **Always call `list_sessions` before executing commands** -- discover session IDs and CWDs; never hardcode them.
2. **Provide a `rationale` with every `execute_command`** -- explains intent to the approval system and appears in audit logs.
3. **Check task status after enqueueing** -- tasks may need approval before they run.
4. **Read terminal content after command execution** -- verify the command produced expected output.
5. **Use `search_history` before re-running work** -- avoid duplicating commands already executed in the session.

---

## Terminal & Sessions

Skill file: [operating/terminals.md](operating/terminals.md)

| Tool | Purpose | Key Parameters |
| --- | --- | --- |
| `list_sessions` | List active terminal sessions with workspace hierarchy | -- |
| `execute_command` | Run a command with approval gating and snapshots | `session_id`, `command`, `rationale` |
| `get_terminal_content` | Read terminal scrollback buffer | `session_id`, `max_lines` (default 100) |
| `type_in_terminal` | Send keystrokes to interactive programs | `session_id`, `input` (supports `\n`, `\r`, `\t`) |

---

## Browser Automation

Skill file: [operating/browser.md](operating/browser.md)

| Tool | Purpose | Key Parameters |
| --- | --- | --- |
| `open_canvas_browser` | Create a new browser panel on the canvas | `url?`, `name?` |
| `browser_navigate` | Navigate to a URL | `url`, `pane?` |
| `browser_back` | Go back in history | -- |
| `browser_forward` | Go forward in history | -- |
| `browser_reload` | Reload current page | -- |
| `browser_read_dom` | Get page text, title, and URL | `pane?` |
| `browser_take_screenshot` | Capture screenshot to vision storage | -- |
| `browser_click` | Click element by selector or text | `pane`, `selector?`, `text?` |
| `browser_type` | Type into an input or textarea | `pane`, `selector`, `text`, `clear?` |
| `browser_scroll` | Scroll page or element | `pane`, `direction`, `amount?`, `selector?` |
| `browser_get_elements` | List interactive elements | `pane`, `filter?`, `limit?` |
| `browser_eval_js` | Execute JavaScript in page context | `pane`, `code` |

---

## Task Queue

Skill file: [operating/tasks.md](operating/tasks.md)

| Tool | Purpose | Key Parameters |
| --- | --- | --- |
| `enqueue_task` | Queue a background task | `description`, `priority?`, `command?`, `session_id?`, `dependencies?`, `scheduled_at?`, `delay_seconds?` |
| `list_tasks` | List all tasks with status | -- |
| `cancel_task` | Cancel a queued or running task | `task_id` |

---

## Subagents

Use provider/model discovery before pinning a spawned agent to a specific runtime config.

| Tool | Purpose | Key Parameters |
| --- | --- | --- |
| `fetch_authenticated_providers` | List authenticated providers that are ready to run child agents | -- |
| `fetch_provider_models` | Fetch remotely available models for one authenticated provider | `provider` |
| `spawn_subagent` | Spawn a bounded child task, optionally with explicit `provider`/`model` overrides | `title`, `description`, `provider?`, `model?`, `runtime?`, `session?` |

Recommended flow:

```
fetch_authenticated_providers   -> choose provider
fetch_provider_models           -> provider: "openai"
spawn_subagent                  -> title, description, provider, model
list_subagents                  -> monitor child progress
```

---

## Goal Runners

Skill file: [operating/goals.md](operating/goals.md)

| Tool | Purpose | Key Parameters |
| --- | --- | --- |
| `start_goal_run` | Start a durable multi-step objective | `goal`, `title?`, `thread_id?`, `session_id?`, `priority?` |
| `list_goal_runs` | List all goal runs | -- |
| `get_goal_run` | Get detailed goal run info | `goal_run_id` |
| `control_goal_run` | Pause, resume, cancel, or retry a goal | `goal_run_id`, `action`, `step_index?` |

---

## Memory

Skill file: [operating/memory.md](operating/memory.md)

| Tool | Purpose | Key Parameters |
| --- | --- | --- |
| `read_memory` | Read a memory file (SOUL.md, MEMORY.md, USER.md) | `file` |
| `write_memory` | Write or append to a memory file | `file`, `content`, `append?` |

---

## Workspaces & Layout

Skill file: [operating/workspaces.md](operating/workspaces.md)

| Tool | Purpose | Key Parameters |
| --- | --- | --- |
| `list_workspaces` | List all workspaces | -- |
| `create_workspace` | Create a new workspace | `name`, `path?` |
| `split_pane` | Split a pane horizontally or vertically | `session_id`, `direction` |
| `resize_pane` | Resize a pane | `session_id`, `rows?`, `cols?` |
| `create_snippet` | Save a reusable command snippet | `name`, `command`, `description?` |
| `list_snippets` | List saved snippets | -- |
| `run_snippet` | Execute a saved snippet | `name`, `session_id` |

---

## Messaging

Skill file: [operating/messaging.md](operating/messaging.md)

| Tool | Purpose | Key Parameters |
| --- | --- | --- |
| `send_message` | Send a message via Slack, Discord, or Telegram | `channel`, `text`, `provider` |
| `read_messages` | Read recent messages from a channel | `channel`, `provider`, `limit?` |

---

## Observability & History

Skill file: [operating/observability.md](operating/observability.md)

| Tool | Purpose | Key Parameters |
| --- | --- | --- |
| `search_history` | Full-text search over command history | `query`, `limit?` |
| `find_symbol` | Search code symbols via AST index | `workspace_root`, `symbol`, `limit?` |
| `list_snapshots` | List filesystem snapshots | `workspace_id?` |
| `restore_snapshot` | Restore a previous snapshot | `snapshot_id` |
| `get_git_status` | Git status for a repository | `path` |

---

## Safety & Utilities

Skill file: [operating/safety.md](operating/safety.md)

| Tool | Purpose | Key Parameters |
| --- | --- | --- |
| `verify_integrity` | Verify WORM telemetry ledgers are intact | -- |
| `scrub_sensitive` | Redact secrets and credentials from text | `text` |

---

## Common Workflows

### Run a command and verify output

```
list_sessions          -> pick session_id
execute_command        -> session_id, command, rationale
get_terminal_content   -> session_id, max_lines: 50
```

### Browse a page and extract data

```
open_canvas_browser    -> url
browser_read_dom       -> pane
browser_eval_js        -> pane, code: "document.querySelector('h1').textContent"
```

### Queue work with dependencies

```
enqueue_task           -> description: "build", command: "cargo build"      -> returns task_a
enqueue_task           -> description: "test",  command: "cargo test", dependencies: [task_a]
list_tasks             -> verify both tasks are queued
```

### Start and monitor a goal

```
start_goal_run         -> goal: "Deploy staging and run smoke tests"
list_goal_runs         -> check status
get_goal_run           -> goal_run_id for step-level detail
control_goal_run       -> goal_run_id, action: "pause"
```
