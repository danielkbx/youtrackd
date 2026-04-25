# Development Process

## Planning

Before implementing any non-trivial task, read all files in `.agents/` and include that step explicitly in the plan.

For public CLI changes, follow `.agents/io-consistency.md` and update `help.rs`, `README.md`, `CLAUDE.md`, relevant `.agents/` files, and user journeys. Public comment outputs must document and preserve reusable encoded comment IDs.

## Tooling

| Task | Command |
|---|---|
| Dev run | `cargo run -- <args>` |
| Build release | `cargo build --release` |
| Test | `cargo test` |
| Lint | `cargo clippy -- -D warnings` |
| Format | `cargo fmt` |
| Size check | `ls -lh target/release/ytd` |

## Commits

- Conventional commits: `feat:`, `fix:`, `refactor:`, `test:`, `chore:`
- Do not use scopes in commit prefixes: use `fix: ...`, not `fix(alias): ...`
- Message describes *why*, not *what*
- Stage specific files — never `git add .`
- Never commit: `.env`, tokens, `target/`

## Branching

- `main` — stable, releasable
- `feat/<name>` — new commands or features
- `fix/<name>` — bug fixes

## Config Files

- `~/.config/ytd/config.json` — user config (XDG)
- `.env` — local dev overrides (never committed)

## Environment Variables

| Variable | Purpose |
|---|---|
| `YTD_CONFIG` | Custom config file path (overrides XDG) |
| `YTD_VISIBILITY_GROUP` | Default visibility group for ticket/article create and new comments |
| `YOUTRACK_URL` | Override config URL |
| `YOUTRACK_TOKEN` | Override config token |
| `YOUTRACK_TEST_URL` | Integration test target |
| `YOUTRACK_TEST_TOKEN` | Integration test token |
| `YOUTRACK_TEST_PROJECT` | Integration test project ID |
