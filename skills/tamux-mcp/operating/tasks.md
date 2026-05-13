# Background Tasks — Daemon-managed task queue with dependencies and scheduling

## Agent Rules

- **Use tasks for work that should survive disconnects** — tasks persist in daemon SQLite and resume after restarts
- **Always provide a descriptive `description`** — this is the primary identifier for the task
- **Set `priority` appropriately** — urgent tasks execute before normal; low tasks yield to everything else
- **Use `dependencies` for ordered execution** — a task won't start until all dependency tasks complete
- **Check task status with `list_tasks` after enqueueing** — tasks may enter `awaiting_approval` state
- **Use `session_id` to bind a task to a terminal** — ensures the command runs in the right context
- **Use scheduling for delayed/timed work** — `delay_seconds`, `scheduled_at` (Unix ms), or `schedule_at` (RFC3339)
- **Cancel tasks that are no longer needed** — don't leave stale tasks in the queue

## Reference

### `enqueue_task`

Queue a daemon-managed background task with dependencies and scheduling.

| Param | Type | Required | Description |
|---|---|---|---|
| `description` | string | Yes | What the task should do |
| `title` | string | No | Short display title |
| `priority` | string | No | `low`, `normal` (default), `high`, `urgent` |
| `command` | string | No | Shell command to execute |
| `session_id` | string | No | Target terminal session UUID |
| `dependencies` | array | No | Task IDs that must complete first |
| `scheduled_at` | integer | No | Unix timestamp (ms) for delayed execution |
| `schedule_at` | string | No | RFC3339 datetime string (alternative to scheduled_at) |
| `delay_seconds` | integer | No | Relative delay from now |

**Returns:** Full task JSON with `id`, `status`, `created_at`, etc.

**Task lifecycle:**

```
Queued -> InProgress -> Completed
                    \-> Failed (retries up to max_retries)
                    \-> AwaitingApproval (if command triggers policy)
Queued -> Blocked (dependencies not met)
Any state -> Cancelled (via cancel_task)
```

### `list_tasks`

List all daemon-managed tasks with status, dependencies, schedule, and metadata.

**Parameters:** None.

**Returns:** Array of task objects with: id, title, description, status, priority, progress, created_at, started_at, completed_at, error, thread_id, session_id, retry_count, dependencies, scheduled_at, logs.

### `cancel_task`

Cancel a queued, blocked, running, or approval-pending task.

| Param | Type | Required | Description |
|---|---|---|---|
| `task_id` | string | Yes | Task UUID to cancel |

**Returns:** `{ task_id, cancelled: true/false }`

### Scheduling Examples

```json
// Run in 5 minutes
{"description": "Run test suite", "command": "cargo test", "delay_seconds": 300}

// Run at specific time
{"description": "Deploy to staging", "command": "./deploy.sh staging", "schedule_at": "2024-03-15T14:00:00Z"}

// Run after dependency completes
{"description": "Run integration tests", "command": "cargo test --test integration", "dependencies": ["task_abc123"]}
```

## Gotchas

- Tasks that were `InProgress` when the daemon restarts are automatically re-queued (not lost).
- `command` is optional -- tasks without commands are executed by the agent's LLM loop, not as shell commands.
- Dependency chains: if a dependency fails, the dependent task becomes `Blocked`.
- Tasks have a `max_retries` (default 3) -- failed tasks retry automatically with exponential backoff.
- Task logs are preserved even after completion -- use `list_tasks` to inspect historical execution.
- The `session_id` must be a valid active session -- get it from `list_sessions`.
