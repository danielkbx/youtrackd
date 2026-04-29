use std::process::Command;

use tempfile::tempdir;

fn run_ytd(args: &[&str], extra_env: &[(&str, &str)]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ytd"));
    command.args(args);
    command.env_remove("YOUTRACK_URL");
    command.env_remove("YOUTRACK_TOKEN");
    command.env_remove("YTD_CONFIG");
    command.env_remove("XDG_CONFIG_HOME");
    command.env_remove("YTD_VISIBILITY_GROUP");
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
fn global_help_explains_agent_skill_command() {
    let output = run_ytd(&["help"], &[]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("Agent Skills"));
    assert!(text.contains("skill"));
    assert!(text.contains("Print latest SKILL.md guidance for AI agents"));
    assert!(text.contains("AI agents can run `ytd skill`"));
    assert!(stderr(&output).is_empty());
}

#[test]
fn skill_help_is_available_from_both_entry_points() {
    let direct = run_ytd(&["help", "skill"], &[]);
    let nested = run_ytd(&["skill", "help"], &[]);

    assert!(direct.status.success());
    assert!(nested.status.success());
    assert_eq!(stdout(&direct), stdout(&nested));
    assert!(stdout(&direct).contains("Generate the latest SKILL.md content"));
    assert!(stdout(&direct).contains("Agents can run this command themselves"));
}

#[test]
fn skill_generates_standard_markdown_without_credentials() {
    let output = run_ytd(&["skill"], &[]);

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());
    let text = stdout(&output);
    assert!(text.starts_with("---\n"));
    assert!(text.contains("name: ytd-youtrack"));
    assert!(text.contains("description: >-\n  Use when working with YouTrack"));
    assert!(text.contains("ytd --version"));
    assert!(text.contains("ytd skill --scope standard > SKILL.md"));
    assert!(text.contains("Prefer `--format json`"));
    assert!(text.contains("ytd schema <resource> <action>"));
    assert!(text.contains("ytd comment attach <comment-id> <file>"));
}

#[test]
fn skill_accepts_brief_scope_without_credentials() {
    let output = run_ytd(&["skill", "--scope", "brief"], &[]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("ytd skill --scope brief > SKILL.md"));
    assert!(stderr(&output).is_empty());
}

#[test]
fn full_skill_lists_comment_attach_command() {
    let output = run_ytd(&["skill", "--scope", "full"], &[]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("ytd schema ticket|article|board|sprint create|update"));
    assert!(stdout(&output).contains("ytd comment get|update|attach|attachments|delete"));
    assert!(stderr(&output).is_empty());
}

#[test]
fn skill_invalid_scope_errors_before_config_load() {
    let output = run_ytd(&["skill", "--scope", "invalid"], &[]);

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        "Invalid scope: invalid. Expected one of: brief, standard, full\n"
    );
}

#[test]
fn skill_rejects_json_format() {
    let output = run_ytd(&["skill", "--format", "json"], &[]);

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        "ytd skill only supports --format text or --format md\n"
    );
}

#[test]
fn project_skill_requires_login() {
    let tmp = tempdir().expect("tempdir");
    let xdg = tmp.path().to_string_lossy().into_owned();
    let output = run_ytd(&["skill", "--project", "DWP"], &[("XDG_CONFIG_HOME", &xdg)]);

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "Not logged in. Run `ytd login`.\n");
}
