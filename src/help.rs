pub fn print_help(resource: Option<&str>, _action: Option<&str>) {
    match resource {
        None | Some("help") => print_global_help(),
        Some("login") => {
            println!("Usage: ytd login\n\nInteractively configure YouTrack URL and token.")
        }
        Some("logout") => println!("Usage: ytd logout\n\nRemove stored credentials."),
        Some("open") => print_open_help(),
        Some("whoami") => println!("Usage: ytd whoami\n\nShow current user info."),
        Some("config") => print_config_help(),
        Some("group") => print_group_help(),
        Some("project") => print_project_help(),
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
        Some(other) => println!("Unknown command: {other}\nRun `ytd help` for a list of commands."),
    }
}

fn print_global_help() {
    println!(
        "ytd - YouTrack CLI

Usage: ytd <command> [options]

Commands:
  login                     Configure credentials
  logout                    Remove credentials
  url <target>              Print web URL
  open <target>             Open web URL in browser
  whoami                    Show current user
  config set visibility-group <group>
                            Store default visibility group
  config get visibility-group
                            Show stored visibility group
  config unset visibility-group
                            Remove stored visibility group
  group list                List visibility groups

  project list              List projects
  project get <id>          Get project details

  article search <q>        Search articles
  article list              List articles (--project required)
  article get <id>          Get article
  article create            Create article (--project, --json)
  article update <id>       Update article (--json)
  article append <id> <t>   Append text to article
  article comment <id> <t>  Add comment to article
  article comments <id>     List article comments
  article attach <id> <f>   Attach file to article
  article attachments <id>  List article attachments
  article delete <id>       Delete article

  ticket search <q>         Search tickets
  ticket list               List tickets (--project required)
  ticket get <id>           Get ticket
  ticket create             Create ticket (--project, --json)
  ticket update <id>        Update ticket (--json)
  ticket comment <id> <t>   Add comment
  ticket comments <id>      List ticket comments
  ticket tag <id> <tag>     Add tag
  ticket untag <id> <tag>   Remove tag
  ticket link <id> <t>      Link to another ticket
  ticket links <id>         Show links
  ticket attach <id> <f>    Attach file
  ticket attachments <id>   List attachments
  ticket log <id> <dur>     Log time (e.g. 2h30m)
  ticket worklog <id>       Show work items
  ticket set <id> <f> <v>   Set custom field
  ticket fields <id>        Show field values
  ticket history <id>       Show activity log
  ticket delete <id>        Delete ticket

  comment get <id>          Get comment details
  comment update <id> <t>   Update comment text
  comment attachments <id>  List comment attachments
  comment delete <id>       Delete comment

  attachment get <id>       Get attachment details
  attachment delete <id>    Delete attachment
  attachment download <id>  Download attachment

  tag list                  List tags
  search list               List saved searches
  search run <name-or-id>   Run saved search
  board list                List agile boards
  board get <id>            Get board details

Global flags:
  --format text|raw|md      Output format (default: text)
  --no-meta                 Suppress IDs, dates, author
  -y                        Skip delete confirmation

Run `ytd help <command>` for command-specific help."
    );
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

fn print_article_help() {
    println!(
        "Usage:
  ytd article search <query> [--project <id>]
  ytd article list --project <id>
  ytd article get <id>
  ytd article create --project <id> --json '{{\"summary\":\"...\",\"content\":\"...\"}}' [--visibility-group <group> | --no-visibility-group]
  ytd article update <id> --json '{{\"summary\":\"...\",\"content\":\"...\"}}' [--visibility-group <group> | --no-visibility-group]
  ytd article append <id> <text>
  ytd article comment <id> <text> [--visibility-group <group> | --no-visibility-group]
  ytd article comments <id>
  ytd article attach <id> <file>
  ytd article attachments <id>
  ytd article delete <id> [-y]

Create/update print only the article ID on stdout."
    );
}

fn print_ticket_help() {
    println!(
        "Usage:
  ytd ticket search <query> [--project <id>]
  ytd ticket list --project <id>
  ytd ticket get <id>
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
  ytd ticket delete <id> [-y]

Durations: 30m, 1h, 2h30m, 90 (plain number = minutes)
Create/update print only the ticket ID on stdout."
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
  ABC-A-1:237-3"
    );
}

fn print_search_help() {
    println!(
        "Usage:
  ytd search list [--project <id>]
  ytd search run <name-or-id>

'search run' accepts a saved search ID or name (case-insensitive)."
    );
}

fn print_board_help() {
    println!(
        "Usage:
  ytd board list [--project <id>]
  ytd board get <id>

--project filters boards client-side by project membership."
    );
}
