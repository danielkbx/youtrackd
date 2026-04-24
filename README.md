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
ytd ticket comment <id> "text" [--visibility-group <group> | --no-visibility-group]
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
ytd ticket attachments <id>                  # List files with reusable attachment IDs
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
ytd article comment <id> "text" [--visibility-group <group> | --no-visibility-group]
ytd article comments <id>
ytd article attach <id> <file>
ytd article attachments <id>
ytd article delete <id> [-y]
```

### Comments
```
ytd comment get <comment-id>
ytd comment update <comment-id> "text" [--visibility-group <group> | --no-visibility-group]
ytd comment attachments <comment-id>
ytd comment delete <comment-id> [-y]
```

Comment IDs are returned by `ytd ticket comments` and `ytd article comments`. Use the returned `id` field with `ytd comment ...`; `ytId` is the raw YouTrack comment ID and is only included for reference.

### Attachments
```
ytd attachment get <attachment-id>
ytd attachment delete <attachment-id> [-y]
ytd attachment download <attachment-id> [--output <path>]
```

Attachment IDs are returned by `ytd ticket attachments`, `ytd article attachments`, and `ytd comment attachments`. Use the returned `id` field with `ytd attachment ...`; `ytId` is the raw YouTrack attachment ID and is only included for reference.

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
ytd board create --name <name> --project <project>[,<project>...] [--template <template>] [--json '{...}']
ytd board update <id> [--name <name>] [--json '{...}']
ytd board delete <id> [-y]
```

Board commands use YouTrack's Agile API. For existing boards, `get`, `update`, and `delete` require only the board ID. For `create`, YouTrack requires a board name and at least one project; `--project` accepts project short names or database IDs.

## Output Flags

| Flag | Description |
|---|---|
| `--format raw` | JSON output (for scripting) |
| `--format text` | Plain text (default) |
| `--format md` | Markdown (title + body + comments) |
| `--no-meta` | Hide IDs, dates, author |

## JSON Input

Many create and update commands accept JSON via `--json` or stdin. Stdin takes precedence over `--json` when both are present.

Commands that accept JSON input:

| Command | Required fields | Optional/common fields |
|---|---|---|
| `ticket create` | `summary` plus `--project` | `description` |
| `ticket update` | at least one field | `summary`, `description` |
| `article create` | `summary` plus `--project` | `content` |
| `article update` | at least one field | `summary`, `content` |
| `board create` | `name` and `projects`, or `--name` and `--project` | YouTrack Agile fields such as `visibleForProjectBased` |
| `board update` | at least one field or `--name` | YouTrack Agile fields such as `orphansAtTheTop` |

JSON input must be valid JSON:

```bash
ytd ticket create --project PROJ --json '{"summary":"Fix login","description":"Details"}'
```

Create and update commands print only the resulting entity ID on stdout, so they can be used in scripts:

```bash
ID=$(ytd ticket create --project PROJ --json '{"summary":"Fix login bug"}')
```

Ticket and article commands use the project short name from `--project`:

```bash
ytd ticket update PROJ-42 --json '{"summary":"New title"}'
ytd article create --project PROJ --json '{"summary":"Runbook","content":"Steps..."}'
ytd article update PROJ-A-1 --json '{"content":"Updated steps..."}'
```

You can also pipe JSON through stdin:

```bash
printf '%s\n' '{"summary":"New article","content":"..."}' | ytd article create --project PROJ
```

Boards are exposed as `board` commands, but YouTrack calls them Agile boards in the REST API. Basic board creation uses flags for the required fields:

```bash
ytd board create --name "Team Scrum Board" --project PROJ --template scrum
```

Use `--json` or stdin for advanced Agile fields. The JSON object is merged with the flag-derived payload; `--name` wins over JSON `name`.

```bash
ytd board create --name "Team Scrum Board" --project PROJ --template scrum --json '{"visibleForProjectBased":true}'
ytd board update 108-4 --name "Renamed Board"
ytd board update 108-4 --json '{"orphansAtTheTop":true}'
```

For `board create`, `--project PROJ` is resolved to the YouTrack project database ID required by the API. Multi-project boards use a comma-separated list:

```bash
ytd board create --name "Multi Project Board" --project PROJ,OPS --template kanban
```

Advanced users may provide the complete board create payload as JSON:

```bash
ytd board create --template scrum --json '{"name":"Team Scrum Board","projects":[{"id":"0-0"}]}'
```

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

### Download an attachment
```bash
ATTACHMENT_ID=$(ytd ticket attachments PROJ-42 --format raw | jq -r '.[0].id')
ytd attachment get "$ATTACHMENT_ID"
ytd attachment download "$ATTACHMENT_ID" --output /tmp/
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

### Attachment IDs

YouTrack attachment operations are scoped to their parent ticket or article, even when an attachment is displayed on a comment. For that reason, `ytd` returns reusable attachment IDs in this format:

```
<parent-id>:<attachment-id>
```

Examples:

```
PROJ-42:8-2897
PROJ-A-1:237-3
```

Use the `id` field returned by `ytd ticket attachments`, `ytd article attachments`, or `ytd comment attachments` with `ytd attachment get|delete|download`. The `commentId` field is included when YouTrack reports that an attachment belongs to a comment.

`ytd` does not support adding files to existing comments. A Curl probe against YouTrack showed that updating a comment with `attachments:[{id}]` returns success but does not assign the attachment to the comment.

### Comment visibility

New comments follow the configured visibility default from `YTD_VISIBILITY_GROUP` or `ytd config set visibility-group ...`. Use `--visibility-group <group>` to set a group explicitly or `--no-visibility-group` to create the comment without inherited default visibility.

For existing comments, `ytd comment update` does not apply defaults automatically. Without visibility flags it only changes the text and preserves the current visibility. Use `--visibility-group <group>` to set visibility or `--no-visibility-group` to clear it.

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
