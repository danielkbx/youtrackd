use crate::args::ParsedArgs;
use crate::error::YtdError;
use crate::format::{Format, OutputOptions};
use crate::types::Project;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SkillScope {
    Brief,
    Standard,
    Full,
}

impl SkillScope {
    fn parse(value: Option<&str>) -> Result<Self, YtdError> {
        match value {
            None => Ok(Self::Standard),
            Some("brief") => Ok(Self::Brief),
            Some("standard") => Ok(Self::Standard),
            Some("full") => Ok(Self::Full),
            Some(other) => Err(YtdError::Input(format!(
                "Invalid scope: {other}. Expected one of: brief, standard, full"
            ))),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Brief => "brief",
            Self::Standard => "standard",
            Self::Full => "full",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkillProjectContext {
    id: String,
    short_name: String,
    name: String,
    archived: Option<bool>,
    description: Option<String>,
}

impl From<Project> for SkillProjectContext {
    fn from(project: Project) -> Self {
        Self {
            id: project.id,
            short_name: project.short_name,
            name: project.name,
            archived: project.archived,
            description: project.description,
        }
    }
}

pub fn project_ref(args: &ParsedArgs) -> Result<Option<&str>, YtdError> {
    match args.flags.get("project").map(|s| s.as_str()) {
        None => Ok(None),
        Some("true") | Some("") => Err(YtdError::Input(
            "Usage: ytd skill [--scope brief|standard|full] [--project <project>]".into(),
        )),
        Some(project) => Ok(Some(project)),
    }
}

pub fn validate(args: &ParsedArgs, opts: &OutputOptions) -> Result<(), YtdError> {
    SkillScope::parse(args.flags.get("scope").map(|s| s.as_str()))?;
    project_ref(args)?;
    if matches!(opts.format, Format::Json | Format::Raw) {
        return Err(YtdError::Input(
            "ytd skill only supports --format text or --format md".into(),
        ));
    }
    Ok(())
}

pub fn run(
    project: Option<SkillProjectContext>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    validate(args, opts)?;
    let scope = SkillScope::parse(args.flags.get("scope").map(|s| s.as_str()))?;
    println!("{}", render_skill(scope, project.as_ref()));
    Ok(())
}

fn render_skill(scope: SkillScope, project: Option<&SkillProjectContext>) -> String {
    let mut out = String::new();
    push_frontmatter(&mut out, project);
    push_intro(&mut out);
    push_version(&mut out, scope, project);
    if let Some(project) = project {
        push_project_context(&mut out, project);
    }
    push_help_guidance(&mut out);
    push_json_guidance(&mut out);
    push_core_commands(&mut out, project);
    push_public_ids(&mut out, project);
    push_safety(&mut out);

    if matches!(scope, SkillScope::Standard | SkillScope::Full) {
        push_workflow(&mut out);
        push_examples(&mut out, project);
        push_recipes(&mut out, project);
        push_visibility(&mut out);
    }

    if matches!(scope, SkillScope::Full) {
        push_full_reference(&mut out);
    }

    out.trim_end().to_string()
}

fn push_frontmatter(out: &mut String, project: Option<&SkillProjectContext>) {
    out.push_str("---\n");
    out.push_str(&format!("name: {}\n", skill_name(project)));
    if let Some(project) = project {
        out.push_str(&format!(
            "description: >-\n  Use when working with YouTrack project {} through the ytd CLI: searching, reading, creating, updating, commenting on, linking, tagging, attaching files to, or deleting project tickets and articles; managing boards, sprints, saved searches, aliases, worklogs, and visibility.\n",
            project.short_name
        ));
    } else {
        out.push_str(
            "description: >-\n  Use when working with YouTrack through the ytd CLI: searching, reading, creating, updating, commenting on, linking, tagging, attaching files to, or deleting tickets and articles; managing projects, users, boards, sprints, saved searches, aliases, worklogs, and visibility.\n",
        );
    }
    out.push_str("---\n\n");
}

fn skill_name(project: Option<&SkillProjectContext>) -> String {
    let Some(project) = project else {
        return "ytd-youtrack".to_string();
    };
    let suffix = sanitize_skill_name_part(&project.short_name);
    let mut name = if suffix.is_empty() {
        "ytd-youtrack".to_string()
    } else {
        format!("ytd-youtrack-{suffix}")
    };
    if name.len() > 64 {
        name.truncate(64);
        while name.ends_with('-') {
            name.pop();
        }
    }
    name
}

fn sanitize_skill_name_part(value: &str) -> String {
    let mut out = String::new();
    let mut last_was_hyphen = false;
    for ch in value.chars().flat_map(|ch| ch.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_was_hyphen = false;
        } else if !last_was_hyphen && !out.is_empty() {
            out.push('-');
            last_was_hyphen = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

fn push_intro(out: &mut String) {
    out.push_str("# ytd YouTrack CLI\n\n");
    out.push_str("Use `ytd` to read and edit YouTrack tickets, articles, comments, attachments, projects, users, boards, sprints, saved searches, aliases, and worklogs.\n\n");
}

fn push_version(out: &mut String, scope: SkillScope, project: Option<&SkillProjectContext>) {
    let version = env!("CARGO_PKG_VERSION");
    out.push_str("## Keeping This Skill Current\n\n");
    out.push_str(&format!("This skill was generated for ytd {version}.\n\n"));
    out.push_str(
        "Agents may regenerate this file whenever they need current ytd instructions:\n\n",
    );
    out.push_str("```bash\n");
    out.push_str(&regeneration_command(scope, project));
    out.push('\n');
    out.push_str("```\n\n");
    out.push_str("Before relying on this file, run:\n\n");
    out.push_str("```bash\n");
    out.push_str("ytd --version\n");
    out.push_str("```\n\n");
    out.push_str(&format!(
        "If the installed ytd version is newer than {version}, regenerate this skill using the same command shape.\n\n"
    ));
}

fn regeneration_command(scope: SkillScope, project: Option<&SkillProjectContext>) -> String {
    match project {
        Some(project) => format!(
            "ytd skill --project {} --scope {} > SKILL.md",
            project.short_name,
            scope.as_str()
        ),
        None => format!("ytd skill --scope {} > SKILL.md", scope.as_str()),
    }
}

fn push_project_context(out: &mut String, project: &SkillProjectContext) {
    out.push_str("## Project Context\n\n");
    out.push_str("Default project:\n");
    out.push_str(&format!("- Short name: {}\n", project.short_name));
    out.push_str(&format!("- Name: {}\n", project.name));
    out.push_str(&format!("- ID: {}\n\n", project.id));
    if project.archived == Some(true) {
        out.push_str("Warning: this project is archived. Prefer read-only operations unless the user explicitly asks to modify it.\n\n");
    }
    if let Some(description) = project.description.as_deref().filter(|s| !s.is_empty()) {
        out.push_str(&format!("Description: {description}\n\n"));
    }
}

fn push_help_guidance(out: &mut String) {
    out.push_str("## Help Lookup\n\n");
    out.push_str("- Run `ytd help` to see all available commands.\n");
    out.push_str(
        "- Run `ytd help <command>` or `ytd <command> help` before using an unfamiliar command.\n",
    );
    out.push_str("- Prefer checking command help over guessing required flags, JSON fields, or delete behavior.\n\n");
}

fn push_json_guidance(out: &mut String) {
    out.push_str("## Output Rules\n\n");
    out.push_str("- Prefer `--format json` for search, list, and get commands when extracting IDs, scripting, comparing data, or planning follow-up commands.\n");
    out.push_str("- Use public normalized `id` fields from JSON for follow-up commands. Do not use raw `ytId` unless a command explicitly asks for a raw YouTrack ID.\n");
    out.push_str("- Use text output only for quick human-readable inspection or when the user asks for a readable summary.\n");
    out.push_str("- Use `--format md` only for Markdown exports.\n");
    out.push_str(
        "- Use `--format raw` only to inspect YouTrack API-shaped fields for debugging.\n\n",
    );
}

fn push_core_commands(out: &mut String, project: Option<&SkillProjectContext>) {
    let optional_project_flag = project
        .map(|p| format!(" --project {}", p.short_name))
        .unwrap_or_default();
    let required_project_flag = project
        .map(|p| format!(" --project {}", p.short_name))
        .unwrap_or_else(|| " --project <project>".to_string());
    out.push_str("## Core Commands\n\n");
    out.push_str("```bash\n");
    out.push_str(&format!(
        "ytd ticket search \"<query>\"{optional_project_flag} --format json\n"
    ));
    out.push_str("ytd ticket get <ticket-id> --format json\n");
    out.push_str(&format!(
        "ytd ticket create{required_project_flag} --json '{{\"summary\":\"...\",\"description\":\"...\"}}'\n"
    ));
    out.push_str("ytd ticket update <ticket-id> --json '{\"summary\":\"...\"}'\n");
    out.push_str("ytd ticket comment <ticket-id> \"text\"\n");
    out.push_str(&format!(
        "ytd article search \"<query>\"{optional_project_flag} --format json\n"
    ));
    out.push_str("ytd article get <article-id> --format json\n");
    out.push_str(&format!(
        "ytd article create{required_project_flag} --json '{{\"summary\":\"...\",\"content\":\"...\"}}'\n"
    ));
    out.push_str("```\n\n");
}

fn push_public_ids(out: &mut String, project: Option<&SkillProjectContext>) {
    let ticket_id = project
        .map(|project| format!("{}-123", project.short_name))
        .unwrap_or_else(|| "PROJ-123".to_string());
    let article_id = project
        .map(|project| format!("{}-A-123", project.short_name))
        .unwrap_or_else(|| "DOCS-A-123".to_string());
    out.push_str("## Public IDs\n\n");
    out.push_str(&format!(
        "- Tickets use readable IDs such as `{ticket_id}`.\n"
    ));
    out.push_str(&format!(
        "- Articles use readable IDs such as `{article_id}`.\n"
    ));
    out.push_str("- Comment IDs encode parent scope: `<ticket-id>:<comment-id>` or `<article-id>:<comment-id>`.\n");
    out.push_str("- Attachment IDs encode parent scope: `<ticket-id>:<attachment-id>` or `<article-id>:<attachment-id>`.\n");
    out.push_str("- Sprint IDs encode board scope: `<board-id>:<sprint-id>`.\n\n");
}

fn push_safety(out: &mut String) {
    out.push_str("## Safety\n\n");
    out.push_str("- Read the current resource state before mutating it.\n");
    out.push_str("- Confirm destructive intent with the user before deleting anything.\n");
    out.push_str("- Use `-y` only when deletion is explicitly requested.\n");
    out.push_str("- Never print, log, or expose YouTrack tokens or credential files.\n\n");
}

fn push_workflow(out: &mut String) {
    out.push_str("## Recommended Agent Workflow\n\n");
    out.push_str("1. Search or list before get/update when the target is ambiguous.\n");
    out.push_str("2. Use `--format json` by default for machine-readable work.\n");
    out.push_str("3. Use returned public `id` fields for follow-up commands.\n");
    out.push_str("4. Read current resource state before mutating.\n");
    out.push_str("5. Explain the exact destructive target before deleting.\n\n");
}

fn push_examples(out: &mut String, project: Option<&SkillProjectContext>) {
    out.push_str("## Examples\n\n");
    out.push_str("```bash\n");
    if let Some(project) = project {
        out.push_str(&format!(
            "ytd ticket list --project {} --format json\n",
            project.short_name
        ));
        out.push_str(&format!(
            "ytd ticket search \"State: Open\" --project {} --format json\n",
            project.short_name
        ));
        out.push_str(&format!(
            "ytd article list --project {} --format json\n",
            project.short_name
        ));
        out.push_str(&format!(
            "ytd ticket create --project {} --json '{{\"summary\":\"...\",\"description\":\"...\"}}'\n",
            project.short_name
        ));
    } else {
        out.push_str("ytd project list --format json\n");
        out.push_str("ytd ticket search \"<query>\" --format json\n");
        out.push_str("ytd article search \"<query>\" --format json\n");
    }
    out.push_str("```\n\n");
}

fn push_recipes(out: &mut String, project: Option<&SkillProjectContext>) {
    let optional_project_flag = project
        .map(|p| format!(" --project {}", p.short_name))
        .unwrap_or_default();
    let required_project_flag = project
        .map(|p| format!(" --project {}", p.short_name))
        .unwrap_or_else(|| " --project <project>".to_string());
    out.push_str("## Common Recipes\n\n");
    out.push_str("- Find a ticket: `ytd ticket search \"<query>");
    out.push_str(&format!("\"{optional_project_flag} --format json`.\n"));
    out.push_str("- Inspect a ticket: `ytd ticket get <ticket-id> --format json`.\n");
    out.push_str("- Add a comment: `ytd ticket comment <ticket-id> \"text\"`.\n");
    out.push_str(&format!(
        "- Create a ticket: `ytd ticket create{required_project_flag} --json '{{\"summary\":\"...\",\"description\":\"...\"}}'`.\n"
    ));
    out.push_str(&format!(
        "- List project articles: `ytd article list{required_project_flag} --format json`.\n"
    ));
    out.push_str("- Export an article as Markdown: `ytd article get <article-id> --format md > article.md`.\n");
    out.push_str("- List current sprints: `ytd sprint current --format json`.\n");
    out.push_str("- Log work: `ytd ticket log <ticket-id> 30m \"work summary\"`.\n\n");
}

fn push_visibility(out: &mut String) {
    out.push_str("## Visibility\n\n");
    out.push_str("- Ticket/article create and new comment commands may inherit configured visibility defaults.\n");
    out.push_str("- Update commands preserve existing visibility unless `--visibility-group <group>` or `--no-visibility-group` is passed explicitly.\n");
    out.push_str("- Use `--no-visibility-group` on create/comment commands to suppress inherited visibility defaults.\n\n");
}

fn push_full_reference(out: &mut String) {
    out.push_str("## Command Reference\n\n");
    out.push_str("```text\n");
    out.push_str("ytd login / logout / whoami\n");
    out.push_str("ytd project list|get\n");
    out.push_str("ytd user list|get\n");
    out.push_str("ytd article search|list|get|create|update|append|comment|comments|attach|attachments|delete\n");
    out.push_str("ytd ticket search|list|get|create|update|comment|comments|tag|untag|link|links|attach|attachments|log|worklog|set|fields|history|sprints|delete\n");
    out.push_str("ytd comment get|update|attachments|delete\n");
    out.push_str("ytd attachment get|delete|download\n");
    out.push_str("ytd alias create|list|delete and ytd <alias> create|list\n");
    out.push_str("ytd tag list / search list|run / board list|get|create|update|delete\n");
    out.push_str("ytd sprint list|current|get|create|update|delete|ticket\n");
    out.push_str("```\n\n");
    out.push_str("## Detailed Output And Input Rules\n\n");
    out.push_str(
        "- `--format json` is stable ytd-normalized JSON and should be the agent default.\n",
    );
    out.push_str(
        "- `--format text` is optimized for humans and compact context-window inspection.\n",
    );
    out.push_str("- `--format md` exports Markdown title/body/comment content.\n");
    out.push_str("- `--format raw` exposes YouTrack API-shaped JSON for debugging.\n");
    out.push_str("- `--no-meta` suppresses metadata where supported.\n");
    out.push_str("- Structured create/update input uses `--json '{...}'` or stdin; stdin takes precedence.\n");
    out.push_str("- Create/update JSON input must be an object.\n");
    out.push_str("- Delete commands ask for interactive confirmation; non-interactive delete requires `-y`.\n\n");
    out.push_str("## Alias, Board, And Sprint Workflows\n\n");
    out.push_str("- Use `ytd alias create <alias> --project <project-id> --user <user-id> [--sprint <sprint-id|none>]` for repeated ticket workflows.\n");
    out.push_str("- Use `ytd <alias> create <text>` and `ytd <alias> list [--all]` after configuring an alias.\n");
    out.push_str(
        "- Use `ytd board list --format json` to discover board IDs before sprint operations.\n",
    );
    out.push_str("- Use `ytd sprint current --format json` or `ytd sprint list --board <board-id> --format json` to discover board-scoped sprint IDs.\n");
    out.push_str("- Use `ytd sprint ticket add <board-id>:<sprint-id> <ticket-id>` and `remove` for sprint assignment.\n\n");
    out.push_str("## Troubleshooting\n\n");
    out.push_str("- `Not logged in. Run ytd login.` means credentials are missing.\n");
    out.push_str("- Invalid formats must be one of `text`, `raw`, `json`, or `md`.\n");
    out.push_str("- `Project not found: <ref>` means the project database ID or short name could not be resolved.\n");
    out.push_str("- Permission errors usually mean the YouTrack token lacks access to that project or operation.\n");
    out.push_str("- Ambiguous user references should be retried with the exact user ID from `ytd user list --format json`.\n\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::Format;
    use std::collections::HashMap;

    fn args(flags: &[(&str, &str)]) -> ParsedArgs {
        ParsedArgs {
            resource: Some("skill".into()),
            action: None,
            positional: vec![],
            flags: flags
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
        }
    }

    fn opts(format: Format) -> OutputOptions {
        OutputOptions {
            format,
            no_meta: false,
        }
    }

    fn project() -> SkillProjectContext {
        SkillProjectContext {
            id: "0-96".into(),
            short_name: "DWP".into(),
            name: "Developer Workflow Platform".into(),
            archived: Some(false),
            description: Some("Project docs".into()),
        }
    }

    #[test]
    fn missing_scope_defaults_to_standard() {
        assert_eq!(SkillScope::parse(None).unwrap(), SkillScope::Standard);
    }

    #[test]
    fn parses_supported_scopes() {
        assert_eq!(SkillScope::parse(Some("brief")).unwrap(), SkillScope::Brief);
        assert_eq!(
            SkillScope::parse(Some("standard")).unwrap(),
            SkillScope::Standard
        );
        assert_eq!(SkillScope::parse(Some("full")).unwrap(), SkillScope::Full);
    }

    #[test]
    fn rejects_invalid_scope() {
        let err = SkillScope::parse(Some("nope")).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Invalid scope: nope. Expected one of: brief, standard, full"
        );
    }

    #[test]
    fn validates_supported_formats() {
        validate(&args(&[]), &opts(Format::Text)).unwrap();
        validate(&args(&[]), &opts(Format::Md)).unwrap();
    }

    #[test]
    fn rejects_json_and_raw_formats() {
        let json = validate(&args(&[]), &opts(Format::Json)).unwrap_err();
        assert_eq!(
            json.to_string(),
            "ytd skill only supports --format text or --format md"
        );
        let raw = validate(&args(&[]), &opts(Format::Raw)).unwrap_err();
        assert_eq!(
            raw.to_string(),
            "ytd skill only supports --format text or --format md"
        );
    }

    #[test]
    fn output_has_frontmatter_with_only_name_and_description() {
        let text = render_skill(SkillScope::Brief, None);
        assert!(text.starts_with("---\nname: ytd-youtrack\ndescription: >-\n  "));
        let frontmatter = text.split("---").nth(1).unwrap();
        assert!(frontmatter.contains("name:"));
        assert!(frontmatter.contains("description:"));
        assert!(!frontmatter.contains("metadata:"));
        assert!(!frontmatter.contains("compatibility:"));
    }

    #[test]
    fn project_suffix_is_sanitized() {
        let project = SkillProjectContext {
            short_name: "DWP_Test--42!".into(),
            ..project()
        };
        let text = render_skill(SkillScope::Brief, Some(&project));
        assert!(text.contains("name: ytd-youtrack-dwp-test-42\n"));
    }

    #[test]
    fn project_context_only_appears_when_project_exists() {
        let generic = render_skill(SkillScope::Brief, None);
        assert!(!generic.contains("## Project Context"));

        let project = project();
        let project_text = render_skill(SkillScope::Brief, Some(&project));
        assert!(project_text.contains("## Project Context"));
        assert!(project_text.contains("- Short name: DWP"));
        assert!(project_text.contains("- Name: Developer Workflow Platform"));
        assert!(project_text.contains("- ID: 0-96"));
    }

    #[test]
    fn archived_project_includes_warning() {
        let mut project = project();
        project.archived = Some(true);
        let text = render_skill(SkillScope::Brief, Some(&project));
        assert!(text.contains("this project is archived"));
    }

    #[test]
    fn version_and_regeneration_command_use_effective_scope() {
        let text = render_skill(SkillScope::Standard, None);
        assert!(text.contains(env!("CARGO_PKG_VERSION")));
        assert!(text.contains("ytd --version"));
        assert!(text.contains("ytd skill --scope standard > SKILL.md"));
    }

    #[test]
    fn project_regeneration_command_uses_resolved_short_name() {
        let project = project();
        let text = render_skill(SkillScope::Full, Some(&project));
        assert!(text.contains("ytd skill --project DWP --scope full > SKILL.md"));
    }

    #[test]
    fn json_first_guidance_appears_in_all_scopes() {
        for scope in [SkillScope::Brief, SkillScope::Standard, SkillScope::Full] {
            let text = render_skill(scope, None);
            assert!(text.contains("Prefer `--format json`"));
        }
    }

    #[test]
    fn help_lookup_guidance_appears_in_all_scopes() {
        for scope in [SkillScope::Brief, SkillScope::Standard, SkillScope::Full] {
            let text = render_skill(scope, None);
            assert!(text.contains("Run `ytd help`"));
            assert!(text.contains("Run `ytd help <command>` or `ytd <command> help`"));
        }
    }

    #[test]
    fn public_id_examples_use_distinct_realistic_prefixes() {
        let text = render_skill(SkillScope::Brief, None);
        assert!(text.contains("Tickets use readable IDs such as `PROJ-123`"));
        assert!(text.contains("Articles use readable IDs such as `DOCS-A-123`"));
    }

    #[test]
    fn project_public_id_examples_use_resolved_short_name() {
        let project = project();
        let text = render_skill(SkillScope::Brief, Some(&project));
        assert!(text.contains("Tickets use readable IDs such as `DWP-123`"));
        assert!(text.contains("Articles use readable IDs such as `DWP-A-123`"));
    }

    #[test]
    fn scopes_increase_in_size_and_detail() {
        let brief = render_skill(SkillScope::Brief, None);
        let standard = render_skill(SkillScope::Standard, None);
        let full = render_skill(SkillScope::Full, None);

        assert!(brief.len() < standard.len());
        assert!(standard.len() < full.len());
        assert!(!brief.contains("## Common Recipes"));
        assert!(standard.contains("## Common Recipes"));
        assert!(full.contains("## Command Reference"));
    }

    #[test]
    fn rejects_missing_project_value() {
        let mut flags = HashMap::new();
        flags.insert("project".to_string(), "true".to_string());
        let args = ParsedArgs {
            resource: Some("skill".into()),
            action: None,
            positional: vec![],
            flags,
        };
        let err = validate(&args, &opts(Format::Text)).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Usage: ytd skill [--scope brief|standard|full] [--project <project>]"
        );
    }
}
