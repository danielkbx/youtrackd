pub fn print_help(resource: Option<&str>, _action: Option<&str>) {
    match resource {
        None | Some("help") => print_global_help(),
        Some("login") => {
            println!("Usage: ytd login\n\nInteractively configure YouTrack URL and token.")
        }
        Some("logout") => println!("Usage: ytd logout\n\nRemove stored credentials."),
        Some("open") => print_open_help(),
        Some("skill") => print_skill_help(),
        Some("whoami") => println!("Usage: ytd whoami\n\nShow current user info."),
        Some("config") => print_config_help(),
        Some("group") => print_group_help(),
        Some("user") => print_user_help(),
        Some("project") => print_project_help(),
        Some("alias") => print_alias_help(),
        Some("article") => print_article_help(),
        Some("ticket") => print_ticket_help(),
        Some("comment") => print_comment_help(),
        Some("attachment") => print_attachment_help(),
        Some("url") => print_url_help(),
        Some("tag") => println!(
            "Usage:\n  ytd tag list [--project <id>]\n\nList tags. --project filters client-side."
        ),
        Some("search") => print_search_help(),
        Some("board") => print_board_help(),
        Some("sprint") => print_sprint_help(),
        Some(other) => println!("Unknown command: {other}\nRun `ytd help` for a list of commands."),
    }
}

fn print_global_help() {
    println!("ytd - YouTrack CLI\n");
    println!("Usage: ytd <command> [options]\n");
    println!("Commands:\n");

    print_help_group(
        "Core",
        &[
            ("login", "Configure credentials"),
            ("logout", "Remove credentials"),
            ("url <target>", "Print web URL"),
            ("open <target>", "Open web URL in browser"),
            ("whoami", "Show current user"),
        ],
    );
    print_help_group(
        "Agent Skills",
        &[("skill", "Print latest SKILL.md guidance for AI agents")],
    );
    print_help_group(
        "Config",
        &[
            (
                "config set visibility-group <group>",
                "Store default visibility group",
            ),
            (
                "config get visibility-group",
                "Show stored visibility group",
            ),
            (
                "config unset visibility-group",
                "Remove stored visibility group",
            ),
            ("group list", "List visibility groups"),
        ],
    );
    print_help_group(
        "Users And Aliases",
        &[
            ("user list", "List users"),
            ("user get <id-or-login>", "Get user details"),
            ("alias create <name>", "Create or update an alias"),
            ("alias list", "List configured aliases"),
            ("alias delete <name>", "Delete an alias"),
            ("<alias> create <text>", "Create ticket through alias"),
            ("<alias> list", "List alias tickets"),
        ],
    );
    print_help_group(
        "Projects",
        &[
            ("project list", "List projects"),
            ("project get <id>", "Get project details"),
        ],
    );
    print_help_group(
        "Articles",
        &[
            ("article search <q>", "Search articles"),
            ("article list", "List articles (--project required)"),
            ("article get <id>", "Get article"),
            ("article create", "Create article (--project, --json)"),
            ("article update <id>", "Update article (--json)"),
            ("article append <id> <t>", "Append text to article"),
            ("article comment <id> <t>", "Add comment to article"),
            ("article comments <id>", "List article comments"),
            ("article attach <id> <f>", "Attach file to article"),
            ("article attachments <id>", "List article attachments"),
            ("article delete <id>", "Delete article"),
        ],
    );
    print_help_group(
        "Tickets",
        &[
            ("ticket search <q>", "Search tickets"),
            ("ticket list", "List tickets (--project required)"),
            ("ticket get <id>", "Get ticket"),
            ("ticket create", "Create ticket (--project, --json)"),
            ("ticket update <id>", "Update ticket (--json)"),
            ("ticket comment <id> <t>", "Add comment"),
            ("ticket comments <id>", "List ticket comments"),
            ("ticket tag <id> <tag>", "Add tag"),
            ("ticket untag <id> <tag>", "Remove tag"),
            ("ticket link <id> <t>", "Link to another ticket"),
            ("ticket links <id>", "Show links"),
            ("ticket attach <id> <f>", "Attach file"),
            ("ticket attachments <id>", "List attachments"),
            ("ticket log <id> <dur>", "Log time (e.g. 2h30m)"),
            ("ticket worklog <id>", "Show work items"),
            ("ticket set <id> <f> <v>", "Set custom field"),
            ("ticket fields <id>", "Show field values"),
            ("ticket history <id>", "Show activity log"),
            ("ticket sprints <id>", "List ticket sprints"),
            ("ticket delete <id>", "Delete ticket"),
            ("tag list", "List tags"),
        ],
    );
    print_help_group(
        "Comments",
        &[
            ("comment get <id>", "Get comment details"),
            ("comment update <id> <t>", "Update comment text"),
            ("comment attachments <id>", "List comment attachments"),
            ("comment delete <id>", "Delete comment"),
        ],
    );
    print_help_group(
        "Attachments",
        &[
            ("attachment get <id>", "Get attachment details"),
            ("attachment delete <id>", "Delete attachment"),
            ("attachment download <id>", "Download attachment"),
        ],
    );
    print_help_group(
        "Saved Searches",
        &[
            ("search list", "List saved searches"),
            ("search run <name-or-id>", "Run saved search"),
        ],
    );
    print_help_group(
        "Boards And Sprints",
        &[
            ("board list", "List agile boards"),
            ("board get <id>", "Get board details"),
            ("board create", "Create agile board"),
            ("board update <id>", "Update agile board"),
            ("board delete <id>", "Delete agile board"),
            ("sprint list", "List board sprints"),
            ("sprint current", "List current sprints"),
            ("sprint get <id>", "Get sprint details"),
            ("sprint create", "Create sprint"),
            ("sprint update <id>", "Update sprint"),
            ("sprint delete <id>", "Delete sprint"),
            ("sprint ticket list <id>", "List sprint tickets"),
            ("sprint ticket add <id> <t>", "Add ticket to sprint"),
            ("sprint ticket remove <id> <t>", "Remove ticket from sprint"),
        ],
    );

    println!("Global flags:");
    print_help_items(&[
        (
            "--format text|raw|json|md",
            "Output format; invalid values are rejected",
        ),
        ("--no-meta", "Suppress IDs, dates, author"),
        ("-y", "Confirm delete without prompting"),
    ]);
    println!("\nAI agents can run `ytd skill` to get current ytd usage instructions.");
    println!("Use `ytd skill --project <project>` for project-specific examples.");
    println!("\nRun `ytd help <command>` for command-specific help.");
}

fn print_help_group(title: &str, items: &[(&str, &str)]) {
    println!("{title}:");
    print_help_items(items);
    println!();
}

fn print_help_items(items: &[(&str, &str)]) {
    const WIDTH: usize = 36;

    for (command, description) in items {
        println!("  {command:<WIDTH$}  {description}");
    }
}

fn print_config_help() {
    println!(
        "Usage:
  ytd config set visibility-group <group>
  ytd config get visibility-group
  ytd config unset visibility-group

Manage stored CLI settings without requiring login.
Currently supported key: visibility-group"
    );
}

fn print_url_help() {
    println!(
        "Usage:
  ytd url <target>

Print the YouTrack web URL for a target.

Examples:
  ytd url ABC-12
  ytd url ABC-A-12
  ytd url ABC
  ytd url ABC-A"
    );
}

fn print_open_help() {
    println!(
        "Usage:
  ytd open <target>

Open the YouTrack web URL for a target in the default browser and print the URL.

Supported targets:
  <PROJECT>-<NUMBER>       Ticket, e.g. ABC-12
  <PROJECT>-A-<NUMBER>     Article, opens in project context, e.g. ABC-A-1
  <PROJECT>                Project overview, e.g. ABC
  <PROJECT>-A              Project knowledge base, e.g. ABC-A

Examples:
  ytd open ABC-12
  ytd open ABC-A-12
  ytd open ABC
  ytd open ABC-A"
    );
}

fn print_skill_help() {
    println!(
        "Usage:
  ytd skill [--scope brief|standard|full] [--project <project>]

Generate the latest SKILL.md content for AI agents using ytd.

Agents can run this command themselves to fetch current ytd usage
instructions instead of relying on a stale checked-in skill file.
Redirect stdout to SKILL.md when a persistent skill file is wanted.

Options:
  --scope brief|standard|full   Detail level for the generated skill; default: standard
  --project <project>           Resolve project and include project-specific context/examples

Examples:
  ytd skill
  ytd skill --scope brief
  ytd skill --project DWP
  ytd skill --project DWP --scope full > SKILL.md"
    );
}

fn print_project_help() {
    println!(
        "Usage:
  ytd project list [--format raw] [--no-meta]
  ytd project get <shortName> [--format raw] [--no-meta]"
    );
}

fn print_group_help() {
    println!(
        "Usage:
  ytd group list [--format raw] [--no-meta]

List known YouTrack groups. Useful for visibility-group selection."
    );
}

fn print_user_help() {
    println!(
        "Usage:
  ytd user list [--format text|json|raw] [--no-meta]
  ytd user get <user-id-or-login> [--format text|json|raw] [--no-meta]

List and inspect YouTrack users. user get accepts a YouTrack user database ID or login.
Use user get to find the user ID for hand-written alias config."
    );
}

fn print_alias_help() {
    println!(
        "Usage:
  ytd alias create <alias> [--project <project-id>] [--user <user-id>] [--sprint <sprint-id|none>]
  ytd alias list [--format text|json|raw] [--no-meta]
  ytd alias delete <alias> [-y]
  ytd <alias> create <text>
  ytd <alias> list [--all] [--format text|json|raw|md] [--no-meta]

Aliases store only IDs in the config file:
  project: YouTrack project database ID
  user: YouTrack user database ID
  sprint: optional ytd sprint ID <board-id>:<sprint-id>

alias list is config-backed, not YouTrack API-backed, so --format json and --format raw return the same ytd alias model.
Alias ticket lists use the same output as ticket list, ticket search, search run, and sprint ticket list.
Delete commands ask for confirmation. Use -y to confirm non-interactively."
    );
}

fn print_article_help() {
    println!(
        "Usage:
  ytd article search <query> [--project <id>]
  ytd article list --project <id>
  ytd article get <id> [--no-comments]
  ytd article create --project <id> --json '{{\"summary\":\"...\",\"content\":\"...\"}}' [--visibility-group <group> | --no-visibility-group]
  ytd article update <id> --json '{{\"summary\":\"...\",\"content\":\"...\"}}' [--visibility-group <group> | --no-visibility-group]
  ytd article append <id> <text>
  ytd article comment <id> <text> [--visibility-group <group> | --no-visibility-group]
  ytd article comments <id>
  ytd article attach <id> <file>
  ytd article attachments <id>
  ytd article delete <id> [-y]

Create/update print only the article ID on stdout.
Create uses configured visibility defaults. Update changes visibility only with explicit visibility flags.
Delete commands ask for confirmation. Use -y to confirm non-interactively.
Text output renders Markdown content as readable terminal text with ASCII tables and prints content after metadata, after a blank line and without a field label. Use article get --no-comments to omit comments from text, json, raw, or md output."
    );
}

fn print_ticket_help() {
    println!(
        "Usage:
  ytd ticket search <query> [--project <id>]
  ytd ticket list --project <id>
  ytd ticket get <id> [--no-comments]
  ytd ticket create --project <id> --json '{{\"summary\":\"...\",\"description\":\"...\"}}' [--visibility-group <group> | --no-visibility-group]
  ytd ticket update <id> --json '{{\"summary\":\"...\",\"description\":\"...\"}}' [--visibility-group <group> | --no-visibility-group]
  ytd ticket comment <id> <text> [--visibility-group <group> | --no-visibility-group]
  ytd ticket comments <id>
  ytd ticket tag <id> <tag>
  ytd ticket untag <id> <tag>
  ytd ticket link <id> <target> [--type <linktype>]
  ytd ticket links <id>
  ytd ticket attach <id> <file>
  ytd ticket attachments <id>
  ytd ticket log <id> <duration> [text] [--date YYYY-MM-DD] [--type <worktype>]
  ytd ticket worklog <id>
  ytd ticket set <id> <field> <value>
  ytd ticket fields <id>
  ytd ticket history <id> [--category <category>]
  ytd ticket sprints <id>
  ytd ticket delete <id> [-y]

Durations: 30m, 1h, 2h30m, 90 (plain number = minutes)
Create/update print only the ticket ID on stdout.
Create uses configured visibility defaults. Update changes visibility only with explicit visibility flags.
Delete commands ask for confirmation. Use -y to confirm non-interactively.

Text output for ticket search/list/get and linked or sprint tickets is specialized:
compact lists show ID, summary, project, important fields, and updated/resolved state;
ticket get shows a detail report with status, custom fields, metadata, then a blank line and description without a label; comments follow the parent content.
Text output renders Markdown content fields as readable terminal text with ASCII tables and prints content after metadata, after a blank line and without a field label. Use ticket get --no-comments to omit comments from text, json, raw, or md output."
    );
}

fn print_comment_help() {
    println!(
        "Usage:
  ytd comment get <comment-id>
  ytd comment update <comment-id> <text> [--visibility-group <group> | --no-visibility-group]
  ytd comment attachments <comment-id>
  ytd comment delete <comment-id> [-y]

Comment IDs are returned by:
  ytd ticket comments <ticket-id>
  ytd article comments <article-id>

Use the returned id field, for example:
  ABC-12:4-17
  ABC-A-1:251-0

New comments use configured visibility defaults. Comment updates change visibility only with explicit visibility flags.
Delete commands ask for confirmation. Use -y to confirm non-interactively.
Text output for comment get renders Markdown text as terminal text after metadata, without a text field label.
Comment attachment upload is not supported by the YouTrack REST API flow verified for ytd."
    );
}

fn print_attachment_help() {
    println!(
        "Usage:
  ytd attachment get <attachment-id>
  ytd attachment delete <attachment-id> [-y]
  ytd attachment download <attachment-id> [--output <path>]

Attachment IDs are returned by:
  ytd ticket attachments <ticket-id>
  ytd article attachments <article-id>
  ytd comment attachments <comment-id>

Use the returned id field, for example:
  ABC-12:8-2897
  ABC-A-1:237-3

Delete commands ask for confirmation. Use -y to confirm non-interactively."
    );
}

fn print_search_help() {
    println!(
        "Usage:
  ytd search list [--project <id>]
  ytd search run <name-or-id>

'search run' accepts a saved search ID or name (case-insensitive).
--project filters saved searches by project reference in the saved query text.
Text output uses the same compact ticket format as ticket search."
    );
}

fn print_board_help() {
    println!(
        "Usage:
  ytd board list [--project <id>]
  ytd board get <id>
  ytd board create --name <name> --project <project>[,<project>...] [--template <template>] [--json '...']
  ytd board update <id> [--name <name>] [--json '...']
  ytd board delete <id> [-y]

--project filters boards client-side by project membership for list.
For create, --project sets the board projects and accepts short names or database IDs.
Templates: kanban, scrum, version, custom, personal.
Delete commands ask for confirmation. Use -y to confirm non-interactively.
Use --json or stdin for advanced YouTrack Agile fields."
    );
}

fn print_sprint_help() {
    println!(
        "Usage:
  ytd sprint list [--board <board-id>]
  ytd sprint current [--board <board-id>]
  ytd sprint get <sprint-id>
  ytd sprint create --board <board-id> --name <name> [--json '...']
  ytd sprint update <sprint-id> [--name <name>] [--json '...']
  ytd sprint delete <sprint-id> [-y]
  ytd sprint ticket list <sprint-id>
  ytd sprint ticket add <sprint-id> <ticket-id>
  ytd sprint ticket remove <sprint-id> <ticket-id>

Use the returned id field with sprint get, update, delete, and sprint ticket commands.
Without --board, sprint list returns sprints from all boards.
Use ytd sprint current to list current sprints across boards, or --board for one board.
current is not accepted as a sprint-id.
Sprint ticket commands list, add, and remove tickets in a sprint.
Sprint ticket list text output uses the same compact ticket format as ticket list.
Delete commands ask for confirmation. Use -y to confirm non-interactively.
Use --json or stdin for advanced YouTrack sprint fields."
    );
}
