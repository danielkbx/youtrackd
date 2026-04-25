# Journey 19: Skill Generation

Purpose: verify that `ytd skill` produces useful AI-agent `SKILL.md` guidance. This journey is review-oriented and must not lock exact wording except for stable commands, scope flags, and project identifiers.

Before running project-specific steps, follow `PROCESS.md`: ask the user for the target project and wait for confirmation.

## Prerequisites

- `ytd` is built and available in PATH.
- For project-specific steps, valid credentials are configured.
- `$PROJECT` is the user-confirmed target project short name or database ID.

## 1. Generic Skill Generation

Run:

```bash
ytd skill > /tmp/ytd-skill-standard.md
ytd skill --scope brief > /tmp/ytd-skill-brief.md
ytd skill --scope full > /tmp/ytd-skill-full.md
```

Review:

- Each file has YAML frontmatter delimited by `---`.
- Frontmatter has `name` and `description`.
- The description clearly says when an agent should use the skill.
- No project-specific context appears.
- Each file includes the current `ytd` version.
- Each file includes update/regeneration instructions.
- Regeneration commands omit `--project`.
- Regeneration commands include the effective scope:
  - brief file uses `--scope brief`
  - standard file uses `--scope standard`
  - full file uses `--scope full`
- Each file tells agents to prefer `--format json` for machine-readable work.
- Each file tells agents to use `ytd help`, `ytd help <command>`, or `ytd <command> help` before guessing command details.

## 2. Scope Review

Review:

- `brief` is materially shorter than `standard`.
- `standard` is materially shorter than `full`.
- `brief` contains only core usage, ID, JSON, version, and safety guidance.
- `standard` contains workflows and common recipes.
- `full` contains a compact command reference and detailed ID/output/input rules.
- Do not fail this section on harmless wording differences.

## 3. Project-Specific Skill Generation

After the user confirms `$PROJECT`, run:

```bash
ytd skill --project "$PROJECT" > /tmp/ytd-skill-project-standard.md
ytd skill --project "$PROJECT" --scope brief > /tmp/ytd-skill-project-brief.md
ytd skill --project "$PROJECT" --scope full > /tmp/ytd-skill-project-full.md
```

Review:

- The resolved project short name appears.
- The resolved project ID appears.
- The resolved project name appears.
- Project-specific examples use the resolved project short name.
- Public ID examples use the resolved project short name, for example `<resolved-shortName>-123` and `<resolved-shortName>-A-123`.
- Regeneration commands include `--project <resolved-shortName>`.
- Regeneration commands include the same effective scope.
- Project-specific files do not use the originally typed project reference if it differs from resolved short name, except in command transcript comments.
- If the project is archived, the skill includes a read-only caution.

## 4. Agent Usability Review

Read the generated skill files as an AI agent would. The journey passes when the answer is yes for every generated variant:

- Does this skill explain how to keep itself current?
- Does it clearly say to regenerate when installed `ytd` is newer than the skill version?
- Does it clearly prefer JSON output for agent automation?
- Does it tell agents how to look up global and command-specific help?
- Does it give enough project context when `--project` was used?
- Does the level of detail match the requested scope?

## 5. Negative Cases

Run:

```bash
ytd skill --scope nope
ytd skill --format json
```

Expected:

- Invalid scope fails with `Invalid scope: nope. Expected one of: brief, standard, full`.
- `--format json` fails with `ytd skill only supports --format text or --format md`.
