---
name: install-skills
description: Import community skills into tamux from the registry or a direct URL, then verify they appear in the local skill catalog
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

# Install Skills — import reusable community skills into tamux

## Agent Rules

- **Use `tamux skill import <source>` as the canonical install flow**
- **Search first when the user does not provide an exact skill name** using `tamux skill search <query>`
- **Verify the imported skill is present** with `tamux skill list` or `tamux skill inspect <name>`
- **Use `--force` only when the user explicitly accepts bypassing security warnings**
- **Treat direct URLs and registry names differently** — keep the source explicit in the command you run

## Reference

### Primary install command

```bash
tamux skill import <source>
```

Where `source` can be:

- a registry skill name
- a direct URL to a published skill archive

### Discover skills before import

```bash
tamux skill search <query>
```

Examples:

```bash
tamux skill search browser
tamux skill search git
tamux skill search debugging
```

### Verify after import

```bash
tamux skill list
tamux skill inspect <name-or-variant-id>
```

Expected outcome:

- the imported skill appears in the local catalog
- its maturity/status is visible in the list output
- inspect returns the variant metadata and content details

### Security override

If the import flow warns about findings and the user explicitly wants to proceed anyway:

```bash
tamux skill import <source> --force
```

Use this only with user consent.

### Related commands

```bash
tamux skill export <name>
tamux skill publish <name>
tamux skill reject <name>
tamux skill promote <name> --to active
```

## Gotchas

- `tamux skill import` is the install command; there is no separate `tamux install skill` CLI flow
- imported skills can be blocked or downgraded by scan results; read the terminal output instead of assuming success
- a skill can appear under a variant ID rather than only the friendly name; `tamux skill inspect` handles both
- `--force` bypasses warnings, not hard blocks imposed by the daemon