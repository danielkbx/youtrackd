# Architecture

## Directory Structure

```
src/
  main.rs           ← entry point, command routing, KNOWN-map validation
  args.rs           ← argument parsing (handwritten, no clap)
  cli_spec.rs       ← canonical public CLI model for completion generation
  completion.rs     ← Bash/Zsh/Fish completion renderers
  client.rs         ← HttpTransport trait + YtClient + UreqTransport
  config.rs         ← credential resolution + storage (XDG, mode 600)
  duration.rs       ← parse_duration (30m, 1h, 2h30m → minutes)
  error.rs          ← YtdError enum
  format.rs         ← OutputOptions, text/JSON formatting, --no-meta
  help.rs           ← help text per command
  input.rs          ← JSON input from --json flag or stdin
  types.rs          ← all data structures (serde Serialize/Deserialize)
  commands/
    mod.rs          ← module declarations + shared command helpers
    config.rs       ← stored non-auth settings (visibility-group)
    open_target.rs  ← shared web target parsing for open/url
    open.rs         ← open YouTrack web URL in default browser
    url.rs          ← print YouTrack web URL
    visibility.rs   ← shared visibility flag/default handling
    group.rs        ← group list
    login.rs        ← interactive login flow
    logout.rs       ← clear credentials
    whoami.rs       ← current user display
    user.rs         ← user list/get
    alias.rs        ← local alias config + dynamic alias ticket workflows
    project.rs      ← project list/get
    article.rs      ← article CRUD + parent move + comments + attachments + delete
    ticket.rs       ← ticket CRUD + tags + links + attachments + time + custom fields + history + delete
    comment.rs      ← global comment get/update/delete + comment attachment upload/listing
    attachment.rs   ← global attachment get/delete/download
    tag.rs          ← tag list (client-side project filter)
    search.rs       ← saved search list/run
    board.rs        ← agile board list/get/create/update/delete (client-side project filter)
    sprint.rs       ← sprint list/current/get/create/update/delete and nested sprint ticket list/add/remove (board-scoped sprint IDs)
    skill.rs        ← generated SKILL.md guidance for AI agents (optional project context)
```

## Module Boundary

Core modules (`client.rs`, `config.rs`, `types.rs`, `error.rs`) have no CLI dependencies.
`commands/` own all I/O (stdout, stderr, stdin prompts). Boundary enforced by Rust module DAG.

API structs may contain raw YouTrack comment IDs. Any CLI-facing comment output must normalize comment IDs before formatting: `id` is the encoded ytd ID, while the raw YouTrack ID is exposed only as `ytId`.

API structs may contain raw YouTrack attachment IDs. Any CLI-facing attachment output must normalize attachment IDs before formatting: `id` is the encoded ytd ID, while the raw YouTrack ID is exposed only as `ytId`.

API structs may contain raw YouTrack sprint IDs. Any CLI-facing sprint output must normalize sprint IDs before formatting: `id` is the ytd sprint-id `<board-id>:<sprint-id>`, while the raw YouTrack sprint ID is exposed only as `ytId`.

## HttpTransport Trait

```rust
pub trait HttpTransport {
    fn get(&self, url: &str, token: &str) -> Result<String, YtdError>;
    fn get_bytes(&self, url: &str, token: &str) -> Result<Vec<u8>, YtdError>;
    fn post(&self, url: &str, token: &str, body: &str) -> Result<String, YtdError>;
    fn post_multipart(&self, url: &str, token: &str, file_path: &Path, file_name: &str) -> Result<String, YtdError>;
    fn delete(&self, url: &str, token: &str) -> Result<(), YtdError>;
}
```

Production: `UreqTransport`. Tests: `MockTransport` with canned responses.

## Command Validation

Command names are validated against a KNOWN-map in `main.rs` **before** loading config. This prevents misleading "Not logged in" errors on typos.

Public command input/output behavior is governed by `.agents/io-consistency.md`.

Dynamic aliases are resolved as runtime commands after built-in command matching. This is the deliberate exception to command-name validation before config loading: if the first word is not a built-in command, `main.rs` may read local config to decide whether it is a configured alias. Alias management commands (`alias create|list|delete`) are config-backed rather than YouTrack API-backed. Stored alias config keeps only IDs:

```json
{ "aliases": { "todo": { "project": "0-96", "user": "1-51", "sprint": "108-4:113-6" } } }
```

The `sprint` key is optional and omitted when none. Because `alias list` has no API-shaped source response, `--format json` and `--format raw` intentionally return the same local alias data model. `ytd <alias> list` must use the same compact ticket formatter as `ticket list`.

## Help System

Both `ytd help` and `ytd <command> help` work. Output is plain text — no Markdown, no ANSI colors.

`ytd completion <bash|zsh|fish>` is a deliberate top-level no-auth command. It renders static shell completion scripts from `cli_spec.rs`, runs before config loading, writes only the generated script to stdout, and never calls YouTrack.

`ytd skill` is a deliberate top-level no-action command like `open` and `url`. It prints Markdown SKILL.md guidance for AI agents. Without `--project`, it must run before config loading and require no login. With `--project`, it loads config, resolves the project through YouTrack, and embeds resolved project context, including project-specific ticket/article ID examples. The global help must clearly state that AI agents can run `ytd skill` themselves to fetch current guidance. Generated skills must also point agents back to `ytd help` and command-specific help.

## Config Module

`config.rs` owns credential loading plus stored CLI defaults:
- `get_config()` — resolves env vars → `~/.config/ytd/config.json` → error
- `save_config(c)` — writes file with `OpenOptionsExt::mode(0o600)` (no race condition)
- `clear_config()` — deletes file
- `resolve_visibility_group()` — resolves CLI flag → `YTD_VISIBILITY_GROUP` → stored config, with `--no-visibility-group` as explicit override
- alias helpers store and read local alias definitions under `aliases`, preserving only project/user/sprint IDs

## Visibility Defaults

- Stored config is `StoredConfig` in `types.rs` with optional `url`, `token`, and `visibility_group`
- Serialized config uses camelCase JSON keys, so the file stores `visibilityGroup`
- Ticket/article create handlers and comment create handlers apply configured visibility defaults
- Ticket/article/comment update handlers build `LimitedVisibilityInput` only from explicit visibility flags
- `ResolvedVisibilityGroup::Clear` becomes an empty `permittedGroups` payload for updates; creates omit the `visibility` field

## Adding a Command

1. API method on `YtClient` in `client.rs`
2. Handler in `commands/<resource>.rs`
3. Register in `is_known_command()` and `match` in `main.rs`
4. Unit tests (inline `#[cfg(test)]` or MockTransport)
5. Update `help.rs`, `README.md`, `CLAUDE.md`, relevant `.agents/` files, and user journeys when public CLI behavior changes

## YouTrack API Client

- Base URL from config, trailing slash stripped, `/api` appended
- All requests: `Authorization: Bearer <token>`, `Accept: application/json`
- Always use `?fields=` to request only needed fields
- Always set `$top` explicitly (server default is 42)
- Attachments: manual multipart/form-data body building; downloads use signed attachment `url` values
- Comment attachment upload uses parent-scoped comment attachment endpoints: `/api/issues/{issueID}/comments/{commentID}/attachments` and `/api/articles/{articleID}/comments/{commentID}/attachments`
- Errors: HTTP status + detail to stderr, exit non-zero

Sprint ticket membership uses an Agile/Sprint-scoped API:

- Add: `POST /api/agiles/{agileID}/sprints/{sprintID}/issues`
- Remove: `DELETE /api/agiles/{agileID}/sprints/{sprintID}/issues/{issueDatabaseID}`
- List: `GET /api/agiles/{agileID}/sprints/{sprintID}?fields=issues(...)`

The CLI accepts readable ticket IDs but the client resolves them to internal issue database IDs before add/remove.
