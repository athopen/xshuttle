# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build              # Build all crates
cargo build --release    # Build optimized release binary
cargo run                # Build and run xshuttle binary
cargo test --workspace   # Run all tests across workspace
cargo test -p config     # Run tests for a specific crate
cargo clippy --workspace # Run linter on all crates
cargo fmt                # Format all code
```

## Planning

When creating a new plan, always create a planning document in `./.claude/plans/`.

## Project Overview

xshuttle is a system tray application for Linux and macOS, written in Rust (2024 edition).

## Architecture

### Workspace Structure

```
xshuttle/
├── Cargo.toml          # Combined package + workspace manifest
├── src/                # Main binary
│   ├── main.rs         # Entry point, event loop
│   └── app.rs          # App logic, event handling
└── crates/             # Library crates
    ├── config/         # Configuration types & loading
    ├── terminal/       # Terminal detection & launching (platform-specific)
    ├── ssh/            # SSH config parsing
    └── tray/           # System tray & menu building
```

### Crate Dependencies

```
xshuttle (binary)
    ├── tray       → config
    ├── config
    ├── terminal
    └── ssh
```

### Internal Crates

| Crate | Purpose |
|-------|---------|
| `config` | Config file types, loading from `~/.xshuttle.json`, serde deserialization |
| `terminal` | Cross-platform terminal detection and command launching (Linux/macOS) |
| `ssh` | Parse `~/.ssh/config` to extract host names |
| `tray` | System tray icon, menu building from config entries |

### External Dependencies

- `tray-icon` - System tray icon (Tauri team)
- `muda` - Cross-platform menus
- `tao` - Event loop (handles GTK init on Linux, NSRunLoop on macOS)
- `image` - Icon loading
- `serde` / `serde_json` - Configuration serialization
- `ssh2-config` - SSH config parsing
- `which` - Binary detection (Linux)
- `open` - Open files with default application
