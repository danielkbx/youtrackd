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

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8")
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf-8")
}

#[test]
fn global_help_lists_open_and_url_commands() {
    let output = run_ytd(&["help"], &[]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("url <target>"));
    assert!(text.contains("open <target>"));
    assert!(stderr(&output).is_empty());
}

#[test]
fn url_help_shows_usage_and_examples() {
    let output = run_ytd(&["help", "url"], &[]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("Usage:\n  ytd url <target>"));
    assert!(text.contains("ytd url ABC-12"));
    assert!(text.contains("ytd url ABC-A"));
}

#[test]
fn open_help_shows_usage_and_examples() {
    let output = run_ytd(&["help", "open"], &[]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("Usage:\n  ytd open <target>"));
    assert!(text.contains("ytd open ABC-12"));
    assert!(text.contains("default browser"));
}

#[test]
fn url_without_target_shows_usage_error() {
    let output = run_ytd(
        &["url"],
        &[
            ("YOUTRACK_URL", "https://example.youtrack.cloud"),
            ("YOUTRACK_TOKEN", "perm:test"),
        ],
    );

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "Usage: ytd url <target>\n");
}

#[test]
fn open_without_target_shows_usage_error() {
    let output = run_ytd(
        &["open"],
        &[
            ("YOUTRACK_URL", "https://example.youtrack.cloud"),
            ("YOUTRACK_TOKEN", "perm:test"),
        ],
    );

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "Usage: ytd open <target>\n");
}
