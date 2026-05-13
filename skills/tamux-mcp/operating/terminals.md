# Terminal Session Management and Command Execution

Discover, manage, and interact with terminal sessions through tamux MCP tools.

## Agent Rules

- **ALWAYS call `list_sessions` first** to discover session IDs and working directories — never assume or hardcode session IDs.
- **Reuse the same `session_id`** across related commands so all work stays in one terminal context (shared shell state, environment variables, working directory).
- **Always provide a `rationale`** when calling `execute_command` — this is shown to the operator during approval and is required.
- **Read terminal content after execution** to verify the command produced expected results — do not assume success from the event stream alone.
- **Use `type_in_terminal` only for interactive programs** (vim, python REPL, TUI apps, password prompts) — prefer `execute_command` for one-shot commands.
- **Handle approval responses** — `execute_command` may return an `approval_required` event if the command matches risk patterns. Inform the user and wait.
- **Escape special characters in `input`** — `type_in_terminal` supports `\n` (newline), `\r` (carriage return), `\t` (tab).

## Reference

### Tool: `list_sessions`

**Description:** List all active terminal sessions and browser panels with workspace/surface hierarchy.

**Parameters:** None

**Returns:** JSON array of sessions with:

| Field | Type | Description |
|---|---|---|
| `session_id` | string (UUID) | Unique identifier for the session |
| `pane_id` | string | Pane identifier within the layout |
| `title` | string | Terminal title (usually the shell name or running program) |
| `cwd` | string | Current working directory of the session |
| `active_command` | string or null | Currently running command, if any |

Workspace and surface hierarchy info is also included.

**Example response:**

```json
{
  "sessions": [
    {
      "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "pane_id": "pane_1",
      "title": "zsh",
      "cwd": "/home/user/project",
      "active_command": null
    }
  ]
}
```

---

### Tool: `execute_command`

**Description:** Execute a managed command in a sandboxed lane with approval gating and filesystem snapshots.

**Parameters:**

| Name | Type | Required | Description |
|---|---|---|---|
| `session_id` | string (UUID) | Yes | Target terminal session |
| `command` | string | Yes | Shell command to execute |
| `rationale` | string | Yes | Why this command is needed (shown to the operator during approval) |

**Returns:** Stream of events:

| Event | Meaning |
|---|---|
| `queued` | Command entered the execution queue |
| `started` | Execution began |
| `finished` | Completed with exit code and output |
| `rejected` | Denied by the policy engine |
| `approval_required` | Paused, waiting for operator decision |

**Approval flow:** If the command matches risk patterns (e.g., `rm -rf`, `git push --force`, `terraform destroy`), it enters the `approval_required` state. The operator sees the command, rationale, risk level, and blast radius, then chooses one of:

- **Allow Once** — permit this single execution
- **Allow For Session** — permit this and similar commands for the rest of the session
- **Deny** — reject the command

**Example call:**

```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "method": "tools/call",
  "params": {
    "name": "execute_command",
    "arguments": {
      "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "command": "cargo test --workspace",
      "rationale": "Run the full test suite to verify refactoring did not break anything"
    }
  }
}
```

---

### Tool: `get_terminal_content`

**Description:** Read the scrollback buffer (visible content) of a terminal session. ANSI escape codes are stripped from the output.

**Parameters:**

| Name | Type | Required | Description |
|---|---|---|---|
| `session_id` | string (UUID) | Yes | Target terminal session |
| `max_lines` | integer | No | Maximum number of lines to return (default: 100) |

**Returns:**

| Field | Type | Description |
|---|---|---|
| `session_id` | string | The session that was read |
| `content` | string | Plain-text terminal content with ANSI codes stripped |
| `line_count` | integer | Number of lines returned |

**Example response:**

```json
{
  "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "content": "$ cargo test\n   Compiling myproject v0.1.0\ntest result: ok. 42 passed; 0 failed",
  "line_count": 3
}
```

---

### Tool: `type_in_terminal`

**Description:** Send keystrokes/input to a terminal session. Used for interactive programs (vim, REPLs, TUI apps, password prompts).

**Parameters:**

| Name | Type | Required | Description |
|---|---|---|---|
| `session_id` | string (UUID) | Yes | Target terminal session |
| `input` | string | Yes | Text to send. Supports escape sequences: `\n` (newline), `\r` (carriage return), `\t` (tab) |

**Returns:**

| Field | Type | Description |
|---|---|---|
| `session_id` | string | The session that received input |
| `bytes_sent` | integer | Number of bytes written |
| `status` | string | `"ok"` on success |

**Example response:**

```json
{
  "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "bytes_sent": 5,
  "status": "ok"
}
```

## Gotchas

- **`execute_command` is not instant.** It queues the command and streams events. Do not assume output is available immediately after the call returns — wait for the `finished` event or read terminal content afterward.
- **Session IDs are UUIDs, not pane names.** Always get them from `list_sessions`. They change between daemon restarts.
- **`get_terminal_content` returns ANSI-stripped text.** You will not see colors, bold, or other formatting codes in the returned content.
- **`type_in_terminal` sends raw keystrokes.** If the terminal is at a shell prompt, the text will be typed as-is. You must append `\n` to the input if you want the shell to execute what you typed.
- **Approval pauses execution.** If a command needs approval, the tool call will return an `approval_required` event. You should inform the user and wait for the approval to resolve before proceeding with dependent commands.
- **Filesystem snapshots are automatic.** Managed commands take automatic filesystem snapshots before execution. If a command causes damage, you can restore via `restore_snapshot` — but you must have the snapshot ID from the execution events.
- **Do not use `type_in_terminal` for regular commands.** It bypasses the approval gate and filesystem snapshots. Use `execute_command` for anything that is not genuinely interactive.
