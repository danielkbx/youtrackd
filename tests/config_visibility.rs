use std::process::Command;

fn run_ytd(args: &[&str], extra_env: &[(&str, &str)]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ytd"));
    command.args(args);
    command.env_remove("YTD_CONFIG");
    command.env_remove("XDG_CONFIG_HOME");
    command.env_remove("YTD_VISIBILITY_GROUP");
    for (key, value) in extra_env {
        command.env(key, value);
    }
    command.output().expect("failed to run ytd")
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf-8")
}

#[test]
fn ticket_create_rejects_conflicting_visibility_flags() {
    let output = run_ytd(
        &[
            "ticket",
            "create",
            "--project",
            "TEST",
            "--json",
            r#"{"summary":"Restricted issue"}"#,
            "--visibility-group",
            "Team Alpha",
            "--no-visibility-group",
        ],
        &[
            ("YOUTRACK_URL", "http://127.0.0.1:1"),
            ("YOUTRACK_TOKEN", "perm:test"),
        ],
    );

    assert!(!output.status.success());
    assert_eq!(
        stderr(&output),
        "--visibility-group cannot be combined with --no-visibility-group\n"
    );
}

#[test]
fn article_update_rejects_conflicting_visibility_flags() {
    let output = run_ytd(
        &[
            "article",
            "update",
            "KB-1",
            "--json",
            r#"{"summary":"Runbook"}"#,
            "--visibility-group",
            "Docs Team",
            "--no-visibility-group",
        ],
        &[
            ("YOUTRACK_URL", "http://127.0.0.1:1"),
            ("YOUTRACK_TOKEN", "perm:test"),
        ],
    );

    assert!(!output.status.success());
    assert_eq!(
        stderr(&output),
        "--visibility-group cannot be combined with --no-visibility-group\n"
    );
}
