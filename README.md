![Tamux Slavic Mythology](docs/assets/tamux-slavic.jpg)
# tamux

**Terminal Agentic Multiplexer and TUI with a super-powerful agent** - a daemon-first environment for long-running AI work.
>Tamux is not a CLI tool. It's a full autonomous agent platform. The mental model gap is huge - most people see "terminal multiplexer with AI" but the architecture underneath is a persistent, multi-agent, self-healing, auditable, learning agent
platform.

Official website: [https://tamux.app](https://tamux.app)

tamux keeps the terminal, the agent, and the runtime in one place. Sessions, threads, tasks, approvals, and goal runs live in the daemon, so work can keep moving even when the UI closes.

In practice that means:

- Electron, the TUI, the CLI, MCP clients, and chat gateways all reconnect to the same daemon state
- the built-in runtime can plan work, run tools, spawn bounded sub-agents, pause for approval, and learn over time
- memory, queue state, and operational history stay durable instead of vanishing with a single terminal tab

## Quick Start

### Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/mkurman/tamux/main/scripts/install.sh | sh
tamux --help
```

The quick installer downloads the same native release bundle family that the npm package uses, installs the binaries into `~/.local/bin` by default, and works without Node.js or npm.

After a quick-install or direct-binary install, `tamux upgrade` reuses the direct installer path for the current install directory. If you installed through npm, `tamux upgrade` continues to use npm.

### NPM

```bash
npm install -g tamux
tamux --help
```
```bash
# or install locally in a project:
npm install tamux
npx tamux --help
```

If `npm install -g tamux` succeeds but `tamux` is still not found on macOS, your npm global bin directory is not on `PATH` yet:

```bash
export PATH="$(npm config get prefix)/bin:$PATH"
exec $SHELL -l
tamux --help
```

If the bin directory is already on `PATH`, opening a new shell is still useful because `zsh` and `bash` can cache command lookups.

When tamux is installed through npm, `tamux upgrade` upgrades through npm as well.

### From sources

```bash
git clone https://github.com/mkurman/tamux.git

cd tamux

# 1. Start the daemon
cargo run --release --bin tamux-daemon

# 2. Launch the TUI in another terminal
cargo run --release --bin tamux setup

# or
cargo run --release --bin tamux-tui

# 3. Or launch the desktop app
cd frontend && npm install && npm run dev:electron
```

If you want to test the core loop fast, start a goal run and give Swarog a concrete objective: investigate a failing build, prepare a release checklist, or trace a bug across a workspace.

## What It Feels Like

tamux is for operators who want a terminal that remembers, an agent that can stay with a task, and a control surface that makes long-running work visible instead of mysterious.

It is still a real terminal multiplexer. It just has a daemon beneath it, durable autonomy above it, and enough structure to let automation run without turning into fog.

## The TUI

The TUI is a keyboard-first control room for the daemon with mouse support.

- inspect sessions, threads, tasks, approvals, and goal runs without leaving the terminal
- move between operator control and agent execution from the same live state used by Electron and the CLI
- keep working over SSH, inside tmux, or anywhere a browser UI is the wrong tool

Thread command reference: [`docs/tui/agent-directives.md`](docs/tui/agent-directives.md)

## The Fires

tamux gives its daemon-side agents a slightly mythic face, but the work stays concrete.

- **Swarog** is the main working fire: planning, tool use, sub-agent orchestration, memory, and durable goal runs
- **Rarog** is the guiding flame: onboarding, check-ins, operator context, and the gentler edge of the system
- **Weles** is the underwatch: governance, risk review, and guarded inspection when the runtime needs a second set of eyes
- **Swarozyc** is the quick worker: narrower, faster execution and focused implementation help
- **Radogost** is the negotiator: tradeoff analysis, comparison, and routing toward the strongest next move
- **Domowoj** is the keeper: local stability, cleanup, careful repair, and environment-aware fixes
- **Swietowit** is the wide watcher: broader architectural awareness and thread-level situational context
- **Perun** is the thunder hand: decisive execution, infrastructure discipline, and security-minded action
- **Mokosh** is the earth keeper: maintenance, reliability, and durable operational care
- **Dazhbog** is the bright giver: synthesis, explanation, and turning ambiguity into a useful next move

### Working With Them

Use `@agent ...` to add or update a visible thread participant on the current thread.

Examples:

- `@weles verify claims before answering`
- `@swarozyc review svarog's implementation and step in when needed`
- `@perun drive the risky migration plan to a concrete decision`
- `@mokosh stabilize this workspace and clean up the rough edges`
- `@dazhbog turn this discussion into a clear plan and operator update`

Behavior:

- the current thread owner does not change
- the participant watches visible thread activity
- when it decides to contribute, it adds a thread message or a queued visible suggestion instead of using hidden internal messages
- repeating `@agent ...` updates that participant instead of creating duplicates
- to stop or remove agent from participants list use @agent stop or @agent leave

Use `!agent ...` for hidden internal DM.

Examples:

- `!weles check whether this is risky`
- `!radogost compare these two approaches before we answer`
- `!perun assess the operational blast radius before we proceed`
- `!dazhbog summarize the strongest next move in plain language`

Behavior:

- the current visible thread stays where it is
- the target agent is contacted on the hidden internal path
- this is for behind-the-scenes coordination, not visible participation

Use handoff when another agent should own future replies in the thread.

Behavior:

- a handoff changes the active responder for the thread
- future operator messages route to that agent until a return handoff
- tamux records the switch as a visible system event while keeping linked handoff context hidden

### Registration And Setup

Builtin personas such as `@swarozyc`, `@radogost`, `@domowoj`, `@swietowit`, `@perun`, `@mokosh`, and `@dazhbog` need their own provider and model configuration before they can participate or receive internal DM.

If one of those personas is available but not configured yet:

- the desktop app opens a provider then model setup flow
- the TUI opens the same provider then model picker flow
- after setup, tamux retries the original `@agent ...` or `!agent ...` request automatically

`@veles` is accepted as an alias for `@weles`, matching the existing `swarog` and `svarog` alias handling.

If the target is not a known built-in persona or registered sub-agent alias, the leading `@agent` form is not treated as an agent directive and remains a normal message/file-mention path.


Together they give the system a little presence without hiding what it is doing.

## Read More

### Documentation

- [Reference](docs/reference.md)
- [Getting Started](docs/getting-started.md)
- [How tamux Works](docs/how-tamux-works.md)
- [Thread Participants](docs/operating/thread-participants.md)
- [Goal Runners](docs/goal-runners.md)
- [Self-Orchestrating Agent Architecture](docs/self-orchestrating-agent.md)
- [Agentic Mission Control](docs/agentic-mission-control.md)
- [TUI Capabilities Map](docs/tui/capabilities-map.md)
- [TUI README](crates/amux-tui/README.md)
- [CDUI YAML Views](docs/cdui-yaml-views.md)
- [Plugin Development](docs/plugin-development.md)

## License

[MIT](LICENSE)
