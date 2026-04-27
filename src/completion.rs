use crate::cli_spec::{CommandSpec, OptionSpec};
use crate::error::YtdError;

pub fn render_completion(shell: &str, spec: &CommandSpec) -> Result<String, YtdError> {
    match shell {
        "bash" => Ok(render_bash(spec)),
        "zsh" => Ok(render_zsh(spec)),
        "fish" => Ok(render_fish(spec)),
        _ => Err(YtdError::Input(format!(
            "unsupported completion shell: {shell}"
        ))),
    }
}

fn render_bash(spec: &CommandSpec) -> String {
    let command_cases = bash_command_cases(spec);
    let option_cases = bash_option_cases(spec);
    let fixed_value_cases = bash_fixed_value_cases(spec);
    let value_options = pattern_words(
        collect_options(spec)
            .iter()
            .filter(|option| option.value_name.is_some())
            .map(|option| option_name(option)),
    );
    let value_option_assignments = pattern_words(
        collect_options(spec)
            .iter()
            .filter(|option| option.value_name.is_some())
            .map(|option| format!("{}=*", option_name(option))),
    );
    let command_words = pattern_words(command_words(spec));

    format!(
        r#"# ytd bash completion
_ytd()
{{
    local cur prev candidates path
    local path_words=()
    COMPREPLY=()
    cur="${{COMP_WORDS[COMP_CWORD]}}"
    prev="${{COMP_WORDS[COMP_CWORD-1]}}"

    for ((i = 1; i < COMP_CWORD; i++)); do
        case "${{COMP_WORDS[i]}}" in
            {value_options})
                ((i++))
                ;;
            {value_option_assignments})
                ;;
            -*)
                ;;
            *)
                case "${{COMP_WORDS[i]}}" in
                    {command_words})
                        path_words+=("${{COMP_WORDS[i]}}")
                        ;;
                esac
                ;;
        esac
    done

    path="${{path_words[*]}}"

    case "$prev" in
{fixed_value_cases}
    esac

    case "$cur" in
        --*)
            case "$path" in
{option_cases}
                *)
                    candidates=""
                    ;;
            esac
            COMPREPLY=( $(compgen -W "$candidates" -- "$cur") )
            ;;
        *)
            case "$path" in
{command_cases}
                *)
                    candidates=""
                    ;;
            esac
            COMPREPLY=( $(compgen -W "$candidates" -- "$cur") )
            ;;
    esac
}}

complete -F _ytd ytd
"#
    )
}

fn bash_command_cases(spec: &CommandSpec) -> String {
    let mut cases = vec![format!(
        "                \"\")\n                    candidates=\"{}\"\n                    ;;",
        words(spec.subcommands.iter().map(|command| command.name))
    )];

    for command in &spec.subcommands {
        collect_bash_command_cases(command, Vec::new(), &mut cases);
    }

    cases.join("\n")
}

fn collect_bash_command_cases(
    command: &CommandSpec,
    mut path: Vec<&'static str>,
    cases: &mut Vec<String>,
) {
    path.push(command.name);

    if !command.subcommands.is_empty() {
        cases.push(format!(
            "                \"{}\")\n                    candidates=\"{}\"\n                    ;;",
            path.join(" "),
            words(command.subcommands.iter().map(|command| command.name))
        ));

        for subcommand in &command.subcommands {
            collect_bash_command_cases(subcommand, path.clone(), cases);
        }
    }
}

fn bash_option_cases(spec: &CommandSpec) -> String {
    let mut cases = vec![format!(
        "                \"\")\n                    candidates=\"{}\"\n                    ;;",
        words(
            spec.options_for_path(&[])
                .iter()
                .map(|option| option_name(option))
        )
    )];

    for path in command_paths_including_non_leaves(spec) {
        cases.push(format!(
            "                \"{}\")\n                    candidates=\"{}\"\n                    ;;",
            path.join(" "),
            words(spec.options_for_path(&path).iter().map(|option| option_name(option)))
        ));
    }

    cases.join("\n")
}

fn bash_fixed_value_cases(spec: &CommandSpec) -> String {
    collect_options(spec)
        .into_iter()
        .filter(|option| !option.values.is_empty())
        .map(|option| {
            format!(
                "        {})\n            case \" ${{path}} \" in\n{}\n            esac\n            ;;",
                option_name(option),
                bash_fixed_value_path_cases(spec, option)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn bash_fixed_value_path_cases(spec: &CommandSpec, option: &OptionSpec) -> String {
    let option_words = words(option.values.iter().copied());
    matching_option_paths(spec, option)
        .into_iter()
        .map(|path| {
            format!(
                "                \" {} \")\n                    COMPREPLY=( $(compgen -W \"{}\" -- \"$cur\") )\n                    return 0\n                    ;;",
                path.join(" "),
                option_words
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_zsh(spec: &CommandSpec) -> String {
    let command_cases = zsh_command_cases(spec);
    let option_cases = zsh_option_cases(spec);
    let fixed_value_cases = zsh_fixed_value_cases(spec);
    let value_options = pattern_words(
        collect_options(spec)
            .iter()
            .filter(|option| option.value_name.is_some())
            .map(|option| option_name(option)),
    );
    let value_option_assignments = pattern_words(
        collect_options(spec)
            .iter()
            .filter(|option| option.value_name.is_some())
            .map(|option| format!("{}=*", option_name(option))),
    );
    let command_words = pattern_words(command_words(spec));

    format!(
        r#"#compdef ytd

_ytd()
{{
  local -a commands options path_words
  local cur prev path word
  cur="${{words[CURRENT]}}"
  prev="${{words[CURRENT-1]}}"

  for (( i = 2; i < CURRENT; i++ )); do
    word="${{words[i]}}"
    case "$word" in
      {value_options})
        (( i++ ))
        ;;
      {value_option_assignments})
        ;;
      -*)
        ;;
      *)
        case "$word" in
          {command_words})
            path_words+=("$word")
            ;;
        esac
        ;;
    esac
  done
  path="${{path_words[*]}}"

  case "$prev" in
{fixed_value_cases}
  esac

  if [[ "$cur" == --* ]]; then
    case "$path" in
{option_cases}
      *)
        options=()
        ;;
    esac
    compadd -- $options
    return
  fi

  case "$path" in
{command_cases}
    *)
      _message 'no more ytd subcommands'
      ;;
  esac
}}

_ytd "$@"
"#
    )
}

fn zsh_command_cases(spec: &CommandSpec) -> String {
    let mut cases = vec![format!(
        "    \"\")\n      commands=({})\n      _describe 'ytd command' commands\n      ;;",
        zsh_command_entries(&spec.subcommands)
    )];

    for command in &spec.subcommands {
        collect_zsh_command_cases(command, Vec::new(), &mut cases);
    }

    cases.join("\n")
}

fn collect_zsh_command_cases(
    command: &CommandSpec,
    mut path: Vec<&'static str>,
    cases: &mut Vec<String>,
) {
    path.push(command.name);

    if !command.subcommands.is_empty() {
        cases.push(format!(
            "    \"{}\")\n      commands=({})\n      _describe 'ytd subcommand' commands\n      ;;",
            path.join(" "),
            zsh_command_entries(&command.subcommands)
        ));

        for subcommand in &command.subcommands {
            collect_zsh_command_cases(subcommand, path.clone(), cases);
        }
    }
}

fn zsh_option_cases(spec: &CommandSpec) -> String {
    let mut cases = vec![format!(
        "      \"\")\n        options=({})\n        ;;",
        words(
            spec.options_for_path(&[])
                .iter()
                .map(|option| option_name(option))
        )
    )];

    for path in command_paths_including_non_leaves(spec) {
        cases.push(format!(
            "      \"{}\")\n        options=({})\n        ;;",
            path.join(" "),
            words(
                spec.options_for_path(&path)
                    .iter()
                    .map(|option| option_name(option)),
            )
        ));
    }

    cases.join("\n")
}

fn zsh_fixed_value_cases(spec: &CommandSpec) -> String {
    collect_options(spec)
        .into_iter()
        .filter(|option| !option.values.is_empty())
        .map(|option| {
            format!(
                "    {})\n      case \" ${{path}} \" in\n{}\n      esac\n      ;;",
                option_name(option),
                zsh_fixed_value_path_cases(spec, option)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn zsh_fixed_value_path_cases(spec: &CommandSpec, option: &OptionSpec) -> String {
    let option_words = words(option.values.iter().copied());
    matching_option_paths(spec, option)
        .into_iter()
        .map(|path| {
            format!(
                "        \" {} \")\n          compadd -- {}\n          return\n          ;;",
                path.join(" "),
                option_words
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn zsh_command_entries(commands: &[CommandSpec]) -> String {
    commands
        .iter()
        .map(|command| {
            format!(
                "'{}:{}'",
                zsh_escape(command.name),
                zsh_escape(command.about)
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_fish(spec: &CommandSpec) -> String {
    let mut lines = vec!["# ytd fish completion".to_string()];

    for option in &spec.options {
        lines.push(fish_option_line(option, None));
    }

    let top_level_commands = command_names(&spec.subcommands);
    for command in &spec.subcommands {
        render_fish_command_lines(command, &[], &top_level_commands, lines.as_mut());
    }

    for path in command_paths_including_non_leaves(spec) {
        if let Some(command) = spec.find(&path) {
            for option in &command.options {
                lines.push(fish_option_line(option, Some(&path)));
            }
        }
    }

    lines.push(String::new());
    lines.join("\n")
}

fn render_fish_command_lines(
    command: &CommandSpec,
    parents: &[&str],
    siblings: &[&str],
    lines: &mut Vec<String>,
) {
    lines.push(format!(
        "complete -c ytd -f -n '{}' -a '{}' -d '{}'",
        fish_command_condition(parents, siblings),
        fish_escape(command.name),
        fish_escape(command.about)
    ));

    let mut child_parents = parents.to_vec();
    child_parents.push(command.name);
    let child_siblings = command_names(&command.subcommands);

    for subcommand in &command.subcommands {
        render_fish_command_lines(subcommand, &child_parents, &child_siblings, lines);
    }
}

fn fish_option_line(option: &OptionSpec, path: Option<&[&str]>) -> String {
    let mut line = "complete -c ytd".to_string();

    if let Some(long) = option.long {
        line.push_str(&format!(" -l {}", fish_escape(long)));
    }

    if let Some(short) = option.short {
        line.push_str(&format!(" -s {short}"));
    }

    if option.value_name.is_some() {
        line.push_str(" -r -f");
    } else {
        line.push_str(" -f");
    }

    if let Some(path) = path {
        line.push_str(&format!(" -n '{}'", fish_exact_path_condition(path)));
    }

    if !option.values.is_empty() {
        line.push_str(&format!(" -a '{}'", words(option.values.iter().copied())));
    }

    line.push_str(&format!(" -d '{}'", fish_escape(option.about)));
    line
}

fn collect_options(spec: &CommandSpec) -> Vec<&OptionSpec> {
    let mut options = Vec::new();
    collect_options_recursive(spec, &mut options);
    options.sort_by_key(|option| (option.long, option.short));
    options.dedup_by_key(|option| (option.long, option.short));
    options
}

fn collect_options_recursive<'a>(command: &'a CommandSpec, options: &mut Vec<&'a OptionSpec>) {
    options.extend(command.options.iter());

    for subcommand in &command.subcommands {
        collect_options_recursive(subcommand, options);
    }
}

fn command_paths_including_non_leaves(spec: &CommandSpec) -> Vec<Vec<&'static str>> {
    let mut paths = Vec::new();
    for command in &spec.subcommands {
        collect_command_paths_including_non_leaves(command, Vec::new(), &mut paths);
    }
    paths
}

fn collect_command_paths_including_non_leaves(
    command: &CommandSpec,
    mut path: Vec<&'static str>,
    paths: &mut Vec<Vec<&'static str>>,
) {
    path.push(command.name);
    paths.push(path.clone());

    for subcommand in &command.subcommands {
        collect_command_paths_including_non_leaves(subcommand, path.clone(), paths);
    }
}

fn matching_option_paths(spec: &CommandSpec, option: &OptionSpec) -> Vec<Vec<&'static str>> {
    let mut paths = vec![Vec::new()];
    paths.extend(command_paths_including_non_leaves(spec));
    paths
        .into_iter()
        .filter(|path| {
            spec.options_for_path(path)
                .iter()
                .any(|candidate| same_option(candidate, option))
        })
        .collect()
}

fn same_option(left: &OptionSpec, right: &OptionSpec) -> bool {
    left.long == right.long && left.short == right.short
}

fn command_words(spec: &CommandSpec) -> Vec<&'static str> {
    let mut names = Vec::new();
    collect_command_words(spec, &mut names);
    names.sort_unstable();
    names.dedup();
    names
}

fn collect_command_words(command: &CommandSpec, names: &mut Vec<&'static str>) {
    for subcommand in &command.subcommands {
        names.push(subcommand.name);
        collect_command_words(subcommand, names);
    }
}

fn command_names(commands: &[CommandSpec]) -> Vec<&str> {
    commands.iter().map(|command| command.name).collect()
}

fn option_name(option: &OptionSpec) -> String {
    match (option.long, option.short) {
        (Some(long), _) => format!("--{long}"),
        (None, Some(short)) => format!("-{short}"),
        (None, None) => String::new(),
    }
}

fn fish_command_condition(parents: &[&str], siblings: &[&str]) -> String {
    if parents.is_empty() {
        "__fish_use_subcommand".to_string()
    } else {
        let mut checks = parents
            .iter()
            .map(|parent| format!("__fish_seen_subcommand_from {}", fish_escape(parent)))
            .collect::<Vec<_>>();

        checks.push(format!(
            "not __fish_seen_subcommand_from {}",
            siblings
                .iter()
                .map(|sibling| fish_escape(sibling))
                .collect::<Vec<_>>()
                .join(" ")
        ));

        checks.join("; and ")
    }
}

fn fish_exact_path_condition(path: &[&str]) -> String {
    let mut checks = path
        .iter()
        .map(|part| format!("__fish_seen_subcommand_from {}", fish_escape(part)))
        .collect::<Vec<_>>();

    let siblings = path_sibling_blockers(path);
    if !siblings.is_empty() {
        checks.push(format!(
            "not __fish_seen_subcommand_from {}",
            siblings
                .into_iter()
                .map(fish_escape)
                .collect::<Vec<_>>()
                .join(" ")
        ));
    }

    checks.join("; and ")
}

fn path_sibling_blockers(path: &[&str]) -> Vec<&'static str> {
    match path {
        ["ticket"] => vec![
            "search",
            "list",
            "get",
            "create",
            "update",
            "comment",
            "comments",
            "tag",
            "untag",
            "link",
            "links",
            "attach",
            "attachments",
            "log",
            "worklog",
            "set",
            "fields",
            "history",
            "sprints",
            "delete",
        ],
        ["article"] => vec![
            "search",
            "list",
            "get",
            "create",
            "update",
            "move",
            "append",
            "comment",
            "comments",
            "attach",
            "attachments",
            "delete",
        ],
        ["board"] => vec!["list", "get", "create", "update", "delete"],
        ["sprint"] => vec![
            "list", "current", "get", "create", "update", "delete", "ticket",
        ],
        ["sprint", "ticket"] => vec!["list", "add", "remove"],
        ["completion"] => vec!["bash", "zsh", "fish"],
        _ => vec![],
    }
}

fn words<I>(values: I) -> String
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    values
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

fn pattern_words<I>(values: I) -> String
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    values
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect::<Vec<_>>()
        .join("|")
}

fn zsh_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "'\\''")
}

fn fish_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(shell: &str) -> String {
        render_completion(shell, &crate::cli_spec::cli_spec()).unwrap()
    }

    #[test]
    fn bash_output_contains_expected_markers() {
        let output = render("bash");

        assert!(!output.is_empty());
        assert!(output.contains("_ytd"));
        assert!(output.contains("complete -F _ytd ytd"));
        assert!(output.contains("ticket"));
        assert!(output.contains("article"));
        assert!(output.contains("--format"));
        assert!(output.contains("--project"));
        assert!(output.contains("json"));
    }

    #[test]
    fn zsh_output_contains_expected_markers() {
        let output = render("zsh");

        assert!(!output.is_empty());
        assert!(output.contains("#compdef ytd"));
        assert!(output.contains("_ytd"));
        assert!(output.contains("ticket"));
        assert!(output.contains("article"));
        assert!(output.contains("--format"));
        assert!(output.contains("json"));
    }

    #[test]
    fn fish_output_contains_expected_markers() {
        let output = render("fish");

        assert!(!output.is_empty());
        assert!(output.contains("complete -c ytd"));
        assert!(output.contains("ticket"));
        assert!(output.contains("article"));
        assert!(output.contains("-l format"));
        assert!(output.contains("json"));
    }

    #[test]
    fn fixed_values_appear_in_generated_output() {
        for shell in crate::cli_spec::COMPLETION_SHELLS {
            let output = render(shell);

            for value in crate::cli_spec::FORMAT_VALUES
                .iter()
                .chain(crate::cli_spec::SKILL_SCOPE_VALUES)
                .chain(crate::cli_spec::BOARD_TEMPLATE_VALUES)
                .chain(crate::cli_spec::COMPLETION_SHELLS)
            {
                assert!(
                    output.contains(value),
                    "{shell} completion output is missing {value}"
                );
            }
        }
    }

    #[test]
    fn unsupported_shell_returns_error() {
        let error = render_completion("powershell", &crate::cli_spec::cli_spec()).unwrap_err();

        assert!(error.to_string().contains("unsupported completion shell"));
        assert!(error.to_string().contains("powershell"));
    }
}
