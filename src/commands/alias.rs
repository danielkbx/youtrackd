use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::commands;
use crate::commands::{ticket, visibility};
use crate::config;
use crate::error::YtdError;
use crate::format::{self, OutputOptions};
use crate::types::{
    parse_sprint_id, CreateIssueInput, Issue, Project, ProjectRef, Sprint, StoredAlias, User,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::{self, BufRead, IsTerminal, Write};

const BUILTIN_COMMANDS: &[&str] = &[
    "login",
    "logout",
    "help",
    "config",
    "group",
    "project",
    "article",
    "ticket",
    "comment",
    "attachment",
    "tag",
    "search",
    "board",
    "sprint",
    "whoami",
    "open",
    "url",
    "alias",
    "user",
];

pub fn run<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("create") => cmd_create(client, args),
        Some("list") => cmd_list(client, opts),
        Some("delete") => cmd_delete(args),
        _ => Err(YtdError::Input(
            "Usage: ytd alias <create|list|delete>".into(),
        )),
    }
}

pub fn run_config_only(args: &ParsedArgs) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("delete") => cmd_delete(args),
        _ => Err(YtdError::Input(
            "Usage: ytd alias delete <alias> [-y]".into(),
        )),
    }
}

pub fn run_runtime<T: HttpTransport>(
    client: &YtClient<T>,
    alias_name: &str,
    alias: &StoredAlias,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("create") => runtime_create(client, alias_name, alias, args),
        Some("list") => runtime_list(client, alias, args, opts),
        _ => Err(YtdError::Input(format!(
            "Usage: ytd {alias_name} <create|list>"
        ))),
    }
}

pub fn validate_alias_name(name: &str) -> Result<(), YtdError> {
    if !is_valid_alias_name(name) {
        return Err(YtdError::Input(
            "Alias names must match ^[a-z0-9][a-z0-9_-]*$".into(),
        ));
    }
    if BUILTIN_COMMANDS.contains(&name) {
        return Err(YtdError::Input(format!(
            "Alias name conflicts with built-in command: {name}"
        )));
    }
    Ok(())
}

fn is_valid_alias_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }
    chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-')
}

fn cmd_create<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let name = args
        .positional
        .first()
        .ok_or_else(|| YtdError::Input("Usage: ytd alias create <alias>".into()))?;
    validate_alias_name(name)?;

    let mut stored = config::load_stored_config()?;
    let existing = stored.aliases.get(name).cloned();
    let alias = build_alias(client, args, existing.as_ref())?;
    stored.aliases.insert(name.clone(), alias);
    config::save_stored_config(&stored)?;
    println!("{name}");
    Ok(())
}

fn cmd_list<T: HttpTransport>(client: &YtClient<T>, opts: &OutputOptions) -> Result<(), YtdError> {
    let stored = config::load_stored_config()?;
    let outputs = stored
        .aliases
        .iter()
        .map(|(name, alias)| alias_output(client, name, alias))
        .collect::<Vec<_>>();

    if matches!(opts.format, format::Format::Text) {
        print!("{}", render_aliases_text(&outputs, opts));
    } else {
        format::print_items(&outputs, opts);
    }
    Ok(())
}

fn cmd_delete(args: &ParsedArgs) -> Result<(), YtdError> {
    let name = args
        .positional
        .first()
        .ok_or_else(|| YtdError::Input("Usage: ytd alias delete <alias> [-y]".into()))?;
    validate_alias_name(name)?;

    let mut stored = config::load_stored_config()?;
    if !stored.aliases.contains_key(name) {
        return Err(YtdError::Input(format!("Alias not found: {name}")));
    }
    if commands::confirm_delete(
        "alias",
        name,
        args.flags
            .get("y")
            .map(|value| value == "true")
            .unwrap_or(false),
    )? {
        stored.aliases.remove(name);
        config::save_stored_config(&stored)?;
        println!("{name}");
    }
    Ok(())
}

fn build_alias<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    existing: Option<&StoredAlias>,
) -> Result<StoredAlias, YtdError> {
    let interactive = io::stdin().is_terminal();

    let project = match args.flags.get("project") {
        Some(project) => {
            require_database_id(project, "project")?;
            client.get_project(project)?;
            project.clone()
        }
        None if interactive => {
            prompt_project(client, existing.map(|alias| alias.project.as_str()))?
        }
        None if let Some(existing) = existing => existing.project.clone(),
        None => {
            return Err(YtdError::Input(
                "--project is required for new aliases in non-interactive mode".into(),
            ))
        }
    };

    let user = match args.flags.get("user") {
        Some(user) => {
            require_database_id(user, "user")?;
            client.get_user(user)?;
            user.clone()
        }
        None if interactive => prompt_user(client, existing.map(|alias| alias.user.as_str()))?,
        None if let Some(existing) = existing => existing.user.clone(),
        None => {
            return Err(YtdError::Input(
                "--user is required for new aliases in non-interactive mode".into(),
            ))
        }
    };

    let sprint = match args.flags.get("sprint") {
        Some(value) if value == "none" => None,
        Some(value) => {
            validate_sprint(client, value)?;
            Some(value.clone())
        }
        None if interactive => prompt_sprint(client, &project, existing.and_then(|a| a.sprint.as_deref()))?,
        None if let Some(existing) = existing => existing.sprint.clone(),
        None => {
            return Err(YtdError::Input(
                "--sprint is required for new aliases in non-interactive mode. Use --sprint none for no sprint.".into(),
            ))
        }
    };

    Ok(StoredAlias {
        project,
        user,
        sprint,
    })
}

fn require_database_id(value: &str, field: &str) -> Result<(), YtdError> {
    let mut parts = value.split('-');
    let valid = matches!(
        (parts.next(), parts.next(), parts.next()),
        (Some(left), Some(right), None)
            if !left.is_empty()
                && !right.is_empty()
                && left.chars().all(|ch| ch.is_ascii_digit())
                && right.chars().all(|ch| ch.is_ascii_digit())
    );
    if valid {
        Ok(())
    } else {
        Err(YtdError::Input(format!(
            "--{field} must be a YouTrack database ID like 0-96"
        )))
    }
}

fn validate_sprint<T: HttpTransport>(client: &YtClient<T>, value: &str) -> Result<(), YtdError> {
    let sprint = parse_sprint_id(value)?;
    client.get_sprint(&sprint.board_id, &sprint.sprint_id)?;
    Ok(())
}

fn prompt_project<T: HttpTransport>(
    client: &YtClient<T>,
    default: Option<&str>,
) -> Result<String, YtdError> {
    let mut projects = client.list_projects()?;
    projects.sort_by_key(|project| {
        (
            project.archived.unwrap_or(false),
            project.name.to_ascii_lowercase(),
        )
    });
    let index = prompt_choice(
        "Project",
        &projects,
        default.and_then(|id| projects.iter().position(|project| project.id == id)),
        |project| format!("{} ({})", project.name, project.short_name),
    )?;
    Ok(projects[index].id.clone())
}

fn prompt_user<T: HttpTransport>(
    client: &YtClient<T>,
    default: Option<&str>,
) -> Result<String, YtdError> {
    let mut users = client.list_users()?;
    users.sort_by_key(|user| {
        user.full_name
            .as_deref()
            .unwrap_or(&user.login)
            .to_ascii_lowercase()
    });
    let index = prompt_choice(
        "User",
        &users,
        default.and_then(|id| users.iter().position(|user| user.id == id)),
        format_user,
    )?;
    Ok(users[index].id.clone())
}

fn prompt_sprint<T: HttpTransport>(
    client: &YtClient<T>,
    project_id: &str,
    default: Option<&str>,
) -> Result<Option<String>, YtdError> {
    let mut choices = vec![SprintChoice {
        id: None,
        label: "none".into(),
    }];
    for board in client.list_agiles()? {
        if !board
            .projects
            .iter()
            .any(|project| project.id == project_id)
        {
            continue;
        }
        let board_id = board.id.clone();
        let board_name = board.name.clone().unwrap_or_else(|| board_id.clone());
        for sprint in board.sprints {
            let Some(name) = sprint.name else {
                continue;
            };
            choices.push(SprintChoice {
                id: Some(format!("{board_id}:{}", sprint.id)),
                label: format!("{name} - {board_name}"),
            });
        }
    }
    let default_index = default.and_then(|id| {
        choices
            .iter()
            .position(|choice| choice.id.as_deref() == Some(id))
    });
    let index = prompt_choice("Sprint", &choices, default_index, |choice| {
        choice.label.clone()
    })?;
    Ok(choices[index].id.clone())
}

struct SprintChoice {
    id: Option<String>,
    label: String,
}

fn prompt_choice<T, F>(
    label: &str,
    choices: &[T],
    default: Option<usize>,
    format: F,
) -> Result<usize, YtdError>
where
    F: Fn(&T) -> String,
{
    if choices.is_empty() {
        return Err(YtdError::Input(format!("No {label} choices available")));
    }

    eprintln!("{label}:");
    for (idx, choice) in choices.iter().enumerate() {
        let marker = if Some(idx) == default {
            " (default)"
        } else {
            ""
        };
        eprintln!("  {}. {}{}", idx + 1, format(choice), marker);
    }
    eprint!("Select {label}: ");
    io::stderr().flush()?;

    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return default.ok_or_else(|| YtdError::Input(format!("{label} selection is required")));
    }
    let number = trimmed
        .parse::<usize>()
        .map_err(|_| YtdError::Input(format!("Invalid {label} selection: {trimmed}")))?;
    if number == 0 || number > choices.len() {
        return Err(YtdError::Input(format!(
            "Invalid {label} selection: {trimmed}"
        )));
    }
    Ok(number - 1)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AliasOutput {
    alias: String,
    project: String,
    user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sprint: Option<String>,
    project_resolved: bool,
    user_resolved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    sprint_resolved: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_short_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_login: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_full_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sprint_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    board_name: Option<String>,
}

fn alias_output<T: HttpTransport>(
    client: &YtClient<T>,
    name: &str,
    alias: &StoredAlias,
) -> AliasOutput {
    let project = client.get_project(&alias.project).ok();
    let user = client.get_user(&alias.user).ok();
    let sprint = alias
        .sprint
        .as_deref()
        .and_then(|id| parse_sprint_id(id).ok())
        .and_then(|id| {
            let sprint = client.get_sprint(&id.board_id, &id.sprint_id).ok()?;
            let board = client.get_agile(&id.board_id).ok();
            Some((sprint, board.and_then(|board| board.name)))
        });

    AliasOutput {
        alias: name.to_string(),
        project: alias.project.clone(),
        user: alias.user.clone(),
        sprint: alias.sprint.clone(),
        project_resolved: project.is_some(),
        user_resolved: user.is_some(),
        sprint_resolved: alias.sprint.as_ref().map(|_| sprint.is_some()),
        project_name: project.as_ref().map(|project| project.name.clone()),
        project_short_name: project.as_ref().map(|project| project.short_name.clone()),
        user_login: user.as_ref().map(|user| user.login.clone()),
        user_full_name: user.as_ref().and_then(|user| user.full_name.clone()),
        sprint_name: sprint.as_ref().and_then(|(sprint, _)| sprint.name.clone()),
        board_name: sprint.and_then(|(_, board_name)| board_name),
    }
}

fn render_aliases_text(aliases: &[AliasOutput], opts: &OutputOptions) -> String {
    let mut out = String::new();
    for (idx, alias) in aliases.iter().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        out.push_str(&alias.alias);
        out.push('\n');
        push_alias_line(
            &mut out,
            "project",
            alias
                .project_short_name
                .as_deref()
                .or(alias.project_name.as_deref()),
            &alias.project,
            alias.project_resolved,
            opts,
        );
        push_alias_line(
            &mut out,
            "user",
            alias
                .user_full_name
                .as_deref()
                .or(alias.user_login.as_deref()),
            &alias.user,
            alias.user_resolved,
            opts,
        );
        match alias.sprint.as_deref() {
            Some(sprint) => {
                let display =
                    alias
                        .sprint_name
                        .as_deref()
                        .map(|name| match alias.board_name.as_deref() {
                            Some(board) => format!("{name} - {board}"),
                            None => name.to_string(),
                        });
                push_alias_line(
                    &mut out,
                    "sprint",
                    display.as_deref(),
                    sprint,
                    alias.sprint_resolved.unwrap_or(false),
                    opts,
                );
            }
            None => out.push_str("  sprint: none\n"),
        }
    }
    out
}

fn push_alias_line(
    out: &mut String,
    label: &str,
    display: Option<&str>,
    id: &str,
    resolved: bool,
    opts: &OutputOptions,
) {
    out.push_str("  ");
    out.push_str(label);
    out.push_str(": ");
    if let Some(display) = display {
        out.push_str(display);
        if !opts.no_meta {
            out.push_str(" (");
            out.push_str(id);
            out.push(')');
        }
    } else {
        out.push_str(id);
    }
    if !resolved {
        out.push_str(" [unresolved]");
    }
    out.push('\n');
}

fn runtime_create<T: HttpTransport>(
    client: &YtClient<T>,
    alias_name: &str,
    alias: &StoredAlias,
    args: &ParsedArgs,
) -> Result<(), YtdError> {
    let text = args.positional.join(" ").trim().to_string();
    if text.is_empty() {
        return Err(YtdError::Input(format!(
            "Usage: ytd {alias_name} create <text>"
        )));
    }

    let input = CreateIssueInput {
        project: ProjectRef {
            id: alias.project.clone(),
            short_name: None,
            name: None,
        },
        summary: text,
        description: None,
        visibility: visibility::build_create_visibility_input(client, args)?,
    };
    let issue = client.create_issue(&input)?;
    let ticket_id = issue.id_readable.unwrap_or(issue.id);
    let user = client.get_user(&alias.user).map_err(|err| {
        YtdError::Input(format!(
            "Created {ticket_id}, but failed to resolve alias user {}: {err}",
            alias.user
        ))
    })?;
    client
        .apply_command(&ticket_id, &format!("Assignee {}", user.login))
        .map_err(|err| {
            YtdError::Input(format!(
                "Created {ticket_id}, but failed to assign user {}: {err}",
                user.login
            ))
        })?;
    if let Some(sprint_id) = alias.sprint.as_deref() {
        let sprint = parse_sprint_id(sprint_id)?;
        client
            .add_issue_to_sprint(&sprint.board_id, &sprint.sprint_id, &ticket_id)
            .map_err(|err| {
                YtdError::Input(format!(
                    "Created {ticket_id}, but failed to add it to sprint {sprint_id}: {err}"
                ))
            })?;
    }
    println!("{ticket_id}");
    Ok(())
}

fn runtime_list<T: HttpTransport>(
    client: &YtClient<T>,
    alias: &StoredAlias,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let project = client.get_project(&alias.project)?;
    let user = client.get_user(&alias.user)?;
    let mut query = format!(
        "project: {{{}}} for: {}",
        project.short_name,
        search_value(&user.login)
    );
    if !args
        .flags
        .get("all")
        .map(|value| value == "true")
        .unwrap_or(false)
    {
        query.push_str(" #Unresolved");
    }
    let issues = if let Some(sprint_id) = alias.sprint.as_deref() {
        let parsed = parse_sprint_id(sprint_id)?;
        let sprint_issues = client.list_sprint_issues(&parsed.board_id, &parsed.sprint_id)?;
        let mut issues = client.search_issues(&query, None)?;
        issues.retain(|issue| issue_in_sprint(issue, &sprint_issues));
        issues
    } else {
        client.search_issues(&query, None)?
    };
    ticket::print_issue_items(&issues, opts);
    Ok(())
}

fn issue_in_sprint(issue: &Issue, sprint_issues: &[Issue]) -> bool {
    sprint_issues.iter().any(|sprint_issue| {
        sprint_issue.id == issue.id
            || match (
                sprint_issue.id_readable.as_deref(),
                issue.id_readable.as_deref(),
            ) {
                (Some(left), Some(right)) => left == right,
                _ => false,
            }
    })
}

fn search_value(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' || ch == '-')
    {
        value.to_string()
    } else {
        format!("{{{value}}}")
    }
}

fn format_user(user: &User) -> String {
    match user.full_name.as_deref() {
        Some(full_name) if !full_name.trim().is_empty() => format!("{full_name} ({})", user.login),
        _ => user.login.clone(),
    }
}

#[allow(dead_code)]
fn _assert_alias_map_is_deterministic(_: &BTreeMap<String, StoredAlias>) {}

#[allow(dead_code)]
fn _assert_project_and_sprint_types(_: &Project, _: &Sprint) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_alias_names() {
        assert!(validate_alias_name("todo").is_ok());
        assert!(validate_alias_name("my-todo_1").is_ok());
        assert!(validate_alias_name("Todo").is_err());
        assert!(validate_alias_name("ticket").is_err());
    }

    #[test]
    fn search_value_wraps_values_with_spaces() {
        assert_eq!(search_value("alice"), "alice");
        assert_eq!(search_value("Alice Example"), "{Alice Example}");
    }
}
