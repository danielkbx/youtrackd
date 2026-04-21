pub fn print_help(resource: Option<&str>, _action: Option<&str>) {
    match resource {
        None | Some("help") => print_global_help(),
        Some("login") => println!("Usage: ytd login\n\nInteractively configure YouTrack URL and token."),
        Some("logout") => println!("Usage: ytd logout\n\nRemove stored credentials."),
        Some("whoami") => println!("Usage: ytd whoami\n\nShow current user info."),
        Some("project") => print_project_help(),
        Some("article") => print_article_help(),
        Some("ticket") => print_ticket_help(),
        Some("tag") => println!("Usage:\n  ytd tag list [--project <id>]\n\nList tags. --project filters client-side."),
        Some("search") => print_search_help(),
        Some("board") => print_board_help(),
        Some(other) => println!("Unknown command: {other}\nRun `ytd help` for a list of commands."),
    }
}

fn print_global_help() {
    println!("ytd - YouTrack CLI

Usage: ytd <command> [options]

Commands:
  login                     Configure credentials
  logout                    Remove credentials
  whoami                    Show current user

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

  tag list                  List tags
  search list               List saved searches
  search run <name-or-id>   Run saved search
  board list                List agile boards
  board get <id>            Get board details

Global flags:
  --format text|raw|md      Output format (default: text)
  --no-meta                 Suppress IDs, dates, author
  -y                        Skip delete confirmation

Run `ytd help <command>` for command-specific help.");
}

fn print_project_help() {
    println!("Usage:
  ytd project list [--format raw] [--no-meta]
  ytd project get <shortName> [--format raw] [--no-meta]");
}

fn print_article_help() {
    println!("Usage:
  ytd article search <query> [--project <id>]
  ytd article list --project <id>
  ytd article get <id>
  ytd article create --project <id> --json '{{\"summary\":\"...\",\"content\":\"...\"}}'
  ytd article update <id> --json '{{\"summary\":\"...\",\"content\":\"...\"}}'
  ytd article append <id> <text>
  ytd article comment <id> <text>
  ytd article comments <id>
  ytd article attach <id> <file>
  ytd article attachments <id>
  ytd article delete <id> [-y]

Create/update print only the article ID on stdout.");
}

fn print_ticket_help() {
    println!("Usage:
  ytd ticket search <query> [--project <id>]
  ytd ticket list --project <id>
  ytd ticket get <id>
  ytd ticket create --project <id> --json '{{\"summary\":\"...\",\"description\":\"...\"}}'
  ytd ticket update <id> --json '{{\"summary\":\"...\",\"description\":\"...\"}}'
  ytd ticket comment <id> <text>
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
Create/update print only the ticket ID on stdout.");
}

fn print_search_help() {
    println!("Usage:
  ytd search list [--project <id>]
  ytd search run <name-or-id>

'search run' accepts a saved search ID or name (case-insensitive).");
}

fn print_board_help() {
    println!("Usage:
  ytd board list [--project <id>]
  ytd board get <id>

--project filters boards client-side by project membership.");
}
