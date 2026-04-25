# Code Review Standards

## Non-negotiable

- Core must not import from `src/cli/` — check every new import
- Tokens and credentials must never appear in logs, errors, or stdout
- Create/update commands print only the resource ID on stdout (nothing else)
- Errors go to stderr, exit code non-zero
- `--format` and `--no-meta` flags must be respected by every command
- Unknown `--format` values must error, not fall back to text
- Delete commands must never mutate in non-TTY contexts without `-y`

## Input Handling

- `--json` flag and stdin both accepted for structured input
- Stdin takes precedence over `--json` when both are provided

## CLI Entry Point

- Kommando-Validierung (KNOWN-Map) muss VOR `getConfig()` stattfinden, sonst erhalten Nutzer bei Tippfehlern irreführende Auth-Fehler
- Help-Routing: `ytd help <cmd>` nutzt `argv.action`, nicht `argv.positional[0]` — beide Varianten (`ytd help ticket` und `ytd ticket help`) müssen abgedeckt sein

## Scope

- Changes should touch only what the task requires
- No cleanup of unrelated code
- No speculative exports, utilities, or abstractions

## Report Format

```
BLOCKER: <issue>
WARN: <issue>
OK
```
