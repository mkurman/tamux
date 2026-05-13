# Investigation and Recovery — History, Symbols, Snapshots, and Git State

Use this as a quick investigation and recovery reference before retrying work. Check prior history, inspect git state, locate the right code, and only then decide whether to retry, replan, or restore.

## Agent Rules

- **Start with `search_history` before retrying or rerunning work** — check prior commands, transcript output, policy responses, and failure text first.
- **Reconstruct the last attempt before changing strategy** — pair history with `get_git_status` so you know what changed and what is still dirty.
- **Use specific history queries** — search exact error text, command fragments, denied actions, filenames, or symbols instead of broad terms.
- **Use `find_symbol` once history tells you where to look** — semantic symbol search is best when you already know the likely code target.
- **Check `list_snapshots` before recovery actions** — understand whether a rollback point exists before proposing destructive recovery.
- **Treat `restore_snapshot` as a last-resort repair step** — inspect history and git state first so you know what current work would be discarded.

## Reference

### Tool: `search_history`

Search daemon-managed command and transcript history with full-text queries.

| Param | Type | Required | Description |
|---|---|---|---|
| `query` | string | Yes | Full-text query |
| `limit` | integer | No | Max results to return |

Practical queries:

- `"cargo test" AND fail*` — find repeated test failures
- `"approval_required" AND git` — inspect git commands stopped for approval
- `denied AND rm` — check earlier policy blocks or denied commands
- `"No such file" AND Cargo.toml` — match exact failure text

Use it first to answer: what already failed, what was approved or denied, what command sequence last worked, and whether this path has already been tried.

### Tool: `find_symbol`

Search indexed code symbols in a workspace using the daemon's semantic index.

| Param | Type | Required | Description |
|---|---|---|---|
| `workspace_root` | string | Yes | Absolute path to the workspace root |
| `symbol` | string | Yes | Symbol name or pattern to search for |
| `limit` | integer | No | Max results |

Use it after history points you to a likely file, function, type, handler, or module.

### Tool: `list_snapshots`

List recorded workspace snapshots. Daemon-managed commands create snapshots automatically before execution; direct terminal input or other work outside daemon-managed execution may have no rollback point.

| Param | Type | Required | Description |
|---|---|---|---|
| `workspace_id` | string | No | Filter by workspace |

Use it to confirm whether a rollback point exists before proposing recovery.

### Tool: `restore_snapshot`

Restore a previously recorded workspace snapshot.

| Param | Type | Required | Description |
|---|---|---|---|
| `snapshot_id` | string | Yes | Snapshot ID to restore |

Warning: this replaces current workspace state. It does not undo external side effects.

### Tool: `get_git_status`

Get git status for a working directory.

| Param | Type | Required | Description |
|---|---|---|---|
| `path` | string | Yes | Absolute path to the git repository |

Use it to check branch, dirty state, staged changes, and untracked files before retrying, reverting, or restoring.

## Typical Investigation Workflows

### Retry or Replan After Failure

```text
1. search_history("exact error text or failed command")
2. search_history("related success, filename, or symbol")
3. get_git_status(path="/home/user/project")
4. find_symbol(workspace_root="/home/user/project", symbol="suspected_handler")
5. choose a new plan instead of repeating the same attempt unchanged
```

### Check Approval or Policy Failures

```text
1. search_history("approval_required OR denied OR rejected")
2. search_history("command fragment AND error text")
3. get_git_status(path="/home/user/project")
4. re-scope the command or switch strategies before retrying
```

### Recover From a Bad Daemon-Managed Command

```text
1. search_history("command text OR resulting error")
2. get_git_status(path="/home/user/project")
3. confirm there is no wanted uncommitted work to keep
4. list_snapshots()
5. restore_snapshot(snapshot_id="...")
```

## Gotchas

- `search_history` only covers daemon-managed command and transcript history; work done outside daemon-managed execution will not appear.
- `search_history` uses full-text query syntax, so exact phrases and operators may need quoting.
- `find_symbol` depends on semantic indexing; first lookup or stale indexes may make searches slower or less complete.
- Snapshots only cover daemon-managed commands, can be pruned, and are not guaranteed for every past action.
- `restore_snapshot` is destructive for current workspace state and should follow investigation, not replace it.
- `get_git_status` only works on a valid git repository path.
