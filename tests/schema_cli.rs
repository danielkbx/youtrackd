use std::process::Command;

fn run_ytd(args: &[&str]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ytd"));
    command.args(args);
    command.env_remove("YOUTRACK_URL");
    command.env_remove("YOUTRACK_TOKEN");
    command.env_remove("YTD_VISIBILITY_GROUP");
    command.env("YTD_CONFIG", "/tmp/nonexistent-ytd-config");
    command.output().expect("failed to run ytd")
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8")
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf-8")
}

#[test]
fn schema_lists_json_commands_without_login() {
    let output = run_ytd(&["schema"]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("ticket create"));
    assert!(text.contains("article update"));
    assert!(text.contains("board create"));
    assert!(text.contains("sprint update"));
    assert!(!text.contains("Not logged in"));
    assert!(stderr(&output).is_empty());
}

#[test]
fn schema_list_json_is_machine_readable_without_login() {
    let output = run_ytd(&["schema", "list", "--format", "json"]);

    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(value
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["command"] == "ticket create"));
    assert!(stderr(&output).is_empty());
}

#[test]
fn ticket_create_schema_includes_fields_and_examples() {
    let output = run_ytd(&["schema", "ticket", "create"]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("summary"));
    assert!(text.contains("description"));
    assert!(text.contains("customFields"));
    assert!(text.contains("tags"));
    assert!(text.contains("--project"));
    assert!(text.contains("stdin takes precedence"));
    assert!(text.contains("ytd ticket create --project PROJ"));
}

#[test]
fn project_schema_requires_login() {
    let output = run_ytd(&["schema", "ticket", "create", "--project", "PROJ"]);

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("Not logged in"));
}

#[test]
fn article_update_schema_json_contains_parent_article() {
    let output = run_ytd(&["schema", "article", "update", "--format", "json"]);

    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["command"], "article update");
    assert!(value["fields"]
        .as_array()
        .unwrap()
        .iter()
        .any(|field| field["name"] == "parentArticle"));
    assert!(value["rules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|rule| rule.as_str().unwrap().contains("parentArticle:null")));
}

#[test]
fn board_create_schema_labels_pass_through() {
    let output = run_ytd(&["schema", "board", "create", "--format", "json"]);

    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["strictFields"], false);
    assert!(value["rules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|rule| rule.as_str().unwrap().contains("pass through")));
}

#[test]
fn schema_rejects_raw_and_md_formats() {
    let raw = run_ytd(&["schema", "sprint", "update", "--format", "raw"]);
    let md = run_ytd(&["schema", "sprint", "update", "--format", "md"]);
    let project_raw = run_ytd(&[
        "schema",
        "ticket",
        "create",
        "--project",
        "PROJ",
        "--format",
        "raw",
    ]);

    assert!(!raw.status.success());
    assert_eq!(stdout(&raw), "");
    assert_eq!(
        stderr(&raw),
        "ytd schema only supports --format text or --format json\n"
    );
    assert!(!md.status.success());
    assert_eq!(stdout(&md), "");
    assert_eq!(
        stderr(&md),
        "ytd schema only supports --format text or --format json\n"
    );
    assert!(!project_raw.status.success());
    assert_eq!(stdout(&project_raw), "");
    assert_eq!(
        stderr(&project_raw),
        "ytd schema only supports --format text or --format json\n"
    );
}

#[test]
fn schema_requires_nested_action() {
    let output = run_ytd(&["schema", "ticket"]);

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        "Usage: ytd schema <ticket|article|board|sprint> <create|update>\n"
    );
}

#[test]
fn schema_rejects_unsupported_target() {
    let output = run_ytd(&["schema", "ticket", "delete"]);

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    let err = stderr(&output);
    assert!(err.contains("Unsupported schema target: ticket delete"));
    assert!(err.contains("ticket create"));
}

#[test]
fn help_mentions_schema_discovery() {
    let global = run_ytd(&["help"]);
    let schema = run_ytd(&["help", "schema"]);
    let ticket = run_ytd(&["help", "ticket"]);
    let article = run_ytd(&["help", "article"]);
    let board = run_ytd(&["help", "board"]);
    let sprint = run_ytd(&["help", "sprint"]);

    assert!(stdout(&global).contains("schema <resource> <action>"));
    assert!(stdout(&schema).contains("ytd schema <ticket|article|board|sprint>"));
    assert!(stdout(&ticket).contains("ytd schema ticket create"));
    assert!(stdout(&article).contains("ytd schema article create"));
    assert!(stdout(&board).contains("ytd schema board create"));
    assert!(stdout(&sprint).contains("ytd schema sprint create"));
}
