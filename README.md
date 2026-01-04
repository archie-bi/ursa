# Ursa

A fast, keyboard-driven tmux session manager built with [Ratatui](https://ratatui.rs/).

## Features

- List all tmux sessions at a glance
- Create, rename, and delete sessions
- Vim-style navigation (hjkl)
- Instant session switching

## Installation

```bash
cargo install --git https://github.com/archie-bi/ursa
```

Or build from source:

```bash
git clone https://github.com/archie-bi/ursa
cd ursa
cargo install --path .
```

## Usage

```bash
ursa
```

## Keybindings

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `→` / `l` | Next action (Enter → Rename → Delete) |
| `←` / `h` | Previous action |
| `Enter` | Confirm selected action |
| `r` | Refresh session list |
| `q` / `Esc` | Quit |

## Actions

Each session has three actions you can cycle through with `←` / `→`:

- **[Enter]** - Attach to the session
- **[Rename]** - Rename the session
- **[Delete]** - Kill the session

## Requirements

- tmux must be installed and available in your PATH

## License

MIT
