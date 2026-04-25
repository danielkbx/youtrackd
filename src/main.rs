mod args;
mod client;
mod commands;
mod config;
mod duration;
mod error;
mod format;
mod help;
mod input;
mod types;

use args::parse_args;
use error::YtdError;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn run() -> Result<(), YtdError> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = parse_args(&argv);

    // Version check
    if args.flags.contains_key("version") {
        println!("ytd {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Help check
    if args.resource.as_deref() == Some("help") || args.resource.is_none() {
        help::print_help(args.action.as_deref(), None);
        return Ok(());
    }
    if args.action.as_deref() == Some("help") {
        help::print_help(
            args.resource.as_deref(),
            args.positional.first().map(|s| s.as_str()),
        );
        return Ok(());
    }

    let resource = args.resource.as_deref().unwrap();
    let action = args.action.as_deref();
    let opts = format::OutputOptions::from_flags(&args.flags)?;

    // Validate command before loading config
    let runtime_alias = if is_known_command(resource, action) {
        None
    } else if matches!(action, Some("create" | "list")) {
        let stored = config::load_stored_config()?;
        stored.aliases.get(resource).cloned()
    } else {
        None
    };

    if runtime_alias.is_none() && !is_known_command(resource, action) {
        return Err(unknown_command(resource, action));
    }

    // Auth commands don't need config
    match resource {
        "login" => return commands::login::run(&args),
        "logout" => return commands::logout::run(),
        "config" => return commands::config::run(&args),
        "alias" if action == Some("delete") => return commands::alias::run_config_only(&args),
        _ => {}
    }

    let cfg = config::get_config()?;

    match resource {
        "open" => return commands::open::run(&cfg, &args),
        "url" => return commands::url::run(&cfg, &args),
        _ => {}
    }

    let transport = client::UreqTransport;
    let mut client = client::YtClient::new(cfg, transport);
    if args.flags.contains_key("verbose") {
        client.set_verbose(true);
    }

    match resource {
        "whoami" => commands::whoami::run(&client, &opts),
        "group" => commands::group::run(&client, &args, &opts),
        "project" => commands::project::run(&client, &args, &opts),
        "article" => commands::article::run(&client, &args, &opts),
        "ticket" => commands::ticket::run(&client, &args, &opts),
        "comment" => commands::comment::run(&client, &args, &opts),
        "attachment" => commands::attachment::run(&client, &args, &opts),
        "tag" => commands::tag::run(&client, &args, &opts),
        "user" => commands::user::run(&client, &args, &opts),
        "alias" => commands::alias::run(&client, &args, &opts),
        "search" => commands::search::run(&client, &args, &opts),
        "board" => commands::board::run(&client, &args, &opts),
        "sprint" => commands::sprint::run(&client, &args, &opts),
        _ if runtime_alias.is_some() => commands::alias::run_runtime(
            &client,
            resource,
            runtime_alias.as_ref().unwrap(),
            &args,
            &opts,
        ),
        _ => Err(YtdError::Input(format!("Unknown resource: {resource}"))),
    }
}

fn unknown_command(resource: &str, action: Option<&str>) -> YtdError {
    YtdError::Input(format!(
        "Unknown command: ytd {}{}",
        resource,
        action.map(|a| format!(" {a}")).unwrap_or_default()
    ))
}

fn is_known_command(resource: &str, action: Option<&str>) -> bool {
    matches!(
        (resource, action),
        ("login", None)
            | ("logout", None)
            | ("open", None)
            | ("url", None)
            | ("whoami", None)
            | ("config", Some("set" | "get" | "unset"))
            | ("alias", Some("create" | "list" | "delete"))
            | ("group", Some("list"))
            | ("user", Some("list" | "get"))
            | ("project", Some("list" | "get"))
            | (
                "article",
                Some(
                    "search"
                        | "list"
                        | "get"
                        | "create"
                        | "update"
                        | "append"
                        | "comment"
                        | "comments"
                        | "attach"
                        | "attachments"
                        | "delete"
                )
            )
            | (
                "ticket",
                Some(
                    "search"
                        | "list"
                        | "get"
                        | "create"
                        | "update"
                        | "comment"
                        | "comments"
                        | "tag"
                        | "untag"
                        | "link"
                        | "links"
                        | "attach"
                        | "attachments"
                        | "log"
                        | "worklog"
                        | "set"
                        | "fields"
                        | "history"
                        | "sprints"
                        | "delete"
                )
            )
            | ("comment", Some("get" | "update" | "delete" | "attachments"))
            | ("attachment", Some("get" | "delete" | "download"))
            | ("tag", Some("list"))
            | ("search", Some("list" | "run"))
            | (
                "board",
                Some("list" | "get" | "create" | "update" | "delete")
            )
            | (
                "sprint",
                Some("list" | "current" | "get" | "create" | "update" | "delete" | "ticket")
            )
    )
}

#[cfg(test)]
mod tests {
    use super::is_known_command;

    #[test]
    fn knows_config_commands() {
        assert!(is_known_command("config", Some("set")));
        assert!(is_known_command("config", Some("get")));
        assert!(is_known_command("config", Some("unset")));
        assert!(!is_known_command("config", None));
    }

    #[test]
    fn knows_group_commands() {
        assert!(is_known_command("group", Some("list")));
        assert!(!is_known_command("group", None));
    }

    #[test]
    fn knows_user_commands() {
        assert!(is_known_command("user", Some("list")));
        assert!(is_known_command("user", Some("get")));
        assert!(!is_known_command("user", Some("create")));
        assert!(!is_known_command("user", None));
    }

    #[test]
    fn knows_alias_commands() {
        assert!(is_known_command("alias", Some("create")));
        assert!(is_known_command("alias", Some("list")));
        assert!(is_known_command("alias", Some("delete")));
        assert!(!is_known_command("alias", Some("get")));
        assert!(!is_known_command("todo", Some("list")));
    }

    #[test]
    fn knows_open_and_url_commands() {
        assert!(is_known_command("open", None));
        assert!(is_known_command("url", None));
        assert!(!is_known_command("open", Some("now")));
        assert!(!is_known_command("url", Some("raw")));
    }

    #[test]
    fn knows_comment_commands() {
        assert!(is_known_command("comment", Some("get")));
        assert!(is_known_command("comment", Some("update")));
        assert!(is_known_command("comment", Some("delete")));
        assert!(is_known_command("comment", Some("attachments")));
        assert!(is_known_command("ticket", Some("comments")));
        assert!(!is_known_command("comment", Some("attach")));
        assert!(!is_known_command("comment", Some("create")));
    }

    #[test]
    fn knows_attachment_commands() {
        assert!(is_known_command("attachment", Some("get")));
        assert!(is_known_command("attachment", Some("delete")));
        assert!(is_known_command("attachment", Some("download")));
        assert!(!is_known_command("attachment", Some("create")));
    }

    #[test]
    fn knows_board_commands() {
        assert!(is_known_command("board", Some("list")));
        assert!(is_known_command("board", Some("get")));
        assert!(is_known_command("board", Some("create")));
        assert!(is_known_command("board", Some("update")));
        assert!(is_known_command("board", Some("delete")));
        assert!(!is_known_command("board", Some("sprint")));
    }

    #[test]
    fn knows_sprint_commands() {
        assert!(is_known_command("sprint", Some("list")));
        assert!(is_known_command("sprint", Some("current")));
        assert!(is_known_command("sprint", Some("get")));
        assert!(is_known_command("sprint", Some("create")));
        assert!(is_known_command("sprint", Some("update")));
        assert!(is_known_command("sprint", Some("delete")));
        assert!(is_known_command("sprint", Some("ticket")));
        assert!(is_known_command("ticket", Some("sprints")));
        assert!(!is_known_command("sprint", Some("attach")));
    }
}
