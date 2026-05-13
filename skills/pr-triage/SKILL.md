---
name: pr-triage
description: Use when the user wants a pull request review queue summary, morning PR check, stale PR scan, or a routine-ready GitHub triage summary delivered to chat.
---

# PR Triage

Use this skill to produce a concise, actionable pull request triage summary from the GitHub plugin.

## When to use

- Morning engineering review
- On-demand PR queue check
- Scheduled status digest
- Review bottleneck scan
- Stale PR follow-up

## Inputs you need

- GitHub plugin configured with token
- repository owner
- repository name
- optional delivery target: Slack, Telegram, or Discord

## Fetch pattern

1. Fetch repository overview when branch or repo context matters.
2. Fetch open pull requests.
3. If needed, fetch open issues for blocking context.
4. Focus on actionable PR state, not full raw output.

Core GitHub calls:

```json
{"plugin_name": "github", "endpoint_name": "get_repo", "params": {"owner": "OWNER", "repo": "REPO"}}
```

```json
{"plugin_name": "github", "endpoint_name": "list_pull_requests", "params": {"owner": "OWNER", "repo": "REPO"}}
```

Optional filter example:

```json
{"plugin_name": "github", "endpoint_name": "list_pull_requests", "params": {"owner": "OWNER", "repo": "REPO", "state": "open", "per_page": 20}}
```

Optional blocking context:

```json
{"plugin_name": "github", "endpoint_name": "list_issues", "params": {"owner": "OWNER", "repo": "REPO", "state": "open", "per_page": 20}}
```

## Triage buckets

Summarize PRs into these buckets when possible:

- **needs-review** — open PRs likely waiting for reviewer attention
- **needs-merge** — PRs that look ready to land
- **blocked** — PRs with obvious blockers, stale state, or unresolved dependency context
- **stale** — PRs old enough or quiet enough to need follow-up

If the available plugin data is limited, say what is inferred versus confirmed.

## Summary format

Keep the summary short and operational:

- repo name
- count of open PRs
- top actionable items by bucket
- notable stale PRs
- clear next actions

Example outline:

```text
PR triage — OWNER/REPO

Open PRs: 7
Needs review: 3
Needs merge: 1
Blocked: 2
Stale: 1

Top actions:
- #123 Add X — needs review
- #118 Fix Y — likely ready to merge
- #111 Refactor Z — blocked by failing checks or missing follow-up
```

## Delivery

For chat delivery, send the final concise summary through one of:

- `send_slack_message`
- `send_telegram_message`
- `send_discord_message`

Only deliver after the summary is clean and human-readable.

## Routine compatibility

This skill is routine-friendly.

Typical trigger windows:
- start of workday
- pre-standup
- afternoon review sweep
- end-of-day handoff

## Error handling

- If GitHub plugin is not configured, say the GitHub token is missing.
- If repo coordinates are missing, ask for owner/repo.
- If plugin output is sparse, provide a best-effort triage and label uncertainty explicitly.
