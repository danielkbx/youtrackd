use std::process::Command;

fn run_ytd(args: &[&str], extra_env: &[(&str, &str)]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ytd"));
    command.args(args);
    command.env_remove("YTD_CONFIG");
    command.env_remove("XDG_CONFIG_HOME");
    command.env_remove("YOUTRACK_URL");
    command.env_remove("YOUTRACK_TOKEN");
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
fn attachment_help_shows_commands() {
    let output = run_ytd(&["help", "attachment"], &[]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("ytd attachment get <attachment-id>"));
    assert!(text.contains("ytd attachment delete <attachment-id> [-y]"));
    assert!(text.contains("ytd attachment download <attachment-id> [--output <path>]"));
    assert!(
        text.contains("Delete commands ask for confirmation. Use -y to confirm non-interactively.")
    );
    assert!(stderr(&output).is_empty());
}

#[test]
fn comment_help_lists_attachments_but_not_attach() {
    let output = run_ytd(&["help", "comment"], &[]);

    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("ytd comment attachments <comment-id>"));
    assert!(
        text.contains("Delete commands ask for confirmation. Use -y to confirm non-interactively.")
    );
    assert!(!text.contains("ytd comment attach <comment-id>"));
}

#[test]
fn ticket_article_and_search_help_describe_consistency_rules() {
    let ticket = run_ytd(&["help", "ticket"], &[]);
    let article = run_ytd(&["help", "article"], &[]);
    let search = run_ytd(&["help", "search"], &[]);

    assert!(ticket.status.success());
    assert!(article.status.success());
    assert!(search.status.success());
    assert!(
        stdout(&ticket).contains("Update changes visibility only with explicit visibility flags.")
    );
    assert!(
        stdout(&article).contains("Update changes visibility only with explicit visibility flags.")
    );
    assert!(stdout(&search).contains(
        "--project filters saved searches by project reference in the saved query text."
    ));
}

#[test]
fn comment_attach_is_rejected_before_config_load() {
    let output = run_ytd(&["comment", "attach", "DWP-1:4-1", "/tmp/a.txt"], &[]);

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "Unknown command: ytd comment attach\n");
}
