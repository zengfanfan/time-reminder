# TimeReminder

A lightweight desktop app that periodically covers your screen with a fullscreen reminder overlay — great for eye breaks, posture checks, water reminders, or anything else you tend to forget while deep in work.

Built with [Tauri v2](https://v2.tauri.app/) + [SvelteKit](https://svelte.dev/).

![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)
![License](https://img.shields.io/badge/license-MIT-blue)

---

## Features

- **Multiple reminders** — create as many as you need, each with its own interval and display duration
- **Fullscreen overlay** — a blurred, distraction-free overlay with a countdown timer
- **Sound alerts** — optional beep when the overlay appears
- **Enable / disable per reminder** — toggle without deleting
- **System tray** — runs quietly in the background; click the tray icon to show/hide
- **Launch at startup** — optional autostart on login
- **Bilingual** — English and Chinese (中文), auto-detected from system locale

## Screenshots

> Add your own screenshots here.

## Installation

Download the latest release for your platform from the [Releases](../../releases) page.

| Platform | Format |
|----------|--------|
| macOS    | `.dmg` |
| Windows  | `.msi` / `.exe` |
| Linux    | `.AppImage` / `.deb` |

## Building from Source

**Prerequisites:** [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/) (or [Bun](https://bun.sh/)), and the [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform.

```bash
# Clone the repo
git clone https://github.com/your-username/time-reminder.git
cd time-reminder

# Install frontend dependencies
bun install   # or: npm install

# Run in development mode
bun run tauri dev

# Build a release bundle
bun run tauri build
```

## Usage

1. Click **Add Reminder** to create a new reminder.
2. Set a title, message text, interval (how often it fires), and display duration (how long the overlay stays up).
3. Toggle the switch on the reminder card to enable or disable it.
4. The overlay will appear automatically — press the dismiss button or wait for the countdown to close it.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
