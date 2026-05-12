---
name: daily-brief
description: Use when the user wants a morning brief, start-of-day summary, routine-ready daily digest, or a concise operational overview combining calendar, tasks, PRs, and issues.
---

# Daily Brief

Use this skill to build a concise daily operating brief that combines calendar context, task status, pull request queue, and issue highlights.

## When to use

- Morning startup routine
- Start-of-day planning
- Pre-standup summary
- On-demand daily overview
- Scheduled digest delivery to chat

## Data sources

Use the smallest set needed for a useful brief:

1. **Calendar** — today's meetings and timing constraints
2. **Task queue** — current work status and bottlenecks
3. **PR queue** — review and merge pressure
4. **Issue backlog** — urgent or blocking open issues

## Fetch sequence

### 1. Calendar agenda
Use the Calendar plugin for the current day window.

Example:

```json
{"plugin_name": "calendar", "endpoint_name": "list_events", "params": {"time_min": "TODAY_START_ISO", "time_max": "TODAY_END_ISO", "max_results": 10}}
```

### 2. Task queue status
Use workspace task tools or equivalent task surfaces to summarize:
- todo
- in progress
- in review
- blocked / notable notices

### 3. PR queue
Use the GitHub plugin directly or reuse the PR triage skill flow.

Core call:

```json
{"plugin_name": "github", "endpoint_name": "list_pull_requests", "params": {"owner": "OWNER", "repo": "REPO"}}
```

### 4. Issue highlights
Use GitHub issues when there are likely blockers or urgent backlog items.

```json
{"plugin_name": "github", "endpoint_name": "list_issues", "params": {"owner": "OWNER", "repo": "REPO", "state": "open", "per_page": 20}}
```

## Summary format

Keep the brief short, prioritized, and operational.

Recommended sections:

```text
Daily brief

Agenda
- 09:00 Team sync
- 13:00 Review block

Tasks
- In progress: 2
- In review: 1
- Blocked: 1

PRs
- Needs review: 3
- Ready to merge: 1

Issues
- 2 likely blockers

Top priorities today
1. Finish X before 13:00
2. Review PR #123
3. Resolve blocker in issue #456
```

## Prioritization rules

- Lead with time-sensitive calendar constraints
- Surface blocked or in-review work before backlog noise
- Prefer actionable PRs over full queue dumps
- Mention only the most relevant issues
- End with 2-4 concrete next actions

## Delivery

For chat delivery, send the final cleaned brief through one of:

- `send_slack_message`
- `send_telegram_message`
- `send_discord_message`

Only deliver after the brief is readable and compressed for skim reading.

## Routine compatibility

This skill is designed for routines.

Typical trigger times:
- start of workday
- 30 minutes before standup
- midday reset

## Error handling

- If calendar access is unavailable, continue with tasks + PRs + issues.
- If GitHub plugin is not configured, omit PR/issues and say GitHub data is unavailable.
- If task tools are unavailable, still produce a calendar + GitHub brief.
- Always degrade gracefully instead of failing the whole brief.
