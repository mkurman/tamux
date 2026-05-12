# Building tamux Plugins

Develop custom plugins with tools, views, and components to extend tamux agent capabilities and UI.

## Agent Rules

- **Follow the existing plugin patterns** — look at `frontend/src/plugins/` for examples
- **Namespace everything** — plugin IDs prefix all components, commands, and tools
- **Plugins can contribute tools to the agent** — this is how you extend agent capabilities
- **Two plugin models exist:** in-tree (compiled with frontend) and runtime (npm-installed)
- **Test plugins in development mode** before publishing

## Reference

### Plugin Architecture

tamux supports two plugin models:

#### 1. In-Tree Plugins (Built-in)

Located at `frontend/src/plugins/<plugin-name>/`

Structure:
```
frontend/src/plugins/my-plugin/
├── registerPlugin.ts    # Entry point — registers components, commands, tools
├── MyComponent.tsx      # React components
└── index.ts            # Exports
```

Registration in `registerPlugin.ts`:
```typescript
import type { PluginRegistration } from "../../lib/pluginTypes";

export const myPlugin: PluginRegistration = {
  id: "my-plugin",
  name: "My Plugin",
  version: "1.0.0",
  register(api) {
    // Register React components
    api.registerComponent("my-plugin:dashboard", MyDashboard);

    // Register command handlers
    api.registerCommand("my-plugin:do-something", async (args) => {
      // Handle command
    });

    // Register agent tools
    api.registerTool({
      type: "function",
      function: {
        name: "my_custom_tool",
        description: "Does something useful for the agent",
        parameters: {
          type: "object",
          properties: {
            input: { type: "string", description: "The input to process" }
          },
          required: ["input"]
        }
      }
    }, async (args) => {
      // Tool executor — return result string
      return `Processed: ${args.input}`;
    });
  }
};
```

Import in app startup (e.g., `frontend/src/App.tsx` or plugin loader):
```typescript
import { myPlugin } from "./plugins/my-plugin/registerPlugin";
pluginManager.register(myPlugin);
```

#### 2. Runtime-Installed Plugins (npm)

Installed via CLI:
```bash
tamux install plugin <npm-package-name>
```

Package.json requirements:
```json
{
  "name": "tamux-plugin-example",
  "version": "1.0.0",
  "tamuxPlugin": {
    "entry": "dist/plugin.js"
  }
}
```

The entry script is a self-contained browser bundle that registers via the global API:
```javascript
window.TamuxApi.registerPlugin({
  id: "example",
  name: "Example Plugin",
  version: "1.0.0",
  register(api) {
    // Same API as in-tree plugins
    api.registerComponent("example:widget", MyWidget);
    api.registerCommand("example:action", handler);
    api.registerTool(toolDef, executor);
  },
  teardown() {
    // Cleanup when plugin is unloaded
  }
});
```

Plugin metadata stored at `~/.tamux/plugins/registry.json`.

### Plugin API Surface

The `api` object passed to `register()` provides:

| Method | Purpose |
|---|---|
| `api.registerComponent(name, ReactComponent)` | Register a React component (namespaced) |
| `api.registerCommand(name, handler)` | Register a command handler |
| `api.registerTool(definition, executor)` | Register an agent-callable tool |
| `api.registerView(yamlConfig)` | Register a YAML view definition |
| `api.onStartup(callback)` | Run code on plugin load |
| `api.onTeardown(callback)` | Run code on plugin unload |

### Contributing Agent Tools

This is the most powerful plugin capability — your plugin can extend what the AI agent can do.

Tool definition follows OpenAI function calling schema:
```typescript
const toolDef = {
  type: "function",
  function: {
    name: "analyze_logs",    // Tool name (must be unique)
    description: "Analyze application logs for error patterns and anomalies",
    parameters: {
      type: "object",
      properties: {
        log_path: {
          type: "string",
          description: "Path to the log file"
        },
        time_range: {
          type: "string",
          description: "Time range to analyze (e.g., '1h', '30m', '7d')"
        },
        severity: {
          type: "string",
          enum: ["error", "warning", "info", "all"],
          description: "Minimum severity level to include"
        }
      },
      required: ["log_path"]
    }
  }
};

const toolExecutor = async (args: { log_path: string; time_range?: string; severity?: string }) => {
  // Execute tool logic
  // Return a string result
  return JSON.stringify({ patterns_found: 3, summary: "..." });
};

api.registerTool(toolDef, toolExecutor);
```

The agent sees this tool alongside all built-in tools and can call it during conversations.

### Contributing YAML Views

Plugins can contribute UI views using tamux's YAML-based declarative UI (CDUI):

```typescript
api.registerView({
  schemaVersion: 1,
  title: "My Dashboard",
  layout: {
    type: "my-plugin:dashboard",
    props: { theme: "dark" }
  }
});
```

Views are persisted under `~/.tamux/views/plugins/`.

### Existing Plugin Examples

| Plugin | Location | What it does |
|---|---|---|
| `coding-agents` | `frontend/src/plugins/coding-agents/` | Integrates external coding agents (Hermes, OpenClaw) with runtime profiles, health checks |
| `ai-training` | `frontend/src/plugins/ai-training/` | Integrates AI training tools (Prime Intellect Verifiers, AutoResearch, AutoRL) |

Study these for patterns on:
- How to structure plugin registration
- How to contribute multiple tools
- How to manage plugin state
- How to integrate with external processes

### Runtime `plugin.json` Python Commands

Runtime-installed manifest plugins can also expose slash commands backed by Python execution plans.

Shape:
```json
{
  "python": {
    "run_path": "workspace",
    "source": "https://example.com/tool.py",
    "env": true,
    "dependencies": ["requests>=2.32"]
  },
  "commands": {
    "sync": {
      "description": "Run sync",
      "python": {
        "command": "python sync.py --full"
      }
    }
  }
}
```

Rules:
- `commands.<name>.python.command` is required
- top-level `python` provides defaults for `run_path`, `source`, `env`, and `dependencies`
- `source` must be an `http(s)` URL or an absolute path
- `env` may be a path string to `source` before execution or a boolean
- `env: true` prefers `uv` for `.venv` setup and falls back to `python -m venv`

### Skill Generation from Goal Runs

tamux goal runners can automatically generate reusable SKILL.md documents from successful execution trajectories. These are saved as markdown files that describe a repeatable procedure. This is a form of "plugin by documentation" — the agent learns procedures and can replay them in future goals.

Generated skills are stored alongside memory and referenced during planning of new goal runs.

## Gotchas

- Plugin IDs must be unique — collisions cause registration failures
- Component names must be namespaced as `plugin-id:component-name`
- Runtime plugins run in the browser context — no Node.js APIs available
- Tool names must be globally unique across all plugins and built-in tools
- Tool executors must return strings — JSON should be stringified
- YAML views require components to be registered in the component registry first
- In-tree plugins are compiled with the frontend — changes require a rebuild
- Runtime plugins are loaded dynamically — they must be self-contained browser bundles
- The plugin API may evolve — check for breaking changes when upgrading tamux
