# Dashy

A fast, lightweight terminal system monitor built in Rust.

![Dashy TUI](https://img.shields.io/badge/TUI-Ratatui-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey)

## Features

- **CPU Monitor** - Real-time CPU usage with 60-second sparkline history
- **Memory Usage** - Visual gauge showing used/total RAM
- **Disk I/O** - Live read/write throughput monitoring
- **Network I/O** - RX/TX traffic rates
- **Open Ports** - List all listening ports with associated process names
- **Process Kill** - Kill processes holding ports directly from the UI

## Installation

### Quick Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/mshaaban0/dashy/main/install.sh | bash
```

### Homebrew (macOS/Linux)

```bash
brew tap mshaaban0/dashy
brew install dashy
```

### Cargo (Build from source)

```bash
cargo install --git https://github.com/mshaaban0/dashy.git
```

### Manual Build

```bash
git clone https://github.com/mshaaban0/dashy.git
cd dashy
cargo build --release
sudo cp target/release/dashy /usr/local/bin/
```

## Usage

Simply run:

```bash
dashy
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `Ctrl+C` | Force quit |
| `j` / `↓` | Select next port |
| `k` / `↑` | Select previous port |
| `Enter` | Kill selected process (opens confirmation) |
| `Tab` | Toggle Yes/No in confirmation dialog |
| `y` | Quick confirm kill |
| `n` | Cancel dialog |

## Screenshots

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ CPU: 12.3%                        │ Memory                                  │
│ ▁▂▃▄▅▆▇█▇▆▅▄▃▂▁▂▃▄▅▆▇█▇▆▅▄▃▂▁    │ ████████░░░░░░░░░░  8.2 GB / 16.0 GB   │
├─────────────────────────────────────────────────────────────────────────────┤
│ Disk I/O                          │ Network I/O                             │
│  Read:  12.5 MB/s                 │  RX: 1.2 MB/s                           │
│  Write: 8.3 MB/s                  │  TX: 256 KB/s                           │
├─────────────────────────────────────────────────────────────────────────────┤
│ Open Ports - [k/j] navigate, [Enter] kill                                   │
│ Port       Process                                                          │
│ 3000       node                                                             │
│ 5432       postgres                                                         │
│ 8080       java                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Requirements

- macOS 10.15+ or Linux
- Terminal with Unicode support

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
