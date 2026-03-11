# rustaria

> A Rust download manager powered by [aria2](https://aria2.github.io/) — terminal UI, browser integration, and NixOS-native configuration.

## Features

- **aria2 backend** – leverages aria2c for reliable, multi-protocol downloads (HTTP/HTTPS, FTP, BitTorrent, Metalink)
- **TUI** – beautiful terminal interface built with Ratatui (progress bars, speed graphs, keybindings)
- **Browser extension** – intercept downloads from Chrome/Firefox and send them to rustaria
- **Clipboard monitoring** – auto-detects URLs matching configurable patterns
- **Post-processing** – auto-organize files, merge HLS/DASH with FFmpeg, extract archives
- **Scheduler** – time-based scheduling, concurrency limits, bandwidth throttling
- **NixOS native** – ships with a Nix flake and Home Manager module

## Architecture

```
┌─────────────┐
│  UI Layer   │  TUI · CLI · Notifications
└─────┬───────┘
      │
┌─────▼───────┐
│Orchestration│  Job Queue · Scheduler · Config
└─────┬───────┘
      │
┌─────▼───────┐
│ aria2 Bridge│  RPC Client · WebSocket Events · Process Manager
└─────┬───────┘
      │
┌─────▼───────┐
│ Integration │  Browser Extension · Clipboard · Media Sniffer
└─────┬───────┘
      │
┌─────▼───────┐
│Post-Process │  Organizer · FFmpeg · Extractor · Hooks
└─────┬───────┘
      │
┌─────▼───────┐
│ Persistence │  SQLite DB · State Store
└─────────────┘
```

## Quick Start

```bash
# Build
cargo build --release

# Run TUI
./target/release/rustaria

# Or run the daemon in headless mode
./target/release/rustaria --daemon
```

## Configuration

Edit `config.toml` in the project root (or `~/.config/rustaria/config.toml`).

```toml
[general]
download_dir = "~/Downloads"
max_concurrent = 5

[aria2]
rpc_url = "http://127.0.0.1:6800/jsonrpc"
rpc_secret = ""
```

## Browser Extension

See [extension/README.md](extension/README.md) for build and install instructions.

## NixOS / Home Manager

```nix
{
  inputs.rustaria.url = "github:me-osano/rustaria";

  # In your home.nix
  imports = [ inputs.rustaria.homeManagerModules.default ];

  programs.rustaria = {
    enable = true;
    settings = {
      general.download_dir = "~/Downloads";
    };
  };
}
```

## License

MIT
