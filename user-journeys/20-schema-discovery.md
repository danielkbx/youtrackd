# Journey 20: Schema Discovery

Purpose: verify that humans and agents can discover JSON input fields. Static schema discovery works without login or config; project-specific ticket schema discovery requires login and reads project custom field metadata. This journey is read-only and creates no YouTrack resources.

## Scope

- No YouTrack mutation.
- Static schema steps require no login or config.
- Project-specific schema steps require valid credentials and `$PROJECT`.
- Verifies `ytd schema` output for commands that accept `--json` or stdin JSON.

## 1. List Schemas Without Login

Run:

```bash
env -u YOUTRACK_URL -u YOUTRACK_TOKEN YTD_CONFIG=/tmp/nonexistent-ytd-config ytd schema
```

Expected:

- Exit code 0.
- Output lists `ticket create`, `ticket update`, `article create`, `article update`, `board create`, `board update`, `sprint create`, and `sprint update`.
- Output does not say `Not logged in`.

## 2. Ticket Create Fields

Run:

```bash
env -u YOUTRACK_URL -u YOUTRACK_TOKEN YTD_CONFIG=/tmp/nonexistent-ytd-config ytd schema ticket create
```

Expected:

- Exit code 0.
- Output includes `summary`, `description`, `customFields`, `tags`, `--project`, and `stdin takes precedence`.
- Output explains that unknown ticket JSON fields are rejected and JSON `project` is not accepted.

## 2a. Project-Specific Ticket Fields

Run:

```bash
ytd schema ticket create --project $PROJECT --format json
```

Expected:

- Exit code 0.
- Output parses with `jq`.
- JSON has `project.shortName` matching `$PROJECT` or the resolved project short name.
- JSON includes `projectFields` and `projectExamples`.
- Typical projects include examples for fields such as `Assignee`, `Priority`, `Type`, or `State` when those fields are attached to the project.

## 3. Article Update JSON Schema

Run:

```bash
env -u YOUTRACK_URL -u YOUTRACK_TOKEN YTD_CONFIG=/tmp/nonexistent-ytd-config ytd schema article update --format json
```

Expected:

- Exit code 0.
- Output parses with `jq`.
- JSON has `command == "article update"`.
- JSON fields include `parentArticle`.
- Rules mention `parentArticle:null` and unknown field rejection.

## 4. Board Create Pass-Through Caveat

Run:

```bash
env -u YOUTRACK_URL -u YOUTRACK_TOKEN YTD_CONFIG=/tmp/nonexistent-ytd-config ytd schema board create
```

Expected:

- Exit code 0.
- Output includes `name`, `projects`, `--project and JSON projects cannot be combined`.
- Output explains that additional JSON fields pass through to YouTrack unchanged.

## 5. Sprint Update Pass-Through Caveat

Run:

```bash
env -u YOUTRACK_URL -u YOUTRACK_TOKEN YTD_CONFIG=/tmp/nonexistent-ytd-config ytd schema sprint update
```

Expected:

- Exit code 0.
- Output includes `name` and `goal`.
- Output explains that additional JSON fields pass through to YouTrack unchanged.

## 6. Unsupported Target

Run:

```bash
env -u YOUTRACK_URL -u YOUTRACK_TOKEN YTD_CONFIG=/tmp/nonexistent-ytd-config ytd schema ticket delete
```

Expected:

- Exit code non-zero.
- Error mentions `Unsupported schema target: ticket delete`.
- Error lists supported targets.

## Cleanup

No cleanup required.
