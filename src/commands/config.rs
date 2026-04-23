use crate::args::ParsedArgs;
use crate::config;
use crate::error::YtdError;

pub fn run(args: &ParsedArgs) -> Result<(), YtdError> {
    if let Some(output) = execute(args)? {
        println!("{output}");
    }
    Ok(())
}

fn execute(args: &ParsedArgs) -> Result<Option<String>, YtdError> {
    match args.action.as_deref() {
        Some("set") => set_config(args),
        Some("get") => get_config_value(args),
        Some("unset") => unset_config(args),
        _ => Err(YtdError::Input(
            "Usage: ytd config <set|get|unset> visibility-group [value]".into(),
        )),
    }
}

fn set_config(args: &ParsedArgs) -> Result<Option<String>, YtdError> {
    let key = config_key(args, "Usage: ytd config set visibility-group <value>")?;
    let value = args
        .positional
        .get(1)
        .ok_or_else(|| YtdError::Input("Usage: ytd config set visibility-group <value>".into()))?;

    if value.is_empty() {
        return Err(YtdError::Input("visibility-group cannot be empty".into()));
    }

    let mut stored = config::load_stored_config()?;
    match key {
        ConfigKey::VisibilityGroup => stored.visibility_group = Some(value.clone()),
    }
    config::save_stored_config(&stored)?;

    Ok(Some(value.clone()))
}

fn get_config_value(args: &ParsedArgs) -> Result<Option<String>, YtdError> {
    let key = config_key(args, "Usage: ytd config get visibility-group")?;
    let stored = config::load_stored_config()?;

    match key {
        ConfigKey::VisibilityGroup => stored
            .visibility_group
            .map(Some)
            .ok_or_else(|| YtdError::Input("visibility-group is not set".into())),
    }
}

fn unset_config(args: &ParsedArgs) -> Result<Option<String>, YtdError> {
    let key = config_key(args, "Usage: ytd config unset visibility-group")?;
    let mut stored = config::load_stored_config()?;

    match key {
        ConfigKey::VisibilityGroup => stored.visibility_group = None,
    }

    config::save_stored_config(&stored)?;
    Ok(None)
}

#[derive(Copy, Clone)]
enum ConfigKey {
    VisibilityGroup,
}

fn config_key(args: &ParsedArgs, usage: &str) -> Result<ConfigKey, YtdError> {
    let key = args
        .positional
        .first()
        .ok_or_else(|| YtdError::Input(usage.into()))?;

    match key.as_str() {
        "visibility-group" => Ok(ConfigKey::VisibilityGroup),
        _ => Err(YtdError::Input(format!("Unknown config key: {key}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::StoredConfig;

    fn clear_env() {
        std::env::remove_var("YTD_CONFIG");
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("YOUTRACK_URL");
        std::env::remove_var("YOUTRACK_TOKEN");
    }

    fn parsed(action: &str, positional: &[&str]) -> ParsedArgs {
        ParsedArgs {
            resource: Some("config".into()),
            action: Some(action.into()),
            positional: positional.iter().map(|value| value.to_string()).collect(),
            flags: Default::default(),
        }
    }

    #[test]
    fn set_get_and_unset_visibility_group() {
        let _lock = crate::config::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        clear_env();
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());

        let set_output = execute(&parsed("set", &["visibility-group", "Developers"])).unwrap();
        assert_eq!(set_output.as_deref(), Some("Developers"));

        let stored = config::load_stored_config().unwrap();
        assert_eq!(stored.visibility_group.as_deref(), Some("Developers"));

        let get_output = execute(&parsed("get", &["visibility-group"])).unwrap();
        assert_eq!(get_output.as_deref(), Some("Developers"));

        let unset_output = execute(&parsed("unset", &["visibility-group"])).unwrap();
        assert_eq!(unset_output, None);

        let stored = config::load_stored_config().unwrap();
        assert_eq!(
            stored,
            StoredConfig {
                visibility_group: None,
                ..StoredConfig::default()
            }
        );

        clear_env();
    }

    #[test]
    fn get_visibility_group_errors_when_unset() {
        let _lock = crate::config::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        clear_env();
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());

        let err = execute(&parsed("get", &["visibility-group"])).unwrap_err();
        assert_eq!(err.to_string(), "visibility-group is not set");

        clear_env();
    }

    #[test]
    fn rejects_unknown_config_key() {
        let err = execute(&parsed("get", &["token"])).unwrap_err();
        assert_eq!(err.to_string(), "Unknown config key: token");
    }

    #[test]
    fn rejects_missing_value_for_set() {
        let err = execute(&parsed("set", &["visibility-group"])).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Usage: ytd config set visibility-group <value>"
        );
    }
}
