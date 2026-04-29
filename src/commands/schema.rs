use crate::args::ParsedArgs;
use crate::error::YtdError;
use crate::format::{Format, OutputOptions};
use serde::Serialize;
use std::io::{self, Write};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonCommandSchema {
    resource: &'static str,
    action: &'static str,
    command: &'static str,
    usage: &'static str,
    accepts: SchemaAccepts,
    strict_fields: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    required_flags: Vec<SchemaFlag>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    fields: Vec<SchemaField>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    rules: Vec<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    examples: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SchemaAccepts {
    json_flag: bool,
    stdin: bool,
}

#[derive(Debug, Clone, Serialize)]
struct SchemaFlag {
    name: &'static str,
    value: &'static str,
    description: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct SchemaField {
    name: &'static str,
    #[serde(rename = "type")]
    value_type: &'static str,
    required: bool,
    requirement: &'static str,
    description: &'static str,
    example: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct SchemaListItem {
    resource: &'static str,
    action: &'static str,
    command: &'static str,
}

pub fn run(args: &ParsedArgs, opts: &OutputOptions) -> Result<(), YtdError> {
    validate_format(opts)?;

    match args.action.as_deref() {
        None | Some("list") => print_list(opts),
        Some(resource @ ("ticket" | "article" | "board" | "sprint")) => {
            let action = args.positional.first().ok_or_else(|| {
                YtdError::Input(
                    "Usage: ytd schema <ticket|article|board|sprint> <create|update>".into(),
                )
            })?;
            let schema = find_schema(resource, action)
                .ok_or_else(|| YtdError::Input(unsupported_target(resource, Some(action))))?;
            print_schema(schema, opts)
        }
        Some(other) => Err(YtdError::Input(unsupported_target(other, None))),
    }
}

fn validate_format(opts: &OutputOptions) -> Result<(), YtdError> {
    match opts.format {
        Format::Text | Format::Json => Ok(()),
        Format::Raw | Format::Md => Err(YtdError::Input(
            "ytd schema only supports --format text or --format json".into(),
        )),
    }
}

fn print_list(opts: &OutputOptions) -> Result<(), YtdError> {
    let items: Vec<SchemaListItem> = schemas()
        .iter()
        .map(|schema| SchemaListItem {
            resource: schema.resource,
            action: schema.action,
            command: schema.command,
        })
        .collect();

    match opts.format {
        Format::Json => {
            println!("{}", serde_json::to_string_pretty(&items)?);
        }
        Format::Text => {
            println!("JSON input schemas:\n");
            for item in &items {
                println!("  {}", item.command);
            }
            println!("\nRun `ytd schema <resource> <action>` for fields.");
            println!("Use `--format json` for machine-readable schema metadata.");
        }
        Format::Raw | Format::Md => unreachable!("validated above"),
    }
    Ok(())
}

fn print_schema(schema: JsonCommandSchema, opts: &OutputOptions) -> Result<(), YtdError> {
    match opts.format {
        Format::Json => {
            println!("{}", serde_json::to_string_pretty(&schema)?);
        }
        Format::Text => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            write_schema_text(&schema, &mut handle)?;
        }
        Format::Raw | Format::Md => unreachable!("validated above"),
    }
    Ok(())
}

fn write_schema_text<W: Write>(schema: &JsonCommandSchema, out: &mut W) -> io::Result<()> {
    writeln!(out, "{}\n", schema.command)?;
    writeln!(out, "Usage:\n  {}\n", schema.usage)?;
    writeln!(out, "JSON input:")?;
    writeln!(out, "  via --json or stdin; stdin takes precedence")?;
    if !schema.required_flags.is_empty() {
        writeln!(out, "\nRequired flags:")?;
        for flag in &schema.required_flags {
            writeln!(
                out,
                "  --{} <{}>  {}",
                flag.name, flag.value, flag.description
            )?;
        }
    }
    writeln!(out, "\nFields:")?;
    for field in &schema.fields {
        writeln!(
            out,
            "  {:<16} {:<14} {:<22} {}",
            field.name, field.value_type, field.requirement, field.description
        )?;
    }
    if !schema.rules.is_empty() {
        writeln!(out, "\nRules:")?;
        for rule in &schema.rules {
            writeln!(out, "  - {rule}")?;
        }
    }
    if !schema.examples.is_empty() {
        writeln!(out, "\nExamples:")?;
        for example in &schema.examples {
            writeln!(out, "  {example}")?;
        }
    }
    Ok(())
}

fn find_schema(resource: &str, action: &str) -> Option<JsonCommandSchema> {
    schemas()
        .into_iter()
        .find(|schema| schema.resource == resource && schema.action == action)
}

fn unsupported_target(resource: &str, action: Option<&str>) -> String {
    let target = action
        .map(|action| format!("{resource} {action}"))
        .unwrap_or_else(|| resource.to_string());
    format!(
        "Unsupported schema target: {target}. Supported targets: {}",
        schemas()
            .iter()
            .map(|schema| schema.command)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn schemas() -> Vec<JsonCommandSchema> {
    vec![
        schema(
            "ticket",
            "create",
            "ytd ticket create --project <id> --json '{\"summary\":\"...\",\"description\":\"...\"}'",
            false,
            vec![flag(
                "project",
                "id",
                "Project short name or YouTrack project ID",
            )],
            vec![
                field("summary", "string", true, "required", "Ticket summary", "Fix login"),
                field(
                    "description",
                    "string",
                    false,
                    "optional",
                    "Markdown-capable ticket description",
                    "Steps...",
                ),
            ],
            vec![
                "JSON input must be an object.",
                "summary is required.",
                "Visibility is controlled by flags/env/config, not JSON.",
                "ytd only consumes summary and description; unknown ticket JSON fields are not part of the ytd contract and are not sent.",
            ],
            vec![
                "ytd ticket create --project PROJ --json '{\"summary\":\"Fix login\",\"description\":\"Steps...\"}'",
            ],
        ),
        schema(
            "ticket",
            "update",
            "ytd ticket update <id> --json '{\"summary\":\"...\",\"description\":\"...\"}'",
            true,
            vec![],
            vec![
                field("summary", "string", false, "optional", "Ticket summary", "Fix login"),
                field(
                    "description",
                    "string",
                    false,
                    "optional",
                    "Markdown-capable ticket description",
                    "Steps...",
                ),
            ],
            vec![
                "JSON input must be an object.",
                "At least one of summary, description, --visibility-group, or --no-visibility-group is required.",
                "Visibility update is flag-only; env/config defaults are ignored on update.",
                "ytd only consumes summary and description; unknown ticket JSON fields are not part of the ytd contract and are not sent.",
            ],
            vec!["ytd ticket update PROJ-42 --json '{\"summary\":\"Fix login\"}'"],
        ),
        schema(
            "article",
            "create",
            "ytd article create --project <id> --json '{\"summary\":\"...\",\"content\":\"...\",\"parentArticle\":{\"id\":\"PROJ-A-1\"}}'",
            true,
            vec![flag(
                "project",
                "id",
                "Project short name or YouTrack project ID",
            )],
            vec![
                field("summary", "string", true, "required", "Article summary", "Release notes"),
                field(
                    "content",
                    "string",
                    false,
                    "optional",
                    "Markdown article content",
                    "## Notes",
                ),
                field(
                    "parentArticle",
                    "object",
                    false,
                    "optional",
                    "Parent article reference shaped as {\"id\":\"<readable-article-id>\"}",
                    "{\"id\":\"PROJ-A-1\"}",
                ),
            ],
            vec![
                "JSON input must be an object.",
                "summary is required.",
                "Unknown fields are rejected. Allowed fields: summary, content, parentArticle.",
                "parentArticle.id uses a readable reusable article ID; ytd resolves the internal YouTrack article ID before sending.",
                "Visibility defaults work like ticket create and are controlled by flags/env/config, not JSON.",
            ],
            vec![
                "ytd article create --project PROJ --json '{\"summary\":\"Release notes\",\"content\":\"## Notes\",\"parentArticle\":{\"id\":\"PROJ-A-1\"}}'",
            ],
        ),
        schema(
            "article",
            "update",
            "ytd article update <id> --json '{\"summary\":\"...\",\"content\":\"...\",\"parentArticle\":{\"id\":\"PROJ-A-1\"}}'",
            true,
            vec![],
            vec![
                field("summary", "string", false, "optional", "Article summary", "Release notes"),
                field(
                    "content",
                    "string",
                    false,
                    "optional",
                    "Markdown article content",
                    "## Notes",
                ),
                field(
                    "parentArticle",
                    "object|null",
                    false,
                    "optional",
                    "Parent reference {\"id\":\"<readable-article-id>\"}; null clears parent",
                    "null",
                ),
            ],
            vec![
                "JSON input must be an object.",
                "Unknown fields are rejected. Allowed fields: summary, content, parentArticle.",
                "At least one allowed field or explicit visibility flag is required.",
                "parentArticle:null clears the parent article.",
                "Visibility update is flag-only; env/config defaults are ignored on update.",
            ],
            vec![
                "ytd article update PROJ-A-2 --json '{\"parentArticle\":{\"id\":\"PROJ-A-1\"}}'",
                "ytd article update PROJ-A-2 --json '{\"parentArticle\":null}'",
            ],
        ),
        schema(
            "board",
            "create",
            "ytd board create --name <name> --project <project>[,<project>...] [--template <template>] [--json '{...}']",
            false,
            vec![],
            vec![
                field(
                    "name",
                    "string",
                    true,
                    "required unless --name",
                    "Agile board name; --name overrides JSON name",
                    "Team Board",
                ),
                field(
                    "projects",
                    "array<object>",
                    true,
                    "required unless --project",
                    "Project references shaped as [{\"id\":\"<project-db-id>\"}]",
                    "[{\"id\":\"0-96\"}]",
                ),
            ],
            vec![
                "JSON input is optional. If provided, JSON must be an object.",
                "Required effective body fields: name and projects.",
                "--project and JSON projects cannot be combined.",
                "--project accepts project short names or IDs and ytd resolves them to database IDs.",
                "--template is a flag, not JSON.",
                "Additional JSON fields pass through to YouTrack unchanged and are outside the strict ytd contract.",
                "Known pass-through examples used in ytd docs/journeys: visibleForProjectBased, orphansAtTheTop.",
            ],
            vec![
                "ytd board create --name \"Team Board\" --project PROJ --template scrum --json '{\"visibleForProjectBased\":true}'",
            ],
        ),
        schema(
            "board",
            "update",
            "ytd board update <id> [--name <name>] [--json '{...}']",
            false,
            vec![],
            vec![
                field(
                    "name",
                    "string",
                    false,
                    "optional",
                    "Agile board name; --name overrides JSON name",
                    "Team Board",
                ),
                field(
                    "projects",
                    "array<object>",
                    false,
                    "optional",
                    "Project references shaped as [{\"id\":\"<project-db-id>\"}]; update accepts this only through JSON",
                    "[{\"id\":\"0-96\"}]",
                ),
            ],
            vec![
                "JSON input is optional. If provided, JSON must be an object.",
                "At least one JSON field or --name is required.",
                "board update rejects --project; use JSON projects for project changes.",
                "Additional JSON fields pass through to YouTrack unchanged and are outside the strict ytd contract.",
            ],
            vec!["ytd board update 108-4 --json '{\"orphansAtTheTop\":true}'"],
        ),
        schema(
            "sprint",
            "create",
            "ytd sprint create --board <board-id> --name <name> [--json '{...}']",
            false,
            vec![flag("board", "board-id", "Board ID that scopes the sprint")],
            vec![field(
                "name",
                "string",
                true,
                "required unless --name",
                "Sprint name; --name overrides JSON name",
                "Sprint 1",
            )],
            vec![
                "JSON input is optional. If provided, JSON must be an object.",
                "Effective name is required.",
                "Additional JSON fields pass through to YouTrack unchanged and are outside the strict ytd contract.",
                "Known pass-through example used in ytd docs/journeys: goal.",
            ],
            vec!["ytd sprint create --board 108-4 --name \"Sprint 1\" --json '{\"goal\":\"Finish onboarding\"}'"],
        ),
        schema(
            "sprint",
            "update",
            "ytd sprint update <sprint-id> [--name <name>] [--json '{...}']",
            false,
            vec![],
            vec![field(
                "name",
                "string",
                false,
                "optional",
                "Sprint name; --name overrides JSON name",
                "Sprint 1",
            )],
            vec![
                "JSON input is optional. If provided, JSON must be an object.",
                "At least one JSON field or --name is required.",
                "Additional JSON fields pass through to YouTrack unchanged and are outside the strict ytd contract.",
                "Known pass-through example used in ytd docs/journeys: goal.",
            ],
            vec!["ytd sprint update 108-4:113-6 --json '{\"goal\":\"Finish onboarding\"}'"],
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn schema(
    resource: &'static str,
    action: &'static str,
    usage: &'static str,
    strict_fields: bool,
    required_flags: Vec<SchemaFlag>,
    fields: Vec<SchemaField>,
    rules: Vec<&'static str>,
    examples: Vec<&'static str>,
) -> JsonCommandSchema {
    JsonCommandSchema {
        resource,
        action,
        command: Box::leak(format!("{resource} {action}").into_boxed_str()),
        usage,
        accepts: SchemaAccepts {
            json_flag: true,
            stdin: true,
        },
        strict_fields,
        required_flags,
        fields,
        rules,
        examples,
    }
}

fn flag(name: &'static str, value: &'static str, description: &'static str) -> SchemaFlag {
    SchemaFlag {
        name,
        value,
        description,
    }
}

fn field(
    name: &'static str,
    value_type: &'static str,
    required: bool,
    requirement: &'static str,
    description: &'static str,
    example: &'static str,
) -> SchemaField {
    SchemaField {
        name,
        value_type,
        required,
        requirement,
        description,
        example,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_for(resource: &str, action: &str) -> String {
        let schema = find_schema(resource, action).unwrap();
        let mut out = Vec::new();
        write_schema_text(&schema, &mut out).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn list_contains_all_schema_targets() {
        let commands: Vec<_> = schemas().iter().map(|schema| schema.command).collect();
        assert_eq!(
            commands,
            vec![
                "ticket create",
                "ticket update",
                "article create",
                "article update",
                "board create",
                "board update",
                "sprint create",
                "sprint update"
            ]
        );
    }

    #[test]
    fn ticket_create_text_includes_fields_project_and_stdin() {
        let text = text_for("ticket", "create");
        assert!(text.contains("summary"));
        assert!(text.contains("description"));
        assert!(text.contains("--project"));
        assert!(text.contains("stdin takes precedence"));
    }

    #[test]
    fn article_update_text_documents_parent_and_unknown_fields() {
        let text = text_for("article", "update");
        assert!(text.contains("parentArticle"));
        assert!(text.contains("parentArticle:null"));
        assert!(text.contains("Unknown fields are rejected"));
    }

    #[test]
    fn board_create_text_documents_conflict_and_pass_through() {
        let text = text_for("board", "create");
        assert!(text.contains("name"));
        assert!(text.contains("projects"));
        assert!(text.contains("--project and JSON projects cannot be combined"));
        assert!(text.contains("pass through"));
    }

    #[test]
    fn sprint_update_text_documents_pass_through() {
        let text = text_for("sprint", "update");
        assert!(text.contains("name"));
        assert!(text.contains("pass through"));
    }

    #[test]
    fn json_schema_shape_is_stable() {
        let schema = find_schema("ticket", "create").unwrap();
        let value = serde_json::to_value(schema).unwrap();
        assert_eq!(value["command"], "ticket create");
        assert_eq!(value["strictFields"], false);
        let fields = value["fields"].as_array().unwrap();
        assert!(fields.iter().any(|field| field["name"] == "summary"));
    }

    #[test]
    fn unsupported_target_lists_supported_targets() {
        let message = unsupported_target("ticket", Some("delete"));
        assert!(message.contains("Unsupported schema target: ticket delete"));
        assert!(message.contains("ticket create"));
    }
}
