# Browser Automation — Canvas and sidebar browser control via tamux MCP tools

## Agent Rules

- **Always call `browser_read_dom` before interacting** — understand the page before clicking or typing
- **Use `browser_get_elements` to discover interactive elements** — don't guess selectors
- **Prefer text-based clicking over CSS selectors** — `text` parameter in `browser_click` is more robust than `selector`
- **Use `open_canvas_browser` to create browser panels** — returns a `pane` ID needed for all canvas browser tools
- **Sidebar browser tools (no `pane` param) are separate from canvas browser tools (require `pane` param)**
- **Read DOM after navigation** — pages need time to load; read DOM to verify content
- **Use `browser_eval_js` for complex interactions** — when click/type tools aren't sufficient, execute JS directly
- **Truncation warning** — DOM text is truncated to 12,000 chars, page text to 20,000 chars

## Reference

### Sidebar Browser Tools (no pane parameter needed)

#### `browser_navigate`

Navigate sidebar browser to a URL.

| Param | Type | Required | Description |
|---|---|---|---|
| `url` | string | Yes | URL to open (must start with https://) |
| `pane` | string | No | If provided, targets canvas browser instead |

#### `browser_back` / `browser_forward` / `browser_reload`

Navigation controls for the sidebar browser. No parameters.

#### `browser_read_dom`

Get page text, title, and URL.

| Param | Type | Required | Description |
|---|---|---|---|
| `pane` | string | No | Canvas browser pane ID (omit for sidebar) |

**Returns:** URL, title, and DOM text content (truncated to 12,000 chars).

#### `browser_take_screenshot`

Capture sidebar browser to vision storage. No parameters.

**Returns:** File path and page info.

### Canvas Browser Tools (require pane parameter)

#### `open_canvas_browser`

Create a new browser panel on the active canvas surface.

| Param | Type | Required | Description |
|---|---|---|---|
| `url` | string | No | Initial URL (default: https://google.com) |
| `name` | string | No | Panel display name |

**Returns:** `pane` ID for use in subsequent browser tool calls.

#### `browser_click`

Click an element in a canvas browser panel.

| Param | Type | Required | Description |
|---|---|---|---|
| `pane` | string | Yes | Canvas browser pane ID |
| `selector` | string | No | CSS selector of element |
| `text` | string | No | Visible text content to match (preferred over selector) |

Scrolls element into view before clicking. Supports A, BUTTON, and elements with leaf text nodes.

#### `browser_type`

Type text into an input field.

| Param | Type | Required | Description |
|---|---|---|---|
| `pane` | string | Yes | Canvas browser pane ID |
| `selector` | string | Yes | CSS selector of input element |
| `text` | string | Yes | Text to type |
| `clear` | boolean | No | Clear existing content first (default: true) |

Dispatches input and change events. Supports input, textarea, and contenteditable elements.

#### `browser_scroll`

Scroll page or element within a canvas browser panel.

| Param | Type | Required | Description |
|---|---|---|---|
| `pane` | string | Yes | Canvas browser pane ID |
| `direction` | string | Yes | "up" or "down" |
| `amount` | integer | No | Pixels to scroll (default: 400) |
| `selector` | string | No | CSS selector of scrollable element (default: window) |

#### `browser_get_elements`

List interactive elements on the page.

| Param | Type | Required | Description |
|---|---|---|---|
| `pane` | string | Yes | Canvas browser pane ID |
| `filter` | string | No | "links", "buttons", "inputs", or "all" (default: "all") |
| `limit` | integer | No | Max elements (default: 50, max: 200) |

**Returns:** Only visible elements with: tag, text, href, type, placeholder, CSS selector hint.

#### `browser_eval_js`

Execute JavaScript in the page context.

| Param | Type | Required | Description |
|---|---|---|---|
| `pane` | string | Yes | Canvas browser pane ID |
| `code` | string | Yes | JavaScript to evaluate |

**Returns:** JSON-serialized result (truncated to 12,000 chars). Has full access to document, window, DOM.

### Typical Workflow

```
1. open_canvas_browser(url="https://example.com") -> get pane ID
2. browser_read_dom(pane=ID) -> understand page
3. browser_get_elements(pane=ID, filter="inputs") -> find form fields
4. browser_type(pane=ID, selector="input[name=q]", text="search query")
5. browser_click(pane=ID, text="Search")
6. browser_read_dom(pane=ID) -> read results
```

## Gotchas

- Canvas browser tools REQUIRE a pane ID from `open_canvas_browser` -- calling them without one fails silently or errors.
- Sidebar browser is a separate instance -- sidebar tools and canvas tools target different browsers entirely.
- `browser_click` with `text` uses tree walker text matching, not substring -- use exact visible text.
- Only visible elements (in viewport, non-zero dimensions) are returned by `browser_get_elements`.
- No CDP (Chrome DevTools Protocol) -- tamux uses Electron webview, not headless Chrome.
- Screenshots only work on sidebar browser, not canvas panels.
- JavaScript execution is in page context -- cannot access Electron or Node.js APIs.
- The browser uses persistent partition `persist:amux-browser` -- cookies and session storage carry across panels.
