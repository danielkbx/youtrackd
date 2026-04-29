# Input/Output Consistency Rules

These rules define the expected CLI surface for new and changed commands. Treat them as normative unless a command documents a deliberate exception.

## Command Shape

- Use `ytd <resource> <action>` for resource operations.
- Use nested actions only when the target is naturally scoped by another resource, for example `sprint ticket add`.
- `ytd skill` is a deliberate top-level no-action exception because it prints current SKILL.md guidance for AI agents.
- `ytd completion <bash|zsh|fish>` is a deliberate top-level no-auth exception because it prints static shell completion scripts to stdout.
- `ytd schema` is a deliberate top-level no-auth exception because it prints static JSON input contracts for commands accepting `--json` or stdin.
- Support `ytd help <resource>` and `ytd <resource> help` for every public resource.
- Validate command/resource/action names before loading config, so typos return "Unknown command" instead of auth errors.
- Validate global output flags before loading config for formatted commands.

## Input

- Required identifiers are positional and come before free text.
- Free text positionals may consume the remaining arguments, for example comment text or worklog text.
- Structured create/update input uses JSON via `--json` or stdin.
- Stdin JSON takes precedence over `--json`.
- Commands accepting JSON input must be documented in `ytd schema`.
- JSON commands must require a JSON object unless the command explicitly documents another shape.
- Structured JSON commands must reject unsupported top-level fields unless the command explicitly documents API pass-through behavior.
- Create commands require all fields needed to create a useful resource.
- Update commands require at least one actual update field or an explicit update flag such as `--visibility-group` or `--no-visibility-group`.
- Boolean/destructive confirmation uses flags, not JSON.

## Filters

- Filter flags must either filter deterministically or be rejected; never silently ignore a supported-looking filter.
- If filtering is client-side, document that in help and README.
- If filtering is approximate because the API lacks a first-class field, document the matching basis. Example: `search list --project` filters by project reference in the saved query text.

## Output Formats

- Supported `--format` values are exactly `text`, `json`, `raw`, and `md`.
- Unknown format values are input errors.
- `text` is the default and optimized for humans and AI-agent context windows.
- `json` is ytd-normalized JSON and is the stable scripting format.
- `raw` is YouTrack API-shaped JSON and should stay as close to the API response as practical.
- `md` is Markdown export, not a general structured format.
- `--no-meta` suppresses metadata fields in formatted output where applicable.
- `ytd skill` is Markdown-first: `--format text` and `--format md` print generated SKILL.md content, while `--format json` and `--format raw` are rejected.
- `ytd completion` is shell-script-first and supports only the positional shells `bash`, `zsh`, and `fish`; it does not use structured output formats.
- `ytd schema` supports `--format text` and `--format json`; `raw` and `md` are rejected.

## Stdout And Stderr

- Successful data output goes to stdout.
- Errors, prompts, and diagnostics go to stderr.
- Create/update/delete commands that return a single mutated resource print only its reusable public ID on stdout.
- Commands that intentionally perform an action without returning a resource may be silent on success unless existing command behavior documents a status line.
- Attachment upload commands (`ticket attach`, `article attach`, `comment attach`) print `Attached <filename>` on success.
- Credentials and tokens must never be printed to stdout, stderr, logs, or error messages.

## Public IDs

- Any CLI-facing `id` must be reusable as input to the corresponding command.
- Raw YouTrack database IDs, when exposed, use `ytId`.
- Ticket outputs use the readable ticket ID as `id`, for example `DWP-28`.
- Article outputs use the readable article ID as `id`, for example `DWP-A-1`.
- Article `parentArticle` outputs use the readable article ID as nested `id` and expose the raw YouTrack article ID as nested `ytId`.
- `article move <id> <parent-id|none>` changes article hierarchy and prints only the moved article ID.
- Comment IDs encode parent scope as `<ticket-id>:<comment-id>` or `<article-id>:<comment-id>`.
- Attachment IDs encode parent scope as `<ticket-id>:<attachment-id>` or `<article-id>:<attachment-id>`.
- Sprint IDs encode board scope as `<board-id>:<sprint-id>`.
- Do not expose API-only `idReadable` in normalized ticket/article outputs.

## Delete

- Delete commands ask for confirmation when run interactively.
- Interactive confirmation requires exact `yes`.
- `-y` confirms deletion without prompting.
- Non-interactive delete without `-y` must fail and must not mutate.
- On successful delete, print the deleted public ID on stdout.

## Visibility

- `ticket create`, `article create`, `ticket comment`, and `article comment` apply configured visibility defaults.
- Create/new-comment default precedence is `--visibility-group` then `YTD_VISIBILITY_GROUP` then stored `visibility-group`.
- `--no-visibility-group` suppresses inherited defaults on create/new-comment commands and sends no visibility payload.
- `ticket update`, `article update`, and `comment update` preserve existing visibility unless an explicit visibility flag is passed.
- `--visibility-group <group>` sets limited visibility.
- `--no-visibility-group` clears visibility on updates by sending an empty permitted-groups payload.
- Combining `--visibility-group` and `--no-visibility-group` is an input error.

## Text Layout

- Markdown content fields (`content`, `description`, `text`) render as readable terminal text in `--format text`.
- Content fields print after metadata and scalar fields, separated by a blank line, without a field label.
- `ticket search`, `ticket list`, `search run`, `sprint ticket list`, and linked ticket text output use compact ticket rows.
- `ticket get` uses a detail report: title, status/custom fields, metadata, blank line, description, then comments.
- `--no-comments` must remove comments from all supported formats for commands that document it.

## Documentation

- Any public CLI behavior change must update `src/help.rs`, `README.md`, `CLAUDE.md`, relevant `.agents/` files, and user journeys when affected.
- Help text and README usage must match parser behavior exactly.
- `ytd help` and `ytd help skill` must clearly state that AI agents can run `ytd skill` themselves to fetch current ytd guidance. Generated skill content must also tell agents to use `ytd help`, `ytd help <command>`, or `ytd <command> help` before guessing command usage.
- `ytd help` must clearly advertise `ytd schema <resource> <action>` for JSON field discovery. Generated skill content must tell agents to use schema discovery before guessing JSON fields.
- If a command has a deliberate exception to these rules, document the exception near the command help and in README.
