#![allow(dead_code)]

pub const FORMAT_VALUES: &[&str] = &["text", "json", "raw", "md"];
pub const SKILL_SCOPE_VALUES: &[&str] = &["brief", "standard", "full"];
pub const BOARD_TEMPLATE_VALUES: &[&str] = &["kanban", "scrum", "version", "custom", "personal"];
pub const COMPLETION_SHELLS: &[&str] = &["bash", "zsh", "fish"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub name: &'static str,
    pub about: &'static str,
    pub subcommands: Vec<CommandSpec>,
    pub options: Vec<OptionSpec>,
    pub positionals: Vec<PositionalSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionSpec {
    pub long: Option<&'static str>,
    pub short: Option<char>,
    pub about: &'static str,
    pub value_name: Option<&'static str>,
    pub repeatable: bool,
    pub values: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionalSpec {
    pub name: &'static str,
    pub about: &'static str,
    pub repeatable: bool,
    pub values: &'static [&'static str],
}

impl CommandSpec {
    pub fn find(&self, path: &[&str]) -> Option<&CommandSpec> {
        match path.split_first() {
            None => Some(self),
            Some((name, rest)) => self
                .subcommands
                .iter()
                .find(|command| command.name == *name)
                .and_then(|command| command.find(rest)),
        }
    }

    pub fn command_paths(&self) -> Vec<Vec<&'static str>> {
        let mut paths = Vec::new();
        for command in &self.subcommands {
            collect_command_paths(command, Vec::new(), &mut paths);
        }
        paths
    }

    pub fn option_long_names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        collect_option_long_names(self, &mut names);
        names.sort_unstable();
        names.dedup();
        names
    }

    pub fn options_for_path(&self, path: &[&str]) -> Vec<&OptionSpec> {
        let mut options = self.options.iter().collect::<Vec<_>>();
        if let Some(command) = self.find(path) {
            options.extend(command.options.iter());
        }
        options.sort_by_key(|option| (option.long, option.short));
        options.dedup_by_key(|option| (option.long, option.short));
        options
    }
}

fn collect_command_paths(
    command: &CommandSpec,
    mut prefix: Vec<&'static str>,
    paths: &mut Vec<Vec<&'static str>>,
) {
    prefix.push(command.name);

    if command.subcommands.is_empty() {
        paths.push(prefix);
        return;
    }

    for subcommand in &command.subcommands {
        collect_command_paths(subcommand, prefix.clone(), paths);
    }
}

fn collect_option_long_names(command: &CommandSpec, names: &mut Vec<&'static str>) {
    names.extend(command.options.iter().filter_map(|option| option.long));

    for subcommand in &command.subcommands {
        collect_option_long_names(subcommand, names);
    }
}

pub fn cli_spec() -> CommandSpec {
    command(
        "ytd",
        "YouTrack CLI",
        vec![
            option_value("format", "Output format", "format", FORMAT_VALUES),
            option_flag("no-meta", "Suppress metadata"),
            option_short_flag('v', "Verbose HTTP diagnostics"),
            option_flag("version", "Print version"),
        ],
        vec![],
        vec![
            command(
                "help",
                "Show help",
                vec![],
                vec![positional("command", "Command to show help for", true, &[])],
                vec![],
            ),
            leaf("login", "Configure credentials"),
            leaf("logout", "Remove credentials"),
            command(
                "url",
                "Print web URL",
                vec![],
                vec![positional(
                    "target",
                    "Ticket, article, or project target",
                    false,
                    &[],
                )],
                vec![],
            ),
            command(
                "open",
                "Open web URL",
                vec![],
                vec![positional(
                    "target",
                    "Ticket, article, or project target",
                    false,
                    &[],
                )],
                vec![],
            ),
            command(
                "skill",
                "Print SKILL.md guidance",
                vec![
                    option_value("scope", "Skill detail level", "scope", SKILL_SCOPE_VALUES),
                    option_value("project", "Project short name or ID", "project", &[]),
                ],
                vec![],
                vec![],
            ),
            leaf("whoami", "Show current user"),
            command(
                "config",
                "Manage stored settings",
                vec![],
                vec![],
                vec![
                    config_visibility_command("set", true),
                    config_visibility_command("get", false),
                    config_visibility_command("unset", false),
                ],
            ),
            command(
                "group",
                "Manage groups",
                vec![],
                vec![],
                vec![leaf("list", "List groups")],
            ),
            command(
                "user",
                "Manage users",
                vec![],
                vec![],
                vec![
                    leaf("list", "List users"),
                    command(
                        "get",
                        "Get user",
                        vec![],
                        vec![positional(
                            "user-id-or-login",
                            "User ID or login",
                            false,
                            &[],
                        )],
                        vec![],
                    ),
                ],
            ),
            command(
                "project",
                "Manage projects",
                vec![],
                vec![],
                vec![
                    leaf("list", "List projects"),
                    command(
                        "get",
                        "Get project",
                        vec![],
                        vec![positional("id", "Project ID", false, &[])],
                        vec![],
                    ),
                ],
            ),
            command(
                "alias",
                "Manage local ticket aliases",
                vec![],
                vec![],
                vec![
                    command(
                        "create",
                        "Create or update alias",
                        vec![
                            option_value("project", "Project ID", "id", &[]),
                            option_value("user", "User ID", "id", &[]),
                            option_value("sprint", "Sprint ID or none", "sprint-id|none", &[]),
                        ],
                        vec![positional("alias", "Alias name", false, &[])],
                        vec![],
                    ),
                    leaf("list", "List aliases"),
                    command(
                        "delete",
                        "Delete alias",
                        vec![option_short_flag('y', "Confirm without prompting")],
                        vec![positional("alias", "Alias name", false, &[])],
                        vec![],
                    ),
                ],
            ),
            article_command(),
            ticket_command(),
            command(
                "comment",
                "Manage comments",
                vec![],
                vec![],
                vec![
                    command(
                        "get",
                        "Get comment",
                        vec![],
                        vec![positional("comment-id", "Comment ID", false, &[])],
                        vec![],
                    ),
                    command(
                        "update",
                        "Update comment",
                        visibility_update_options(),
                        vec![
                            positional("comment-id", "Comment ID", false, &[]),
                            positional("text", "Comment text", true, &[]),
                        ],
                        vec![],
                    ),
                    command(
                        "attachments",
                        "List comment attachments",
                        vec![],
                        vec![positional("comment-id", "Comment ID", false, &[])],
                        vec![],
                    ),
                    command(
                        "attach",
                        "Attach file to comment",
                        vec![],
                        vec![
                            positional("comment-id", "Comment ID", false, &[]),
                            positional("file", "File path", false, &[]),
                        ],
                        vec![],
                    ),
                    command(
                        "delete",
                        "Delete comment",
                        vec![option_short_flag('y', "Confirm without prompting")],
                        vec![positional("comment-id", "Comment ID", false, &[])],
                        vec![],
                    ),
                ],
            ),
            command(
                "attachment",
                "Manage attachments",
                vec![],
                vec![],
                vec![
                    command(
                        "get",
                        "Get attachment",
                        vec![],
                        vec![positional("attachment-id", "Attachment ID", false, &[])],
                        vec![],
                    ),
                    command(
                        "delete",
                        "Delete attachment",
                        vec![option_short_flag('y', "Confirm without prompting")],
                        vec![positional("attachment-id", "Attachment ID", false, &[])],
                        vec![],
                    ),
                    command(
                        "download",
                        "Download attachment",
                        vec![option_value(
                            "output",
                            "Output file or directory",
                            "path",
                            &[],
                        )],
                        vec![positional("attachment-id", "Attachment ID", false, &[])],
                        vec![],
                    ),
                ],
            ),
            command(
                "tag",
                "Manage tags",
                vec![],
                vec![],
                vec![command(
                    "list",
                    "List tags",
                    vec![option_value("project", "Project filter", "id", &[])],
                    vec![],
                    vec![],
                )],
            ),
            command(
                "search",
                "Manage saved searches",
                vec![],
                vec![],
                vec![
                    command(
                        "list",
                        "List saved searches",
                        vec![option_value("project", "Project filter", "id", &[])],
                        vec![],
                        vec![],
                    ),
                    command(
                        "run",
                        "Run saved search",
                        vec![],
                        vec![positional(
                            "name-or-id",
                            "Saved search name or ID",
                            false,
                            &[],
                        )],
                        vec![],
                    ),
                ],
            ),
            board_command(),
            sprint_command(),
            command(
                "completion",
                "Generate shell completions",
                vec![],
                vec![],
                COMPLETION_SHELLS
                    .iter()
                    .map(|shell| leaf(shell, "Generate completion script"))
                    .collect(),
            ),
        ],
    )
}

fn article_command() -> CommandSpec {
    command(
        "article",
        "Manage articles",
        vec![],
        vec![],
        vec![
            command(
                "search",
                "Search articles",
                vec![option_value("project", "Project filter", "id", &[])],
                vec![positional("query", "Search query", true, &[])],
                vec![],
            ),
            command(
                "list",
                "List articles",
                vec![option_value("project", "Project ID", "id", &[])],
                vec![],
                vec![],
            ),
            content_get_command("get", "Get article", "article-id"),
            command(
                "create",
                "Create article; JSON fields: summary, content, parentArticle",
                article_create_options(),
                vec![],
                vec![],
            ),
            command(
                "update",
                "Update article; parentArticle null clears parent",
                article_update_options(),
                vec![positional("article-id", "Article ID", false, &[])],
                vec![],
            ),
            command(
                "move",
                "Move article under another parent or clear parent",
                vec![],
                vec![
                    positional("article-id", "Article ID", false, &[]),
                    positional("parent-id", "Parent article ID or none", false, &["none"]),
                ],
                vec![],
            ),
            command(
                "append",
                "Append article text",
                vec![],
                vec![
                    positional("article-id", "Article ID", false, &[]),
                    positional("text", "Text", true, &[]),
                ],
                vec![],
            ),
            command(
                "comment",
                "Add article comment",
                visibility_create_options(),
                vec![
                    positional("article-id", "Article ID", false, &[]),
                    positional("text", "Comment text", true, &[]),
                ],
                vec![],
            ),
            command(
                "comments",
                "List article comments",
                vec![],
                vec![positional("article-id", "Article ID", false, &[])],
                vec![],
            ),
            attach_command("article-id"),
            attachments_command("article-id"),
            delete_command("article-id"),
        ],
    )
}

fn ticket_command() -> CommandSpec {
    command(
        "ticket",
        "Manage tickets",
        vec![],
        vec![],
        vec![
            command(
                "search",
                "Search tickets",
                vec![option_value("project", "Project filter", "id", &[])],
                vec![positional("query", "Search query", true, &[])],
                vec![],
            ),
            command(
                "list",
                "List tickets",
                vec![option_value("project", "Project ID", "id", &[])],
                vec![],
                vec![],
            ),
            content_get_command("get", "Get ticket", "ticket-id"),
            command("create", "Create ticket", create_options(), vec![], vec![]),
            command(
                "update",
                "Update ticket",
                update_options(),
                vec![positional("ticket-id", "Ticket ID", false, &[])],
                vec![],
            ),
            command(
                "comment",
                "Add ticket comment",
                visibility_create_options(),
                vec![
                    positional("ticket-id", "Ticket ID", false, &[]),
                    positional("text", "Comment text", true, &[]),
                ],
                vec![],
            ),
            command(
                "comments",
                "List ticket comments",
                vec![],
                vec![positional("ticket-id", "Ticket ID", false, &[])],
                vec![],
            ),
            command(
                "tag",
                "Add tag",
                vec![],
                vec![
                    positional("ticket-id", "Ticket ID", false, &[]),
                    positional("tag", "Tag", false, &[]),
                ],
                vec![],
            ),
            command(
                "untag",
                "Remove tag",
                vec![],
                vec![
                    positional("ticket-id", "Ticket ID", false, &[]),
                    positional("tag", "Tag", false, &[]),
                ],
                vec![],
            ),
            command(
                "link",
                "Link ticket",
                vec![option_value("type", "Link type", "type", &[])],
                vec![
                    positional("ticket-id", "Ticket ID", false, &[]),
                    positional("target", "Target ticket", false, &[]),
                ],
                vec![],
            ),
            command(
                "links",
                "List ticket links",
                vec![],
                vec![positional("ticket-id", "Ticket ID", false, &[])],
                vec![],
            ),
            attach_command("ticket-id"),
            attachments_command("ticket-id"),
            command(
                "log",
                "Log work time",
                vec![
                    option_value("date", "Work date", "YYYY-MM-DD", &[]),
                    option_value("type", "Work type", "worktype", &[]),
                ],
                vec![
                    positional("ticket-id", "Ticket ID", false, &[]),
                    positional("duration", "Duration", false, &[]),
                    positional("text", "Work text", true, &[]),
                ],
                vec![],
            ),
            command(
                "worklog",
                "Show work items",
                vec![],
                vec![positional("ticket-id", "Ticket ID", false, &[])],
                vec![],
            ),
            command(
                "set",
                "Set custom field",
                vec![],
                vec![
                    positional("ticket-id", "Ticket ID", false, &[]),
                    positional("field", "Field", false, &[]),
                    positional("value", "Value", false, &[]),
                ],
                vec![],
            ),
            command(
                "fields",
                "Show field values",
                vec![],
                vec![positional("ticket-id", "Ticket ID", false, &[])],
                vec![],
            ),
            command(
                "history",
                "Show activity log",
                vec![option_value(
                    "category",
                    "History category",
                    "category",
                    &[],
                )],
                vec![positional("ticket-id", "Ticket ID", false, &[])],
                vec![],
            ),
            command(
                "sprints",
                "List ticket sprints",
                vec![],
                vec![positional("ticket-id", "Ticket ID", false, &[])],
                vec![],
            ),
            delete_command("ticket-id"),
        ],
    )
}

fn board_command() -> CommandSpec {
    command(
        "board",
        "Manage agile boards",
        vec![],
        vec![],
        vec![
            command(
                "list",
                "List agile boards",
                vec![option_value("project", "Project filter", "id", &[])],
                vec![],
                vec![],
            ),
            command(
                "get",
                "Get board",
                vec![],
                vec![positional("board-id", "Board ID", false, &[])],
                vec![],
            ),
            command(
                "create",
                "Create agile board",
                vec![
                    option_value("name", "Board name", "name", &[]),
                    option_value("project", "Project IDs", "project[,project]", &[]),
                    option_value(
                        "template",
                        "Board template",
                        "template",
                        BOARD_TEMPLATE_VALUES,
                    ),
                    option_value("json", "JSON object", "json", &[]),
                ],
                vec![],
                vec![],
            ),
            command(
                "update",
                "Update agile board",
                vec![
                    option_value("name", "Board name", "name", &[]),
                    option_value("json", "JSON object", "json", &[]),
                ],
                vec![positional("board-id", "Board ID", false, &[])],
                vec![],
            ),
            delete_command("board-id"),
        ],
    )
}

fn sprint_command() -> CommandSpec {
    command(
        "sprint",
        "Manage sprints",
        vec![],
        vec![],
        vec![
            command(
                "list",
                "List sprints",
                vec![option_value("board", "Board ID", "board-id", &[])],
                vec![],
                vec![],
            ),
            command(
                "current",
                "List current sprints",
                vec![option_value("board", "Board ID", "board-id", &[])],
                vec![],
                vec![],
            ),
            command(
                "get",
                "Get sprint",
                vec![],
                vec![positional("sprint-id", "Sprint ID", false, &[])],
                vec![],
            ),
            command(
                "create",
                "Create sprint",
                vec![
                    option_value("board", "Board ID", "board-id", &[]),
                    option_value("name", "Sprint name", "name", &[]),
                    option_value("json", "JSON object", "json", &[]),
                ],
                vec![],
                vec![],
            ),
            command(
                "update",
                "Update sprint",
                vec![
                    option_value("name", "Sprint name", "name", &[]),
                    option_value("json", "JSON object", "json", &[]),
                ],
                vec![positional("sprint-id", "Sprint ID", false, &[])],
                vec![],
            ),
            delete_command("sprint-id"),
            command(
                "ticket",
                "Manage sprint tickets",
                vec![],
                vec![],
                vec![
                    command(
                        "list",
                        "List sprint tickets",
                        vec![],
                        vec![positional("sprint-id", "Sprint ID", false, &[])],
                        vec![],
                    ),
                    command(
                        "add",
                        "Add ticket to sprint",
                        vec![],
                        vec![
                            positional("sprint-id", "Sprint ID", false, &[]),
                            positional("ticket-id", "Ticket ID", false, &[]),
                        ],
                        vec![],
                    ),
                    command(
                        "remove",
                        "Remove ticket from sprint",
                        vec![],
                        vec![
                            positional("sprint-id", "Sprint ID", false, &[]),
                            positional("ticket-id", "Ticket ID", false, &[]),
                        ],
                        vec![],
                    ),
                ],
            ),
        ],
    )
}

fn config_visibility_command(name: &'static str, needs_value: bool) -> CommandSpec {
    let positionals = if needs_value {
        vec![
            positional("key", "Config key", false, &["visibility-group"]),
            positional("group", "Visibility group", false, &[]),
        ]
    } else {
        vec![positional(
            "key",
            "Config key",
            false,
            &["visibility-group"],
        )]
    };
    command(
        name,
        "Manage visibility-group config",
        vec![],
        positionals,
        vec![],
    )
}

fn content_get_command(
    name: &'static str,
    about: &'static str,
    id_name: &'static str,
) -> CommandSpec {
    command(
        name,
        about,
        vec![option_flag("no-comments", "Omit comments")],
        vec![positional(id_name, "ID", false, &[])],
        vec![],
    )
}

fn create_options() -> Vec<OptionSpec> {
    let mut options = vec![
        option_value("project", "Project ID", "id", &[]),
        option_value("json", "JSON object", "json", &[]),
    ];
    options.extend(visibility_create_options());
    options
}

fn update_options() -> Vec<OptionSpec> {
    let mut options = vec![option_value("json", "JSON object", "json", &[])];
    options.extend(visibility_update_options());
    options
}

fn article_create_options() -> Vec<OptionSpec> {
    let mut options = vec![
        option_value("project", "Project ID", "id", &[]),
        option_value(
            "json",
            "Article JSON: summary, content, parentArticle",
            "json",
            &[],
        ),
    ];
    options.extend(visibility_create_options());
    options
}

fn article_update_options() -> Vec<OptionSpec> {
    let mut options = vec![option_value(
        "json",
        "Article JSON: summary, content, parentArticle; parentArticle:null clears parent",
        "json",
        &[],
    )];
    options.extend(visibility_update_options());
    options
}

fn visibility_create_options() -> Vec<OptionSpec> {
    vec![
        option_value("visibility-group", "Visibility group", "group", &[]),
        option_flag("no-visibility-group", "Suppress visibility default"),
    ]
}

fn visibility_update_options() -> Vec<OptionSpec> {
    vec![
        option_value("visibility-group", "Visibility group", "group", &[]),
        option_flag("no-visibility-group", "Clear visibility"),
    ]
}

fn attach_command(parent: &'static str) -> CommandSpec {
    command(
        "attach",
        "Attach file",
        vec![],
        vec![
            positional(parent, "Parent ID", false, &[]),
            positional("file", "File path", false, &[]),
        ],
        vec![],
    )
}

fn attachments_command(parent: &'static str) -> CommandSpec {
    command(
        "attachments",
        "List attachments",
        vec![],
        vec![positional(parent, "Parent ID", false, &[])],
        vec![],
    )
}

fn delete_command(id_name: &'static str) -> CommandSpec {
    command(
        "delete",
        "Delete resource",
        vec![option_short_flag('y', "Confirm without prompting")],
        vec![positional(id_name, "ID", false, &[])],
        vec![],
    )
}

fn command(
    name: &'static str,
    about: &'static str,
    options: Vec<OptionSpec>,
    positionals: Vec<PositionalSpec>,
    subcommands: Vec<CommandSpec>,
) -> CommandSpec {
    CommandSpec {
        name,
        about,
        subcommands,
        options,
        positionals,
    }
}

fn leaf(name: &'static str, about: &'static str) -> CommandSpec {
    command(name, about, vec![], vec![], vec![])
}

fn option_flag(long: &'static str, about: &'static str) -> OptionSpec {
    OptionSpec {
        long: Some(long),
        short: None,
        about,
        value_name: None,
        repeatable: false,
        values: &[],
    }
}

fn option_short_flag(short: char, about: &'static str) -> OptionSpec {
    OptionSpec {
        long: None,
        short: Some(short),
        about,
        value_name: None,
        repeatable: false,
        values: &[],
    }
}

fn option_value(
    long: &'static str,
    about: &'static str,
    value_name: &'static str,
    values: &'static [&'static str],
) -> OptionSpec {
    OptionSpec {
        long: Some(long),
        short: None,
        about,
        value_name: Some(value_name),
        repeatable: false,
        values,
    }
}

fn positional(
    name: &'static str,
    about: &'static str,
    repeatable: bool,
    values: &'static [&'static str],
) -> PositionalSpec {
    PositionalSpec {
        name,
        about,
        repeatable,
        values,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_level_commands_exist() {
        let spec = cli_spec();
        let commands = spec
            .subcommands
            .iter()
            .map(|command| command.name)
            .collect::<Vec<_>>();

        assert_eq!(
            commands,
            vec![
                "help",
                "login",
                "logout",
                "url",
                "open",
                "skill",
                "whoami",
                "config",
                "group",
                "user",
                "project",
                "alias",
                "article",
                "ticket",
                "comment",
                "attachment",
                "tag",
                "search",
                "board",
                "sprint",
                "completion",
            ]
        );
    }

    #[test]
    fn completion_exposes_supported_shells() {
        let spec = cli_spec();
        let completion = spec.find(&["completion"]).unwrap();
        let shells = completion
            .subcommands
            .iter()
            .map(|command| command.name)
            .collect::<Vec<_>>();

        assert_eq!(shells, COMPLETION_SHELLS);
    }

    #[test]
    fn fixed_values_are_exposed() {
        let spec = cli_spec();
        let format = spec
            .options
            .iter()
            .find(|option| option.long == Some("format"))
            .unwrap();
        let scope = spec
            .find(&["skill"])
            .unwrap()
            .options
            .iter()
            .find(|option| option.long == Some("scope"))
            .unwrap();
        let template = spec
            .find(&["board", "create"])
            .unwrap()
            .options
            .iter()
            .find(|option| option.long == Some("template"))
            .unwrap();

        assert_eq!(format.values, FORMAT_VALUES);
        assert_eq!(scope.values, SKILL_SCOPE_VALUES);
        assert_eq!(template.values, BOARD_TEMPLATE_VALUES);
    }

    #[test]
    fn command_paths_include_nested_leaves() {
        let paths = cli_spec().command_paths();

        assert!(paths.contains(&vec!["config", "set"]));
        assert!(paths.contains(&vec!["sprint", "ticket", "list"]));
        assert!(paths.contains(&vec!["sprint", "ticket", "add"]));
        assert!(paths.contains(&vec!["sprint", "ticket", "remove"]));
        assert!(paths.contains(&vec!["completion", "bash"]));
    }

    #[test]
    fn options_for_path_are_context_specific() {
        let spec = cli_spec();
        let ticket = spec
            .options_for_path(&["ticket"])
            .into_iter()
            .filter_map(|option| option.long)
            .collect::<Vec<_>>();
        let board_create = spec
            .options_for_path(&["board", "create"])
            .into_iter()
            .filter_map(|option| option.long)
            .collect::<Vec<_>>();

        assert!(ticket.contains(&"format"));
        assert!(!ticket.contains(&"template"));
        assert!(board_create.contains(&"format"));
        assert!(board_create.contains(&"template"));
        assert!(board_create.contains(&"json"));
    }
}
