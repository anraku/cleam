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
│ [Tab] Switch Panel  [Enter] Open Stream  [q] Quit       │
└──────────────────────────────────────────────────────────┘
```

## Features

- Browse log groups and streams side by side
- View log events (last 1 hour) in a full-screen list
- Filter events with CloudWatch filter patterns
- Open any event in a full-screen read-only viewer
- AWS SSO authentication support (`aws sso login`)
- Pagination with lazy loading

## Installation

```bash
cargo install --path .
```

Or build and run directly:

## Requirements

- AWS credentials configured (`~/.aws/config` or environment variables)
- For SSO: run `aws sso login` before starting cleam
- Set `AWS_PROFILE`

## Key Bindings

### Main screen (Log Groups / Streams)

| Key | Action |
|-----|--------|
| `Tab` | Switch between Groups and Streams panel |
| `j` / `↓` | Move cursor down |
| `k` / `↑` | Move cursor up |
| `Enter` | Open selected stream |
| `q` | Quit |

### Events screen

| Key | Action |
|-----|--------|
| `j` / `↓` | Move cursor down |
| `k` / `↑` | Move cursor up |
| `/` | Enter filter mode |
| `Enter` | Open selected event in viewer |
| `q` | Back to main screen |

### Filter input

| Key | Action |
|-----|--------|
| `Enter` | Apply filter pattern |
| `Esc` | Cancel filter input |

### Viewer screen

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `q` | Back to events screen |

## Configuration

cleam uses the standard AWS SDK configuration chain:

- `AWS_DEFAULT_REGION` environment variable
- `~/.aws/config` default profile
- AWS SSO profiles (`aws sso login`)
- IAM roles (EC2/ECS/Lambda environments)

No additional configuration needed.

## License

MIT
