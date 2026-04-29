use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::error::YtdError;
use crate::format::{Format, OutputOptions};
use crate::types::{Project, ProjectCustomField};
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
    rules: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    examples: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project: Option<SchemaProject>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    project_fields: Vec<ProjectSchemaField>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    project_examples: Vec<ProjectSchemaExample>,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SchemaProject {
    id: String,
    short_name: String,
    name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectSchemaField {
    name: String,
    #[serde(rename = "type")]
    value_type: String,
    required: bool,
    value_shape: String,
    example: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectSchemaExample {
    field: String,
    json: serde_json::Value,
}

pub fn run(args: &ParsedArgs, opts: &OutputOptions) -> Result<(), YtdError> {
    validate(opts)?;

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

pub fn run_project<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    validate(opts)?;
    let project_ref = args
        .flags
        .get("project")
        .map(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| YtdError::Input("--project must not be empty".into()))?;
    let project = client.resolve_project(project_ref)?;

    match args.action.as_deref() {
        None | Some("list") => print_project_list(&project, opts),
        Some(resource @ ("ticket" | "article" | "board" | "sprint")) => {
            let action = args.positional.first().ok_or_else(|| {
                YtdError::Input(
                    "Usage: ytd schema <ticket|article|board|sprint> <create|update>".into(),
                )
            })?;
            let mut schema = find_schema(resource, action)
                .ok_or_else(|| YtdError::Input(unsupported_target(resource, Some(action))))?;
            schema.project = Some(SchemaProject::from(project.clone()));
            if resource == "ticket" && matches!(action.as_str(), "create" | "update") {
                let fields = client.list_project_custom_fields(&project.id)?;
                add_project_ticket_fields(&mut schema, &fields);
            } else {
                schema.rules.push(
                    "--project does not add resource-specific JSON fields for this schema target."
                        .into(),
                );
            }
            print_schema(schema, opts)
        }
        Some(other) => Err(YtdError::Input(unsupported_target(other, None))),
    }
}

pub fn validate(opts: &OutputOptions) -> Result<(), YtdError> {
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

fn print_project_list(project: &Project, opts: &OutputOptions) -> Result<(), YtdError> {
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
            let value = serde_json::json!({
                "project": SchemaProject::from(project.clone()),
                "schemas": items,
                "detail": "Run ytd schema ticket create --project <project> or ytd schema ticket update --project <project> for project custom field examples."
            });
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
        Format::Text => {
            println!(
                "JSON input schemas for project {} ({})\n",
                project.short_name, project.name
            );
            for item in &items {
                println!("  {}", item.command);
            }
            println!(
                "\nRun `ytd schema ticket create --project {}` or `ytd schema ticket update --project {}` for project custom field examples.",
                project.short_name, project.short_name
            );
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
    if let Some(project) = &schema.project {
        writeln!(
            out,
            "\nProject:\n  {} ({}, {})",
            project.short_name, project.name, project.id
        )?;
    }
    if !schema.project_fields.is_empty() {
        writeln!(out, "\nProject custom fields:")?;
        for field in &schema.project_fields {
            writeln!(
                out,
                "  {:<24} {:<28} {}",
                field.name, field.value_type, field.value_shape
            )?;
        }
    }
    if !schema.examples.is_empty() {
        writeln!(out, "\nExamples:")?;
        for example in &schema.examples {
            writeln!(out, "  {example}")?;
        }
    }
    if !schema.project_examples.is_empty() {
        writeln!(out, "\nProject custom field examples:")?;
        for example in &schema.project_examples {
            let json = serde_json::to_string(&example.json).map_err(io::Error::other)?;
            writeln!(out, "  {}: {}", example.field, json)?;
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

impl From<Project> for SchemaProject {
    fn from(project: Project) -> Self {
        Self {
            id: project.id,
            short_name: project.short_name,
            name: project.name,
        }
    }
}

fn add_project_ticket_fields(schema: &mut JsonCommandSchema, fields: &[ProjectCustomField]) {
    schema.rules.push(
        "customFields is API-shaped and validated by YouTrack using the project field configuration below."
            .into(),
    );

    for project_field in fields {
        let Some(name) = project_field_name(project_field) else {
            continue;
        };
        let issue_type = issue_custom_field_type(project_field);
        let example_value = example_value_for_project_field(project_field, &issue_type);
        let custom_field = serde_json::json!({
            "name": name,
            "$type": issue_type,
            "value": example_value
        });
        schema.project_fields.push(ProjectSchemaField {
            name: name.clone(),
            value_type: issue_type.clone(),
            required: !project_field.can_be_empty.unwrap_or(true),
            value_shape: value_shape_for_type(&issue_type).to_string(),
            example: custom_field.clone(),
        });
        schema.project_examples.push(ProjectSchemaExample {
            field: name,
            json: serde_json::json!({ "customFields": [custom_field] }),
        });
    }
}

fn project_field_name(field: &ProjectCustomField) -> Option<String> {
    field
        .field
        .as_ref()
        .and_then(|prototype| prototype.name.clone())
        .or_else(|| field.name.clone())
        .filter(|name| !name.trim().is_empty())
}

fn issue_custom_field_type(field: &ProjectCustomField) -> String {
    let is_multi = field
        .field
        .as_ref()
        .and_then(|prototype| prototype.field_type.as_ref())
        .and_then(|field_type| field_type.is_multi_value)
        .unwrap_or(false);
    if let Some(field_type) = field.field_type.as_deref() {
        match field_type {
            "UserProjectCustomField" => {
                return if is_multi {
                    "MultiUserIssueCustomField"
                } else {
                    "SingleUserIssueCustomField"
                }
                .to_string();
            }
            "EnumProjectCustomField" => {
                return if is_multi {
                    "MultiEnumIssueCustomField"
                } else {
                    "SingleEnumIssueCustomField"
                }
                .to_string();
            }
            "OwnedProjectCustomField" => {
                return if is_multi {
                    "MultiOwnedIssueCustomField"
                } else {
                    "SingleOwnedIssueCustomField"
                }
                .to_string();
            }
            "VersionProjectCustomField" => {
                return if is_multi {
                    "MultiVersionIssueCustomField"
                } else {
                    "SingleVersionIssueCustomField"
                }
                .to_string();
            }
            "StateProjectCustomField" => return "StateIssueCustomField".to_string(),
            "PeriodProjectCustomField" => return "PeriodIssueCustomField".to_string(),
            "SimpleProjectCustomField" => return "SimpleIssueCustomField".to_string(),
            _ => {}
        }
        if let Some(issue_type) = field_type
            .strip_suffix("ProjectCustomField")
            .map(|prefix| format!("{prefix}IssueCustomField"))
        {
            return issue_type;
        }
        if field_type.ends_with("IssueCustomField") {
            return field_type.to_string();
        }
    }

    let value_type = field
        .field
        .as_ref()
        .and_then(|prototype| prototype.field_type.as_ref())
        .and_then(|field_type| field_type.value_type.as_deref())
        .unwrap_or("");
    match (is_multi, value_type) {
        (true, "user") => "MultiUserIssueCustomField",
        (false, "user") => "SingleUserIssueCustomField",
        (true, "enum") => "MultiEnumIssueCustomField",
        (false, "enum") => "SingleEnumIssueCustomField",
        (true, "version") => "MultiVersionIssueCustomField",
        (false, "version") => "SingleVersionIssueCustomField",
        (true, "ownedField") => "MultiOwnedIssueCustomField",
        (false, "ownedField") => "SingleOwnedIssueCustomField",
        (_, "state") => "StateIssueCustomField",
        (_, "period") => "PeriodIssueCustomField",
        (_, "date") => "DateIssueCustomField",
        (_, "integer") => "IntegerIssueCustomField",
        (_, "float") => "FloatIssueCustomField",
        (_, "string") | (_, "text") => "SimpleIssueCustomField",
        _ => "IssueCustomField",
    }
    .to_string()
}

fn example_value_for_project_field(
    field: &ProjectCustomField,
    issue_type: &str,
) -> serde_json::Value {
    if issue_type.contains("User") || issue_type.contains("Owned") {
        return first_bundle_ref(field, "login")
            .unwrap_or_else(|| serde_json::json!({ "login": "jane.doe" }));
    }
    if issue_type.contains("Enum") || issue_type.contains("State") || issue_type.contains("Version")
    {
        let item = first_bundle_ref(field, "name")
            .unwrap_or_else(|| serde_json::json!({ "name": "Example" }));
        if issue_type.starts_with("Multi") {
            return serde_json::json!([item]);
        }
        return item;
    }
    if issue_type.contains("Period") {
        return serde_json::json!({ "minutes": 60 });
    }
    if issue_type.contains("Date") {
        return serde_json::json!(1735689600000_i64);
    }
    if issue_type.contains("Integer") {
        return serde_json::json!(1);
    }
    if issue_type.contains("Float") {
        return serde_json::json!(1.0);
    }
    serde_json::json!("Example")
}

fn first_bundle_ref(field: &ProjectCustomField, preferred_key: &str) -> Option<serde_json::Value> {
    let value = field.bundle.as_ref()?.values.first()?;
    let obj = value.as_object()?;
    if let Some(preferred) = obj.get(preferred_key).and_then(|value| value.as_str()) {
        return Some(serde_json::json!({ preferred_key: preferred }));
    }
    if let Some(name) = obj.get("name").and_then(|value| value.as_str()) {
        return Some(serde_json::json!({ "name": name }));
    }
    if let Some(login) = obj.get("login").and_then(|value| value.as_str()) {
        return Some(serde_json::json!({ "login": login }));
    }
    None
}

fn value_shape_for_type(issue_type: &str) -> &'static str {
    if issue_type.starts_with("Multi") {
        "array of value refs"
    } else if issue_type.contains("User") || issue_type.contains("Owned") {
        "object, for example {\"login\":\"...\"}"
    } else if issue_type.contains("Enum")
        || issue_type.contains("State")
        || issue_type.contains("Version")
    {
        "object, for example {\"name\":\"...\"}"
    } else if issue_type.contains("Period") {
        "object, for example {\"minutes\":60}"
    } else {
        "primitive or API-shaped value"
    }
}

fn board_schema_fields(name_required: bool, projects_required: bool) -> Vec<SchemaField> {
    vec![
        field(
            "name",
            "string",
            name_required,
            if name_required {
                "required unless --name"
            } else {
                "optional"
            },
            "Agile board name; --name overrides JSON name",
            "Team Board",
        ),
        field(
            "projects",
            "array<object>",
            projects_required,
            if projects_required {
                "required unless --project"
            } else {
                "optional"
            },
            "Project references shaped as [{\"id\":\"<project-db-id>\"}]",
            "[{\"id\":\"0-96\"}]",
        ),
        field(
            "owner",
            "object",
            false,
            "optional",
            "Board owner reference",
            "{\"id\":\"1-51\"}",
        ),
        field(
            "visibleFor",
            "object|null",
            false,
            "optional",
            "Deprecated visibility group reference",
            "{\"id\":\"3-7\"}",
        ),
        field(
            "visibleForProjectBased",
            "boolean",
            false,
            "optional",
            "Deprecated project-based read visibility flag",
            "true",
        ),
        field(
            "updateableBy",
            "object|null",
            false,
            "optional",
            "Deprecated update group reference",
            "{\"id\":\"3-7\"}",
        ),
        field(
            "updateableByProjectBased",
            "boolean",
            false,
            "optional",
            "Deprecated project-based update permission flag",
            "true",
        ),
        field(
            "orphansAtTheTop",
            "boolean",
            false,
            "optional",
            "Place orphan swimlane at the top",
            "true",
        ),
        field(
            "hideOrphansSwimlane",
            "boolean",
            false,
            "optional",
            "Hide the orphan swimlane",
            "false",
        ),
        field(
            "estimationField",
            "object|null",
            false,
            "optional",
            "Estimation custom field reference",
            "{\"id\":\"58-1\"}",
        ),
        field(
            "originalEstimationField",
            "object|null",
            false,
            "optional",
            "Original estimation custom field reference",
            "{\"id\":\"58-2\"}",
        ),
        field(
            "swimlaneSettings",
            "object|null",
            false,
            "optional",
            "API-shaped swimlane settings",
            "{\"$type\":\"AttributeBasedSwimlaneSettings\"}",
        ),
        field(
            "colorCoding",
            "object|null",
            false,
            "optional",
            "API-shaped color coding settings",
            "{\"$type\":\"FieldBasedColorCoding\"}",
        ),
    ]
}

fn sprint_schema_fields(name_required: bool) -> Vec<SchemaField> {
    vec![
        field(
            "name",
            "string",
            name_required,
            if name_required {
                "required unless --name"
            } else {
                "optional"
            },
            "Sprint name; --name overrides JSON name",
            "Sprint 1",
        ),
        field(
            "goal",
            "string|null",
            false,
            "optional",
            "Sprint goal",
            "Finish onboarding",
        ),
        field(
            "start",
            "integer|null",
            false,
            "optional",
            "Start timestamp in milliseconds since epoch",
            "1735689600000",
        ),
        field(
            "finish",
            "integer|null",
            false,
            "optional",
            "Finish timestamp in milliseconds since epoch",
            "1736294400000",
        ),
        field(
            "archived",
            "boolean",
            false,
            "optional",
            "Whether the sprint is archived",
            "false",
        ),
        field(
            "isDefault",
            "boolean",
            false,
            "optional",
            "Whether matching new issues are added to this sprint by default",
            "true",
        ),
        field(
            "issues",
            "array<object>",
            false,
            "optional",
            "API-shaped issue references for sprint membership",
            "[{\"id\":\"2-42\"}]",
        ),
        field(
            "previousSprint",
            "object",
            false,
            "create-only optional",
            "Previous sprint reference used by YouTrack to move unresolved issues",
            "{\"id\":\"113-6\"}",
        ),
    ]
}

fn schemas() -> Vec<JsonCommandSchema> {
    vec![
        schema(
            "ticket",
            "create",
            "ytd ticket create --project <id> --json '{\"summary\":\"...\",\"description\":\"...\"}'",
            true,
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
                field(
                    "customFields",
                    "array<object>",
                    false,
                    "optional",
                    "API-shaped YouTrack issue custom field values",
                    "[{\"name\":\"Priority\",\"$type\":\"SingleEnumIssueCustomField\",\"value\":{\"name\":\"Critical\"}}]",
                ),
                field(
                    "tags",
                    "array<object>",
                    false,
                    "optional",
                    "API-shaped tag references",
                    "[{\"name\":\"backend\"}]",
                ),
            ],
            vec![
                "JSON input must be an object.",
                "summary is required.",
                "JSON project is not accepted; use --project so ytd can resolve the project ID.",
                "Visibility is controlled by flags/env/config, not JSON.",
                "Unknown ticket JSON fields are rejected. Allowed fields: customFields, description, summary, tags.",
                "Use --project with ytd schema ticket create for project-specific custom field examples.",
            ],
            vec![
                "ytd ticket create --project PROJ --json '{\"summary\":\"Fix login\",\"description\":\"Steps...\",\"customFields\":[{\"name\":\"Priority\",\"$type\":\"SingleEnumIssueCustomField\",\"value\":{\"name\":\"Critical\"}}]}'",
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
                field(
                    "customFields",
                    "array<object>",
                    false,
                    "optional",
                    "API-shaped YouTrack issue custom field values",
                    "[{\"name\":\"Assignee\",\"$type\":\"SingleUserIssueCustomField\",\"value\":{\"login\":\"jane.doe\"}}]",
                ),
                field(
                    "tags",
                    "array<object>",
                    false,
                    "optional",
                    "API-shaped tag references",
                    "[{\"name\":\"backend\"}]",
                ),
            ],
            vec![
                "JSON input must be an object.",
                "At least one of summary, description, customFields, tags, --visibility-group, or --no-visibility-group is required.",
                "Visibility update is flag-only; env/config defaults are ignored on update.",
                "Unknown ticket JSON fields are rejected. Allowed fields: customFields, description, summary, tags.",
                "Use --project with ytd schema ticket update for project-specific custom field examples.",
            ],
            vec!["ytd ticket update PROJ-42 --json '{\"customFields\":[{\"name\":\"Assignee\",\"$type\":\"SingleUserIssueCustomField\",\"value\":{\"login\":\"jane.doe\"}}]}'"],
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
            board_schema_fields(true, true),
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
            board_schema_fields(false, false),
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
            sprint_schema_fields(true),
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
            sprint_schema_fields(false),
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
        rules: rules.into_iter().map(String::from).collect(),
        examples,
        project: None,
        project_fields: vec![],
        project_examples: vec![],
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
        assert_eq!(value["strictFields"], true);
        let fields = value["fields"].as_array().unwrap();
        assert!(fields.iter().any(|field| field["name"] == "summary"));
        assert!(fields.iter().any(|field| field["name"] == "customFields"));
    }

    #[test]
    fn project_ticket_schema_generates_custom_field_examples() {
        let fields: Vec<ProjectCustomField> = serde_json::from_value(serde_json::json!([
            {
                "id": "92-1",
                "field": {
                    "id": "58-1",
                    "name": "Assignee",
                    "fieldType": {"id": "user[1]", "valueType": "user", "isMultiValue": false}
                },
                "$type": "UserProjectCustomField"
            },
            {
                "id": "92-2",
                "field": {
                    "id": "58-2",
                    "name": "Plattform",
                    "fieldType": {"id": "enum[*]", "valueType": "enum", "isMultiValue": true}
                },
                "bundle": {
                    "values": [{"id": "67-1", "name": "iOS", "$type": "EnumBundleElement"}]
                },
                "$type": "EnumProjectCustomField"
            }
        ]))
        .unwrap();
        let mut schema = find_schema("ticket", "create").unwrap();

        add_project_ticket_fields(&mut schema, &fields);

        let value = serde_json::to_value(schema).unwrap();
        let project_fields = value["projectFields"].as_array().unwrap();
        assert!(project_fields
            .iter()
            .any(|field| field["name"] == "Assignee"
                && field["type"] == "SingleUserIssueCustomField"));
        assert!(project_fields
            .iter()
            .any(|field| field["name"] == "Plattform"
                && field["type"] == "MultiEnumIssueCustomField"));
        let examples = value["projectExamples"].as_array().unwrap();
        assert!(examples.iter().any(|example| {
            example["json"]["customFields"][0]["value"] == serde_json::json!({"login":"jane.doe"})
        }));
        assert!(examples.iter().any(|example| {
            example["json"]["customFields"][0]["value"] == serde_json::json!([{"name":"iOS"}])
        }));
    }

    #[test]
    fn unsupported_target_lists_supported_targets() {
        let message = unsupported_target("ticket", Some("delete"));
        assert!(message.contains("Unsupported schema target: ticket delete"));
        assert!(message.contains("ticket create"));
    }
}
