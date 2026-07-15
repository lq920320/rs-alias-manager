# rs-alias-manager

<p align="center">
  <img src="docs/logo.png" alt="rs-alias-manager" width="200" />
</p>

<p align="center">
  <strong>🦀 A modern desktop Shell alias manager built with Rust</strong>
</p>

<p align="center">
  <a href="README-CN.md">中文文档</a> ·
  <a href="#-background">Background</a> ·
  <a href="#-features">Features</a> ·
  <a href="#-quick-start">Quick Start</a> ·
  <a href="#-project-structure">Structure</a> ·
  <a href="#-tech-stack">Tech Stack</a> ·
  <a href="#-contributing">Contributing</a>
</p>

---

## Background

Shell aliases are a daily productivity booster for terminal users. However, managing them typically requires manually editing `.bashrc` or `.zshrc`, which is error-prone and tedious to sync across machines.

**rs-alias-manager** provides a visual alias management solution:

- No more manual config file editing — manage aliases through a GUI
- Built-in template library for quick environment setup on new machines
- Changes take effect immediately without `source` or terminal restart
- Supports Bash, Zsh, and Fish with automatic detection

## Features

| Feature | Description |
|---------|-------------|
| Alias List | Reads and parses shell config files, displays as card list |
| CRUD | Visual form for add/edit/delete with atomic file writes |
| Template Library | Pre-built templates for Git, Docker, File Ops, Network — one-click import |
| Search & Filter | Real-time search by alias name, command, or tags |
| Tags | Custom color-coded tags for organizing aliases |
| Import/Export | JSON format for cross-machine migration |
| Batch Operations | Batch add/delete aliases in a single operation |
| Multi Shell | Auto-detect and support Bash / Zsh / Fish |
| Dark Mode | Light/dark theme toggle, follows system preference |
| i18n | English and Chinese UI, switchable in Settings |
| Safe Write | Temp file + atomic rename prevents config corruption |
| Settings Cache | Backend caches settings to reduce file I/O |
| CSP Security | Content Security Policy configured for WebView |

## Quick Start

### Requirements

- **Rust** 1.75+ ([Install](https://www.rust-lang.org/tools/install))
- **macOS** 10.15+ / **Linux** (Wayland / X11) / **Windows** 10+
- **Trunk** (WASM build tool)

```bash
# Install Trunk
cargo install trunk

# Add WASM target
rustup target add wasm32-unknown-unknown
```

### Development

```bash
# Clone
git clone https://github.com/your-username/rs-alias-manager.git
cd rs-alias-manager

# Start dev mode (Tauri desktop window + hot reload)
cargo tauri dev
```

### Frontend Only (Browser Preview)

```bash
# Start frontend via Trunk (no Tauri backend, uses mock data)
trunk serve
# Open http://127.0.0.1:1420
```

### Production Build

```bash
cargo tauri build

# macOS: src-tauri/target/release/bundle/
# Linux: .deb / .AppImage
# Windows: .msi / .exe
```

### Installing Prebuilt Releases

Download from the [Releases page](../../releases).

**macOS**: Because the app is not signed with an Apple Developer certificate, macOS Gatekeeper may show a "file is damaged" error on first launch. This is a security policy, not actual corruption. To fix:

```bash
sudo xattr -rd com.apple.quarantine /Applications/rs-alias-manager.app
```

Alternatively, right-click the app in Finder → **Open** → confirm the security prompt.

**Windows**: SmartScreen may warn about an unrecognized publisher. Click **More info** → **Run anyway** to proceed.

## Project Structure

```
rs-alias-manager/
├── index.html                 # HTML entry
├── style.css                  # Global styles (dark theme variables)
├── Trunk.toml                 # Trunk build config
├── Cargo.toml                 # Frontend Rust dependencies
├── clippy.toml                # Clippy lint config
│
├── src/                       # Frontend (Leptos WASM)
│   ├── main.rs                # WASM entry, mounts root component
│   ├── app.rs                 # Root component, routing & layout
│   ├── i18n.rs                # Internationalization (EN/ZH)
│   ├── utils.rs               # Shared utility functions
│   ├── api/commands.rs        # Tauri backend API wrappers
│   ├── state/app_state.rs     # Global reactive state
│   ├── components/
│   │   ├── sidebar.rs         # Sidebar navigation + theme toggle
│   │   ├── alias_list.rs      # Alias list + multi-select
│   │   ├── alias_form.rs      # Alias add/edit form
│   │   ├── search_bar.rs      # Search filter bar
│   │   ├── template_category_tabs.rs  # Template category tabs
│   │   ├── template_list.rs   # Template list
│   │   └── settings_form.rs   # Settings form (language, shell, etc.)
│   └── pages/
│       ├── alias_page.rs      # Alias management page
│       ├── template_page.rs   # Template library page
│       └── settings_page.rs   # Settings page
│
├── src-tauri/                 # Backend (Tauri Rust)
│   ├── Cargo.toml
│   ├── tauri.conf.json        # Tauri app config
│   ├── clippy.toml            # Backend clippy config
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs             # Plugin registration + command handlers
│   │   ├── state.rs           # App state with settings cache
│   │   ├── error.rs           # Unified error types
│   │   ├── commands/          # Tauri commands
│   │   │   ├── alias_cmds.rs  # Alias CRUD + batch operations
│   │   │   ├── template_cmds.rs
│   │   │   └── settings_cmds.rs
│   │   ├── models/            # Data models
│   │   │   ├── alias.rs
│   │   │   ├── shell_type.rs
│   │   │   └── template.rs
│   │   └── services/          # Business logic
│   │       ├── alias_parser.rs    # Config file parsing (with tag comments)
│   │       ├── shell_config.rs    # Shell config read/write
│   │       ├── safe_writer.rs     # Atomic safe file writes
│   │       ├── template_library.rs # Template data (JSON-based)
│   │       ├── templates.json     # Externalized template definitions
│   │       └── app_settings.rs    # App settings persistence
│   └── icons/                 # App icons
│
└── docs/                      # Documentation
    ├── prd.md                 # Product requirements
    ├── architecture.md        # Architecture design
    ├── CODE_STYLE.md          # Code style guide
    ├── class-diagram.mermaid
    └── sequence-diagram.mermaid
```

## Tech Stack

| Layer | Technology | Description |
|-------|-----------|-------------|
| **Frontend** | [Leptos 0.8](https://leptos.dev/) | Rust WASM reactive UI framework (CSR) |
| **Routing** | [leptos_router 0.8](https://docs.rs/leptos_router/) | Client-side routing |
| **Desktop** | [Tauri v2](https://v2.tauri.app/) | Rust desktop app framework |
| **Build** | [Trunk](https://trunkrs.dev/) | WASM bundler & dev server |
| **Backend** | Rust | Pure Rust backend, no Node.js |
| **Plugins** | tauri-plugin-fs / dialog / shell | File system, dialogs, shell commands |
| **Styles** | Vanilla CSS | BEM naming, CSS custom properties theming |
| **i18n** | Custom (pure Rust) | Signal-driven, zero-dependency translation |

## Contributing

Contributions welcome! Bug reports, feature requests, and PRs are all appreciated.

### How to Contribute

1. **Fork** this repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'feat: add amazing feature'`
4. Push: `git push origin feature/amazing-feature`
5. Submit a **Pull Request**

### Development Guide

```bash
# Frontend check
cargo check
trunk build

# Backend check + tests
cd src-tauri && cargo check && cargo test
```

### Commit Convention

Follows [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation
- `style:` Style changes
- `refactor:` Code refactoring
- `perf:` Performance optimization
- `test:` Tests
- `chore:` Build/tooling changes

## License

[MIT License](LICENSE)

---

<p align="center">
  Made with Rust 🦀 and Tauri ❤️
</p>
