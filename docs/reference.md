# tamux Reference

This document collects the practical reference material that used to live in the top-level README: providers, configuration, shortcuts, packaging, runtime integration, and development notes.

For onboarding, see [getting-started.md](getting-started.md). For runtime architecture, see [how-tamux-works.md](how-tamux-works.md). For deeper agent internals, see [self-orchestrating-agent.md](self-orchestrating-agent.md). For the canonical memory architecture, see [tamux-memory.md](tamux-memory.md). For the canonical security and governance model, see [tamux-security.md](tamux-security.md).

## Paths And Locations

### Config File Locations

| Platform | Path |
|---|---|
| Linux | `~/.config/tamux/config.json` |
| macOS | `~/Library/Application Support/tamux/config.json` |
| Windows | `%APPDATA%\tamux\config.json` |

Data directory: `~/.tamux/` on Unix, `%LOCALAPPDATA%\tamux\` on Windows. Existing `amux` directories are migrated forward when possible.

### Install Locations

- Desktop app binaries (packaged): inside application resources, typically `resources/bin`
- Runtime data directory: `~/.tamux` on Unix or `%LOCALAPPDATA%\tamux` on Windows
- Source builds: binaries are produced in `target/debug` or `target/release`

## Supported Providers

| Provider | ID | Default Model | Base URL |
|---|---|---|---|
| Featherless | `featherless` | meta-llama/Llama-3.3-70B-Instruct | api.featherless.ai |
| OpenAI | `openai` | gpt-4o | api.openai.com |
| Anthropic | `anthropic` | claude-sonnet-4-20250514 | api.anthropic.com |
| Qwen | `qwen` | qwen-max | api.qwen.com |
| Qwen (DeepInfra) | `qwen-deepinfra` | Qwen/Qwen2.5-72B-Instruct | api.deepinfra.com |
| Kimi (Moonshot) | `kimi` | moonshot-v1-32k | api.moonshot.ai |
| Kimi Coding Plan | `kimi-coding-plan` | kimi-for-coding | api.kimi.com/coding |
| Z.AI (GLM) | `z.ai` | glm-4-plus | api.z.ai |
| Z.AI Coding Plan | `z.ai-coding-plan` | glm-5 | api.z.ai/api/coding/paas/v4 |
| OpenRouter | `openrouter` | arcee-ai/trinity-large-thinking | openrouter.ai |
| Cerebras | `cerebras` | llama-3.3-70b | api.cerebras.ai |
| Together | `together` | meta-llama/Llama-3.3-70B-Instruct-Turbo | api.together.xyz |
| Groq | `groq` | llama-3.3-70b-versatile | api.groq.com |
| Ollama (local) | `ollama` | llama3.1 | localhost:11434 |
| Chutes | `chutes` | deepseek-ai/DeepSeek-V3 | llm.chutes.ai |
| Hugging Face | `huggingface` | meta-llama/Llama-3.3-70B-Instruct | api-inference.huggingface.co |
| MiniMax | `minimax` | MiniMax-M1-80k | api.minimax.io |
| MiniMax Coding Plan | `minimax-coding-plan` | MiniMax-M2.7 | api.minimax.io/anthropic |
| Alibaba Coding Plan | `alibaba-coding-plan` | qwen3-coder | coding-intl.dashscope.aliyuncs.com |
| OpenCode Go | `opencode-go` | glm-5.1 | opencode.ai/zen/go |
| OpenCode Zen | `opencode-zen` | claude-sonnet-4-5 | opencode.ai/zen |
| Custom | `custom` | user-defined | user-defined |

### API Formats

- Most providers use OpenAI-compatible `/chat/completions` endpoints
- Anthropic, MiniMax, and MiniMax Coding Plan use the Anthropic Messages API at `/v1/messages`
- Alibaba Coding Plan supports both OpenAI-compatible `/v1` and Anthropic-compatible `/apps/anthropic`, auto-selected by model name
- OpenCode Go uses Anthropic Messages for `minimax-m2.x` models and OpenAI-compatible chat completions for the rest
- OpenCode Zen auto-selects Anthropic format for Claude models and OpenAI-compatible format for others

Switch providers at any time from the Settings panel. Each provider's base URL, model, and API key are independently configurable, and models can be selected from a searchable list or entered manually.

When OpenRouter is selected, tamux automatically sends app attribution headers using `https://tamux.app` and the `tamux` title so usage can appear in OpenRouter analytics and rankings.

## Build And Run

### Build

```bash
# Rust workspace
cargo build --release

# Individual Rust crates
cargo build --release -p tamux-daemon
cargo build --release -p tamux-cli
cargo build --release -p tamux-gateway
cargo build --release -p tamux-mcp
cargo build --release -p tamux-protocol

# Frontend web bundle
cd frontend
npm install
npm run build

# Electron desktop package
cd frontend
npm run build:electron
```

Recommended slice:

- backend-only changes: `cargo build --release` or the specific `-p` crate
- frontend UI changes: `cd frontend && npm run build`
- Electron bridge or preload changes: `cd frontend && npm run build` and then `cd frontend && npm run dev:electron`
- packaging changes: `cd frontend && npm run build:electron`

### Run

```bash
# Start the daemon
cargo run --release --bin tamux-daemon

# Launch the Electron app
cd frontend
npm run dev:electron

# Or use the CLI directly
cargo run --release --bin tamux -- list
cargo run --release --bin tamux -- new --shell bash
cargo run --release --bin tamux -- attach <session-id>
cargo run --release --bin tamux -- kill <session-id>
cargo run --release --bin tamux -- ping
```

## Release Packaging

Use the platform-specific scripts when you want upload-ready artifacts gathered under `dist-release/`.

```bash
# Linux native release bundle
./scripts/build-release.sh

# Rebuild the full dist-release layout
./scripts/build-production-releases.sh

# Native macOS release bundle (run on macOS)
./scripts/build-release-macos.sh

# Windows cross-build from WSL/Linux
./scripts/build-release-wsl.sh
```

Windows native builds:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
scripts\build-release.bat
```

The production wrapper `./scripts/build-production-releases.sh` deletes and recreates `dist-release/`, builds the native Linux release into `dist-release/linux/`, and builds the Windows cross-release into `dist-release/windows/` when `mingw-w64` is installed.

Typical output layout:

```text
dist-release/
  linux/
    tamux
    tamux-daemon
    tamux-gateway
    tamux-mcp
    GETTING_STARTED.md
    tamux-<version>.AppImage
    tamux_<version>_amd64.deb
    tamux-linux-x86_64.zip
    tamux-linux-aarch64.zip
    SHA256SUMS.txt
    RELEASE_NOTES.md
  windows/
    tamux.exe
    tamux-daemon.exe
    tamux-gateway.exe
    tamux-mcp.exe
    GETTING_STARTED.md
    tamux-portable.exe
    tamux Setup <version>.exe
    tamux-windows-x64.zip
    SHA256SUMS.txt
    RELEASE_NOTES.md
```

Wrapper options:

- `--native-only` builds only the native platform release
- `--windows-only` builds only the Windows cross-release
- `--skip-rust`, `--skip-frontend`, and `--skip-electron` are passed through to the native release script
- `--target <triple>` is passed through to the native release script
- `--sign` enables signing in child scripts

Host expectations:

- Linux host: can build Linux Rust binaries and Linux Electron artifacts
- Linux/WSL host: can also cross-compile Windows Rust binaries and package Windows Electron artifacts when `mingw-w64` is installed
- macOS host: required for signed macOS app bundles, DMGs, and notarization
- Windows host: required for the most reliable signed Windows installers

Recommended prerequisites on Linux:

```bash
sudo apt update
sudo apt install -y mingw-w64
cargo build --release
cd frontend && npm ci && cd ..
```

Signing environment variables:

- `TAMUX_SIGN_CERT` / `TAMUX_SIGN_PASSWORD` for PFX-based signing
- `TAMUX_SIGN_THUMBPRINT` for Windows certificate store signing
- `TAMUX_SIGN_IDENTITY` for macOS `codesign`

Legacy `AMUX_*` signing variables are still accepted for compatibility.

## Notifications

tamux supports in-app attention notifications emitted from terminal output using OSC sequences.

Supported formats:

- `OSC 9`: `9;<message>`
- `OSC 777`: `777;notify;<title>;<body>`
- `OSC 99`: `99;<text>` or metadata + `;<text>`

You can open the in-app notification panel with `Ctrl+I`.

Quick smoke test commands:

```bash
printf '\033]9;Claude needs attention\007'
printf '\033]777;notify;Claude;Waiting for your input\007'
printf '\033]99;Codex finished task\007'
```

Optional shell helpers:

```bash
osc9() {
  printf '\033]9;%s\007' "$*"
}

osc777() {
  local title="$1"; shift
  printf '\033]777;notify;%s;%s\007' "$title" "$*"
}

osc99() {
  printf '\033]99;%s\007' "$*"
}
```

## Configuration

The built-in Settings panel exposes configuration in these sections:

- Appearance: font family, font size, theme, opacity, line height, padding, and custom terminal colors
- Cursor: style, blink toggle, and blink interval
- Terminal: default shell, shell arguments, scrollback, bell, visual bell, bracketed paste
- Behavior: session restore, close confirmation, auto-copy, URL opening, auto-save, transcript capture, and retention periods
- Infrastructure: sandbox, network policy, snapshot backend, WORM integrity, Cerbos PDP endpoint
- Agent: provider selection, per-provider API keys and models, system prompt, streaming, memory, compaction, bash tool, and web search settings
- Keybindings: custom key combinations and reset-to-defaults

## Keyboard Shortcuts

All keybindings are customizable from the Settings panel or by editing `keybindings.json`.

### Pane Management

| Action | Default Binding |
|---|---|
| Split horizontal | `Ctrl+D` |
| Split vertical | `Ctrl+Shift+D` |
| Close active pane | `Ctrl+Shift+W` |
| Toggle zoom pane | `Ctrl+Shift+Z` |
| Focus left pane | `Ctrl+Alt+Left` |
| Focus right pane | `Ctrl+Alt+Right` |
| Focus upper pane | `Ctrl+Alt+Up` |
| Focus lower pane | `Ctrl+Alt+Down` |

### Surfaces

| Action | Default Binding |
|---|---|
| New surface | `Ctrl+T` |
| Close surface | `Ctrl+W` |
| Next surface | `Ctrl+Tab` |
| Previous surface | `Ctrl+Shift+Tab` |

### Workspaces

| Action | Default Binding |
|---|---|
| New workspace | `Ctrl+Shift+N` |
| Switch workspace 1-9 | `Ctrl+1` through `Ctrl+9` |
| Next workspace | `Ctrl+PageDown` |
| Previous workspace | `Ctrl+PageUp` |

### Panels And Overlays

| Action | Default Binding |
|---|---|
| Command palette | `Ctrl+Shift+P` |
| Toggle sidebar | `Ctrl+B` |
| Toggle notifications | `Ctrl+I` |
| Toggle settings | `Ctrl+,` |
| Toggle session vault | `Ctrl+Shift+V` |
| Toggle command log | `Ctrl+Shift+L` |
| Toggle search | `Ctrl+Shift+F` |
| Toggle command history | `Ctrl+Alt+H` |
| Toggle snippets | `Ctrl+S` |
| Toggle agent panel | `Ctrl+Shift+A` |
| Toggle system monitor | `Ctrl+Shift+M` |
| Toggle execution canvas | `Ctrl+Shift+G` |
| Toggle time-travel snapshots | `Ctrl+Shift+T` |

## Plugins And MCP

Runtime-installed plugins are supported through `tamux install plugin <npm-package-or-local-path>`.

External npm plugins should declare `tamuxPlugin.entry` in `package.json`. The legacy `amuxPlugin.entry` field is still accepted for compatibility. The entry should be a self-contained browser script that registers itself through `window.TamuxApi.registerPlugin(...)` or `window.AmuxApi.registerPlugin(...)`.

Register `tamux-mcp` with Claude Code, Cursor, or any MCP-compatible client like this:

```json
{
  "mcpServers": {
    "tamux": {
      "command": "tamux-mcp"
    }
  }
}
```

For deeper plugin details, see [plugin-development.md](plugin-development.md). For MCP connection setup, see [skills/connection/setup.md](skills/connection/setup.md).

## Development Reference

### Crate Layout

| Crate | Role |
|---|---|
| `tamux-protocol` | Shared message types, length-prefixed bincode codec, and configuration |
| `tamux-daemon` | Background daemon: PTY management, task queue, snapshots, policy engine, credential scrubbing, telemetry, and history |
| `tamux-cli` | Command-line client that builds the `tamux` binary |
| `tamux-gateway` | Chat platform bridge crate |
| `tamux-mcp` | MCP server crate |
| `tamux-tui` | Keyboard-first terminal UI client for the daemon |

### Key Dependencies

Rust side:

| Crate | Purpose |
|---|---|
| `tokio` | Async runtime |
| `portable-pty` | Cross-platform PTY spawning |
| `rusqlite` | SQLite with FTS5 |
| `tree-sitter` / `tree-sitter-bash` | AST parsing and indexing |
| `sysinfo` | System telemetry |
| `sha2` | WORM hash chains |
| `regex` | Risk matching and credential scrubbing |
| `serde` / `serde_json` / `bincode` | Serialization for IPC and persistence |

Frontend side:

| Package | Purpose |
|---|---|
| `@xterm/xterm` | Terminal emulation |
| `@xyflow/react` | Execution graph UI |
| `react-resizable-panels` | Split pane layout |
| `zustand` | State management |
| `vite` | Build tooling and dev server |
| `electron` | Desktop shell |
| `electron-builder` | Packaging |

### IPC Protocol

The daemon and all clients communicate via length-prefixed bincode frames over Unix domain sockets on Linux/macOS or localhost TCP on Windows. The protocol lives in `amux-protocol` and is centered on two enums:

- `ClientMessage`: requests from clients to the daemon
- `DaemonMessage`: responses and events from the daemon back to clients
