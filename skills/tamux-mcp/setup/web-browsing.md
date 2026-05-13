---
name: setup-web-browsing
description: Configure tamux web browsing with a headless browser (Lightpanda, Chrome, or Chromium)
compatibility:
  - tamux
  - claude-code
allowed_tools:
  - bash
  - write_file
  - read_file
metadata:
  category: setup
  platform: linux,macos,windows
---

# Setup Web Browsing — configure headless browser for tamux

## Agent Rules

- **Detect what's available first** — check for `lightpanda`, `google-chrome`, `chromium-browser`, `chromium` on PATH before installing anything
- **Prefer Lightpanda** — it's lightweight, fast, and purpose-built for agents
- **Fall back to Chrome/Chromium** — if already installed on the system, no extra install needed
- **Always verify after setup** — test with `fetch_url` tool on a known URL
- **Set `browse_provider` to `auto`** unless the user specifically requests a particular browser

## Reference

### How tamux web browsing works

The `fetch_url` tool in tamux can use a headless browser to render JavaScript before extracting page text. Without a browser, it falls back to raw HTTP (no JS rendering).

The `browse_provider` setting controls which browser is used:

| Value | Behavior |
|-------|----------|
| `auto` | Auto-detect: Lightpanda > Chrome > Chromium > raw HTTP |
| `lightpanda` | Use Lightpanda only |
| `chrome` | Use Chrome/Chromium headless only |
| `none` | Raw HTTP only (no JS rendering) |

### Step 1: Detect available browsers

Run these commands to check what's already on the system:

```bash
# Check Lightpanda
which lightpanda && lightpanda --version

# Check Chrome
which google-chrome-stable || which google-chrome

# Check Chromium
which chromium-browser || which chromium
```

### Step 2: Install a browser if needed

**If nothing is found**, install Lightpanda (recommended):

```bash
npm install -g @aspect-build/lightpanda
```

**If npm is not available**, install Chrome headless:

- **Ubuntu/Debian**: `sudo apt-get install -y chromium-browser`
- **Fedora**: `sudo dnf install -y chromium`
- **macOS**: `brew install --cask google-chrome` (or use existing Chrome)
- **Windows/WSL**: Chrome is usually already installed; ensure `google-chrome` is on PATH in WSL

### Step 3: Configure tamux

Set the browse provider in tamux settings:

#### Via TUI

1. `/settings` > **Web Search** tab
2. Set **Browser** to desired value (recommend: `auto`)

#### Via config directly

The `browse_provider` field in agent config. Value: `auto`, `lightpanda`, `chrome`, or `none`.

### Step 4: Verify

Ask the tamux agent to browse a JavaScript-heavy page:

```
Browse https://news.ycombinator.com and summarize the top stories
```

Or test directly:

```bash
# Lightpanda
lightpanda fetch --output dom-text https://example.com

# Chrome headless
google-chrome --headless=new --no-sandbox --disable-gpu --dump-dom https://example.com
```

### Detection logic

tamux auto-detects browsers in this order:

1. `lightpanda` on PATH → uses `lightpanda fetch --output dom-text <url>`
2. `google-chrome-stable` on PATH → uses `--headless=new --dump-dom <url>`
3. `google-chrome` on PATH → same flags
4. `chromium-browser` on PATH → same flags
5. `chromium` on PATH → same flags
6. None found → falls back to raw HTTP GET with HTML tag stripping

## Gotchas

- The `auto` setting re-detects on every `fetch_url` call — installing a browser mid-session works immediately.
- Chrome headless outputs full DOM HTML; Lightpanda can output clean text directly.
- Some corporate environments block browser installs — use `none` and rely on raw HTTP.
- The `web_browse` tool toggle in settings must be enabled for `fetch_url` to appear in the agent's tool list.
