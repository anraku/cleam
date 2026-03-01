# cleam

AWS CloudWatch Logs TUI viewer written in Rust.

Browse log groups, streams, and events interactively from your terminal.

```
┌─ cleam ──────────────────────────────────────────────────┐
│ AWS CloudWatch Logs                                      │
├──────────────────────┬───────────────────────────────────┤
│ Log Groups           │ Streams: /app/prod                │
│                      │                                   │
│ > /app/prod          │ > 2024/01/15/[$LATEST]abc123...   │
│   /app/stg           │   2024/01/15/[prod]def456...      │
│   /lambda/fn-a       │   2024/01/14/[$LATEST]xyz999...   │
├──────────────────────┴───────────────────────────────────┤
│ [h/l] Switch Panel  [Enter] Open Stream  [q] Quit       │
└──────────────────────────────────────────────────────────┘
```

## Features

- Browse log groups and streams side by side with vim-like navigation
- Incremental search for log groups and streams (`/`)
- View log events in a full-screen list with CloudWatch filter pattern support
- Cross-stream event search by time range and filter pattern (`g`)
- Open any event in a full-screen scrollable viewer
- Download events as JSONL file (`d`)
- AWS SSO authentication support (`aws sso login`)
- Pagination with lazy loading

## Installation
- use `cargo install`
```bash
cargo install cleam
```

- install from source
```bash
cargo install --path .
```

Or build and run directly:
```bash
cargo run
```

## Requirements

- Rust toolchain
- AWS credentials configured (`~/.aws/config` or environment variables)
- Set `AWS_PROFILE`
- Set `AWS_REGION` or region set in `~/.aws/config`
- For SSO: run `aws sso login` before starting cleam

## Key Bindings

### Main screen (Log Groups / Streams)

| Key | Action |
|-----|--------|
| `h` | Focus Log Groups panel |
| `l` | Focus Streams panel |
| `j` / `↓` | Move cursor down |
| `k` / `↑` | Move cursor up |
| `Enter` | Open selected stream (or move focus to Streams) |
| `/` | Start incremental search |
| `g` | Open event search form for selected group |
| `q` | Quit |

#### Incremental search

| Key | Action |
|-----|--------|
| Any char | Narrow down the list |
| `Backspace` | Delete last character |
| `Enter` | Confirm and exit search mode |
| `Esc` | Clear search and restore selection |

### Events screen

| Key | Action |
|-----|--------|
| `j` / `↓` | Move cursor down |
| `k` / `↑` | Move cursor up |
| `/` | Enter filter mode |
| `d` | Enter download mode (save events as JSONL) |
| `Enter` | Open selected event in viewer |
| `q` | Back to main screen |

#### Filter input

| Key | Action |
|-----|--------|
| Any char | Edit CloudWatch filter pattern |
| `Backspace` | Delete last character |
| `Enter` | Apply filter (reload events) |
| `Esc` | Cancel |

#### Download path input

| Key | Action |
|-----|--------|
| Any char | Edit output file path |
| `Backspace` | Delete last character |
| `Enter` | Save all loaded events to JSONL |
| `Esc` | Cancel |

### Viewer screen

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `q` | Back |

### Event search form

Accessed from the main screen with `g`. Searches across all streams in the selected log group.

| Key | Action |
|-----|--------|
| `Tab` | Move to next field |
| `Shift+Tab` | Move to previous field |
| Any char | Edit current field |
| `Backspace` | Delete last character |
| `Enter` | Execute search |
| `q` / `Esc` | Cancel and go back |

Fields: Start time, End time, Filter pattern (all optional).
Date format: `YYYY-MM-DD HH:MM:SS` (UTC)

### Group events screen

| Key | Action |
|-----|--------|
| `j` / `↓` | Move cursor down |
| `k` / `↑` | Move cursor up |
| `Enter` | Open selected event in viewer |
| `q` | Back to event search form |

## License

MIT
