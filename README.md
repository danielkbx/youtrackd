# ytd - YouTrack CLI

[![CI](https://github.com/danielkbx/youtrackd/actions/workflows/ci.yml/badge.svg)](https://github.com/danielkbx/youtrackd/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/danielkbx/youtrackd)](https://github.com/danielkbx/youtrackd/releases/latest)

`ytd` is a command-line client for YouTrack. It works with tickets, knowledge base articles, comments, attachments, users, projects, tags, saved searches, Agile boards, sprints, time tracking, and local ticket aliases.

The default output is compact plain text for humans and AI agents. Use JSON when you want stable scriptable output.

`ytd` can also generate current `SKILL.md` guidance for AI agents with `ytd skill`. Agents can run that command themselves to fetch up-to-date usage instructions, including project-specific examples with `ytd skill --project <project>`. Use `ytd schema <resource> <action>` to inspect JSON fields before creating or updating with `--json`. See [AI Agent Skill File](#ai-agent-skill-file).

## Installation

### Homebrew

```bash
brew tap danielkbx/tap
brew install ytd
```

The Homebrew formula installs Bash, Zsh, and Fish completion files to Homebrew's standard completion directories.

### Download

Download the latest archive for your platform from the [Releases](../../releases) page.

| Platform | Archive |
|---|---|
| Linux x86_64 | `ytd-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `ytd-aarch64-unknown-linux-gnu.tar.gz` |
| macOS Intel | `ytd-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `ytd-aarch64-apple-darwin.tar.gz` |

```bash
tar xzf ytd-aarch64-apple-darwin.tar.gz
sudo mv ytd /usr/local/bin/
```

### Build From Source

Requires [Rust](https://rustup.rs/) 1.70+.

```bash
git clone https://github.com/danielkbx/youtrackd.git
cd youtrackd
cargo build --release
```

The binary is written to `target/release/ytd`.

## Shell Completions

`ytd` can generate completion scripts for `bash`, `zsh`, and `fish`.
Generated scripts are written to stdout and do not require login.

Homebrew installs generated completions automatically. The manual steps below are for downloaded binaries, source builds, or custom shell setups.

### Bash

Install and enable Bash completion support first. With Homebrew:

```bash
brew install bash-completion@2
```

Then add this to `~/.bashrc` or another Bash startup file:

```bash
[[ -r /opt/homebrew/etc/profile.d/bash_completion.sh ]] && source /opt/homebrew/etc/profile.d/bash_completion.sh
```

On Intel macOS/Homebrew installations, use `/usr/local/etc/profile.d/bash_completion.sh` instead.

```bash
mkdir -p ~/.local/share/bash-completion/completions
ytd completion bash > ~/.local/share/bash-completion/completions/ytd
```

Reload your shell, or source the generated file directly for the current session:

```bash
source ~/.local/share/bash-completion/completions/ytd
```

### Zsh

Zsh completion support is built in through `compinit`. Homebrew installs `_ytd` into its standard `site-functions` directory; most Homebrew Zsh setups only need:

```zsh
autoload -Uz compinit
compinit
```

For a manual install:

```bash
mkdir -p ~/.zfunc
ytd completion zsh > ~/.zfunc/_ytd
```

Add the completion directory to your `fpath` before `compinit` in `~/.zshrc`:

```zsh
fpath=(~/.zfunc $fpath)
autoload -Uz compinit
compinit
```

Reload your shell after updating `~/.zshrc`.

### Fish

```fish
mkdir -p ~/.config/fish/completions
ytd completion fish > ~/.config/fish/completions/ytd.fish
```

Fish loads files from that directory automatically in new shells.

### Troubleshooting

Check whether your shell has loaded the completion:

```bash
complete -p ytd
```

```zsh
print $_comps[ytd]
```

```fish
complete --do-complete "ytd "
```

For Zsh, make sure custom completion directories are added to `fpath` before `compinit`. If an old completion is still used, clear the cache and reinitialize:

```zsh
rm -f ~/.zcompdump*
unfunction _ytd 2>/dev/null
autoload -Uz compinit
compinit -u
```

## Getting Started

Configure your YouTrack URL and a permanent token:

```bash
ytd login
```

Create a token in YouTrack under Profile -> Account Security -> New Token.

Check the login:

```bash
ytd whoami
```

Explore your instance:

```bash
ytd project list
ytd ticket list --project MYPROJECT
ytd user list
ytd group list
ytd help
```

Use command-specific help whenever you need exact syntax:

```bash
ytd help ticket
ytd ticket help
ytd help sprint
```

### AI Agent Skill File

`ytd` can generate a current `SKILL.md` file for AI agents:

```bash
ytd skill > SKILL.md
```

Agents can also run `ytd skill` themselves to fetch up-to-date usage guidance instead of relying on stale checked-in instructions. Add `--project <project>` when the agent should receive project-specific examples:

```bash
ytd skill --project MYPROJECT > SKILL.md
ytd skill --project MYPROJECT --scope full
```

Agents should run `ytd schema <resource> <action>` before guessing JSON fields for create/update commands.

## Command Guide

### Authentication And Help

```bash
ytd login
ytd logout
ytd whoami
ytd help
ytd help <command>
ytd <command> help
ytd completion <bash|zsh|fish>
ytd url <target>
ytd open <target>
```

`url` prints a YouTrack web URL. `open` opens the same URL in your browser and prints it. Targets can be tickets, articles, projects, or project knowledge bases:

```bash
ytd url ABC-12
ytd open ABC-A-1
ytd open ABC
ytd open ABC-A
```

### Projects, Users, And Groups

```bash
ytd project list
ytd project get <id>

ytd user list
ytd user get <user-id-or-login>

ytd group list
```

Groups are useful when choosing a visibility group for restricted tickets, articles, and comments.

### Configuration Commands

```bash
ytd config set visibility-group <group>
ytd config get visibility-group
ytd config unset visibility-group
```

These commands manage stored CLI settings. Authentication settings are created by `ytd login`.

### Tickets

```bash
ytd ticket search <query> [--project <id>]
ytd ticket list --project <id>
ytd ticket get <id> [--no-comments]
ytd ticket create --project <id> --json '{"summary":"...","description":"..."}'
ytd ticket update <id> --json '{"summary":"...","description":"..."}'
ytd ticket comment <id> <text>
ytd ticket comments <id>
ytd ticket delete <id> [-y]
```

Ticket management commands:

```bash
ytd ticket tag <id> <tag>
ytd ticket untag <id> <tag>
ytd ticket link <id> <target> [--type <type>]
ytd ticket links <id>
ytd ticket attach <id> <file>
ytd ticket attachments <id>
ytd ticket log <id> <duration> [text] [--date YYYY-MM-DD] [--type <worktype>]
ytd ticket worklog <id>
ytd ticket set <id> <field> <value>
ytd ticket fields <id>
ytd ticket history <id> [--category <category>]
ytd ticket sprints <id>
```

Durations can be written as `30m`, `1h`, `2h30m`, or a plain number of minutes.

### Articles

```bash
ytd article search <query> [--project <id>]
ytd article list --project <id>
ytd article get <id> [--no-comments]
ytd article create --project <id> --json '{"summary":"...","content":"...","parentArticle":{"id":"PROJ-A-1"}}'
ytd article update <id> --json '{"summary":"...","content":"...","parentArticle":{"id":"PROJ-A-1"}}'
ytd article update <id> --json '{"parentArticle":null}'
ytd article move <id> <parent-id|none>
ytd article append <id> <text>
ytd article comment <id> <text>
ytd article comments <id>
ytd article attach <id> <file>
ytd article attachments <id>
ytd article delete <id> [-y]
```

Use `parentArticle.id` with the reusable readable article ID. `parentArticle: null` on update clears the parent article. `article move <id> <parent-id>` is a shortcut for changing the parent, and `article move <id> none` clears it. `article get --format json` includes normalized `parentArticle` data with public `id`, raw `ytId`, and `summary`; `--format raw` includes YouTrack-shaped `parentArticle(id,idReadable,summary)`.

Use `--format md` with `article get` when you want a Markdown export.

### Comments

```bash
ytd comment get <comment-id>
ytd comment update <comment-id> <text>
ytd comment attach <comment-id> <file>
ytd comment attachments <comment-id>
ytd comment delete <comment-id> [-y]
```

Comment IDs are returned by `ticket comments` and `article comments`. Use the returned `id` value with `ytd comment ...`.

Use `comment attach` to upload a file to an existing ticket or article comment. Use `comment attachments` to list files attached to the comment. Attachment IDs returned by comment attachment listings can be used with `ytd attachment get|download|delete`.

### Attachments

```bash
ytd attachment get <attachment-id>
ytd attachment delete <attachment-id> [-y]
ytd attachment download <attachment-id> [--output <path>]
```

Attachment IDs are returned by `ticket attachments`, `article attachments`, and `comment attachments`. Use the returned `id` value with `ytd attachment ...`.

### Aliases

Aliases are local shortcuts for recurring ticket workflows. They can bind a project, a user, and optionally a sprint.

```bash
ytd alias create <alias> [--project <project-id>] [--user <user-id>] [--sprint <sprint-id|none>]
ytd alias list
ytd alias delete <alias> [-y]
ytd <alias> create <text>
ytd <alias> list [--all]
```

Example:

```bash
ytd alias create todo --project 0-96 --user 1-51 --sprint 108-4:113-6
ytd todo create "Follow up with customer"
ytd todo list
ytd todo list --all
```

Use `--sprint none` to create or update an alias without a sprint binding.

### Tags And Saved Searches

```bash
ytd tag list [--project <id>]
ytd search list [--project <id>]
ytd search run <name-or-id>
```

`search run` accepts a saved search ID or name. `search list --project <id>` filters saved searches by project references in the saved query text.

### Boards And Sprints

```bash
ytd board list [--project <id>]
ytd board get <id>
ytd board create --name <name> --project <project>[,<project>...] [--template <template>]
ytd board update <id> [--name <name>]
ytd board delete <id> [-y]

ytd sprint list [--board <board-id>]
ytd sprint current [--board <board-id>]
ytd sprint get <sprint-id>
ytd sprint create --board <board-id> --name <name>
ytd sprint update <sprint-id> [--name <name>]
ytd sprint delete <sprint-id> [-y]
ytd sprint ticket list <sprint-id>
ytd sprint ticket add <sprint-id> <ticket-id>
ytd sprint ticket remove <sprint-id> <ticket-id>
```

Board templates are `kanban`, `scrum`, `version`, `custom`, and `personal`.

Sprint IDs include the board ID:

```text
<board-id>:<sprint-id>
```

Use sprint IDs returned by `sprint list`, `sprint current`, `sprint create`, or `ticket sprints`.

### Agent Skills

```bash
ytd skill [--scope brief|standard|full] [--project <project>]
```

`ytd skill` prints current `SKILL.md` guidance for AI agents. Without `--project`, it works without login. With `--project`, it resolves the project and includes project-specific examples.

`ytd skill` supports `--format text` and `--format md`. Both print Markdown. `--format json` and `--format raw` are not supported for this command.

### JSON Schemas

```bash
ytd schema [list]
ytd schema [--project <project>]
ytd schema <ticket|article|board|sprint> <create|update> [--project <project>]
```

`ytd schema` shows the JSON input contract for commands that accept `--json` or stdin. Static schema discovery works without login; `--project` requires login and adds ticket custom field examples. It supports `--format text` and `--format json`.

## Working With Output

Global output flags:

| Flag | Description |
|---|---|
| `--format text` | Plain text, default |
| `--format json` | Stable ytd-normalized JSON for scripts and agents |
| `--format raw` | YouTrack API-shaped JSON |
| `--format md` | Markdown export where supported |
| `--no-meta` | Hide metadata such as IDs, dates, and author fields where supported |
| `-y` | Confirm delete commands without prompting |

Text output renders Markdown content as readable terminal text. Ticket lists, saved search results, sprint ticket lists, and ticket links use compact ticket rows. `ticket get` and `article get` show details first, then the description or content.

Article detail output includes parent article metadata when present.

Use `--no-comments` with `ticket get` or `article get` to omit comments.

Create, update, and delete commands that return one changed resource print only its ID on stdout, which makes them easy to use in scripts:

```bash
ID=$(ytd ticket create --project PROJ --json '{"summary":"Fix login"}')
ytd ticket get "$ID"
```

Delete commands ask for confirmation when run interactively. In scripts, pass `-y`.

## JSON Input

Create and update commands accept JSON through `--json` or stdin. Stdin takes precedence when both are present.

```bash
ytd ticket create --project PROJ --json '{"summary":"Fix login","description":"Steps..."}'
printf '%s\n' '{"summary":"Runbook","content":"Steps..."}' | ytd article create --project PROJ
```

Common JSON commands:

| Command | Required input |
|---|---|
| `ticket create` | `--project` and JSON `summary`; allowed JSON fields are `summary`, `description`, `customFields`, `tags` |
| `ticket update` | At least one JSON field or an explicit visibility flag; allowed JSON fields are `summary`, `description`, `customFields`, `tags` |
| `article create` | `--project` and JSON `summary`; allowed JSON fields are `summary`, `content`, `parentArticle` |
| `article update` | At least one JSON field or an explicit visibility flag; allowed JSON fields are `summary`, `content`, `parentArticle` |
| `board create` | `--name` and `--project`, or equivalent JSON |
| `board update` | At least one field or `--name` |
| `sprint create` | `--board` and `--name`, or `--board` plus JSON `name` |
| `sprint update` | At least one field or `--name` |

Boards and sprints also accept `--json` for additional YouTrack Agile fields:

```bash
ytd board create --name "Team Board" --project PROJ --template scrum --json '{"visibleForProjectBased":true}'
ytd sprint update 108-4:113-6 --json '{"goal":"Finish onboarding"}'
```

## JSON Schema / Field Discovery

Use `ytd schema` to discover the JSON input contract before using `--json` or stdin. Static schema discovery does not require login.

```bash
ytd schema
ytd schema --project PROJ
ytd schema ticket create
ytd schema ticket create --project PROJ
ytd schema article update --format json
```

`ytd schema <resource> <action>` shows required flags, required and optional JSON fields, flag-vs-JSON precedence rules, examples, and pass-through caveats for board and sprint advanced fields. Without `--project`, schema discovery is static and requires no login. With `--project`, ytd resolves the project, requires login, and adds ticket custom field examples based on the project's YouTrack field configuration.

Ticket `customFields` uses YouTrack API shape:

```bash
ytd ticket create --project PROJ --json '{"summary":"Fix login","customFields":[{"name":"Assignee","$type":"SingleUserIssueCustomField","value":{"login":"jane.doe"}}]}'
```

## Common Workflows

### Create And Update A Ticket

```bash
ID=$(ytd ticket create --project PROJ --json '{"summary":"Fix login bug"}')
ytd ticket set "$ID" Priority Critical
ytd ticket tag "$ID" backend
ytd ticket comment "$ID" "Investigating"
```

### Log Time

```bash
ytd ticket log PROJ-42 2h30m "Implemented feature"
ytd ticket log PROJ-42 45m --date 2026-04-25
```

### Export An Article

```bash
ytd article get PROJ-A-1 --format md > article.md
```

### Update A Comment

```bash
COMMENT_ID=$(ytd ticket comments PROJ-42 --format json | jq -r '.[0].id')
ytd comment get "$COMMENT_ID"
ytd comment update "$COMMENT_ID" "Updated comment text"
```

### Download An Attachment

```bash
ATTACHMENT_ID=$(ytd ticket attachments PROJ-42 --format json | jq -r '.[0].id')
ytd attachment download "$ATTACHMENT_ID" --output /tmp/
```

### Work With Sprints

```bash
SPRINT_ID=$(ytd sprint create --board 108-4 --name "Sprint 1")
ytd sprint ticket add "$SPRINT_ID" PROJ-42
ytd sprint ticket list "$SPRINT_ID"
ytd sprint ticket remove "$SPRINT_ID" PROJ-42
```

### Configure Visibility Defaults

```bash
ytd config set visibility-group "Engineering"
ytd config get visibility-group
ytd config unset visibility-group
```

Ticket and article creation, plus new ticket and article comments, use configured visibility defaults. You can override visibility per command:

```bash
ytd ticket create --project PROJ --json '{"summary":"Private"}' --visibility-group "Engineering"
ytd ticket comment PROJ-42 "Public note" --no-visibility-group
ytd ticket update PROJ-42 --visibility-group "Engineering" --json '{"summary":"Restricted"}'
```

Updates preserve existing visibility unless you pass `--visibility-group` or `--no-visibility-group`.

## ID Formats

Use the `id` field returned by `ytd` as input to later commands. Fields named `ytId` are raw YouTrack IDs and are included for reference.

| Type | Format | Example |
|---|---|---|
| Project | Short name or YouTrack project ID | `PROJ` or `0-96` |
| User | YouTrack user ID or login | `1-51` or `alice` |
| Ticket | `<PROJECT>-<NUMBER>` | `PROJ-42` |
| Article | `<PROJECT>-A-<NUMBER>` | `PROJ-A-1` |
| Comment | `<parent-id>:<comment-id>` | `PROJ-42:4-17` |
| Attachment | `<parent-id>:<attachment-id>` | `PROJ-A-1:237-3` |
| Sprint | `<board-id>:<sprint-id>` | `108-4:113-6` |

`current` is not a sprint ID. Use `ytd sprint current` to list current sprints and then pass one of the returned IDs to sprint commands.

## Configuration

Stored config file:

```text
~/.config/ytd/config.json
```

Example:

```json
{
  "url": "https://your-instance.youtrack.cloud",
  "token": "perm:...",
  "visibilityGroup": "Engineering",
  "aliases": {
    "todo": { "project": "0-96", "user": "1-51", "sprint": "108-4:113-6" },
    "backlog": { "project": "0-96", "user": "1-51" }
  }
}
```

Environment variables:

| Variable | Purpose |
|---|---|
| `YOUTRACK_URL` | YouTrack instance URL |
| `YOUTRACK_TOKEN` | Permanent token |
| `YTD_CONFIG` | Custom config file path |
| `YTD_VISIBILITY_GROUP` | Default visibility group for creates and new comments |

`YOUTRACK_URL` and `YOUTRACK_TOKEN` take precedence over stored credentials.

Use `YTD_CONFIG` for multiple YouTrack instances:

```bash
alias ytd-work='YTD_CONFIG=~/.config/ytd/work.json ytd'
alias ytd-oss='YTD_CONFIG=~/.config/ytd/oss.json ytd'

YTD_CONFIG=~/.config/ytd/work.json ytd login
YTD_CONFIG=~/.config/ytd/oss.json ytd login
```

## License

GPL-3.0-only - see [LICENSE.md](LICENSE.md).
