use crate::error::YtdError;
use crate::types::YtdConfig;
use std::fs;
use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg).join("ytd")
    } else if let Some(dir) = dirs::config_dir() {
        dir.join("ytd")
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

    // Config file
    let path = config_path();
    let content = fs::read_to_string(&path).map_err(|_| YtdError::NotLoggedIn)?;
    let config: YtdConfig = serde_json::from_str(&content)?;

    if config.url.is_empty() || config.token.is_empty() {
        return Err(YtdError::NotLoggedIn);
    }

    Ok(config)
}

pub fn save_config(config: &YtdConfig) -> Result<(), YtdError> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;

    let path = config_path();
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
    use std::sync::Mutex;

    // Config tests must run serially because they modify shared env vars
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_env() {
        std::env::remove_var("YTD_CONFIG");
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("YOUTRACK_URL");
        std::env::remove_var("YOUTRACK_TOKEN");
    }

    #[test]
    fn config_path_uses_ytd_config_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        std::env::set_var("YTD_CONFIG", "/tmp/my-ytd.json");
        let p = config_path();
        assert_eq!(p, PathBuf::from("/tmp/my-ytd.json"));
        clear_env();
    }

    #[test]
    fn config_path_uses_xdg() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());
        let p = config_path();
        assert!(p.starts_with(tmp.path()));
        assert!(p.ends_with("ytd/config.json"));
        clear_env();
    }

    #[test]
    fn get_config_from_env() {
        let _lock = ENV_LOCK.lock().unwrap();
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
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());
        let result = get_config();
        assert!(result.is_err());
        clear_env();
    }

    #[test]
    fn save_and_read_config() {
        let _lock = ENV_LOCK.lock().unwrap();
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
        let _lock = ENV_LOCK.lock().unwrap();
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
    fn clear_config_removes_file() {
        let _lock = ENV_LOCK.lock().unwrap();
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
