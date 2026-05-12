# Goal Runs

## Agent Rules

- **Use goal runs for outcomes, not scripts** — start a goal run when it should plan and adapt across multiple steps, not when you already know every exact task
- **Treat goal runs as durable background work** — the run, its steps, events, and status stay available across UI disconnects and daemon restarts
- **Write the `goal` as the desired result** — include scope, constraints, and success criteria instead of step-by-step instructions
- **Use `session_id` when terminal context matters** — it targets an existing active terminal session; it does not create or guarantee one
- **Monitor with `get_goal_run` before intervening** — inspect status, steps, and events to confirm whether the goal run is progressing normally, waiting on approval, or actually needs action
- **Use `control_goal_run` only for supported controls** — `pause`, `resume`, `cancel`, `retry_step`, and `rerun_from_step`
- **Do not split one objective into many background tasks unless you need exact queue control** — goal runs are for supervised autonomy; tasks are for explicit dependency/scheduling management

## Reference

### `start_goal_run`

Start a durable goal run for a bounded objective that may take multiple steps over time. Provide the `goal` text, and the run stays active in the background.

| Param | Type | Required | Description |
|---|---|---|---|
| `goal` | string | Yes | Goal text describing the outcome to accomplish |
| `title` | string | No | Short display title |
| `thread_id` | string | No | Existing agent thread for conversation context; does not target terminal execution |
| `session_id` | string | No | UUID from an existing active terminal session to target |
| `priority` | string | No | `low`, `normal`, `high`, `urgent` |

**Returns:** Full goal run JSON object.

### Goal Run Lifecycle

```text
queued -> planning -> running -> completed
                           -> awaiting_approval -> running
                           -> failed
                           -> paused -> running
any -> cancelled
```

- `queued` — the goal run was accepted and is waiting to start planning
- `planning` — the daemon is building the initial plan from the `goal` text
- `running` — the goal run is carrying out work or tracking task progress
- `completed` — the run finished successfully and will not do more work unless you start a new run
- `awaiting_approval` — a task hit an approval gate; resolve the approval on that task, not with extra `control_goal_run` actions, before the run can continue to `running`
- `paused` — operator intervention stopped future orchestration until resumed; a child task already running is not terminated automatically
- `cancelled` — operator intervention ended the run; this is terminal, even though a child task already running is not terminated automatically
- `failed` — the run could not recover or continue safely

### `list_goal_runs`

List durable goal runs with status, current step, and summary metrics.

**Parameters:** None.

### `get_goal_run`

Fetch one goal run with its plan, step status, events, and derived summaries.

| Param | Type | Required | Description |
|---|---|---|---|
| `goal_run_id` | string | Yes | Goal run UUID |

Use this to see what the goal run has already tried before retrying a step or rerunning from an earlier point. For `retry_step` and `rerun_from_step`, take `step_index` from the step data returned here.

### `control_goal_run`

Change run state or rerun work from a known point.

| Param | Type | Required | Description |
|---|---|---|---|
| `goal_run_id` | string | Yes | Goal run UUID |
| `action` | string | Yes | `pause`, `resume`, `cancel`, `retry_step`, `rerun_from_step` |
| `step_index` | integer | No | Required for `retry_step` and `rerun_from_step` |

No other user-facing controls are supported. If the run changes course or needs attention, check its status and events. `retry_step` and `rerun_from_step` may repeat already-attempted work and can duplicate side effects.

## When To Use Goal Runs

- Use a goal run when you want the daemon's goal runner to decide the next steps, react to outcomes, and keep working durably in the background
- Use a task when you already know the exact unit of work, command, dependency chain, or schedule you want queued
- Use a normal chat turn when you only need reasoning, advice, or a quick answer without durable execution

## Gotchas

- Goal runs use tasks for executable work; they do not replace the task system
- Approval gates pause autonomous progress; check the run state before assuming it is stuck
- Pausing a goal run stops future orchestration, but it does not terminate a child task that is already running
- If a run is failing repeatedly, inspect `events` and step results before using `retry_step` or `rerun_from_step`
- Goal runs currently require the built-in `daemon` backend, the service that owns run state and orchestration, so planning, supervision, and reflection consume model tokens
