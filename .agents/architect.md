# Architecture

## Directory Structure

```
src/
  main.rs           ← entry point, command routing, KNOWN-map validation
  args.rs           ← argument parsing (handwritten, no clap)
  client.rs         ← HttpTransport trait + YtClient + UreqTransport
  config.rs         ← credential resolution + storage (XDG, mode 600)
  duration.rs       ← parse_duration (30m, 1h, 2h30m → minutes)
  error.rs          ← YtdError enum
  format.rs         ← OutputOptions, text/JSON formatting, --no-meta
  help.rs           ← help text per command
  input.rs          ← JSON input from --json flag or stdin
  types.rs          ← all data structures (serde Serialize/Deserialize)
  commands/
    mod.rs          ← module declarations
    config.rs       ← stored non-auth settings (visibility-group)
    group.rs        ← group list
    login.rs        ← interactive login flow
    logout.rs       ← clear credentials
    whoami.rs       ← current user display
    project.rs      ← project list/get
    article.rs      ← article CRUD + comments + attachments + delete
    ticket.rs       ← ticket CRUD + tags + links + attachments + time + custom fields + history + delete
    comment.rs      ← global comment get/update/delete
    tag.rs          ← tag list (client-side project filter)
    search.rs       ← saved search list/run
    board.rs        ← agile board list/get (client-side project filter)
```

## Module Boundary

Core modules (`client.rs`, `config.rs`, `types.rs`, `error.rs`) have no CLI dependencies.
`commands/` own all I/O (stdout, stderr, stdin prompts). Boundary enforced by Rust module DAG.

API structs may contain raw YouTrack comment IDs. Any CLI-facing comment output must normalize comment IDs before formatting: `id` is the encoded ytd ID, while the raw YouTrack ID is exposed only as `ytId`.

## HttpTransport Trait

```rust
pub trait HttpTransport {
    fn get(&self, url: &str, token: &str) -> Result<String, YtdError>;
    fn post(&self, url: &str, token: &str, body: &str) -> Result<String, YtdError>;
    fn post_multipart(&self, url: &str, token: &str, file_path: &Path, file_name: &str) -> Result<String, YtdError>;
    fn delete(&self, url: &str, token: &str) -> Result<(), YtdError>;
}
```

Production: `UreqTransport`. Tests: `MockTransport` with canned responses.

## Command Validation

Command names are validated against a KNOWN-map in `main.rs` **before** loading config. This prevents misleading "Not logged in" errors on typos.

## Help System

Both `ytd help` and `ytd <command> help` work. Output is plain text — no Markdown, no ANSI colors.

## Config Module

`config.rs` owns credential loading plus stored CLI defaults:
- `get_config()` — resolves env vars → `~/.config/ytd/config.json` → error
- `save_config(c)` — writes file with `OpenOptionsExt::mode(0o600)` (no race condition)
- `clear_config()` — deletes file
- `resolve_visibility_group()` — resolves CLI flag → `YTD_VISIBILITY_GROUP` → stored config, with `--no-visibility-group` as explicit override

## Visibility Defaults

- Stored config is `StoredConfig` in `types.rs` with optional `url`, `token`, and `visibility_group`
- Serialized config uses camelCase JSON keys, so the file stores `visibilityGroup`
- Ticket/article create and update handlers build `LimitedVisibilityInput` in `commands/ticket.rs` and `commands/article.rs`
- `ResolvedVisibilityGroup::Clear` becomes an empty `permittedGroups` payload only for updates; creates omit the `visibility` field

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
- Attachments: manual multipart/form-data body building
- Errors: HTTP status + detail to stderr, exit non-zero
