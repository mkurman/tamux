# Workspace Layout Management

Manage workspaces, surfaces, panes, and snippets to organize your terminal environment.

## Agent Rules

- **Understand the hierarchy:** Workspace → Surface (tab) → Pane (terminal/browser panel)
- **Use `list_sessions`** to see the current workspace/surface/pane structure before making changes
- **Use layout presets for common arrangements** — faster than manual splits
- **Don't create unnecessary panes** — reuse existing ones when possible
- **Snippets are reusable text fragments** — use them for frequently-run command sequences

## Reference

These tools are available via the daemon agent (internal tools), not exposed directly via MCP. External agents can request workspace changes by chatting with the daemon agent or via goal runs.

### Workspace Tools

**`list_workspaces`** — List all workspaces with their surface and pane structure.
No parameters. Returns workspace hierarchy.

**`create_workspace`** — Create a new workspace.
| Param | Type | Required | Description |
|---|---|---|---|
| `title` | string | No | Workspace display name |

**`set_active_workspace`** — Switch to a workspace.
| Param | Type | Required | Description |
|---|---|---|---|
| `workspace_id` | string | Yes | Target workspace ID |

### Surface Tools

**`create_surface`** — Create a new surface (tab) within the active workspace.
| Param | Type | Required | Description |
|---|---|---|---|
| `title` | string | No | Surface display name |

**`set_active_surface`** — Switch to a surface within the current workspace.
| Param | Type | Required | Description |
|---|---|---|---|
| `surface_id` | string | Yes | Target surface ID |

### Pane Tools

**`split_pane`** — Split a pane horizontally or vertically.
| Param | Type | Required | Description |
|---|---|---|---|
| `direction` | string | Yes | `horizontal` or `vertical` |
| `pane_id` | string | No | Pane to split (defaults to active pane) |

**`rename_pane`** — Rename a pane for easier identification.
| Param | Type | Required | Description |
|---|---|---|---|
| `pane_id` | string | Yes | Target pane ID |
| `name` | string | Yes | New display name |

### Layout Presets

**`set_layout_preset`** — Apply a preset layout to the current surface.
| Param | Type | Required | Description |
|---|---|---|---|
| `preset` | string | Yes | One of: `single`, `2-columns`, `3-columns`, `grid-2x2`, `main-stack` |

Preset descriptions:
- `single` — One full-screen pane
- `2-columns` — Two side-by-side panes
- `3-columns` — Three equal columns
- `grid-2x2` — Four panes in a 2x2 grid
- `main-stack` — Large left pane, stacked smaller panes on right

**`equalize_layout`** — Balance all split ratios to equal sizes.
No parameters.

### Snippet Tools

**`list_snippets`** — List all saved snippets.
No parameters. Returns array of snippet objects.

**`create_snippet`** — Create a reusable text snippet.
| Param | Type | Required | Description |
|---|---|---|---|
| `title` | string | Yes | Snippet display name |
| `content` | string | Yes | Snippet text content |

**`run_snippet`** — Execute a snippet in a pane.
| Param | Type | Required | Description |
|---|---|---|---|
| `snippet_id` | string | Yes | Snippet UUID |
| `pane_id` | string | No | Target pane (defaults to active) |

### Example: Setting Up a Development Environment

```
1. create_workspace(title="Backend Dev")
2. set_layout_preset(preset="main-stack")
3. rename_pane(pane_id="pane_1", name="Editor")
4. rename_pane(pane_id="pane_2", name="Tests")
5. rename_pane(pane_id="pane_3", name="Logs")
6. create_snippet(title="Run Tests", content="cargo test --workspace\n")
7. run_snippet(snippet_id="...", pane_id="pane_2")
```

## Gotchas

- Workspace tools are daemon-internal, not exposed via MCP — use chat or goal runs
- Layout presets replace the current pane arrangement — existing panes may be reorganized
- Pane IDs change when layout presets are applied — re-query with `list_sessions` after
- Snippets persist across daemon restarts
- `equalize_layout` only affects the current surface
