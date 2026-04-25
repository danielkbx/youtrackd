# Testing

## Test Runner

Rust's built-in test framework. Run with `cargo test`.

## Test Types

| Type | Location | When to run |
|---|---|---|
| Unit | `src/*.rs` (inline `#[cfg(test)]`) | Always |
| CLI | `tests/` (integration) | Always |
| User Journeys | `user-journeys/` | Manually by AI agent |

## Unit Tests

Inline `#[cfg(test)]` modules in each source file. Mock the HTTP layer via `MockTransport` — never mock business logic.

- `config.rs` — config read/write/clear (uses tempfile, Mutex for env var isolation)
- `commands/config.rs` — config set/get/unset for `visibility-group`
- `args.rs` — argument parsing variants
- `format.rs` — output options, timestamp formatting, user/bool formatting
- `duration.rs` — duration parsing (30m, 1h, 2h30m, edge cases)
- `client.rs` — API methods with MockTransport (get_me, list_projects, create_issue, etc.)

## CLI Integration Tests

Spawn `target/debug/ytd` as a subprocess. Assert on stdout, stderr, and exit code.

For each command, verify:
1. Happy path output is correct for `--format raw` and `--format text`
2. `--no-meta` suppresses metadata fields
3. Create/update commands print only the ID on stdout
4. Invalid input exits non-zero with a message on stderr
5. Stdin JSON input works equivalently to `--json` flag
6. Public input/output behavior follows `.agents/io-consistency.md`

Visibility-specific coverage:
1. `config set|get|unset visibility-group` works without login and persists only stored config fields
2. Create and new-comment visibility resolution precedence is CLI flag → `YTD_VISIBILITY_GROUP` → stored config
3. `--visibility-group` combined with `--no-visibility-group` is rejected as input error
4. Ticket/article/comment update builders ignore env/config defaults unless explicit visibility flags are passed
5. Update builders turn `--no-visibility-group` into a clear payload; create builders omit `visibility`

Delete-specific coverage:
1. Non-interactive delete without `-y` exits non-zero and does not mutate
2. Delete with `-y` mutates and prints the deleted public ID

Group-specific coverage:
1. `group list` requests `usersCount`

User-specific coverage:
1. `user list` returns reusable user IDs and login/name fields where available
2. `user get <user-id-or-login>` accepts both a YouTrack user ID and a login

Alias-specific coverage:
1. `alias create` persists only project/user/sprint IDs under `aliases`
2. `--sprint none` clears or omits the stored `sprint` field
3. `alias list` reads local config and returns the same data model for `--format json` and `--format raw`
4. `alias delete` follows standard delete confirmation behavior
5. `ytd <alias> create <text>` creates a ticket using the alias context and prints only the ticket ID
6. `ytd <alias> list [--all]` uses the shared compact ticket formatter and matches `ticket list` output shape

## Rust-specific Test Notes

- Config tests share env vars → use `Mutex` to serialize tests that modify `XDG_CONFIG_HOME` / `YOUTRACK_URL` / `YOUTRACK_TOKEN`
- MockTransport uses `RefCell<Vec<String>>` for response queue
- `tempfile::tempdir()` for filesystem isolation
- No external test framework needed — `#[test]` + `assert_eq!` suffices

## User Journey Tests

Verzeichnis: `user-journeys/`

End-to-End-Tests, die ein AI-Agent gegen eine echte YouTrack-Instanz ausführt. Jede Journey-Datei beschreibt einen vollständigen Durchlauf mit Schritten, Erwartungen und Cleanup.

**Ablauf**: Siehe `user-journeys/PROCESS.md`. Wichtigste Regel: Der Agent muss den User **vor dem Start explizit nach dem Zielprojekt fragen** und die Bestätigung abwarten.

| Journey | Datei | Testet |
|---|---|---|
| Auth & Projekte | `01-auth-and-projects.md` | whoami, project list/get, --format, --no-meta |
| Ticket-Lifecycle | `02-ticket-lifecycle.md` | ticket create/get/update/comment/search/list |
| Artikel-Lifecycle | `03-article-lifecycle.md` | article create/get/update/append/comment/comments/search/list |
| Tags & Links | `04-tags-and-links.md` | tag list, ticket tag/untag/link/links |
| Attachments | `05-attachments.md` | ticket/article attach/attachments, global attachment get/delete/download, comment attachment listing |
| Time Tracking | `06-time-tracking.md` | ticket log/worklog |
| Custom Fields | `07-custom-fields.md` | ticket fields/set |
| Searches & Boards | `08-search-and-boards.md` | search list/run, board list/get |
| History | `09-history.md` | ticket history |
| Kommentare | `12-comments.md` | ticket/article comments, global comment get/update/delete/attachments, encoded comment IDs |
| Board CRUD | `13-board-crud.md` | board create/update/get/list/delete, JSON/stdin input, validation |
| Sprint CRUD | `14-sprint-crud.md` | sprint create/update/get/list/delete/current, JSON input, sprint-id validation |
| Current Sprints | `15-current-sprints.md` | sprint current across all boards and reusable sprint IDs |
| Ticket Sprints | `16-ticket-sprints.md` | ticket sprints output and reusable sprint IDs |
| Sprint Ticket Assignment | `17-sprint-ticket-assignment.md` | sprint ticket list/add/remove, board-scoped sprint IDs, duplicate add, remove errors |
| Aliases | `18-aliases.md` | user list/get, alias create/list/delete, dynamic alias create/list, config-backed output |
| Skill Generation | `19-skill-generation.md` | agent SKILL.md generation, scopes, version/update instructions, project context |

**Cleanup-Regeln**: Tickets und Artikel per `delete -y` löschen, Tags vor Delete entfernen. Details in `PROCESS.md`.
Boards per `board delete -y` löschen.

When extending the ticket/article journeys, include visibility coverage for inherited create defaults, update preservation without explicit visibility flags, explicit `--visibility-group`, and `--no-visibility-group` clear behavior.


**Naming**: Alle Test-Entities verwenden Prefix `[YTD-TEST]` in Summary/Kommentaren.

**Comment IDs**: Any command that exposes comment objects must return a reusable encoded `id` accepted by `ytd comment get`. Raw YouTrack comment IDs may appear only as `ytId`.

**Attachment IDs**: Any command that exposes attachment objects must return a reusable encoded `id` accepted by `ytd attachment get|delete|download`. Raw YouTrack attachment IDs may appear only as `ytId`.

**Sprint IDs**: Any command that exposes sprint objects must return a reusable sprint-id in `id` using `<board-id>:<sprint-id>`, accepted by `ytd sprint get|update|delete`. Raw YouTrack sprint IDs may appear only as `ytId`.

**Sprint ticket assignment**: Use encoded public sprint IDs in the form `<board-id>:<sprint-id>`. Tests must verify that removing from one sprint does not assume anything about other board sprint assignments returned by `ticket sprints`.

**Comment visibility**: Journey 12 requires `$VIS_GROUP`. It must verify that comment creation applies defaults, `comment update` without visibility flags preserves existing visibility, `--no-visibility-group` clears it, and `--visibility-group` sets it again.

**Aliases**: Journey 18 must use an isolated `YTD_CONFIG` file with valid credentials so alias create/list/delete does not mutate the user's normal config. Alias config stores only IDs; names and readable labels must not be persisted in `aliases`.

**Skill Generation**: Journey 19 is review-oriented rather than exact-text oriented. It must verify that generated skills include valid frontmatter, JSON-first agent guidance, help lookup guidance (`ytd help`, `ytd help <command>`, `ytd <command> help`), current ytd version, regeneration instructions with the same effective scope, and resolved project context/examples when `--project` is used.

## Conventions

No shared test utilities unless used in 3+ test files.
