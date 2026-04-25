# Journey 18: Aliases

Tests: `user list`, `user get`, `alias create`, `alias list`, `alias delete`, isolated alias config, `--sprint none`, dynamic `ytd <alias> create <text>`, and dynamic `ytd <alias> list [--all]`.

## Cleanup

This journey creates up to two tickets, one Agile board, one sprint, and two local aliases in an isolated config file. Always run cleanup, even if an intermediate step fails:

```bash
env YTD_CONFIG="$ALIAS_CONFIG" ytd alias delete todo -y || true
env YTD_CONFIG="$ALIAS_CONFIG" ytd alias delete backlog -y || true
env YTD_CONFIG="$ALIAS_CONFIG" ytd ticket delete "$TODO_TICKET_ID" -y || true
env YTD_CONFIG="$ALIAS_CONFIG" ytd ticket delete "$BACKLOG_TICKET_ID" -y || true
env YTD_CONFIG="$ALIAS_CONFIG" ytd sprint delete "$SPRINT_ID" -y || true
env YTD_CONFIG="$ALIAS_CONFIG" ytd board delete "$BOARD_ID" -y || true
rm -f "$ALIAS_CONFIG"
```

## Prerequisites

- `$PROJECT` is confirmed by the user.
- The current user can create and delete tickets in `$PROJECT`.
- The current user can create and delete Agile boards and sprints for `$PROJECT`.
- `jq` is available for extracting IDs from JSON.
- All created YouTrack entities use the prefix `[YTD-TEST]`.

If the current user lacks Agile board or sprint permissions, document the permission error and mark the sprint-bound parts as blocked. Still run the user and config-only alias checks when possible.

## Setup

### 1. Create isolated config

```bash
ALIAS_CONFIG=$(mktemp /tmp/ytd-aliases.XXXXXX.json)
```

Populate `$ALIAS_CONFIG` with valid `url` and `token` values for the target instance. If the current login uses the default config file, copy it as a starting point. If auth comes from `YOUTRACK_URL` and `YOUTRACK_TOKEN`, write those values into this file.

Expected: `env YTD_CONFIG="$ALIAS_CONFIG" ytd whoami` succeeds.

### 2. Resolve project database ID

```bash
PROJECT_DB_ID=$(env YTD_CONFIG="$ALIAS_CONFIG" ytd project get "$PROJECT" --format raw | jq -r '.id')
```

Expected: `$PROJECT_DB_ID` is a YouTrack project database ID such as `0-96`, not only the project short name.

### 3. List users

```bash
env YTD_CONFIG="$ALIAS_CONFIG" ytd user list --format json
```

Expected: Exit code 0. Output is a JSON array. Each user has a reusable `id`; login/name fields are present when YouTrack returns them.

### 4. Resolve current or test user

Pick a user the test may assign tickets to. Prefer the current user. Set:

```bash
USER_ID=<user-id>
USER_LOGIN=<user-login>
```

Expected: `$USER_ID` is a YouTrack user ID such as `1-51`. `$USER_LOGIN` is set when available.

### 5. Get user by ID

```bash
env YTD_CONFIG="$ALIAS_CONFIG" ytd user get "$USER_ID" --format json
```

Expected: Exit code 0. Output has `id == "$USER_ID"`.

### 6. Get user by login

```bash
env YTD_CONFIG="$ALIAS_CONFIG" ytd user get "$USER_LOGIN" --format json
```

Expected: Exit code 0. Output has `id == "$USER_ID"`. If the instance does not expose a usable login, document that and skip only this step.

## Sprint-bound alias

### 7. Create board

```bash
BOARD_ID=$(env YTD_CONFIG="$ALIAS_CONFIG" ytd board create --name "[YTD-TEST] Alias Board" --project "$PROJECT" --template scrum)
```

Expected: Exit code 0. Stdout contains only the board ID.

### 8. Create sprint

```bash
SPRINT_ID=$(env YTD_CONFIG="$ALIAS_CONFIG" ytd sprint create --board "$BOARD_ID" --name "[YTD-TEST] Alias Sprint")
```

Expected: Exit code 0. Stdout contains a public sprint ID in the format `<board-id>:<sprint-id>`.

### 9. Create `todo` alias

```bash
env YTD_CONFIG="$ALIAS_CONFIG" ytd alias create todo --project "$PROJECT_DB_ID" --user "$USER_ID" --sprint "$SPRINT_ID"
```

Expected: Exit code 0. Stdout contains only `todo`.

### 10. Verify stored alias config contains IDs only

```bash
jq '.aliases.todo' "$ALIAS_CONFIG"
```

Expected: The object is exactly ID-backed data:

```json
{ "project": "0-96", "user": "1-51", "sprint": "108-4:113-6" }
```

Values should match `$PROJECT_DB_ID`, `$USER_ID`, and `$SPRINT_ID`. The object must not contain project names, user names, login labels, or board/sprint names.

### 11. Alias list text

```bash
env YTD_CONFIG="$ALIAS_CONFIG" ytd alias list
```

Expected: Exit code 0. Text output includes `todo` and the stored project/user/sprint IDs.

### 12. Alias list JSON and raw match

```bash
env YTD_CONFIG="$ALIAS_CONFIG" ytd alias list --format json > /tmp/ytd-alias-json.out
env YTD_CONFIG="$ALIAS_CONFIG" ytd alias list --format raw > /tmp/ytd-alias-raw.out
diff -u /tmp/ytd-alias-json.out /tmp/ytd-alias-raw.out
```

Expected: Exit code 0 for both `alias list` commands and for `diff`. `json` and `raw` intentionally return the same config-backed alias data model.

### 13. Dynamic alias create

```bash
TODO_TICKET_ID=$(env YTD_CONFIG="$ALIAS_CONFIG" ytd todo create "[YTD-TEST] Alias Todo Ticket")
```

Expected: Exit code 0. Stdout contains only the readable ticket ID.

### 14. Created ticket is assigned to alias context

```bash
env YTD_CONFIG="$ALIAS_CONFIG" ytd ticket get "$TODO_TICKET_ID" --format raw
env YTD_CONFIG="$ALIAS_CONFIG" ytd ticket sprints "$TODO_TICKET_ID" --format json
```

Expected: The ticket belongs to `$PROJECT`, references `$USER_ID` as assignee where the YouTrack fields expose it, and has a sprint with `id == "$SPRINT_ID"`.

### 15. Dynamic alias list uses compact ticket formatter

```bash
env YTD_CONFIG="$ALIAS_CONFIG" ytd todo list
env YTD_CONFIG="$ALIAS_CONFIG" ytd ticket list --project "$PROJECT"
```

Expected: Both commands use the same compact ticket row shape. `ytd todo list` contains `$TODO_TICKET_ID` and its summary. Field ordering and labels should match the shared `ticket list` formatter.

### 16. Dynamic alias list JSON

```bash
env YTD_CONFIG="$ALIAS_CONFIG" ytd todo list --all --format json
```

Expected: Exit code 0. Output is a JSON array using the same normalized ticket issue model as `ticket list`, with reusable ticket IDs in `id` and raw database IDs only in `ytId`.

## Alias without sprint

### 17. Create `backlog` alias with no sprint

```bash
env YTD_CONFIG="$ALIAS_CONFIG" ytd alias create backlog --project "$PROJECT_DB_ID" --user "$USER_ID" --sprint none
```

Expected: Exit code 0. Stdout contains only `backlog`.

### 18. Verify `sprint` is omitted

```bash
jq '.aliases.backlog' "$ALIAS_CONFIG"
```

Expected: The object has `project` and `user` only. It does not contain a `sprint` key.

### 19. Dynamic no-sprint alias create

```bash
BACKLOG_TICKET_ID=$(env YTD_CONFIG="$ALIAS_CONFIG" ytd backlog create "[YTD-TEST] Alias Backlog Ticket")
```

Expected: Exit code 0. Stdout contains only the readable ticket ID.

### 20. No-sprint ticket has no forced sprint

```bash
env YTD_CONFIG="$ALIAS_CONFIG" ytd ticket sprints "$BACKLOG_TICKET_ID" --format json
```

Expected: Output does not require `$SPRINT_ID`. If YouTrack board defaults add the ticket to a sprint automatically, document that as instance behavior; the alias config itself must still omit `sprint`.

### 21. Delete aliases

```bash
env YTD_CONFIG="$ALIAS_CONFIG" ytd alias delete todo -y
env YTD_CONFIG="$ALIAS_CONFIG" ytd alias delete backlog -y
```

Expected: Exit code 0. Each command prints the deleted alias name.

### 22. Deleted dynamic alias is unavailable

```bash
env YTD_CONFIG="$ALIAS_CONFIG" ytd todo list
```

Expected: Exit code non-zero. Error indicates the command or alias is unknown.

## Cleanup

### 23. Delete created tickets

```bash
env YTD_CONFIG="$ALIAS_CONFIG" ytd ticket delete "$TODO_TICKET_ID" -y
env YTD_CONFIG="$ALIAS_CONFIG" ytd ticket delete "$BACKLOG_TICKET_ID" -y
```

Expected: Exit code 0 for each existing ticket.

### 24. Delete sprint and board

```bash
env YTD_CONFIG="$ALIAS_CONFIG" ytd sprint delete "$SPRINT_ID" -y
env YTD_CONFIG="$ALIAS_CONFIG" ytd board delete "$BOARD_ID" -y
```

Expected: Exit code 0.

### 25. Remove isolated config

```bash
rm -f "$ALIAS_CONFIG" /tmp/ytd-alias-json.out /tmp/ytd-alias-raw.out
```

Expected: Temporary files are removed. The user's default `~/.config/ytd/config.json` was not modified by this journey.
