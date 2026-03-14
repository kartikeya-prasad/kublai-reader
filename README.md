# Kublai Reader

<p align="center">
  <img src="app-icon.png" alt="Kublai Reader" width="128" height="128" />
</p>

<p align="center">
  A beautiful, fast, native RSS reader for Windows 11 — with Mica effects, a three-pane layout, and a built-in article parser.
</p>

<p align="center">
  <a href="https://github.com/kartikeya/kublai-reader/releases/latest">
    <img src="https://img.shields.io/github/v/release/kartikeya/kublai-reader?style=flat-square&label=Download&color=6366f1" alt="Download" />
  </a>
  <img src="https://img.shields.io/badge/platform-Windows%2011-blue?style=flat-square" alt="Platform" />
  <img src="https://img.shields.io/badge/built%20with-Tauri%20v2-orange?style=flat-square" alt="Tauri" />
  <img src="https://img.shields.io/badge/license-MIT-green?style=flat-square" alt="License" />
</p>

---

## Features

- **Three-pane layout** — Feeds sidebar, article list, and article reader. All panes are drag-resizable.
- **Windows 11 Mica / Acrylic** — The window background follows the system theme with native blur effects.
- **Day / Night / Auto theme** — Follows your system setting automatically, or pin to light/dark.
- **Article parser** — Enable auto-parsing per feed, or click the "Parse" button on any article to extract clean, readable content using a built-in extractor.
- **Smart filters** — All, Unread, Starred, Read Later — always one click away.
- **Folder organisation** — Group feeds into nested folders by topic.
- **Auto mark-as-read on scroll** — Articles scroll past? They're marked read automatically.
- **Right-click context menu** — Mark Above as Read, Star, Read Later, Open in Browser, and more.
- **Full-text search** — SQLite FTS5 powers instant search across titles, summaries, and article content.
- **Background refresh** — Feeds update silently in the background at your chosen interval.
- **OPML import / export** — Bring your subscriptions from any other reader.
- **Reader typography** — Choose from serif and sans-serif fonts, adjust font size, line height, and content width.
- **Tags** — Colour-coded tags to organise articles your way.
- **Low memory footprint** — Rust backend keeps RAM usage minimal.

## Screenshots

> *(Screenshots coming soon)*

## Download

Head to the **[Releases](../../releases)** page and grab:

| File | Description |
|---|---|
| `Kublai-Reader_x.x.x_x64-setup.exe` | NSIS installer — recommended for most users |
| `Kublai-Reader_x.x.x_x64_en-US.msi` | MSI installer — for enterprise / Group Policy deployment |
| `kublai-reader.exe` | Portable executable — no installation needed, just run it |

## Requirements

- **Windows 10** (version 1903+) or **Windows 11**
- Mica effect requires Windows 11 22H2+; Acrylic is used as fallback on older builds.

## Building from Source

### Prerequisites

1. [Rust](https://rustup.rs/) (stable, MSVC toolchain)
2. [Node.js](https://nodejs.org/) v18+
3. [VS Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with:
   - **Desktop development with C++** workload
   - **Windows 11 SDK**

### Steps

```bash
# Clone the repo
git clone https://github.com/kartikeya/kublai-reader.git
cd kublai-reader

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Build release installers
npm run tauri build
```

Built artifacts will appear in:
```
src-tauri/target/release/                          ← portable kublai-reader.exe
src-tauri/target/release/bundle/nsis/              ← NSIS installer (.exe)
src-tauri/target/release/bundle/msi/               ← MSI installer
```

## Tech Stack

| Layer | Technology |
|---|---|
| UI framework | Svelte 5 (runes) + TypeScript |
| Build tool | Vite 6 |
| Styling | Tailwind CSS v4 + CSS custom properties |
| Desktop shell | Tauri v2 |
| Backend language | Rust 1.x (stable) |
| Database | SQLite (rusqlite + FTS5) |
| Feed parsing | feed-rs (RSS, Atom, JSON Feed) |
| HTTP | reqwest (async, gzip/brotli) |
| Async runtime | Tokio |

## Contributing

PRs and issues are very welcome! Please open an issue before starting large changes.

## License

MIT — see [LICENSE](LICENSE) for details.
