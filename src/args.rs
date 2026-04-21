use std::collections::HashMap;

#[derive(Debug)]
pub struct ParsedArgs {
    pub resource: Option<String>,
    pub action: Option<String>,
    pub positional: Vec<String>,
    pub flags: HashMap<String, String>,
}

pub fn parse_args(argv: &[String]) -> ParsedArgs {
    let mut positional = Vec::new();
    let mut flags = HashMap::new();
    let mut i = 0;

    while i < argv.len() {
        let arg = &argv[i];
        if let Some(key) = arg.strip_prefix("--") {
            if let Some((k, v)) = key.split_once('=') {
                flags.insert(k.to_string(), v.to_string());
            } else if i + 1 < argv.len() && !argv[i + 1].starts_with("--") {
                flags.insert(key.to_string(), argv[i + 1].clone());
                i += 1;
            } else {
                flags.insert(key.to_string(), "true".to_string());
            }
        } else if arg == "-y" {
            flags.insert("y".to_string(), "true".to_string());
        } else if arg == "-v" {
            flags.insert("verbose".to_string(), "true".to_string());
        } else {
            positional.push(arg.clone());
        }
        i += 1;
    }

    let resource = positional.first().cloned();
    let action = positional.get(1).cloned();
    let rest = if positional.len() > 2 {
        positional[2..].to_vec()
    } else {
        vec![]
    };

    ParsedArgs {
        resource,
        action,
        positional: rest,
        flags,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(input: &[&str]) -> ParsedArgs {
        parse_args(&input.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn parse_resource_and_action() {
        let a = args(&["project", "list"]);
        assert_eq!(a.resource.as_deref(), Some("project"));
        assert_eq!(a.action.as_deref(), Some("list"));
        assert!(a.positional.is_empty());
    }

    #[test]
    fn parse_positionals() {
        let a = args(&["article", "get", "PROJ-A-1"]);
        assert_eq!(a.positional, vec!["PROJ-A-1"]);
    }

    #[test]
    fn parse_key_value_flag() {
        let a = args(&["ticket", "create", "--project", "MYPROJ"]);
        assert_eq!(a.flags.get("project").unwrap(), "MYPROJ");
    }

    #[test]
    fn parse_key_equals_value() {
        let a = args(&["ticket", "create", "--project=MYPROJ"]);
        assert_eq!(a.flags.get("project").unwrap(), "MYPROJ");
    }

    #[test]
    fn parse_boolean_flag() {
        let a = args(&["project", "list", "--no-meta"]);
        assert_eq!(a.flags.get("no-meta").unwrap(), "true");
    }

    #[test]
    fn parse_y_flag() {
        let a = args(&["ticket", "delete", "PROJ-42", "-y"]);
        assert_eq!(a.flags.get("y").unwrap(), "true");
        assert_eq!(a.positional, vec!["PROJ-42"]);
    }

    #[test]
    fn parse_empty() {
        let a = args(&[]);
        assert!(a.resource.is_none());
        assert!(a.action.is_none());
    }

    #[test]
    fn parse_help() {
        let a = args(&["help", "ticket"]);
        assert_eq!(a.resource.as_deref(), Some("help"));
        assert_eq!(a.action.as_deref(), Some("ticket"));
    }

    #[test]
    fn flags_between_positionals() {
        let a = args(&["ticket", "--format", "raw", "list", "--project", "PROJ"]);
        assert_eq!(a.resource.as_deref(), Some("ticket"));
        assert_eq!(a.action.as_deref(), Some("list"));
        assert_eq!(a.flags.get("format").unwrap(), "raw");
        assert_eq!(a.flags.get("project").unwrap(), "PROJ");
    }
}
