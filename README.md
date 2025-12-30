<p align="center">
  <img src="https://raw.githubusercontent.com/athopen/xshuttle/master/assets/icon.svg" width="128" height="128" alt="xshuttle icon">
</p>

<h1 align="center">xshuttle</h1>

<p align="center">
  <a href="https://github.com/athopen/xshuttle/actions/workflows/ci.yml"><img src="https://github.com/athopen/xshuttle/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/athopen/xshuttle/actions/workflows/release.yml"><img src="https://github.com/athopen/xshuttle/actions/workflows/release.yml/badge.svg" alt="Release"></a>
  <a href="LICENSE.md"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
</p>

A system tray application that displays your SSH hosts from `~/.ssh/config` for quick one-click connections.

## Features

- Reads hosts from `~/.ssh/config` automatically
- One-click to open SSH connection in your terminal
- Supports Linux and macOS
- Lightweight, runs in the background

## Installation

### Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/athopen/xshuttle/master/install.sh | bash
```

### Manual Install

Download the latest release for your platform from [Releases](https://github.com/athopen/xshuttle/releases) and extract to `/usr/local/bin`:

```bash
# Linux (x86_64)
curl -fsSL https://github.com/athopen/xshuttle/releases/latest/download/xshuttle-x86_64-unknown-linux-gnu.tar.gz | sudo tar xz -C /usr/local/bin

# macOS (Intel)
curl -fsSL https://github.com/athopen/xshuttle/releases/latest/download/xshuttle-x86_64-apple-darwin.tar.gz | sudo tar xz -C /usr/local/bin

# macOS (Apple Silicon)
curl -fsSL https://github.com/athopen/xshuttle/releases/latest/download/xshuttle-aarch64-apple-darwin.tar.gz | sudo tar xz -C /usr/local/bin
```

### Build from Source

```bash
git clone https://github.com/athopen/xshuttle.git
cd xshuttle
cargo build --release
sudo cp target/release/xshuttle /usr/local/bin/
```

## Usage

Run xshuttle:

```bash
xshuttle
```

## License

Released under the [MIT](https://github.com/athopen/xshuttle/blob/master/LICENSE.md) license.

© [Andreas Penz](https://github.com/athopen/)
