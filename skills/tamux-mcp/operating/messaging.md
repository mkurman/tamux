# Gateway Messaging — Slack, Discord, Telegram

Send and receive messages across chat platforms via the tamux gateway.

## Agent Rules

- **Confirm with the operator before sending messages** — messages to chat platforms are visible to others and irreversible.
- **Use the correct platform tool** — each platform has its own tool (`send_slack_message`, `send_discord_message`, etc.).
- **Target the right channel/user** — specify the destination explicitly.
- **Keep messages concise and actionable** — chat platform recipients expect brief, clear messages.
- **Gateway must be configured** — messaging requires tokens in the agent config (`gateway` section).

## Reference

These tools are available via the daemon agent (internal tools). External MCP agents can trigger messaging by chatting with the daemon agent or via goal runs that include notification steps.

### Gateway Configuration

Located in `~/.tamux/agent/config.json` under the `gateway` section:

```json
{
  "gateway": {
    "enabled": true,
    "slack_token": "xoxb-...",
    "telegram_token": "123456:ABC-DEF...",
    "discord_token": "...",
    "command_prefix": "!"
  }
}
```

### Tool: `send_slack_message`

| Param | Type | Required | Description |
|---|---|---|---|
| `channel` | string | Yes | Slack channel name or ID (e.g., `#dev-ops`, `C01234567`) |
| `message` | string | Yes | Message text (supports Slack mrkdwn formatting) |

### Tool: `send_discord_message`

| Param | Type | Required | Description |
|---|---|---|---|
| `channel` | string | Yes | Discord channel ID |
| `message` | string | Yes | Message text (supports Discord markdown) |

### Tool: `send_telegram_message`

| Param | Type | Required | Description |
|---|---|---|---|
| `chat_id` | string | Yes | Telegram chat ID |
| `message` | string | Yes | Message text |

### Tool: `send_whatsapp_message`

| Param | Type | Required | Description |
|---|---|---|---|
| `to` | string | Yes | WhatsApp phone number or group ID |
| `message` | string | Yes | Message text |

### Incoming Messages

The gateway also receives messages from connected platforms. The daemon polls for incoming messages and surfaces them as `GatewayIncoming` events with:

- `platform` — slack, discord, telegram
- `sender` — who sent the message
- `content` — message text
- `channel` — where it was sent

The daemon agent can respond to incoming gateway messages using the gateway command prefix.

### Tool: `notify_user`

For simpler notifications (not tied to a specific platform):

| Param | Type | Required | Description |
|---|---|---|---|
| `title` | string | Yes | Notification title |
| `body` | string | Yes | Notification body |
| `severity` | string | No | `info`, `warning`, `alert`, `error` |
| `channels` | array | No | Platform channels to notify |

## Gotchas

- Gateway tokens must be configured before messaging tools work — missing tokens cause silent failures.
- Slack uses Bot tokens (`xoxb-`) — ensure the bot has the right channel permissions.
- Discord requires the bot to be in the server and have send-message permissions.
- Messages are real and visible — there is no sandbox mode for messaging.
- The `command_prefix` in gateway config controls how incoming messages trigger daemon commands.
- WhatsApp integration may require additional setup (business API).
- Rate limits apply per platform — do not spam channels.
