# ytd — YouTrack CLI

[![CI](https://github.com/danielkbx/youtrackd/actions/workflows/ci.yml/badge.svg)](https://github.com/danielkbx/youtrackd/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/danielkbx/youtrackd)](https://github.com/danielkbx/youtrackd/releases/latest)

CLI for YouTrack tickets, articles, time tracking, and more. Single binary, no runtime dependencies.

## Installation

### Homebrew (macOS / Linux)

```bash
brew tap danielkbx/tap
brew install ytd
```

### Download

Grab the latest binary for your platform from the [Releases](../../releases) page:

| Platform | Archive |
|---|---|
| Linux x86_64 | `ytd-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `ytd-aarch64-unknown-linux-gnu.tar.gz` |
| macOS Intel | `ytd-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `ytd-aarch64-apple-darwin.tar.gz` |

```bash
# Example: macOS Apple Silicon
tar xzf ytd-aarch64-apple-darwin.tar.gz
sudo mv ytd /usr/local/bin/
```

### Build from Source

Requires [Rust](https://rustup.rs/) 1.70+.

```bash
git clone https://github.com/danielkbx/youtrackd.git
cd youtrackd
cargo build --release
# Binary at target/release/ytd
```

## Getting Started

### 1. Login

```
ytd login
```

You'll be prompted for your YouTrack URL and a permanent token.
To create a token: YouTrack → Profile → Account Security → New Token.

### 2. Verify

```
ytd whoami
```

### 3. Explore

```
ytd project list
ytd ticket list --project MYPROJECT
ytd config set visibility-group "Engineering"
ytd group list
```

## Commands

### Projects
```
ytd project list
ytd project get <shortName>
```

### Groups
```
ytd group list
```

### Tickets
```
ytd ticket list --project <id>
ytd ticket get <id>
ytd ticket search "<query>" [--project <id>]
ytd ticket create --project <id> --json '{"summary":"...","description":"..."}' [--visibility-group <group> | --no-visibility-group]
ytd ticket update <id> --json '{"summary":"..."}' [--visibility-group <group> | --no-visibility-group]
ytd ticket comment <id> "text"
ytd ticket comments <id>
ytd ticket delete <id> [-y]
```

### Ticket Management
```
ytd ticket tag <id> <tag>                    # Add tag
ytd ticket untag <id> <tag>                  # Remove tag
ytd ticket link <id> <target> [--type <t>]   # Link issues
ytd ticket links <id>                        # Show links
ytd ticket set <id> <field> <value>          # Set custom field
ytd ticket fields <id>                       # Show field values
ytd ticket attach <id> <file>                # Upload file
ytd ticket attachments <id>                  # List files
ytd ticket log <id> <duration> [text]        # Log time
ytd ticket worklog <id>                      # Show time log
ytd ticket history <id> [--category <cat>]   # Activity log
```

### Articles (Knowledge Base)
```
ytd article list --project <id>
ytd article get <id>
ytd article search "<query>" [--project <id>]
ytd article create --project <id> --json '{"summary":"...","content":"..."}' [--visibility-group <group> | --no-visibility-group]
ytd article update <id> --json '{"content":"..."}' [--visibility-group <group> | --no-visibility-group]
ytd article append <id> "text"
ytd article comment <id> "text"
ytd article comments <id>
ytd article attach <id> <file>
ytd article attachments <id>
ytd article delete <id> [-y]
```

### Comments
```
ytd comment get <comment-id>
ytd comment update <comment-id> "text"
ytd comment delete <comment-id> [-y]
```

Comment IDs are returned by `ytd ticket comments` and `ytd article comments`. Use the returned `id` field with `ytd comment ...`; `ytId` is the raw YouTrack comment ID and is only included for reference.

### Tags, Searches, Boards
```
ytd config set visibility-group <group>
ytd config get visibility-group
ytd config unset visibility-group
ytd group list
ytd tag list [--project <id>]
ytd search list [--project <id>]
ytd search run <name-or-id>
ytd board list [--project <id>]
ytd board get <id>
```

## Output Flags

| Flag | Description |
|---|---|
| `--format raw` | JSON output (for scripting) |
| `--format text` | Plain text (default) |
| `--format md` | Markdown (title + body + comments) |
| `--no-meta` | Hide IDs, dates, author |

## Examples

### Create and reference a ticket
```bash
ID=$(ytd ticket create --project PROJ --json '{"summary":"Fix login bug"}')
ytd ticket set "$ID" Priority Critical
ytd ticket tag "$ID" backend
```

### Log time
```bash
ytd ticket log PROJ-42 2h30m "Implemented feature"
ytd ticket log PROJ-42 45m --date 2025-01-15
```

### Pipe JSON from stdin
```bash
echo '{"summary":"New article","content":"..."}' | ytd article create --project PROJ
```

### Set custom fields
```bash
ytd ticket set PROJ-42 State "In Progress"
ytd ticket set PROJ-42 Assignee alice.smith
```

### Export article as Markdown file
```bash
ytd article get PROJ-A-1 --format md > article.md
```

### Update a comment
```bash
COMMENT_ID=$(ytd ticket comments PROJ-42 --format raw | jq -r '.[0].id')
ytd comment get "$COMMENT_ID"
ytd comment update "$COMMENT_ID" "Updated comment text"
```

### Delete with confirmation skip
```bash
ytd ticket delete PROJ-42 -y
```

## Good to know

### Comment IDs

YouTrack comment operations are scoped to their parent ticket or article. A raw YouTrack comment ID like `4-17` is not enough to load, update, or delete the comment because the API also needs the parent resource.

For that reason, `ytd` returns reusable comment IDs in this format:

```
<parent-id>:<comment-id>
```

Examples:

```
PROJ-42:4-17       # comment on ticket PROJ-42
PROJ-A-1:187-62   # comment on article PROJ-A-1
```

Use the `id` field returned by `ytd ticket comments` or `ytd article comments` with `ytd comment get|update|delete`. The `ytId` field is only the raw YouTrack comment ID and is included for reference.

## Configuration

Config file: `~/.config/ytd/config.json` (used for stored credentials and CLI settings)

Alternatively, set environment variables:
- `YOUTRACK_URL` — Your YouTrack instance URL
- `YOUTRACK_TOKEN` — Your permanent token
- `YTD_VISIBILITY_GROUP` — Default group for ticket/article visibility

Environment variables take precedence over the config file.

### Visibility Defaults

`ticket create|update` and `article create|update` accept `--visibility-group <group>` to send limited visibility for that group.

`--no-visibility-group` disables inherited defaults from `YTD_VISIBILITY_GROUP` or `ytd config set visibility-group ...`. On `update`, it clears the existing visibility restriction. On `create`, it sends no visibility restriction.

Resolution order is: `--visibility-group` → `YTD_VISIBILITY_GROUP` → stored `visibility-group` config. Combining `--visibility-group` and `--no-visibility-group` is an input error.

### Multiple Instances

Use `YTD_CONFIG` to point to a specific config file:

```bash
# Shell aliases for different YouTrack instances
alias ytd-work='YTD_CONFIG=~/.config/ytd/work.json ytd'
alias ytd-oss='YTD_CONFIG=~/.config/ytd/oss.json ytd'

# First-time setup for each
YTD_CONFIG=~/.config/ytd/work.json ytd login
YTD_CONFIG=~/.config/ytd/oss.json ytd login
```

## ID Formats

| Type | Format | Example |
|---|---|---|
| Project | Short name | `MYPROJECT` |
| Ticket | Project-Number | `MYPROJECT-42` |
| Article | Project-A-Number | `MYPROJECT-A-1` |
| Comment | Parent ID + YouTrack comment ID | `MYPROJECT-42:4-17` or `MYPROJECT-A-1:187-62` |

Comment IDs encode their parent because YouTrack comment operations are scoped to either an issue or an article. `ytd` infers whether the parent is a ticket or article from the parent ID shape: article IDs use `<PROJECT>-A-<NUMBER>`, tickets use `<PROJECT>-<NUMBER>`. Use the `id` returned by `ytd`; do not pass `ytId` to `ytd comment`.

## License

GPL-3.0-only — see [LICENSE.md](LICENSE.md).
