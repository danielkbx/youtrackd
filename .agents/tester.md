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
| Attachments | `05-attachments.md` | ticket/article attach/attachments |
| Time Tracking | `06-time-tracking.md` | ticket log/worklog |
| Custom Fields | `07-custom-fields.md` | ticket fields/set |
| Searches & Boards | `08-search-and-boards.md` | search list/run, board list/get |
| History | `09-history.md` | ticket history |

**Cleanup-Regeln**: Tickets und Artikel per `delete -y` löschen, Tags vor Delete entfernen. Details in `PROCESS.md`.

**Naming**: Alle Test-Entities verwenden Prefix `[YTD-TEST]` in Summary/Kommentaren.

## Conventions

No shared test utilities unless used in 3+ test files.
