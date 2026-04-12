# Kublai Reader

<p align="center">
  A native RSS reader for Windows — three-pane layout, full-text extraction, AI summaries, and FreshRSS/Miniflux sync.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-Windows%2011-blue?style=flat-square" alt="Platform" />
  <img src="https://img.shields.io/badge/built%20with-Tauri%20v2-orange?style=flat-square" alt="Tauri" />
  <img src="https://img.shields.io/badge/license-MIT-green?style=flat-square" alt="License" />
</p>

---

> **Disclaimer: This is an entirely vibe-coded project. No human wrote any of the code — it was generated end-to-end by Claude (Anthropic). Use at your own risk. There are no guarantees of correctness, security, or stability.**

---

## Features

- Three-pane layout (sidebar, article list, reader) with drag-resizable panes
- Per-feed auto full-text extraction — set a feed to always load in reader mode
- AI article summaries via OpenRouter (press `A`)
- FreshRSS and Miniflux sync via Google Reader API
- Keyboard-driven: `W` extract, `R` read/unread, `S` star, `B` bookmark, `A` AI summary
- Magazine/grid view, compact view, card view
- SQLite FTS5 full-text search
- Favicon caching, OPML import/export, tag system
- Background feed refresh with cache cleanup
- Claude-inspired warm parchment theme (light) and near-black theme (dark)

## Download

See the [Releases](../../releases) page.

| File | Description |
|---|---|
| `Kublai-Reader_x.x.x_x64-setup.exe` | NSIS installer |
| `Kublai-Reader_x.x.x_x64_en-US.msi` | MSI installer |

## Requirements

- Windows 10 (1903+) or Windows 11
- WebView2 runtime (pre-installed on Windows 11)

## Building from Source

**Prerequisites:** Rust (stable MSVC), Node.js 18+, VS Build Tools 2022 with C++ workload.

```bash
git clone https://github.com/kartikeya-prasad/kublai-reader.git
cd kublai-reader
npm install
npm run tauri dev      # development
npm run tauri build    # release installers
```

## Tech Stack

| Layer | Technology |
|---|---|
| UI | Svelte 5 (runes) + TypeScript |
| Build | Vite 8 |
| Styling | Tailwind CSS v4 |
| Desktop | Tauri v2 |
| Backend | Rust |
| Database | SQLite (rusqlite + FTS5) |
| Feed parsing | feed-rs |
| AI | OpenRouter API |

## License

MIT
