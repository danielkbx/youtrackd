use crate::error::YtdError;
use crate::types::{ResolvedVisibilityGroup, StoredConfig, YtdConfig};
use std::fs;
use std::path::PathBuf;

#[cfg(test)]
pub static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn config_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg).join("ytd")
    } else {
        PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".config")
            .join("ytd")
    }
}

pub fn config_path() -> PathBuf {
    if let Ok(path) = std::env::var("YTD_CONFIG") {
        return PathBuf::from(path);
    }
    config_dir().join("config.json")
}

pub fn get_config() -> Result<YtdConfig, YtdError> {
    // Env vars take precedence
    if let (Ok(url), Ok(token)) = (
        std::env::var("YOUTRACK_URL"),
        std::env::var("YOUTRACK_TOKEN"),
    ) {
        if !url.is_empty() && !token.is_empty() {
            return Ok(YtdConfig { url, token });
        }
    }

    let stored = load_stored_config()?;
    match (stored.url, stored.token) {
        (Some(url), Some(token)) if !url.is_empty() && !token.is_empty() => {
            Ok(YtdConfig { url, token })
        }
        _ => Err(YtdError::NotLoggedIn),
    }
}

pub fn save_config(config: &YtdConfig) -> Result<(), YtdError> {
    let mut stored = load_stored_config()?;
    stored.url = Some(config.url.clone());
    stored.token = Some(config.token.clone());
    save_stored_config(&stored)
}

pub fn resolve_visibility_group(
    cli_group: Option<&str>,
    no_visibility_group: bool,
) -> Result<ResolvedVisibilityGroup, YtdError> {
    if cli_group.is_some() && no_visibility_group {
        return Err(YtdError::Input(
            "--visibility-group cannot be combined with --no-visibility-group".into(),
        ));
    }

    if no_visibility_group {
        return Ok(ResolvedVisibilityGroup::Clear);
    }

    if let Some(group) = cli_group {
        let trimmed = group.trim();
        if trimmed.is_empty() {
            return Err(YtdError::Input("visibility-group cannot be empty".into()));
        }
        return Ok(ResolvedVisibilityGroup::Group(trimmed.to_string()));
    }

    if let Ok(group) = std::env::var("YTD_VISIBILITY_GROUP") {
        let trimmed = group.trim();
        if !trimmed.is_empty() {
            return Ok(ResolvedVisibilityGroup::Group(trimmed.to_string()));
        }
    }

    let stored = load_stored_config()?;
    if let Some(group) = stored.visibility_group.as_deref() {
        let trimmed = group.trim();
        if !trimmed.is_empty() {
            return Ok(ResolvedVisibilityGroup::Group(trimmed.to_string()));
        }
    }

    Ok(ResolvedVisibilityGroup::None)
}

pub fn load_stored_config() -> Result<StoredConfig, YtdError> {
    let path = config_path();
    match fs::read_to_string(&path) {
        Ok(content) => Ok(serde_json::from_str(&content)?),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(StoredConfig::default()),
        Err(err) => Err(err.into()),
    }
}

pub fn save_stored_config(config: &StoredConfig) -> Result<(), YtdError> {
    if config.is_empty() {
        return clear_config();
    }

    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(config)?;

    // Write with mode 600 atomically (no race condition)
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)?;
        file.write_all(json.as_bytes())?;
    }

    #[cfg(not(unix))]
    {
        fs::write(&path, &json)?;
    }

    Ok(())
}

pub fn clear_config() -> Result<(), YtdError> {
    let path = config_path();
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clear_env() {
        std::env::remove_var("YTD_CONFIG");
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("YOUTRACK_URL");
        std::env::remove_var("YOUTRACK_TOKEN");
        std::env::remove_var("YTD_VISIBILITY_GROUP");
    }

    #[test]
    fn config_path_uses_ytd_config_env() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        std::env::set_var("YTD_CONFIG", "/tmp/my-ytd.json");
        let p = config_path();
        assert_eq!(p, PathBuf::from("/tmp/my-ytd.json"));
        clear_env();
    }

    #[test]
    fn config_path_uses_xdg() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());
        let p = config_path();
        assert!(p.starts_with(tmp.path()));
        assert!(p.ends_with("ytd/config.json"));
        clear_env();
    }

    #[test]
    fn config_path_defaults_to_home_dot_config() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        let original_home = std::env::var_os("HOME");
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", tmp.path());

        let p = config_path();
        assert_eq!(
            p,
            tmp.path().join(".config").join("ytd").join("config.json")
        );

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
        clear_env();
    }

    #[test]
    fn get_config_from_env() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        std::env::set_var("YOUTRACK_URL", "https://test.youtrack.cloud");
        std::env::set_var("YOUTRACK_TOKEN", "perm:test123");
        let cfg = get_config().unwrap();
        assert_eq!(cfg.url, "https://test.youtrack.cloud");
        assert_eq!(cfg.token, "perm:test123");
        clear_env();
    }

    #[test]
    fn get_config_not_logged_in() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());
        let result = get_config();
        assert!(result.is_err());
        clear_env();
    }

    #[test]
    fn save_and_read_config() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());

        let cfg = YtdConfig {
            url: "https://example.youtrack.cloud".into(),
            token: "perm:abc".into(),
        };
        save_config(&cfg).unwrap();

        let loaded = get_config().unwrap();
        assert_eq!(loaded.url, cfg.url);
        assert_eq!(loaded.token, cfg.token);
        let stored = load_stored_config().unwrap();
        assert_eq!(stored.visibility_group, None);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let meta = fs::metadata(config_path()).unwrap();
            assert_eq!(meta.permissions().mode() & 0o777, 0o600);
        }

        clear_env();
    }

    #[test]
    fn save_and_read_via_ytd_config() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("custom.json");
        std::env::set_var("YTD_CONFIG", &path);

        let cfg = YtdConfig {
            url: "https://company-b.youtrack.cloud".into(),
            token: "perm:companyb".into(),
        };
        save_config(&cfg).unwrap();

        let loaded = get_config().unwrap();
        assert_eq!(loaded.url, "https://company-b.youtrack.cloud");
        assert_eq!(loaded.token, "perm:companyb");

        clear_env();
    }

    #[test]
    fn load_stored_config_returns_default_when_missing() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());

        let stored = load_stored_config().unwrap();
        assert_eq!(stored, StoredConfig::default());

        clear_env();
    }

    #[test]
    fn save_stored_config_supports_visibility_group_without_login() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());

        let cfg = StoredConfig {
            visibility_group: Some("Developers".into()),
            ..StoredConfig::default()
        };
        save_stored_config(&cfg).unwrap();

        let stored = load_stored_config().unwrap();
        assert_eq!(stored.visibility_group.as_deref(), Some("Developers"));
        assert!(get_config().is_err());

        clear_env();
    }

    #[test]
    fn save_config_preserves_existing_visibility_group() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());

        save_stored_config(&StoredConfig {
            visibility_group: Some("Maintainers".into()),
            ..StoredConfig::default()
        })
        .unwrap();

        save_config(&YtdConfig {
            url: "https://example.youtrack.cloud".into(),
            token: "perm:abc".into(),
        })
        .unwrap();

        let stored = load_stored_config().unwrap();
        assert_eq!(
            stored.url.as_deref(),
            Some("https://example.youtrack.cloud")
        );
        assert_eq!(stored.token.as_deref(), Some("perm:abc"));
        assert_eq!(stored.visibility_group.as_deref(), Some("Maintainers"));

        clear_env();
    }

    #[test]
    fn save_stored_config_clears_file_when_empty() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());

        save_stored_config(&StoredConfig {
            visibility_group: Some("Maintainers".into()),
            ..StoredConfig::default()
        })
        .unwrap();
        assert!(config_path().exists());

        save_stored_config(&StoredConfig::default()).unwrap();
        assert!(!config_path().exists());

        clear_env();
    }

    #[test]
    fn resolve_visibility_group_respects_precedence() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());

        save_stored_config(&StoredConfig {
            visibility_group: Some("Config Team".into()),
            ..StoredConfig::default()
        })
        .unwrap();

        assert_eq!(
            resolve_visibility_group(Some("CLI Team"), false).unwrap(),
            ResolvedVisibilityGroup::Group("CLI Team".into())
        );

        std::env::set_var("YTD_VISIBILITY_GROUP", "Env Team");
        assert_eq!(
            resolve_visibility_group(None, false).unwrap(),
            ResolvedVisibilityGroup::Group("Env Team".into())
        );

        std::env::remove_var("YTD_VISIBILITY_GROUP");
        assert_eq!(
            resolve_visibility_group(None, false).unwrap(),
            ResolvedVisibilityGroup::Group("Config Team".into())
        );

        let err = resolve_visibility_group(Some("CLI Team"), true).unwrap_err();
        assert_eq!(
            err.to_string(),
            "--visibility-group cannot be combined with --no-visibility-group"
        );

        clear_env();
    }

    #[test]
    fn resolve_visibility_group_returns_none_when_unset() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());

        assert_eq!(
            resolve_visibility_group(None, false).unwrap(),
            ResolvedVisibilityGroup::None
        );

        clear_env();
    }

    #[test]
    fn resolve_visibility_group_rejects_empty_cli_value() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();

        let err = resolve_visibility_group(Some("   "), false).unwrap_err();
        assert_eq!(err.to_string(), "visibility-group cannot be empty");
    }

    #[test]
    fn clear_config_removes_file() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());

        let cfg = YtdConfig {
            url: "https://x.youtrack.cloud".into(),
            token: "perm:x".into(),
        };
        save_config(&cfg).unwrap();
        let path = config_path();
        assert!(path.exists());

        clear_config().unwrap();
        assert!(!path.exists());

        // Clearing again is a no-op
        clear_config().unwrap();

        clear_env();
    }
}
