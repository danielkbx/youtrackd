# ytd — YouTrack CLI

A CLI tool for reading and editing YouTrack tickets and knowledge base articles. Designed for both human and AI-agent use, with compact output to minimize context window usage.

## Project Status

Implemented in Rust. All commands functional, Rust test suite passing, binary builds to ~1.3 MB.

## Architecture

```
src/
  main.rs         ← entry point, command routing
  args.rs         ← argument parsing
  client.rs       ← HTTP transport trait + YouTrack API client
  config.rs       ← credential resolution + storage
  duration.rs     ← duration parsing (30m, 1h, 2h30m)
  error.rs        ← error types
  format.rs       ← output formatting (text/JSON/metadata)
  help.rs         ← help system
  input.rs        ← JSON input handling
  types.rs        ← all data structures
  commands/       ← command handlers (one file per resource)
user-journeys/    ← end-to-end test scripts for AI agents
.agents/          ← project context files
```

Core logic (client, config, types) has no CLI dependencies. Command handlers in `commands/` own formatting and I/O. Boundary enforced by Rust module visibility.

## Tech Stack

- **Language**: Rust
- **HTTP**: ureq 3 (sync, no async runtime)
- **JSON**: serde + serde_json
- **Build**: `cargo build --release` → standalone binary (~1.3 MB)
- **Distribution**: Binary via GitHub Releases
- **Auth**: YouTrack Permanent Token
- **Config**: `~/.config/ytd/config.json` (XDG)

## Commands

```
ytd help / ytd help <command> / ytd <command> help
ytd login / logout / whoami

ytd project list
ytd project get <id>

ytd user list
ytd user get <user-id-or-login>

ytd article search <query> [--project <id>]
ytd article list --project <id>
ytd article get <id> [--no-comments]
ytd article create --project <id> --json '...'
ytd article update <id> --json '...'
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
ytd ticket link <id> <target> [--type <t>]
ytd ticket links <id>
ytd ticket attach <id> <file>
ytd ticket attachments <id>
ytd ticket log <id> <duration> [text] [--date YYYY-MM-DD] [--type <worktype>]
ytd ticket worklog <id>
ytd ticket set <id> <field> <value>
ytd ticket fields <id>
ytd ticket history <id> [--category <cat>]
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

### Output flags (global)

| Flag | Default | Description |
|---|---|---|
| `--format json` | — | ytd-normalized JSON output (same data model as text) |
| `--format raw` | — | YouTrack API JSON, as original as possible |
| `--format text` | ✓ | Plain text; Markdown content is rendered as readable terminal text, including ASCII tables |
| `--format md` | — | Markdown (H1 title + body + comments) |
| `--no-meta` | — | Suppress metadata (IDs, dates, author) |
| `-y` | — | Confirm delete without prompting |

Create/update commands output only the ID on stdout (pipeable).
JSON input via `--json '{...}'` or stdin. Stdin takes precedence.
`--format` accepts only `text`, `raw`, `json`, or `md`; unknown values are input errors.
Delete commands ask for confirmation when interactive. Non-interactive delete requires `-y`.
`search list --project <id>` filters saved searches by project reference in the saved query text.
Input/output consistency rules are documented in `.agents/io-consistency.md`; public CLI changes should follow that checklist.

Ticket text output is specialized: `ticket search`, `ticket list`, `search run`, and `sprint ticket list` render compact ticket rows with ID, summary, project, important custom fields, and updated/resolved state. `ticket get` renders a detail report with status, all custom fields, metadata, then a blank line and the description without a label; comments follow the parent content. `--format text` renders Markdown content fields (`content`, `description`, `text`) as readable terminal text with ASCII tables and prints those content fields last after a blank line, without a field label. `--no-comments` omits comments from text, json, raw, and md output. `article get` also accepts `--no-comments`. `ticket links` renders linked issues with the same compact ticket fields. Use `--format json` for stable ytd-normalized JSON. Use `--format raw` for YouTrack API-shaped JSON.

The unlabeled Markdown content layout applies to `article get`, `article search` and `article list` when content is included, `article comments`, `ticket comments`, `comment get`, `ticket history` activity text, and future text output with `content`, `description`, or `text` fields. `ticket get` uses the same layout for its description; embedded ticket comments remain after the parent description.

Ticket issue outputs expose the reusable ticket ID as `id` (for example `<PROJECT>-<NUMBER>`). Raw YouTrack database IDs, when included, are exposed as `ytId`; ticket issue outputs do not expose `idReadable`.

Article outputs expose the reusable article ID as `id` (for example `<PROJECT>-A-<NUMBER>`). Raw YouTrack database IDs, when included, are exposed as `ytId`; article outputs do not expose `idReadable`.

Comment IDs returned by `ytd` encode the parent resource because YouTrack comment operations are parent-scoped:
`<ticket-id>:<comment-id>` or `<article-id>:<comment-id>`.
`ytd` infers the parent type from the parent ID shape: article IDs use `<PROJECT>-A-<NUMBER>`, tickets use `<PROJECT>-<NUMBER>`.
Use the public `id` field with `ytd comment ...`; raw YouTrack comment IDs may appear only as `ytId`.
Ticket/article creates and new comments apply configured visibility defaults. Ticket/article/comment updates preserve existing visibility unless `--visibility-group` or `--no-visibility-group` is passed explicitly.

Attachment IDs returned by `ytd` also encode the parent resource because YouTrack attachment operations are parent-scoped:
`<ticket-id>:<attachment-id>` or `<article-id>:<attachment-id>`.
Use the public `id` field with `ytd attachment ...`; raw YouTrack attachment IDs may appear only as `ytId`.
Comment attachments can be listed, but adding files to existing comments is not implemented because the verified REST API flow does not assign uploaded parent attachments to comments.

Sprint IDs returned by `ytd` encode the board because YouTrack sprint operations are board-scoped:
`<board-id>:<sprint-id>`.
Use the public `id` field with `ytd sprint get|update|delete` and `ytd sprint ticket ...`; raw YouTrack sprint IDs may appear only as `ytId`.
Use `ytd sprint current` to list current sprints across boards, or `ytd sprint current --board <board-id>` for one board. `current` is not accepted as a sprint-id.
Sprint ticket assignment is board-scoped. The public sprint ID must be `<board-id>:<sprint-id>`. `ytd` resolves readable ticket IDs to YouTrack internal issue database IDs before calling the Agile Sprint issues API.

Aliases are local config entries for dynamic ticket workflows. `alias create` stores only IDs under `aliases`, for example:
`{ "aliases": { "todo": { "project": "0-96", "user": "1-51", "sprint": "108-4:113-6" }, "backlog": { "project": "0-96", "user": "1-51" } } }`.
The `sprint` field is optional and omitted when none; `--sprint none` clears/omits it. `alias list` is config-backed, not YouTrack API-backed, so `--format json` and `--format raw` intentionally return the same alias data model. Dynamic alias commands are the deliberate exception to command-name validation before config loading: after checking built-in commands, `ytd` may read local config to resolve `ytd <alias> ...`. Dynamic alias listing (`ytd <alias> list [--all]`) uses the shared compact ticket formatter and must match `ticket list` output.

## Configuration

```json
~/.config/ytd/config.json
{
  "url": "https://your-instance.youtrack.cloud",
  "token": "perm:...",
  "aliases": {
    "todo": { "project": "0-96", "user": "1-51", "sprint": "108-4:113-6" },
    "backlog": { "project": "0-96", "user": "1-51" }
  }
}
```

File permissions: `600` (set atomically via `OpenOptionsExt::mode`).

Also readable from env: `YOUTRACK_URL`, `YOUTRACK_TOKEN` (takes precedence over config file).

Credential resolution order: env vars → config file → error ("Not logged in. Run `ytd login`.").

Custom config path via `YTD_CONFIG` env var (overrides XDG path):
```bash
# Multiple YouTrack instances via shell aliases
alias ytd-work='YTD_CONFIG=~/.config/ytd/work.json ytd'
alias ytd-oss='YTD_CONFIG=~/.config/ytd/oss.json ytd'
```

## Core Principles

### Think Before Coding
Don't assume. Don't hide confusion. Surface tradeoffs. State assumptions explicitly. Ask clarifying questions before implementing.

### Simplicity First
Minimum code that solves the problem. Nothing speculative. No unrequested features, no single-use abstractions, no impossible error handling.

### Surgical Changes
Touch only what you must. Clean up only your own mess. Match existing style. Every changed line must trace to the user's request.

### Goal-Driven Execution
Define success criteria before coding. Write tests before fixes. Verify refactored code still passes. Loop until verified.

## Agent Files

Read these files at the start of any non-trivial task:

| File | Contents |
|---|---|
| `.agents/architect.md` | Directory structure, module boundary, plugin setup |
| `.agents/reviewer.md` | Code review standards and report format |
| `.agents/tester.md` | Test types, conventions, user journeys |
| `.agents/memory.md` | Discovered API quirks and decisions not in the code |
| `.agents/process.md` | Tooling commands, commit rules, branching, env vars |
