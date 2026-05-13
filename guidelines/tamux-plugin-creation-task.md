---
name: tamux-plugin-creation-task
description: Use when creating a new Tamux plugin package, adding a plugin.json manifest, or preparing a plugin for install and validation.
recommended_skills:
  - plugin-creator
  - documentation-task
  - api-integration-task
---

# Tamux Plugin Creation Task Guideline

Use this guideline when the operator asks to create a Tamux plugin or package an integration as a plugin. A plugin should be a real extension boundary, not a place to hide unfinished core work.

## Decide The Plugin Shape

Before writing files, identify the smallest plugin model that fits the need:

1. API-backed plugin: use `plugin.json` with `api.endpoints`, `settings`, optional `auth`, and commands that route to endpoint `action` names. Use this for integrations like GitHub, GitLab, Gmail, or Calendar.
2. Python-backed plugin: use `plugin.json` with `python` defaults and command-level `python.command` entries. Use this for local workflows, repository tools, training scripts, or command wrappers.
3. Multi-plugin package: use one npm package that ships multiple plugin directories, each with its own `plugin.json`. Use this when the integrations are related but should install as separate Tamux plugins.
4. Frontend runtime plugin: use a self-contained browser script and `tamuxPlugin.entry` only when the plugin must register React components, views, commands, or assistant tools in the renderer. This is the legacy runtime package path and installs with `tamux install plugin`.

Do not force a frontend runtime plugin when a manifest-only plugin can express the integration.

## File Layout

Single manifest plugin package:

```text
tamux-plugin-example/
  package.json
  README.md
  example/
    plugin.json
    skills/
      example.md
```

Multi-plugin package:

```text
tamux-plugin-suite/
  package.json
  README.md
  first-plugin/
    plugin.json
  second-plugin/
    plugin.json
```

Python-backed plugin with bundled scripts:

```text
tamux-plugin-worker/
  package.json
  README.md
  worker/
    plugin.json
    scripts/
      worker_helper.py
    skills/
      worker.md
```

The outer `package.json` should publish the plugin directories:

```json
{
  "name": "tamux-plugin-example",
  "version": "1.0.0",
  "files": [
    "example/",
    "README.md"
  ]
}
```

## Manifest Rules

Every installable plugin directory must contain `plugin.json`.

Required fields:

```json
{
  "name": "example",
  "version": "1.0.0",
  "schema_version": 1
}
```

Use these manifest conventions:

1. Keep `name` stable, lowercase, and path-safe. It becomes the install directory under `~/.tamux/plugins/<name>/`.
2. Use semantic `version` strings such as `1.0.0`.
3. Set `schema_version` to `1`.
4. Add `description`, `author`, `license`, and `tamux_version` for user-facing plugins.
5. Put user-provided values in `settings`; mark tokens, passwords, client secrets, and private keys with `secret: true`.
6. Put reusable agent instructions in `skills` and ship those files inside the plugin directory.
7. Keep unknown future fields out unless the current code or docs require them.

## API-Backed Plugins

Use this shape when Tamux should call a remote HTTP API through the plugin API proxy:

```json
{
  "name": "example-api",
  "version": "1.0.0",
  "schema_version": 1,
  "description": "Example API integration",
  "tamux_version": ">=2.0.0",
  "settings": {
    "token": {
      "type": "string",
      "label": "API Token",
      "required": true,
      "secret": true,
      "description": "Token used for authenticated API requests"
    }
  },
  "api": {
    "base_url": "https://api.example.com",
    "endpoints": {
      "get_item": {
        "method": "GET",
        "path": "/v1/items/{{params.item_id}}",
        "headers": {
          "Authorization": "Bearer {{settings.token}}"
        },
        "response_template": "## {{name}}\n\n{{description}}"
      }
    },
    "rate_limit": {
      "requests_per_minute": 60
    }
  },
  "commands": {
    "item": {
      "description": "Fetch an item by id",
      "action": "get_item"
    }
  }
}
```

API plugin checklist:

1. Verify the provider's current auth, endpoint, pagination, and rate-limit behavior before writing the manifest.
2. Use `{{settings.key}}` for configured values, `{{params.key}}` for command inputs, and `{{auth.access_token}}` for OAuth tokens.
3. Use helpers such as `{{default params.per_page "20"}}` and `{{urlencode params.project}}` where existing plugin examples use them.
4. Keep response templates short and useful for the agent; avoid dumping entire API payloads unless the command requires it.
5. Ensure every `commands.<name>.action` points to an existing `api.endpoints` key.

## OAuth Plugins

Use `auth` when Tamux should manage OAuth2:

```json
{
  "auth": {
    "type": "oauth2",
    "authorization_url": "https://provider.example/oauth/authorize",
    "token_url": "https://provider.example/oauth/token",
    "scopes": ["read:data"],
    "pkce": true
  },
  "settings": {
    "client_id": {
      "type": "string",
      "label": "Client ID",
      "required": true,
      "secret": false
    },
    "client_secret": {
      "type": "string",
      "label": "Client Secret",
      "required": true,
      "secret": true
    }
  }
}
```

OAuth plugins should use the narrowest scopes that satisfy the command set.

## Python-Backed Plugins

Use this shape when a command should execute a local Python workflow:

```json
{
  "name": "example-worker",
  "version": "1.0.0",
  "schema_version": 1,
  "description": "Example local workflow",
  "python": {
    "run_path": "workspace",
    "env": true,
    "dependencies": ["requests>=2.32"]
  },
  "commands": {
    "check": {
      "description": "Validate the current workspace",
      "python": {
        "command": "python scripts/worker_helper.py check"
      }
    }
  }
}
```

Python plugin checklist:

1. `commands.<name>.python.command` is required and must be non-empty.
2. `python.source`, when used, must be an `http(s)` URL or absolute path.
3. `python.env: true` means Tamux should prefer `uv` for virtualenv setup and fall back to `python -m venv`.
4. Put shared defaults under top-level `python`; override only the command-specific values that differ.
5. Do not depend on untracked local scripts. Ship required scripts inside the plugin package or document the external prerequisite clearly.

## Frontend Runtime Plugins

Use this only when the plugin must run in the renderer and register UI/runtime extensions. The package exposes a `tamuxPlugin` field:

```json
{
  "name": "tamux-plugin-renderer-example",
  "version": "1.0.0",
  "tamuxPlugin": {
    "entry": "dist/tamux-plugin.js",
    "format": "script"
  }
}
```

The entry bundle must be self-contained and register itself:

```js
window.TamuxApi.registerPlugin({
  id: "renderer-example",
  name: "Renderer Example",
  version: "1.0.0",
  components: {},
  commands: {},
  onLoad() {},
  onUnload() {}
});
```

Runtime plugin rules:

1. Only `format: "script"` is currently supported.
2. The entry must already be built because `tamux install plugin` uses `npm install --ignore-scripts`.
3. Runtime plugin code runs in the browser context; use Electron bridge layers for filesystem, process, or host access.
4. Namespace contributed components, commands, and views with the plugin id.
5. Do not install this package with `tamux plugin add`; that command expects a v2 `plugin.json` manifest at the package root or in immediate subdirectories.

## Install And Validation Workflow

1. Validate `plugin.json` is valid JSON before installing.
2. Install a v2 manifest plugin from a local package or plugin directory:

```bash
tamux plugin add ./tamux-plugin-example
```

3. Install a legacy frontend runtime package only when it uses `tamuxPlugin.entry`:

```bash
tamux install plugin ./tamux-plugin-renderer-example
```

4. List installed plugins:

```bash
tamux plugin ls
```

5. List registered plugin commands:

```bash
tamux plugin commands
```

6. Enable or disable as needed:

```bash
tamux plugin enable example
tamux plugin disable example
```

7. Confirm files were copied to `~/.tamux/plugins/<name>/`.
8. For API plugins, run at least one read-only command with realistic parameters and verify the response template.
9. For Python plugins, run each command in a workspace that matches the declared prerequisites.
10. For frontend runtime plugins, launch Electron and check the renderer console for registration or load errors.

## Quality Gate

Do not consider a plugin finished until it installs cleanly, appears in `tamux plugin ls`, exposes the intended commands, and at least one real command path has been exercised. Never ship a plugin with placeholder commands, fake endpoint responses, untracked scripts, or undocumented secrets.
