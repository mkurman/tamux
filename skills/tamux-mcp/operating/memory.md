# Memory — Persistent built-in agent memory (SOUL.md, MEMORY.md, USER.md)

## Agent Rules

- **Treat memory as long-lived context** — save stable preferences, constraints, conventions, recurring corrections, and workspace facts that should still matter later
- **Do not use memory as a scratchpad** — avoid task progress, temporary failures, one-off outputs, or anything easily rediscovered from files, history, or the environment
- **Use the right file** — see the Memory Model below; `MEMORY.md` is the normal target for ongoing updates
- **Keep entries short and high-signal** — these files are loaded into later prompt context and have hard size limits
- **Do not store secrets or sensitive one-off data unless explicitly required** — memory is persistent context that may be loaded into prompts
- **Use markdown structure sparingly** — headings and bullets are fine; verbosity is not
- **If nothing durable was learned, do not update memory**

## Quick Reference

MCP clients use `read_memory` and `write_memory`.

The built-in agent uses `update_memory` instead.

| Tool | Purpose | Key params |
|---|---|---|
| `read_memory` | Read `SOUL.md`, `MEMORY.md`, or `USER.md` | `file` |
| `write_memory` | Write or append memory content | `file`, `content`, `append?` |

### Built-in agent tool: `update_memory`

| Param | Type | Required | Description |
|---|---|---|---|
| `target` | string | Yes | `soul`, `memory`, or `user` |
| `mode` | string | Yes | `replace`, `append`, or `remove` |
| `content` | string | Yes | Concise markdown content to apply |

### Memory Model

| File | Purpose | Notes |
|---|---|---|
| `SOUL.md` | Stable agent identity and principles | Rarely updated |
| `MEMORY.md` | Learned project facts, conventions, and environment knowledge worth keeping | Main place for ongoing updates |
| `USER.md` | Saved operator profile summary | Daemon-managed; direct edits may be overwritten |

### How memory is used

1. The daemon maintains `SOUL.md`, `MEMORY.md`, and `USER.md` as persistent markdown memory.
2. MCP clients read and write those files through `read_memory` and `write_memory`.
3. The built-in agent updates them through `update_memory`.
4. The daemon injects these files into later built-in agent prompts.
5. Goal runs can save lasting project learnings to `MEMORY.md`.
6. `USER.md` stays tied to saved operator profile state rather than freeform notes.

## Gotchas

- `USER.md` is daemon-managed — reconciliation can overwrite direct edits with the saved profile summary
- Memory is intentionally bounded — hard limits are `SOUL.md` 1500 chars, `MEMORY.md` 2200 chars, `USER.md` 1375 chars, so each write or append must still fit within the target file's limit
- Store durable signal only — if a fact can be re-derived quickly, leave it out
- Goal runs may write stable memory, but detailed run history, events, and transcripts remain in daemon persistence rather than memory files
- Memory files are plain markdown — no special frontmatter or custom syntax required
