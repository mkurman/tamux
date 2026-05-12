---
name: inbox-triage
description: Use when the user wants an inbox cleanup summary, morning email triage, scheduled inbox sweep, or a concise action-oriented Gmail digest delivered to chat.
---

# Inbox Triage

Use this skill to turn a Gmail inbox scan into a short, actionable triage summary.

## When to use

- Inbox-heavy days
- Morning email cleanup
- Scheduled inbox sweep
- On-demand mailbox review
- End-of-day reply triage

## Inputs you need

- Gmail plugin configured
- optional Gmail search query
- optional delivery target: Slack, Telegram, or Discord

## Fetch pattern

Gmail inbox triage is a two-step retrieval flow.

1. List inbox message IDs.
2. Fetch details for the most relevant messages.
3. Bucket messages into actionable groups.
4. Summarize next actions, not raw email data.

Core Gmail calls:

```json
{"plugin_name": "gmail", "endpoint_name": "list_inbox", "params": {"max_results": 10}}
```

Then fetch message details:

```json
{"plugin_name": "gmail", "endpoint_name": "get_message", "params": {"message_id": "ID_HERE"}}
```

Optional filtered search:

```json
{"plugin_name": "gmail", "endpoint_name": "search_messages", "params": {"query": "is:unread", "max_results": 10}}
```

## Triage buckets

Summarize messages into these buckets when possible:

- **needs-reply** — messages likely needing a direct response
- **needs-action** — tasks, requests, deadlines, or follow-ups
- **can-archive** — informational or already-resolved mail
- **starred / high-priority** — notable items already flagged or clearly urgent

If certainty is low, label the bucket as inferred.

## Summary format

Keep the result concise and operational.

Example outline:

```text
Inbox triage

Messages scanned: 10
Needs reply: 3
Needs action: 2
Can archive: 4
High priority: 1

Top actions:
- Reply to Alice about project timeline
- Review document request from Bob
- Archive status update thread from ops
```

## Action guidance

Where useful, recommend follow-up actions such as:
- reply
- archive
- star
- mark read
- restore from trash

Use Gmail plugin actions only when the user explicitly asks to modify mailbox state.

## Delivery

For chat delivery, send the final concise summary through one of:

- `send_slack_message`
- `send_telegram_message`
- `send_discord_message`

Only deliver after the summary is readable and skim-friendly.

## Routine compatibility

This skill is routine-friendly.

Typical trigger windows:
- morning inbox check
- post-lunch cleanup
- end-of-day sweep

## Error handling

- If Gmail plugin is not configured, say Gmail access is unavailable.
- If inbox listing is sparse, provide a best-effort summary from available messages.
- If search results are empty, say so clearly instead of inventing categories.
- Degrade gracefully rather than failing the whole triage.
