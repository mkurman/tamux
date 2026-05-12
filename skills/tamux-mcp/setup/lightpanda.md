---
name: install-lightpanda
description: Install and configure Lightpanda headless browser for tamux web browsing
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

# Install Lightpanda — headless browser for tamux web browsing

## Agent Rules

- **Check if Lightpanda is already installed** before attempting installation — run `lightpanda --version` first
- **Prefer the prebuilt binary** over building from source — it's faster and more reliable
- **Verify the installation works** after install by running `lightpanda fetch --output dom-text https://example.com`
- **Configure tamux to use it** by setting `browse_provider` to `lightpanda` in agent settings after successful install

## Reference

### What is Lightpanda?

Lightpanda is a fast, Rust-native headless browser designed for AI agents. It renders JavaScript-heavy pages and returns clean DOM text — ideal for the tamux `fetch_url` tool which needs to browse real web pages.

When configured, tamux uses Lightpanda automatically for the `fetch_url` (web browse) tool instead of raw HTTP fetching. This means the agent can read JavaScript-rendered SPAs, dynamic content, and modern web apps.

### Installation Methods

#### Method 1: npm (recommended)

```bash
npm install -g @aspect-build/lightpanda
```

Verify:

```bash
lightpanda --version
```

#### Method 2: Prebuilt binary (Linux x86_64)

```bash
curl -LO https://github.com/nicholasgasior/lightpanda-releases/releases/latest/download/lightpanda-x86_64-linux
chmod +x lightpanda-x86_64-linux
sudo mv lightpanda-x86_64-linux /usr/local/bin/lightpanda
```

#### Method 3: Homebrew (macOS)

```bash
brew install nicholasgasior/tap/lightpanda
```

### Configuring tamux

After installation, configure tamux to use Lightpanda as the browse provider:

#### Option A: TUI settings

1. Open `/settings` in the TUI
2. Navigate to the **Web Search** tab
3. Set **Browser** to `lightpanda` (or leave as `auto` — it will be detected automatically)

#### Option B: Via agent config

The `browse_provider` setting in agent config controls which browser the `fetch_url` tool uses:

- `auto` — auto-detect: tries Lightpanda first, then Chrome, then raw HTTP (default)
- `lightpanda` — always use Lightpanda (fails if not installed)
- `chrome` — always use Chrome/Chromium headless
- `none` — raw HTTP only, no JavaScript rendering

### Verification

Test that the full pipeline works:

```bash
# Direct test
lightpanda fetch --output dom-text https://example.com

# Test via tamux agent — ask the agent to browse a page
# The agent should use fetch_url which will invoke Lightpanda
```

### Troubleshooting

- **`lightpanda: command not found`** — ensure the binary is on your PATH. If installed via npm, check `npm bin -g` is in PATH.
- **Timeout on complex pages** — Lightpanda has a default timeout; very heavy SPAs may need the Chrome fallback.
- **Permission denied** — ensure the binary is executable: `chmod +x $(which lightpanda)`
- **WSL users** — Lightpanda works in WSL2; no GUI required since it's headless.

## Gotchas

- Lightpanda is headless-only — no visual browser window. It outputs DOM text to stdout.
- Some sites with aggressive bot detection may block Lightpanda. For those, Chrome headless with its full browser fingerprint may work better.
- The `auto` browse_provider setting is recommended — it provides the best fallback chain.
