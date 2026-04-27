# ytd - YouTrack CLI

`ytd` is a Rust CLI for reading and editing YouTrack tickets, knowledge base articles, comments, attachments, users, projects, tags, saved searches, Agile boards, sprints, time tracking, and local ticket aliases. It is designed for humans and AI agents, with compact text output and stable normalized JSON.

## Architecture

```text
src/
  main.rs         entry point, command routing, command validation
  args.rs         handwritten argument parsing
  cli_spec.rs     canonical public CLI model for completion generation
  completion.rs   Bash/Zsh/Fish completion renderers
  client.rs       HTTP transport trait and YouTrack API client
  config.rs       credential resolution and local settings
  duration.rs     duration parsing
  error.rs        error types
  format.rs       output formatting
  help.rs         help text
  input.rs        JSON input handling
  types.rs        shared data structures
  commands/       command handlers
user-journeys/    end-to-end test scripts for AI agents
.agents/          maintainer and agent context
```

Core modules (`client.rs`, `config.rs`, `types.rs`, `error.rs`) must not depend on CLI command handlers. Command handlers own stdout, stderr, prompts, and command-specific formatting decisions.

## Tech Stack

- Language: Rust 2021
- HTTP: `ureq` 3, synchronous
- JSON: `serde` and `serde_json`
- Markdown-to-terminal rendering: `termimad`
- Config path: `~/.config/ytd/config.json`, overridden by `YTD_CONFIG`
- Auth: YouTrack permanent token
- Distribution: standalone release binary and Homebrew formula

## Command Surface

```text
ytd help / ytd help <command> / ytd <command> help
ytd login / logout / whoami
ytd url <target>
ytd open <target>
ytd completion <bash|zsh|fish>
ytd skill [--scope brief|standard|full] [--project <project>]

ytd config set visibility-group <group>
ytd config get visibility-group
ytd config unset visibility-group
ytd group list

ytd project list
ytd project get <id>

ytd user list
ytd user get <user-id-or-login>

ytd article search <query> [--project <id>]
ytd article list --project <id>
ytd article get <id> [--no-comments]
ytd article create --project <id> --json '...'
ytd article update <id> --json '...'
ytd article move <id> <parent-id|none>
ytd article append <id> <text>
ytd article comment <id> <text> [--visibility-group <group> | --no-visibility-group]
ytd article comments <id>
ytd article attach <id> <file>
ytd article attachments <id>
ytd article delete <id> [-y]

ytd ticket search <query> [--project <id>]
ytd ticket list --project <id>
ytd ticket get <id> [--no-comments]
ytd ticket create --project <id> --json '...'
ytd ticket update <id> --json '...'
ytd ticket comment <id> <text> [--visibility-group <group> | --no-visibility-group]
ytd ticket comments <id>
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
ytd ticket delete <id> [-y]

ytd comment get <comment-id>
ytd comment update <comment-id> <text> [--visibility-group <group> | --no-visibility-group]
ytd comment attachments <comment-id>
ytd comment delete <comment-id> [-y]

ytd attachment get <attachment-id>
ytd attachment delete <attachment-id> [-y]
ytd attachment download <attachment-id> [--output <path>]

ytd alias create <alias> [--project <project-id>] [--user <user-id>] [--sprint <sprint-id|none>]
ytd alias list
ytd alias delete <alias> [-y]
ytd <alias> create <text>
ytd <alias> list [--all]

ytd tag list [--project <id>]
ytd search list [--project <id>]
ytd search run <name-or-id>

ytd board list [--project <id>]
ytd board get <id>
ytd board create --name <name> --project <project>[,<project>...] [--template <template>] [--json '{...}']
ytd board update <id> [--name <name>] [--json '{...}']
ytd board delete <id> [-y]

ytd sprint list [--board <board-id>]
ytd sprint current [--board <board-id>]
ytd sprint get <sprint-id>
ytd sprint create --board <board-id> --name <name> [--json '{...}']
ytd sprint update <sprint-id> [--name <name>] [--json '{...}']
ytd sprint delete <sprint-id> [-y]
ytd sprint ticket list <sprint-id>
ytd sprint ticket add <sprint-id> <ticket-id>
ytd sprint ticket remove <sprint-id> <ticket-id>
```

## Public CLI Contracts

Public command behavior is governed by `.agents/io-consistency.md`.

- Validate command names and global output formats before loading auth config.
- Support `ytd help <command>` and `ytd <command> help`.
- Supported formats are exactly `text`, `json`, `raw`, and `md`.
- `text` is default and optimized for humans and AI-agent context windows.
- `json` is the stable ytd-normalized scripting format.
- `raw` is YouTrack API-shaped JSON.
- `md` is Markdown export where supported.
- `--no-meta` suppresses metadata fields where applicable.
- Successful data goes to stdout; errors, prompts, and diagnostics go to stderr.
- Tokens and credentials must never be printed.
- Create/update/delete commands that return one changed resource print only its reusable public ID on stdout.
- Delete commands require interactive `yes` or `-y`; non-interactive delete without `-y` must not mutate.

## Public IDs

Any CLI-facing `id` must be reusable as input to the corresponding command. Raw YouTrack IDs, when exposed, use `ytId`.

| Resource | Public ID |
|---|---|
| Ticket | readable issue ID, for example `DWP-28` |
| Article | readable article ID, for example `DWP-A-1` |
| Comment | `<ticket-id>:<comment-id>` or `<article-id>:<comment-id>` |
| Attachment | `<ticket-id>:<attachment-id>` or `<article-id>:<attachment-id>` |
| Sprint | `<board-id>:<sprint-id>` |

Normalized ticket and article outputs do not expose `idReadable`.
Article detail output includes normalized `parentArticle` when present: `id` is the readable reusable article ID and `ytId` is the raw YouTrack ID.

## Text Output

- Markdown content fields (`content`, `description`, `text`) render as readable terminal text in `--format text`.
- Content fields print after metadata and scalar fields, separated by a blank line, without a field label.
- `ticket search`, `ticket list`, `search run`, `sprint ticket list`, and linked ticket text output use compact ticket rows.
- `ticket get` uses a detail report: title, status/custom fields, metadata, blank line, description, then comments.
- `--no-comments` removes comments from every supported format for commands that document it.

## Visibility

- `ticket create`, `article create`, `ticket comment`, and `article comment` apply configured visibility defaults.
- Default precedence for creates and new comments is CLI flag, then `YTD_VISIBILITY_GROUP`, then stored `visibilityGroup`.
- `--no-visibility-group` suppresses inherited defaults on creates and new comments.
- `ticket update`, `article update`, and `comment update` preserve existing visibility unless an explicit visibility flag is passed.
- `--visibility-group <group>` sets limited visibility.
- `--no-visibility-group` clears visibility on updates.
- Combining `--visibility-group` and `--no-visibility-group` is an input error.

## Article Parent

- `article create` and `article update` accept JSON `parentArticle: {"id":"PROJ-A-1"}` where `id` is a readable reusable article ID.
- ytd resolves the readable parent article ID and sends the internal YouTrack article ID to the API.
- `article update <id> --json '{"parentArticle":null}'` clears the parent article.
- `article move <id> <parent-id>` is a convenience command for changing the parent article; `article move <id> none` clears it.
- `article create` and `article update` reject unknown top-level JSON fields. Allowed article JSON fields are `summary`, `content`, and `parentArticle`.

## Aliases

Aliases are local config entries under `aliases`, not YouTrack resources. Stored alias values keep only IDs:

```json
{
  "aliases": {
    "todo": { "project": "0-96", "user": "1-51", "sprint": "108-4:113-6" },
    "backlog": { "project": "0-96", "user": "1-51" }
  }
}
```

The `sprint` key is optional. `alias create --sprint none` omits or clears it. `alias list` is config-backed, so `--format json` and `--format raw` intentionally return the same local alias model.

Dynamic alias commands are the exception to command-name validation before config loading: after checking built-in commands, `main.rs` may read local config to resolve `ytd <alias> create` and `ytd <alias> list`.

## Agent Skill Generation

`ytd skill` prints generated SKILL.md guidance for AI agents.

- Without `--project`, it must not require login.
- With `--project`, it resolves the project and embeds project-specific ticket/article examples.
- It accepts `--format text` and `--format md`.
- It rejects `--format json` and `--format raw`.
- Generated skills include the ytd version, regeneration command, JSON-first automation guidance, and reminders to use `ytd help`, `ytd help <command>`, or `ytd <command> help`.

## Configuration

Config file:

```text
~/.config/ytd/config.json
```

Environment variables:

| Variable | Purpose |
|---|---|
| `YTD_CONFIG` | Custom config path |
| `YTD_VISIBILITY_GROUP` | Default visibility group for creates and new comments |
| `YOUTRACK_URL` | Override config URL |
| `YOUTRACK_TOKEN` | Override config token |
| `YOUTRACK_TEST_URL` | Integration test target |
| `YOUTRACK_TEST_TOKEN` | Integration test token |
| `YOUTRACK_TEST_PROJECT` | Integration test project ID |

Credential resolution order is env vars, then config file, then an input error telling the user to run `ytd login`.

## YouTrack API Notes

- Base URL strips trailing slash and appends `/api`.
- Requests use `Authorization: Bearer <token>` and `Accept: application/json`.
- API calls should request explicit `fields`.
- List calls should set `$top` explicitly.
- Attachments use parent-scoped endpoints; downloads use signed attachment URLs.
- Sprint ticket assignment uses Agile/Sprint-scoped endpoints and resolves readable ticket IDs to internal issue database IDs.

## Agent Files

Read these files at the start of any non-trivial task:

| File | Contents |
|---|---|
| `.agents/architect.md` | Directory structure, module boundaries, command wiring |
| `.agents/io-consistency.md` | Public CLI contracts |
| `.agents/reviewer.md` | Code review standards |
| `.agents/tester.md` | Test types, conventions, user journeys |
| `.agents/memory.md` | Non-obvious YouTrack API quirks and durable external facts |
| `.agents/process.md` | Tooling commands, commit rules, env vars |

## Development Rules

- Prefer the existing style and helper APIs.
- Keep public behavior aligned across `src/help.rs`, `README.md`, `CLAUDE.md`, relevant `.agents/` files, and user journeys.
- Core modules must stay independent from command handlers.
- For public CLI changes, update tests and `.agents/io-consistency.md`.
- Stage specific files; never use `git add .`.
- Do not commit tokens, `.env`, or `target/`.
