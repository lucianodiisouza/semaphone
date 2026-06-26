# Semaphore

Floating traffic light for AI coding agents. Know at a glance when your agent is idle, thinking, or writing files.

| Light | Meaning |
|-------|---------|
| **Green** | Ready for a new task |
| **Yellow** | Thinking / running tools |
| **Red** | Writing or editing files |

## Download

Pre-built binaries for macOS (Apple Silicon & Intel), Linux, and Windows:

**[Download latest release](https://github.com/lucianodiisouza/semaphore/releases/latest)**

Each release includes the Semaphore desktop app (`.dmg`, `.msi`, `.deb`, or `.AppImage`) and the `semctl` CLI for hook installation.

## Quick start

1. Download a release for your OS (or [build from source](#development))
2. Launch Semaphore — it stays in the system tray
3. Open **Settings** and connect your AI tools
4. Use your tools normally; hooks update the light automatically

No terminal required for day-to-day use.

## Using the app

### Move the widget

**Click and drag the traffic light body** (the dark housing with the three lights). Do not drag the empty space around it — grab the semáforo itself.

On hover, a tooltip shows *"Click and drag here to move"* (or the Portuguese equivalent).

### Settings

Open settings in either way:

- **Hover** the widget and click the **⚙** button (top-right corner)
- **Right-click** the tray icon → **Settings**

Settings opens in its own window with theme, language, stealth mode, tool connection, and an About section.

### Tray menu

Right-click the Semaphore icon in the system tray:

| Menu item | Action |
|-----------|--------|
| **Show Semaphore** | Show the floating widget |
| **Hide Window** | Hide the floating widget |
| **Settings** | Open the settings window |
| **Toggle Stealth** | Hide the widget from screen capture |
| **Always on Top** | Keep the widget above other windows (not in macOS fullscreen) |
| **Quit** | Exit Semaphore |

Left-click the tray icon to show/focus the widget.

## Supported tools (v0.1)

| Tool | Status | Install |
|------|--------|---------|
| Cursor | Supported | Settings → Connect, or `semctl install cursor` |
| Claude Code | Supported | Settings → Connect, or `semctl install claude-code` |
| Codex CLI | Supported (Bash hooks; file edit limited) | Settings → Connect, or `semctl install codex` |
| Gemini CLI | Supported | Settings → Connect, or `semctl install gemini-cli` |
| Copilot CLI | Best-effort (varies by version) | Settings → Connect, or `semctl install copilot-cli` |

```bash
semctl install --all
semctl doctor
```

See [adapters/README.md](adapters/README.md) for per-tool hook mapping.

## Development

Requirements: Rust, Node.js 20+, npm.

```bash
npm install
npm run tauri dev
```

Build CLI:

```bash
cargo build -p semctl --release
```

## Architecture

```
AI tool hooks → sem-hook → semctl → Unix socket / named pipe → Semaphore app
```

- **sem-core** — state machine, session aggregation, IPC
- **semctl** — CLI for hooks and installer
- **semaphore** (Tauri) — floating UI, tray, settings window

## Themes & i18n

Built-in themes: Classic, Minimal, Neon (`src/themes/*.json`). English (default) and Portuguese. See `locales/CONTRIBUTING-i18n.md`.

## Stealth mode

Hides the window from many screen-capture tools. Works best on Windows; macOS 15+ may still capture in some apps. Enable in Settings.

## License

MIT — see [LICENSE](LICENSE).
