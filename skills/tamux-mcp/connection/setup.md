# Connecting to tamux via MCP

How to register and connect to the tamux daemon through its MCP server interface.

## Agent Rules

- tamux-mcp is a JSON-RPC 2.0 server over stdio transport — all communication happens via stdin/stdout of the spawned process
- Always verify connection by calling `list_sessions` after setup — if you get a valid response, the connection is working
- Session IDs are UUIDs — never guess or hardcode them; always discover them dynamically
- All tools return JSON responses — parse them accordingly

## Reference

### What is tamux-mcp?

tamux-mcp is a standalone binary that exposes the tamux daemon as a Model Context Protocol (MCP) server. It connects to the running tamux daemon via Unix socket (Linux/macOS) or TCP (Windows) and translates MCP tool calls into daemon commands.

### Registering with Claude Code / Cursor

Add to your MCP config (e.g., `.claude/claude.json` or Cursor's MCP settings):

```json
{
  "mcpServers": {
    "tamux": {
      "command": "tamux-mcp"
    }
  }
}
```

No additional arguments are required. The binary auto-discovers the daemon socket.

### Registering with Claude Agent SDK / Custom Agents

Spawn `tamux-mcp` as a subprocess with stdio communication. Send JSON-RPC messages on stdin, read responses from stdout.

**Step 1 — Initialize the MCP session:**

```json
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "my-agent", "version": "1.0"}}}
```

**Step 2 — Send the initialized notification:**

```json
{"jsonrpc": "2.0", "method": "initialized"}
```

**Step 3 — List available tools:**

```json
{"jsonrpc": "2.0", "id": 2, "method": "tools/list"}
```

**Step 4 — Call a tool:**

```json
{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "list_sessions", "arguments": {}}}
```

### Transport Framing

| Mode | Description | How to enable |
|---|---|---|
| Newline-delimited JSON | One JSON object per line (default) | No configuration needed |
| Content-Length framing | HTTP-style `Content-Length` header before each message | Set `TAMUX_MCP_FRAMING=content-length` env var |

The read side auto-detects the framing format, so a Content-Length client can talk to a newline-delimited server and vice versa.

### Prerequisites

- **tamux daemon must be running** — the `tamux-daemon` process must be active before starting `tamux-mcp`
- **`tamux-mcp` binary must be in PATH** — or provide the full path in your MCP config
- **Socket / port availability:**
  - Linux/macOS: Unix socket at `$XDG_RUNTIME_DIR/tamux-daemon.sock` or `/tmp/tamux-daemon.sock`
  - Windows: TCP at `localhost:17563`

### Verification

After registration, call `list_sessions`. If you get a response containing session data (even an empty array), the connection is working correctly.

## Gotchas

- **Daemon must be running first.** If `tamux-mcp` fails to start or returns connection errors, ensure the tamux daemon is running. The MCP server needs the daemon socket — it will not work if the daemon has not created it yet.
- **Windows uses TCP, not Unix sockets.** Do not look for a socket file on Windows; the daemon listens on `localhost:17563` instead.
- **The `initialize` handshake is mandatory.** The handshake must complete (both `initialize` request and `initialized` notification) before any `tools/call` requests. Sending tool calls before initialization will result in errors.
- **Do not hardcode session IDs.** Sessions are ephemeral and identified by UUIDs that change between daemon restarts. Always discover them via `list_sessions`.
- **Framing mismatch on write side.** While the read side auto-detects framing, make sure your write side consistently uses one framing format. Mixing formats in outgoing messages will confuse the server.
