use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::commands;
use crate::error::YtdError;
use crate::format::{self, OutputOptions};
use crate::input;
use serde_json::{Map, Value};

const VALID_TEMPLATES: &[&str] = &["kanban", "scrum", "version", "custom", "personal"];

pub fn run<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("list") => {
            let mut agiles = client.list_agiles()?;

            // Client-side project filter
            if let Some(project) = args.flags.get("project") {
                agiles.retain(|a| {
                    a.projects.iter().any(|p| {
                        p.short_name
                            .as_deref()
                            .map(|s| s.eq_ignore_ascii_case(project))
                            .unwrap_or(false)
                            || p.id == *project
                    })
                });
            }

            format::print_items(&agiles, opts);
            Ok(())
        }
        Some("get") => {
            let id = args
                .positional
                .first()
                .ok_or_else(|| YtdError::Input("Usage: ytd board get <id>".into()))?;
            let agile = client.get_agile(id)?;
            format::print_single(&agile, opts);
            Ok(())
        }
        Some("create") => {
            let json = input::read_optional_json_input(&args.flags)?;
            let template = validate_template(args.flags.get("template").map(|s| s.as_str()))?;
            let body = build_create_agile_body(client, args, json)?;
            let agile = client.create_agile(template, &body)?;
            println!("{}", agile.id);
            Ok(())
        }
        Some("update") => {
            let id = args.positional.first().ok_or_else(|| {
                YtdError::Input(
                    "Usage: ytd board update <id> [--name <name>] [--json '...']".into(),
                )
            })?;
            let json = input::read_optional_json_input(&args.flags)?;
            let body = build_update_agile_body(args, json)?;
            let agile = client.update_agile(id, &body)?;
            println!("{}", agile.id);
            Ok(())
        }
        Some("delete") => {
            let id = args
                .positional
                .first()
                .ok_or_else(|| YtdError::Input("Usage: ytd board delete <id> [-y]".into()))?;
            if commands::confirm_delete(
                "board",
                id,
                args.flags.get("y").map(|v| v == "true").unwrap_or(false),
            )? {
                client.delete_agile(id)?;
                println!("{id}");
            }
            Ok(())
        }
        _ => Err(YtdError::Input(
            "Usage: ytd board <list|get|create|update|delete>".into(),
        )),
    }
}

fn validate_template(template: Option<&str>) -> Result<Option<&str>, YtdError> {
    if let Some(template) = template {
        if !VALID_TEMPLATES.contains(&template) {
            return Err(YtdError::Input(format!(
                "Invalid template: {template}. Expected one of: {}",
                VALID_TEMPLATES.join(", ")
            )));
        }
    }
    Ok(template)
}

fn build_create_agile_body<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    json: Option<Value>,
) -> Result<Value, YtdError> {
    let mut body = object_or_empty(json, "board create")?;

    if args.flags.contains_key("project") && body.contains_key("projects") {
        return Err(YtdError::Input(
            "Use either --project or JSON projects, not both.".into(),
        ));
    }

    if let Some(projects) = args.flags.get("project") {
        let project_refs = resolve_project_refs(client, projects)?;
        body.insert("projects".into(), Value::Array(project_refs));
    }

    if let Some(name) = args.flags.get("name") {
        body.insert("name".into(), Value::String(name.clone()));
    }

    require_non_empty_string(&body, "name", "--name or JSON name is required")?;
    require_non_empty_array(&body, "projects", "--project or JSON projects is required")?;

    Ok(Value::Object(body))
}

fn build_update_agile_body(args: &ParsedArgs, json: Option<Value>) -> Result<Value, YtdError> {
    if args.flags.contains_key("project") {
        return Err(YtdError::Input(
            "board update does not accept --project; use JSON projects for project changes.".into(),
        ));
    }

    let mut body = object_or_empty(json, "board update")?;

    if let Some(name) = args.flags.get("name") {
        body.insert("name".into(), Value::String(name.clone()));
    }

    if body.is_empty() {
        return Err(YtdError::Input(
            "At least one update field is required. Use --name or --json.".into(),
        ));
    }

    Ok(Value::Object(body))
}

fn object_or_empty(json: Option<Value>, command: &str) -> Result<Map<String, Value>, YtdError> {
    match json {
        Some(Value::Object(map)) => Ok(map),
        Some(_) => Err(YtdError::Input(format!(
            "{command} requires a JSON object."
        ))),
        None => Ok(Map::new()),
    }
}

fn resolve_project_refs<T: HttpTransport>(
    client: &YtClient<T>,
    projects: &str,
) -> Result<Vec<Value>, YtdError> {
    let refs: Vec<&str> = projects
        .split(',')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();

    if refs.is_empty() {
        return Err(YtdError::Input("--project must not be empty".into()));
    }

    refs.into_iter()
        .map(|project| {
            let id = client.resolve_project_id(project)?;
            Ok(serde_json::json!({ "id": id }))
        })
        .collect()
}

fn require_non_empty_string(
    body: &Map<String, Value>,
    key: &str,
    message: &str,
) -> Result<(), YtdError> {
    if body
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
    {
        return Ok(());
    }
    Err(YtdError::Input(message.into()))
}

fn require_non_empty_array(
    body: &Map<String, Value>,
    key: &str,
    message: &str,
) -> Result<(), YtdError> {
    if body
        .get(key)
        .and_then(|v| v.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false)
    {
        return Ok(());
    }
    Err(YtdError::Input(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::YtClient;
    use crate::types::YtdConfig;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::path::Path;

    struct MockTransport {
        responses: RefCell<Vec<String>>,
    }

    impl MockTransport {
        fn new(responses: Vec<&str>) -> Self {
            Self {
                responses: RefCell::new(responses.into_iter().rev().map(String::from).collect()),
            }
        }
    }

    impl HttpTransport for MockTransport {
        fn get(&self, _url: &str, _token: &str) -> Result<String, YtdError> {
            self.responses
                .borrow_mut()
                .pop()
                .ok_or_else(|| YtdError::Http("No more mock responses".into()))
        }

        fn get_bytes(&self, _url: &str, _token: &str) -> Result<Vec<u8>, YtdError> {
            Ok(vec![])
        }

        fn post(&self, _url: &str, _token: &str, _body: &str) -> Result<String, YtdError> {
            self.responses
                .borrow_mut()
                .pop()
                .ok_or_else(|| YtdError::Http("No more mock responses".into()))
        }

        fn post_multipart(
            &self,
            _url: &str,
            _token: &str,
            _file_path: &Path,
            _file_name: &str,
        ) -> Result<String, YtdError> {
            Ok(String::new())
        }

        fn delete(&self, _url: &str, _token: &str) -> Result<(), YtdError> {
            Ok(())
        }
    }

    fn args(flags: &[(&str, &str)]) -> ParsedArgs {
        ParsedArgs {
            resource: Some("board".into()),
            action: Some("create".into()),
            positional: vec![],
            flags: flags
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<HashMap<_, _>>(),
        }
    }

    fn client(responses: Vec<&str>) -> YtClient<MockTransport> {
        YtClient::new(
            YtdConfig {
                url: "https://test.youtrack.cloud".into(),
                token: "perm:test".into(),
            },
            MockTransport::new(responses),
        )
    }

    #[test]
    fn validates_templates() {
        assert_eq!(validate_template(Some("scrum")).unwrap(), Some("scrum"));
        assert!(validate_template(Some("invalid")).is_err());
    }

    #[test]
    fn create_body_uses_name_and_resolved_project_flags() {
        let client = client(vec![
            r#"{"id":"0-96","name":"DW Playground","shortName":"DWP","archived":false,"description":null}"#,
        ]);
        let args = args(&[("name", "Board"), ("project", "DWP")]);

        let body = build_create_agile_body(&client, &args, None).unwrap();

        assert_eq!(
            body,
            serde_json::json!({"name":"Board","projects":[{"id":"0-96"}]})
        );
    }

    #[test]
    fn create_body_rejects_project_flag_and_json_projects() {
        let client = client(vec![]);
        let args = args(&[("name", "Board"), ("project", "DWP")]);
        let json = serde_json::json!({"projects":[{"id":"0-96"}]});

        assert!(build_create_agile_body(&client, &args, Some(json)).is_err());
    }

    #[test]
    fn create_body_allows_json_only_required_fields() {
        let client = client(vec![]);
        let args = args(&[]);
        let json = serde_json::json!({"name":"Board","projects":[{"id":"0-96"}]});

        let body = build_create_agile_body(&client, &args, Some(json)).unwrap();

        assert_eq!(
            body,
            serde_json::json!({"name":"Board","projects":[{"id":"0-96"}]})
        );
    }

    #[test]
    fn create_body_requires_name_and_projects() {
        let client = client(vec![]);
        assert!(build_create_agile_body(&client, &args(&[("project", "0-96")]), None).is_err());
        assert!(build_create_agile_body(&client, &args(&[("name", "Board")]), None).is_err());
    }

    #[test]
    fn update_body_name_overrides_json_name() {
        let args = ParsedArgs {
            resource: Some("board".into()),
            action: Some("update".into()),
            positional: vec!["108-4".into()],
            flags: HashMap::from([("name".into(), "Flag Name".into())]),
        };
        let json = serde_json::json!({"name":"JSON Name","orphansAtTheTop":true});

        let body = build_update_agile_body(&args, Some(json)).unwrap();

        assert_eq!(
            body,
            serde_json::json!({"name":"Flag Name","orphansAtTheTop":true})
        );
    }

    #[test]
    fn update_body_rejects_empty_and_non_object_json() {
        let args = ParsedArgs {
            resource: Some("board".into()),
            action: Some("update".into()),
            positional: vec!["108-4".into()],
            flags: HashMap::new(),
        };

        assert!(build_update_agile_body(&args, None).is_err());
        assert!(build_update_agile_body(&args, Some(serde_json::json!([]))).is_err());
    }
}
