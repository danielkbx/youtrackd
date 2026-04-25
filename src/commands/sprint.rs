use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::error::YtdError;
use crate::format::{self, OutputOptions};
use crate::input;
use crate::types::{current_sprint_output, encode_sprint_id, parse_sprint_id, sprint_output};
use serde_json::{Map, Value};
use std::io::{self, BufRead, IsTerminal, Write};

pub fn run<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("list") => cmd_list(client, args, opts),
        Some("current") => cmd_current(client, args, opts),
        Some("get") => cmd_get(client, args, opts),
        Some("create") => cmd_create(client, args),
        Some("update") => cmd_update(client, args),
        Some("delete") => cmd_delete(client, args),
        Some("ticket") => cmd_ticket(client, args, opts),
        _ => Err(YtdError::Input(
            "Usage: ytd sprint <list|current|get|create|update|delete|ticket>".into(),
        )),
    }
}

fn require_board(args: &ParsedArgs) -> Result<&str, YtdError> {
    args.flags
        .get("board")
        .map(|s| s.as_str())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| YtdError::Input("--board is required".into()))
}

fn require_sprint_id(
    args: &ParsedArgs,
    usage: &str,
) -> Result<crate::types::ParsedSprintId, YtdError> {
    let id = args
        .positional
        .first()
        .ok_or_else(|| YtdError::Input(usage.into()))?;
    parse_sprint_id(id)
}

fn cmd_list<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let board_id = require_board(args)?;
    let sprints = client.list_sprints(board_id)?;
    let outputs: Vec<_> = sprints
        .into_iter()
        .map(|sprint| sprint_output(board_id, sprint))
        .collect();
    format::print_items(&outputs, opts);
    Ok(())
}

fn cmd_current<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    if let Some(board_id) = args.flags.get("board").map(|s| s.as_str()) {
        if board_id.trim().is_empty() {
            return Err(YtdError::Input("--board must not be empty".into()));
        }
        let board = client.get_agile(board_id)?;
        let output = current_sprint_output(&board)
            .ok_or_else(|| YtdError::Input(format!("Board {board_id} has no current sprint")))?;
        format::print_single(&output, opts);
        return Ok(());
    }

    let boards = client.list_agiles()?;
    let outputs: Vec<_> = boards.iter().filter_map(current_sprint_output).collect();
    format::print_items(&outputs, opts);
    Ok(())
}

fn cmd_get<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let id = require_sprint_id(args, "Usage: ytd sprint get <sprint-id>")?;
    let sprint = client.get_sprint(&id.board_id, &id.sprint_id)?;
    let output = sprint_output(&id.board_id, sprint);
    format::print_single(&output, opts);
    Ok(())
}

fn cmd_create<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let board_id = require_board(args)?;
    let json = input::read_optional_json_input(&args.flags)?;
    let body = build_create_sprint_body(args, json)?;
    let sprint = client.create_sprint(board_id, &body)?;
    println!("{}", encode_sprint_id(board_id, &sprint.id));
    Ok(())
}

fn cmd_update<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_sprint_id(
        args,
        "Usage: ytd sprint update <sprint-id> [--name <name>] [--json '...']",
    )?;
    let json = input::read_optional_json_input(&args.flags)?;
    let body = build_update_sprint_body(args, json)?;
    let sprint = client.update_sprint(&id.board_id, &id.sprint_id, &body)?;
    println!("{}", encode_sprint_id(&id.board_id, &sprint.id));
    Ok(())
}

fn cmd_delete<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_sprint_id(args, "Usage: ytd sprint delete <sprint-id> [-y]")?;
    let encoded = args.positional[0].as_str();
    if args.flags.get("y").map(|v| v == "true").unwrap_or(false) || confirm_delete(encoded)? {
        client.delete_sprint(&id.board_id, &id.sprint_id)?;
        println!("{encoded}");
    }
    Ok(())
}

fn cmd_ticket<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    match args.positional.first().map(|s| s.as_str()) {
        Some("list") => cmd_ticket_list(client, args, opts),
        Some("add") => cmd_ticket_add(client, args),
        Some("remove") => cmd_ticket_remove(client, args),
        _ => Err(YtdError::Input(
            "Usage: ytd sprint ticket <list|add|remove>".into(),
        )),
    }
}

fn require_nested_sprint_id(
    args: &ParsedArgs,
    index: usize,
    usage: &str,
) -> Result<crate::types::ParsedSprintId, YtdError> {
    let id = args
        .positional
        .get(index)
        .ok_or_else(|| YtdError::Input(usage.into()))?;
    parse_sprint_id(id)
}

fn require_nested_ticket_id<'a>(
    args: &'a ParsedArgs,
    index: usize,
    usage: &str,
) -> Result<&'a str, YtdError> {
    args.positional
        .get(index)
        .map(|s| s.as_str())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| YtdError::Input(usage.into()))
}

fn cmd_ticket_list<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let id = require_nested_sprint_id(args, 1, "Usage: ytd sprint ticket list <sprint-id>")?;
    let issues = client.list_sprint_issues(&id.board_id, &id.sprint_id)?;
    format::print_items(&issues, opts);
    Ok(())
}

fn cmd_ticket_add<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
) -> Result<(), YtdError> {
    let id = require_nested_sprint_id(
        args,
        1,
        "Usage: ytd sprint ticket add <sprint-id> <ticket-id>",
    )?;
    let ticket_id = require_nested_ticket_id(
        args,
        2,
        "Usage: ytd sprint ticket add <sprint-id> <ticket-id>",
    )?;
    let issue = client.add_issue_to_sprint(&id.board_id, &id.sprint_id, ticket_id)?;
    println!("{}", issue.id_readable.as_deref().unwrap_or(ticket_id));
    Ok(())
}

fn cmd_ticket_remove<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
) -> Result<(), YtdError> {
    let id = require_nested_sprint_id(
        args,
        1,
        "Usage: ytd sprint ticket remove <sprint-id> <ticket-id>",
    )?;
    let ticket_id = require_nested_ticket_id(
        args,
        2,
        "Usage: ytd sprint ticket remove <sprint-id> <ticket-id>",
    )?;
    let issue = client.remove_issue_from_sprint(&id.board_id, &id.sprint_id, ticket_id)?;
    println!("{}", issue.id_readable.as_deref().unwrap_or(ticket_id));
    Ok(())
}

fn build_create_sprint_body(args: &ParsedArgs, json: Option<Value>) -> Result<Value, YtdError> {
    let mut body = object_or_empty(json, "sprint create")?;

    if let Some(name) = args.flags.get("name") {
        body.insert("name".into(), Value::String(name.clone()));
    }

    require_non_empty_string(&body, "name", "--name or JSON name is required")?;
    Ok(Value::Object(body))
}

fn build_update_sprint_body(args: &ParsedArgs, json: Option<Value>) -> Result<Value, YtdError> {
    let mut body = object_or_empty(json, "sprint update")?;

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

fn confirm_delete(id: &str) -> Result<bool, YtdError> {
    if !io::stdin().is_terminal() {
        return Ok(false);
    }

    eprint!("Delete sprint {id}? Type 'yes' to confirm: ");
    io::stderr().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim() == "yes")
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
            _file: &Path,
            _name: &str,
        ) -> Result<String, YtdError> {
            Ok(String::new())
        }

        fn delete(&self, _url: &str, _token: &str) -> Result<(), YtdError> {
            Ok(())
        }
    }

    fn client() -> YtClient<MockTransport> {
        YtClient::new(
            YtdConfig {
                url: "https://test.youtrack.cloud".into(),
                token: "perm:test".into(),
            },
            MockTransport::new(vec![]),
        )
    }

    fn args(flags: &[(&str, &str)]) -> ParsedArgs {
        ParsedArgs {
            resource: Some("sprint".into()),
            action: Some("create".into()),
            positional: vec![],
            flags: flags
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect::<HashMap<_, _>>(),
        }
    }

    #[test]
    fn create_body_uses_name_and_json() {
        let body = build_create_sprint_body(
            &args(&[("name", "Sprint")]),
            Some(serde_json::json!({"goal":"Goal"})),
        )
        .unwrap();

        assert_eq!(body, serde_json::json!({"name":"Sprint","goal":"Goal"}));
    }

    #[test]
    fn name_flag_wins_over_json_name() {
        let body = build_create_sprint_body(
            &args(&[("name", "Flag")]),
            Some(serde_json::json!({"name":"JSON"})),
        )
        .unwrap();

        assert_eq!(body, serde_json::json!({"name":"Flag"}));
    }

    #[test]
    fn create_body_requires_name() {
        assert!(build_create_sprint_body(&args(&[]), None).is_err());
    }

    #[test]
    fn update_body_requires_fields() {
        assert!(build_update_sprint_body(&args(&[]), None).is_err());
    }

    #[test]
    fn create_body_rejects_non_object_json() {
        assert!(build_create_sprint_body(&args(&[]), Some(serde_json::json!([]))).is_err());
    }

    #[test]
    fn update_body_rejects_non_object_json() {
        assert!(build_update_sprint_body(&args(&[]), Some(serde_json::json!([]))).is_err());
    }

    #[test]
    fn current_without_board_lists_all_current_sprints() {
        let client = YtClient::new(
            YtdConfig {
                url: "https://test.youtrack.cloud".into(),
                token: "perm:test".into(),
            },
            MockTransport::new(vec![
                r#"[{"id":"108-4","name":"Board","projects":[{"id":"0-96","shortName":"DWP","name":"DW Playground"}],"sprints":[],"currentSprint":{"id":"113-6","name":"Sprint 1"}}]"#,
            ]),
        );
        let mut parsed = args(&[]);
        parsed.action = Some("current".into());

        assert!(cmd_current(
            &client,
            &parsed,
            &OutputOptions {
                format: format::Format::Raw,
                no_meta: false,
            }
        )
        .is_ok());
    }

    #[test]
    fn current_with_board_errors_without_current_sprint() {
        let client = YtClient::new(
            YtdConfig {
                url: "https://test.youtrack.cloud".into(),
                token: "perm:test".into(),
            },
            MockTransport::new(vec![
                r#"{"id":"108-4","name":"Board","projects":[],"sprints":[],"currentSprint":null}"#,
            ]),
        );
        let mut parsed = args(&[("board", "108-4")]);
        parsed.action = Some("current".into());

        assert!(cmd_current(
            &client,
            &parsed,
            &OutputOptions {
                format: format::Format::Raw,
                no_meta: false,
            }
        )
        .is_err());
    }

    #[test]
    fn current_as_sprint_id_is_rejected() {
        let mut parsed = args(&[]);
        parsed.positional = vec!["108-4:current".into()];
        assert!(require_sprint_id(&parsed, "usage").is_err());
    }

    #[test]
    fn sprint_ticket_requires_nested_action() {
        let parsed = ParsedArgs {
            resource: Some("sprint".into()),
            action: Some("ticket".into()),
            positional: vec![],
            flags: HashMap::new(),
        };

        assert!(cmd_ticket(
            &client(),
            &parsed,
            &OutputOptions {
                format: format::Format::Raw,
                no_meta: false,
            },
        )
        .is_err());
    }

    #[test]
    fn sprint_ticket_list_requires_sprint_id() {
        let parsed = ParsedArgs {
            resource: Some("sprint".into()),
            action: Some("ticket".into()),
            positional: vec!["list".into()],
            flags: HashMap::new(),
        };

        assert!(cmd_ticket_list(
            &client(),
            &parsed,
            &OutputOptions {
                format: format::Format::Raw,
                no_meta: false,
            },
        )
        .is_err());
    }

    #[test]
    fn sprint_ticket_add_requires_ticket_id() {
        let parsed = ParsedArgs {
            resource: Some("sprint".into()),
            action: Some("ticket".into()),
            positional: vec!["add".into(), "108-4:113-6".into()],
            flags: HashMap::new(),
        };

        assert!(cmd_ticket_add(&client(), &parsed).is_err());
    }

    #[test]
    fn sprint_ticket_remove_requires_ticket_id() {
        let parsed = ParsedArgs {
            resource: Some("sprint".into()),
            action: Some("ticket".into()),
            positional: vec!["remove".into(), "108-4:113-6".into()],
            flags: HashMap::new(),
        };

        assert!(cmd_ticket_remove(&client(), &parsed).is_err());
    }

    #[test]
    fn sprint_ticket_commands_reject_current_sprint_id() {
        for action in ["list", "add", "remove"] {
            let parsed = ParsedArgs {
                resource: Some("sprint".into()),
                action: Some("ticket".into()),
                positional: vec![action.into(), "108-4:current".into(), "DWP-1".into()],
                flags: HashMap::new(),
            };
            assert!(cmd_ticket(
                &client(),
                &parsed,
                &OutputOptions {
                    format: format::Format::Raw,
                    no_meta: false,
                },
            )
            .is_err());
        }
    }

    #[test]
    fn require_board_rejects_missing_board() {
        assert!(require_board(&args(&[])).is_err());
        let _ = client();
    }
}
