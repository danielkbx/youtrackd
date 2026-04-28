use std::process::Command;

fn run_ytd(args: &[&str]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ytd"));
    command.args(args);
    command.env_remove("YOUTRACK_URL");
    command.env_remove("YOUTRACK_TOKEN");
    command.env_remove("YTD_CONFIG");
    command.env_remove("XDG_CONFIG_HOME");
    command.env_remove("YTD_VISIBILITY_GROUP");
    command.output().expect("failed to run ytd")
}

fn completion_output(shell: &str) -> std::process::Output {
    run_ytd(&["completion", shell])
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8")
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf-8")
}

#[test]
fn bash_completion_succeeds() {
    let output = completion_output("bash");

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());

    let completion = stdout(&output);
    assert!(completion.contains("_ytd"));
    assert!(completion.contains("complete -F _ytd ytd"));
    assert!(completion.contains("ticket"));
    assert!(completion.contains("article"));
    assert!(completion.contains("sprint"));
    assert!(completion.contains("completion"));
    assert!(completion.contains("--format"));
    assert!(completion.contains("--project"));
    assert!(completion.contains("json"));
}

#[test]
fn generated_bash_completion_is_context_aware_when_sourced() {
    let output = completion_output("bash");
    assert!(output.status.success());

    let temp = tempfile::tempdir().unwrap();
    let completion_path = temp.path().join("ytd.bash");
    std::fs::write(&completion_path, stdout(&output)).unwrap();

    let script = format!(
        r#"
source {}

COMP_WORDS=(ytd "")
COMP_CWORD=1
_ytd
printf 'top:%s\n' "${{COMPREPLY[*]}}"

COMP_WORDS=(ytd ticket "")
COMP_CWORD=2
_ytd
printf 'ticket:%s\n' "${{COMPREPLY[*]}}"

COMP_WORDS=(ytd comment "")
COMP_CWORD=2
_ytd
printf 'comment:%s\n' "${{COMPREPLY[*]}}"

COMP_WORDS=(ytd sprint ticket "")
COMP_CWORD=3
_ytd
printf 'sprint-ticket:%s\n' "${{COMPREPLY[*]}}"

COMP_WORDS=(ytd completion "")
COMP_CWORD=2
_ytd
printf 'completion:%s\n' "${{COMPREPLY[*]}}"

COMP_WORDS=(ytd ticket --)
COMP_CWORD=2
_ytd
printf 'ticket-options:%s\n' "${{COMPREPLY[*]}}"

COMP_WORDS=(ytd board create --)
COMP_CWORD=3
_ytd
printf 'board-create-options:%s\n' "${{COMPREPLY[*]}}"

COMP_WORDS=(ytd --format "")
COMP_CWORD=2
_ytd
printf 'format:%s\n' "${{COMPREPLY[*]}}"
"#,
        completion_path.display()
    );

    let complete_output = Command::new("bash")
        .args(["--noprofile", "--norc", "-c", &script])
        .output()
        .unwrap();

    assert!(
        complete_output.status.success(),
        "{}",
        String::from_utf8_lossy(&complete_output.stderr)
    );

    let candidates = String::from_utf8_lossy(&complete_output.stdout);
    assert!(candidates.contains("top:help login logout url open skill whoami config group user project alias article ticket comment attachment tag search board sprint completion"));
    assert!(candidates.contains("ticket:search list get create update comment comments tag untag link links attach attachments log worklog set fields history sprints delete"));
    assert!(candidates.contains("comment:get update attachments attach delete"));
    assert!(candidates.contains("sprint-ticket:list add remove"));
    assert!(candidates.contains("completion:bash zsh fish"));
    assert!(candidates.contains("ticket-options:--format --no-meta --version"));
    assert!(!line(candidates.as_ref(), "ticket-options:").contains("--template"));
    assert!(candidates.contains("board-create-options:"));
    assert!(line(candidates.as_ref(), "board-create-options:").contains("--name"));
    assert!(line(candidates.as_ref(), "board-create-options:").contains("--project"));
    assert!(line(candidates.as_ref(), "board-create-options:").contains("--template"));
    assert!(line(candidates.as_ref(), "board-create-options:").contains("--json"));
    assert!(line(candidates.as_ref(), "board-create-options:").contains("--format"));
    assert!(candidates.contains("format:text json raw md"));
}

#[test]
fn zsh_completion_succeeds() {
    let output = completion_output("zsh");

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());

    let completion = stdout(&output);
    assert!(completion.contains("#compdef ytd"));
    assert!(completion.contains("_ytd"));
    assert!(completion.contains("\"ticket\""));
    assert!(completion.contains("'search:Search tickets'"));
    assert!(completion.contains("\"sprint ticket\""));
    assert!(completion.contains("'add:Add ticket to sprint'"));
    assert!(completion.contains("--format"));
    assert!(completion.contains("json"));
}

#[test]
fn fish_completion_succeeds() {
    let output = completion_output("fish");

    assert!(output.status.success());
    assert!(stderr(&output).is_empty());

    let completion = stdout(&output);
    assert!(completion.contains("complete -c ytd"));
    assert!(completion.contains("ticket"));
    assert!(completion.contains("article"));
    assert!(completion.contains("-l format"));
    assert!(completion.contains("json"));
}

#[test]
fn generated_fish_completion_produces_context_aware_candidates_when_sourced() {
    if Command::new("fish").arg("--version").output().is_err() {
        return;
    }

    let output = completion_output("fish");
    assert!(output.status.success());

    let temp = tempfile::tempdir().unwrap();
    let completion_path = temp.path().join("ytd.fish");
    std::fs::write(&completion_path, stdout(&output)).unwrap();

    let complete_output = Command::new("fish")
        .args([
            "--no-config",
            "-c",
            &format!(
                "source {}; echo TOP; complete -C 'ytd '; echo SPRINT_TICKET; complete -C 'ytd sprint ticket '; echo COMPLETION; complete -C 'ytd completion '; echo FORMAT; complete -C 'ytd --format '; echo BOARD_CREATE_OPTIONS; complete -C 'ytd board create --'; echo TICKET_OPTIONS; complete -C 'ytd ticket --'",
                completion_path.display()
            ),
        ])
        .output()
        .unwrap();

    assert!(
        complete_output.status.success(),
        "{}",
        String::from_utf8_lossy(&complete_output.stderr)
    );

    let candidates = String::from_utf8_lossy(&complete_output.stdout);
    assert!(candidates.contains("ticket\tManage tickets"));
    assert!(candidates.contains("add\tAdd ticket to sprint"));
    assert!(candidates.contains("bash\tGenerate completion script"));
    assert!(candidates.contains("json"));
    assert!(
        section(candidates.as_ref(), "BOARD_CREATE_OPTIONS").contains("--template\tBoard template")
    );
    assert!(!section(candidates.as_ref(), "TICKET_OPTIONS").contains("--template\tBoard template"));
}

#[test]
fn completion_output_has_exactly_one_trailing_newline() {
    for shell in ["bash", "zsh", "fish"] {
        let output = completion_output(shell);
        let completion = stdout(&output);

        assert!(
            completion.ends_with('\n'),
            "{shell} completion should end with a newline"
        );
        assert!(
            !completion.ends_with("\n\n"),
            "{shell} completion should not end with multiple newlines"
        );
    }
}

#[test]
fn invalid_completion_shell_exits_non_zero_with_useful_error() {
    let output = completion_output("powershell");

    assert!(!output.status.success());
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("unsupported completion shell: powershell"));
}

#[test]
fn completion_rejects_extra_arguments() {
    let output = run_ytd(&["completion", "bash", "extra"]);

    assert!(!output.status.success());
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("Unknown command: ytd completion bash extra"));
}

#[test]
fn completion_help_documents_supported_shells_and_runtime_behavior() {
    for args in [["help", "completion"], ["completion", "help"]] {
        let output = run_ytd(&args);

        assert!(output.status.success());
        assert!(stderr(&output).is_empty());

        let help = stdout(&output);
        assert!(help.contains("ytd completion <bash|zsh|fish>"));
        assert!(help.contains("Bash"));
        assert!(help.contains("Zsh"));
        assert!(help.contains("Fish"));
        assert!(help.contains("stdout"));
        assert!(help.contains("does not require login"));
    }
}

fn line<'a>(text: &'a str, prefix: &str) -> &'a str {
    text.lines()
        .find(|line| line.starts_with(prefix))
        .unwrap_or_default()
}

fn section<'a>(text: &'a str, marker: &str) -> &'a str {
    text.split_once(marker)
        .map(|(_, rest)| rest)
        .unwrap_or_default()
}
