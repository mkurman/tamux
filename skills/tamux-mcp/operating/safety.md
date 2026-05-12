# Approval, Sandbox, and Recovery Safety

Use the managed execution path: approval gates, sandbox controls, automatic snapshots, WORM telemetry ledgers, and scrubbing tools.

These protections apply to managed commands such as `execute_command`; direct terminal input via `type_in_terminal` and other interactive terminal flows bypass approval gates and filesystem snapshots.

Snapshots are created for managed commands and can be pruned or unavailable depending on backend and retention, so rollback is not guaranteed indefinitely.

## Agent Rules

- **Always provide a clear `rationale` with `execute_command`** — the operator sees it during approval, and it should explain why the command is needed now.
- **Use the right terminal path** — use `execute_command` for normal one-shot commands, and reserve `type_in_terminal` for genuinely interactive terminal flows.
- **Treat `approval_required` as a hard pause** — report the event, wait for the operator decision, and do not continue dependent follow-up actions until that approval is resolved.
- **Do not repeat denied or failing approaches unchanged** — if a command is denied, rejected, or keeps failing, inspect fresh state and change strategy before trying again.
- **Respect sandbox and policy controls** — do not try to bypass managed execution, approval gates, or platform sandboxing.
- **Keep scope and impact clear** — keep commands scoped, and describe expected impact plainly when risky work is necessary.
- **Use rollback, redaction, and integrity tools when needed** — snapshots help with rollback, `scrub_sensitive` can redact sensitive text before sharing or storing it, and `verify_integrity` checks the WORM telemetry ledgers.

## Reference

### Safety Layers

The safety model is operator-visible and layered:

1. **Policy evaluation** — risky managed commands can be stopped before execution.
2. **Sandbox isolation** — managed commands run in a sandboxed lane with platform-specific restrictions.
3. **Approval workflow** — higher-risk commands pause for operator review.
4. **Filesystem snapshots** — automatic checkpoints are created before managed commands.
5. **Append-only WORM telemetry ledgers** — the daemon records execution telemetry for audit and tamper detection.
6. **Sensitive-data scrubbing** — secrets can be redacted before display or logging.

### Approval and Denial Flow

When `execute_command` hits a risky pattern, it can return an `approval_required` event with the fields below. Treat that event as a workflow stop for the affected action until the operator allows it or you choose a different plan:

- `approval_id`
- `command`
- `rationale`
- `risk_level`
- `blast_radius`
- `reasons`

Some risky commands never reach `approval_required`; policy can reject them before approval is offered.

The operator can then choose:

- **Allow Once** — run this execution only
- **Allow For Session** — allow this and similar commands for the current session
- **Deny** — reject the command

Approval can allow one execution or, with **Allow For Session**, similar commands for the current session; it does not bypass sandbox restrictions or policy enforcement.

If the operator denies the command, take that as a stop signal. Do not resubmit the same risky approach without materially new context or a narrower plan.

### Replan Triggers

If execution becomes repetitive, stuck, or keeps failing after approval, stop and replan from fresh state instead of repeating the same command unchanged.

### Practical Risk Patterns

Common approval-triggering categories include:

| Category | Examples |
|---|---|
| Destructive filesystem | `rm -rf`, `shred`, `mkfs` |
| Git force/destructive operations | `git push --force`, `git reset --hard`, `git clean -fdx` |
| Infrastructure mutations | `terraform destroy`, `kubectl delete`, `docker rm` |
| System/package mutation | `apt install`, `pip install --system`, `npm install -g` |
| Pipe-to-shell | `curl ... \| bash`, `wget ... \| sh` |
| Network exfiltration | `curl -X POST` with data, `scp`, `rsync` to remote |
| Privilege or permission changes | `sudo`, `su`, `chmod 777` |
| Database mutation | `DROP TABLE`, `DELETE FROM`, `TRUNCATE` |

### Recovery, Integrity, and Scrubbing

**Snapshots**

Managed commands create snapshots automatically. If a command causes damage or unexpected state drift:

```text
1. list_snapshots() -> find the relevant pre-execution snapshot
2. restore_snapshot(snapshot_id="...") -> roll back the workspace
```

`restore_snapshot` restores workspace/filesystem state only; it does not undo external side effects such as network calls, package installs, infrastructure changes, or database mutations.

**WORM telemetry ledgers**

- `verify_integrity` validates the WORM telemetry ledgers for tampering.

**Scrubbing**

- `scrub_sensitive(text="...")` redacts secrets, tokens, passwords, private keys, and similar sensitive values before sharing or storing text.

## Gotchas

- Not every command needs approval, but risky patterns still do.
- `Allow For Session` is broader than one exact command; similar commands may be auto-approved afterward.
- Sandbox restrictions vary by platform and configuration; do not assume unrestricted filesystem or network access.
- `restore_snapshot` is a destructive rollback of current workspace state.
- Snapshot creation adds latency by design.
- External MCP agents receive an `approval_required` event in the tool response and should surface it instead of retrying.
