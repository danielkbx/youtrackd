use std::fs;
use std::process::Command;

use tempfile::tempdir;

fn run_ytd(args: &[&str], extra_env: &[(&str, &str)]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ytd"));
    command.args(args);
    command.env_remove("YOUTRACK_URL");
    command.env_remove("YOUTRACK_TOKEN");
    command.env_remove("YTD_VISIBILITY_GROUP");
    command.env_remove("XDG_CONFIG_HOME");
    for (key, value) in extra_env {
        command.env(key, value);
    }
    command.output().expect("failed to run ytd")
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8")
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf-8")
}

#[test]
fn global_help_lists_config_commands() {
    let output = run_ytd(&["help"], &[]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("config set visibility-group <group>"));
    assert!(text.contains("config get visibility-group"));
    assert!(text.contains("config unset visibility-group"));
    assert!(text.contains("--format text|raw|json|md"));
    assert!(text.contains("Output format; invalid values are rejected"));
    assert!(text.contains("-y"));
    assert!(text.contains("Confirm delete without prompting"));
    assert!(stderr(&output).is_empty());
}

#[test]
fn config_help_is_available_from_both_entry_points() {
    let direct = run_ytd(&["help", "config"], &[]);
    let nested = run_ytd(&["config", "help"], &[]);

    assert!(direct.status.success());
    assert!(nested.status.success());
    assert_eq!(stdout(&direct), stdout(&nested));
    assert!(stdout(&direct).contains("Manage stored CLI settings without requiring login."));
    assert!(stdout(&direct).contains("visibility-group"));
}

#[test]
fn invalid_format_errors_before_config_load() {
    let output = run_ytd(&["project", "list", "--format", "yaml"], &[]);

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        "Invalid format: yaml. Expected one of: text, raw, json, md\n"
    );
}

#[test]
fn config_set_get_and_unset_work_without_login() {
    let tmp = tempdir().expect("tempdir");
    let config_path = tmp.path().join("config.json");
    let config_path_str = config_path.to_string_lossy().into_owned();
    let env = [("YTD_CONFIG", config_path_str.as_str())];

    let set = run_ytd(&["config", "set", "visibility-group", "Docs Team"], &env);
    assert!(set.status.success());
    assert_eq!(stdout(&set), "Docs Team\n");
    assert!(stderr(&set).is_empty());

    let stored = fs::read_to_string(&config_path).expect("config file should exist");
    assert!(stored.contains("\"visibilityGroup\": \"Docs Team\""));
    assert!(!stored.contains("\"url\""));
    assert!(!stored.contains("\"token\""));

    let get = run_ytd(&["config", "get", "visibility-group"], &env);
    assert!(get.status.success());
    assert_eq!(stdout(&get), "Docs Team\n");
    assert!(stderr(&get).is_empty());

    let unset = run_ytd(&["config", "unset", "visibility-group"], &env);
    assert!(unset.status.success());
    assert_eq!(stdout(&unset), "");
    assert!(stderr(&unset).is_empty());
    assert!(!config_path.exists());

    let get_missing = run_ytd(&["config", "get", "visibility-group"], &env);
    assert!(!get_missing.status.success());
    assert_eq!(stdout(&get_missing), "");
    assert_eq!(stderr(&get_missing), "visibility-group is not set\n");
}
