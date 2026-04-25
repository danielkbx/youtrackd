# Project Memory

Discoveries and decisions not derivable from the code. Append new entries — never overwrite existing ones unless factually wrong.

---

## YouTrack API: Always use `?fields=` filter
Date: 2026-04-21
YouTrack returns all fields by default, which produces very large responses. Always request only needed fields via `?fields=id,summary,description,...`. This is critical for agent use cases where output goes into the context window. Apply this in every core API function from the start — it's hard to retrofit.

## YouTrack REST API Reference
Date: 2026-04-21
Official docs: https://www.jetbrains.com/help/youtrack/devportal/rest-api-reference.html

Key endpoint paths (verified from docs):
- Issues: `GET/POST /api/issues`, `GET/POST /api/issues/{issueID}`
- Issue Comments: `GET/POST /api/issues/{issueID}/comments`
- Articles: `GET/POST /api/articles`, `GET/POST /api/articles/{articleID}`
- Projects: `GET/POST /api/admin/projects`, `GET/POST /api/admin/projects/{projectID}`
- Current User: `GET /api/users/me` (returns `Me` entity, extends `User`)

## YouTrack API: Default $top is 42
Date: 2026-04-21
The server default for `$top` (max results) is 42 entries, not 100. Always set `$top` explicitly when listing resources to get predictable results. Use `$top=100` or higher as needed.

## YouTrack API: Article project is read-only after creation
Date: 2026-04-21
The Article entity's `project` field is marked read-only. It can be set during creation (POST /api/articles) but not changed afterward. The POST body for article creation requires `{ project: { id: "..." }, summary: "...", content: "..." }`.

## YouTrack API: Issue creation requires project database ID
Date: 2026-04-21
POST /api/issues requires `{ project: { id: "<databaseID>" }, summary: "..." }`. The `id` here is the internal database ID, not the shortName. Either resolve shortName → ID first via GET /api/admin/projects/{shortName}?fields=id, or test if the API also accepts shortName in the id field.

## YouTrack API: Articles support query parameter for search
Date: 2026-04-21
GET /api/articles accepts a `query` parameter, same as issues. No separate search endpoint needed. Filter by project using YouTrack search syntax in the query string.

## Implementierung: Route-Validierung vor Config-Laden
Date: 2026-04-21
In `src/cli/index.ts` muss die Validierung des Kommandos (ob es überhaupt existiert) VOR dem Laden der Config via `getConfig()` passieren. Sonst bekommt der Nutzer bei einem Tippfehler "Not logged in" statt "Unknown command" — irreführend und schwer zu debuggen. Lösung: `KNOWN`-Map im Entry-Point, die alle gültigen resource/action-Kombinationen kennt.

## Implementierung: fetch-Mock in Bun-Tests
Date: 2026-04-21
Bun's `typeof fetch` erfordert das `preconnect`-Property, das normale Mock-Funktionen nicht haben. Lösung: `as unknown as typeof fetch` beim Zuweisen von `globalThis.fetch` in Tests. Außerdem: `RequestInfo` ist in Bun nicht global verfügbar — stattdessen `string | URL | Request` verwenden.

## Implementierung: Bun + pnpm PATH-Problem
Date: 2026-04-21
Im Claude-Agent-Kontext sind Shell-Profile nicht geladen. Bun liegt in `~/.bun/bin/`, Node/pnpm benötigen nvm. Alle CLI-Befehle müssen mit:
```bash
export NVM_DIR="$HOME/.nvm" && source "$NVM_DIR/nvm.sh" && export PATH="$HOME/.bun/bin:$PATH"
```
…präfixiert werden. Alternativ: In `.agents/process.md` dokumentiert.

## YouTrack API: Issue comment POST requires only text field
Date: 2026-04-21
POST /api/issues/{issueID}/comments — required body: `{ text: "..." }`. Optional: visibility object. The muteUpdateNotifications query param suppresses notifications.

## Implementierung: Visibility-Group-Auflösung und Clear-Verhalten
Date: 2026-04-22
Verifiziert durch Code und Tests: Die Auflösung für Ticket/Article-Create+Update ist `--visibility-group` → `YTD_VISIBILITY_GROUP` → gespeicherter `config visibility-group`-Wert. `--no-visibility-group` überschreibt geerbte Defaults. Bei `update` wird dann ein `LimitedVisibility`-Payload mit leerem `permittedGroups` gesendet (Clear). Bei `create` wird das `visibility`-Feld stattdessen komplett weggelassen. Die Kombination `--visibility-group` + `--no-visibility-group` ist ein Input-Fehler.

Superseded for ticket/article updates on 2026-04-25: updates now preserve visibility unless explicit visibility flags are passed. See "Update visibility is explicit only" below.

## YouTrack API: Comment operations are parent-scoped
Date: 2026-04-24
Specific comment get/update/delete operations require the parent resource path: `/api/issues/{issueID}/comments/{commentID}` or `/api/articles/{articleID}/comments/{commentID}`. `ytd` therefore exposes encoded comment IDs as `<ticket-id>:<comment-id>` and `<article-id>:<comment-id>`. Parent type is inferred from ID shape: article IDs use `<PROJECT>-A-<NUMBER>`, ticket IDs use `<PROJECT>-<NUMBER>`. The public `id` field for any CLI comment output must always be encoded; raw YouTrack comment IDs may only appear as `ytId`.

## Implementierung: Comment visibility semantics
Date: 2026-04-24
New comments (`ticket comment`, `article comment`) apply the configured visibility default from `--visibility-group`, `YTD_VISIBILITY_GROUP`, or stored `config visibility-group`. `--no-visibility-group` suppresses inherited defaults for comment creation. Existing comment updates (`comment update`) intentionally ignore env/config defaults unless an explicit visibility flag is present. `comment update --visibility-group <group>` sets comment visibility; `comment update --no-visibility-group` clears it with an empty `permittedGroups` payload.

## YouTrack API: Existing comment attachment upload unsupported via comment update
Date: 2026-04-24
Curl probe in DWP showed that updating an existing issue comment with `attachments:[{id}]` returns HTTP 200 but does not assign the attachment to the comment. The attachment remains a parent issue attachment with `comment: null`, and the comment response returns `attachments: []`. `ytd` does not implement `comment attach` until a supported API flow is found. Follow-up ticket: DWP-24.

## Implementierung: Attachment IDs are parent-scoped
Date: 2026-04-24
YouTrack attachment get/delete endpoints are scoped to tickets or articles, even when an attachment belongs to a comment. `ytd` therefore exposes encoded attachment IDs as `<ticket-id>:<attachment-id>` and `<article-id>:<attachment-id>`. The public `id` field for any CLI attachment output must always be encoded; raw YouTrack attachment IDs may only appear as `ytId`. Comment-owned attachments include an encoded `commentId` when YouTrack reports `comment(id)`.

## Implementierung: Sprint IDs are board-scoped
Date: 2026-04-24
YouTrack sprint get/update/delete endpoints are scoped under Agile boards: `/api/agiles/{agileID}/sprints/{sprintID}`. `ytd` therefore exposes sprint IDs as `<board-id>:<sprint-id>`. The public `id` field for CLI sprint output must always use this sprint-id, while the raw YouTrack sprint ID may only appear as `ytId`. `current` is intentionally not accepted as a sprint-id; use `ytd sprint current` or `ytd sprint current --board <board-id>` to resolve current sprints to real IDs.

## YouTrack API: Sprint ticket assignment is Agile/Sprint-scoped
Date: 2026-04-25

Verified against DWP. Adding an issue to a sprint uses:

`POST /api/agiles/{agileID}/sprints/{sprintID}/issues`

with body:

`{"id":"<issue-database-id>","$type":"Issue"}`

The readable issue ID (`DWP-28`) is not accepted in this body; YouTrack expects the internal issue database ID (`2-14318`). `$type` is accepted but the database ID is the important part.

Removing an issue from a sprint uses:

`DELETE /api/agiles/{agileID}/sprints/{sprintID}/issues/{issueDatabaseID}`

`POST /api/issues/{issueID}/sprints` returned HTTP 405, so issue-scoped sprint writes are not supported. Duplicate add returned HTTP 200. Removing an unassigned issue returned HTTP 404. A ticket can appear in sprints on multiple boards, so CLI write commands must require the encoded public sprint ID `<board-id>:<sprint-id>`.

## Implementierung: Specialized ticket text output
Date: 2026-04-25

Ticket text output is intentionally no longer the generic `key: value` formatter for commands that render real issue objects. `ticket search`, `ticket list`, `search run`, and `sprint ticket list` use a shared compact ticket formatter. `ticket get` uses a shared detail-report formatter. `ticket links` reuses compact ticket rendering for embedded linked issues. `--format json` is the stable ytd-normalized JSON form for scripts; `--format raw` is reserved for YouTrack API-shaped JSON as original as possible.

## Implementierung: Article public IDs
Date: 2026-04-25

Article CLI outputs use the reusable readable article ID (`<PROJECT>-A-<NUMBER>`) as the public `id`. The raw YouTrack database ID is exposed only as `ytId`. Article outputs should not expose `idReadable`; this matches the public-ID pattern used for comments, attachments, and sprints.

## Implementierung: Ticket issue public IDs
Date: 2026-04-25

Ticket issue CLI outputs use the reusable readable ticket ID (`<PROJECT>-<NUMBER>`) as the public `id`. The raw YouTrack database ID is exposed only as `ytId`. Ticket issue outputs should not expose `idReadable`; this applies to direct issue outputs and nested linked/sprint issue outputs. Commands that only print an ID (`ticket create`, `ticket update`, `sprint ticket add/remove`) already print the reusable readable ID.

## Implementierung: Text output renders Markdown content as plain text
Date: 2026-04-25

`--format text` is plain text, not raw Markdown. Fields that can contain Markdown (`content`, `description`, `text`) are rendered through `termimad` with no ANSI styling and ASCII table borders before printing. Generic text output prints metadata and other scalar fields first, then a blank line and content fields without a label. `ticket get` follows the same shape: status/custom fields/metadata first, then a blank line and the plain-text description without a label; embedded comments still follow the parent content and their own Markdown text is also rendered as plain text.

This applies to `article get`, `article search` and `article list` when content is included, `article comments`, `ticket comments`, `comment get`, `ticket history` activity text, and any future text output with `content`, `description`, or `text` fields. Compact ticket list outputs intentionally do not show content.

## Implementierung: Update visibility is explicit only
Date: 2026-04-25

Ticket/article creates and new comments apply configured visibility defaults from `--visibility-group`, `YTD_VISIBILITY_GROUP`, or stored `config visibility-group`. Ticket/article/comment updates preserve existing visibility unless an explicit visibility flag is passed. `ticket update` and `article update` no longer apply env/config defaults. `--visibility-group <group>` sets limited visibility; `--no-visibility-group` clears existing visibility on updates. Empty ticket/article updates without JSON fields or explicit visibility flags are input errors.

## Implementierung: Delete confirmation is uniform
Date: 2026-04-25

All delete commands (`ticket`, `article`, `comment`, `attachment`, `board`, `sprint`) share the same confirmation behavior. `-y` confirms without prompting. Interactive deletes require typing `yes`. Non-interactive deletes without `-y` fail with an input error and do not mutate.

## YouTrack API: User list/get commands
Date: 2026-04-25

User commands expose YouTrack users for alias setup and scripting. `user list` lists reusable user IDs plus login/name fields where available. `user get <user-id-or-login>` accepts either the raw YouTrack user ID or a login. User IDs are YouTrack IDs and do not need ytd-specific parent encoding.

## Implementierung: Alias config stores IDs only
Date: 2026-04-25

Aliases are local config entries under `aliases`, not YouTrack resources. Stored alias values keep only IDs, for example:

`{ "aliases": { "todo": { "project": "0-96", "user": "1-51", "sprint": "108-4:113-6" }, "backlog": { "project": "0-96", "user": "1-51" } } }`

The `sprint` field is optional and omitted when no sprint is bound. `alias create --sprint none` clears or omits `sprint`. Because `alias list` is config-backed, `--format json` and `--format raw` intentionally return the same local alias data model. Dynamic alias commands are the deliberate exception to command-name validation before config loading: after built-in command matching, `ytd` may read local config to resolve `ytd <alias> ...`. Dynamic alias listing (`ytd <alias> list [--all]`) reuses the shared compact ticket formatter and must match `ticket list` text output.
