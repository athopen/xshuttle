# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build          # Build the project
cargo build --release # Build optimized release binary
cargo run            # Build and run
cargo test           # Run all tests
cargo test <name>    # Run a specific test by name
cargo clippy         # Run linter
cargo fmt            # Format code
```

## Planning

When creating a new plan, always create a planning document in `./.claude/plans/`.

## Project Overview

xshuttle is a system tray application for Linux and macOS, written in Rust (2024 edition).

## Architecture

### Crates Used
- `tray-icon` - System tray icon (Tauri team)
- `muda` - Menus
- `tao` - Event loop (handles GTK init on Linux, NSRunLoop on macOS)