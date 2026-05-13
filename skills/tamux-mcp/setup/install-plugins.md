---
name: install-plugins
description: Install tamux runtime plugins from npm, GitHub, or a local directory and verify they are registered with the daemon
compatibility:
  - tamux
  - claude-code
allowed_tools:
  - bash
  - read_file
metadata:
  category: setup
  platform: linux,macos,windows
---

# Install Plugins — add runtime extensions to tamux

## Agent Rules

- **Prefer `tamux plugin add <source>`** for current runtime plugin installs
- **Allow `tamux install plugin <package>` only as the legacy npm-compatible shortcut**
- **Verify the daemon sees the plugin** after installation by running `tamux plugin ls`
- **Use plugin source that is explicit** — npm package name, GitHub URL, or local directory path
- **Do not claim success on file copy alone** — plugin install is only complete once registration succeeds or the daemon-facing status is explained clearly

## Reference

### What this installs

Runtime plugins extend tamux with extra commands, tools, settings, and UI integrations. They are installed into the tamux data directory and then registered with the daemon so they become active.

### Preferred command

```bash
tamux plugin add <source>
```

Supported `source` forms:

- npm package name: `tamux plugin add tamux-plugin-example`
- GitHub URL: `tamux plugin add https://github.com/org/repo`
- local directory: `tamux plugin add /path/to/plugin-dir`

### Legacy compatibility command

For npm-style installs only, the older shorthand still works:

```bash
tamux install plugin <package>
```

Use this only when the user explicitly asks for the legacy install form or already uses it.

### Verify after install

Run:

```bash
tamux plugin ls
```

Expected outcome:

- the plugin appears in the installed list
- it shows as enabled if registration succeeded
- daemon registration errors are surfaced clearly if activation failed

### Useful follow-up commands

```bash
tamux plugin commands
tamux plugin disable <name>
tamux plugin enable <name>
tamux plugin remove <name>
```

## Gotchas

- `tamux plugin add` is the primary v2 flow; prefer it over the legacy install shortcut
- local installs must point to a plugin directory, not a single file
- registration can fail even if files were copied successfully; always verify with `tamux plugin ls`
- some plugins expose commands only after the daemon refreshes its plugin registry