use crate::args::ParsedArgs;
use crate::error::YtdError;
use crate::types::YtdConfig;
use std::process::Command;

use super::open_target;

pub fn run(config: &YtdConfig, args: &ParsedArgs) -> Result<(), YtdError> {
    let target = args
        .positional
        .first()
        .ok_or_else(|| YtdError::Input("Usage: ytd open <target>".into()))?;
    let parsed = open_target::parse_target(target)?;
    let url = open_target::build_url(config, &parsed);
    open_in_browser(&url)?;
    println!("{url}");
    Ok(())
}

fn open_in_browser(url: &str) -> Result<(), YtdError> {
    let (program, args) = launcher_for_platform(std::env::consts::OS, url)?;
    let status = Command::new(program).args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(YtdError::Input(format!(
            "Browser command exited with status {status}"
        )))
    }
}

fn launcher_for_platform<'a>(
    os: &str,
    url: &'a str,
) -> Result<(&'static str, Vec<&'a str>), YtdError> {
    match os {
        "macos" => Ok(("open", vec![url])),
        "linux" => Ok(("xdg-open", vec![url])),
        "windows" => Ok(("cmd", vec!["/C", "start", "", url])),
        other => Err(YtdError::Input(format!(
            "Opening a browser is not supported on {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::launcher_for_platform;

    #[test]
    fn builds_macos_launcher() {
        let (program, args) = launcher_for_platform("macos", "https://example.com").unwrap();
        assert_eq!(program, "open");
        assert_eq!(args, vec!["https://example.com"]);
    }

    #[test]
    fn builds_linux_launcher() {
        let (program, args) = launcher_for_platform("linux", "https://example.com").unwrap();
        assert_eq!(program, "xdg-open");
        assert_eq!(args, vec!["https://example.com"]);
    }

    #[test]
    fn builds_windows_launcher() {
        let (program, args) = launcher_for_platform("windows", "https://example.com").unwrap();
        assert_eq!(program, "cmd");
        assert_eq!(args, vec!["/C", "start", "", "https://example.com"]);
    }

    #[test]
    fn rejects_unknown_platform() {
        let err = launcher_for_platform("plan9", "https://example.com").unwrap_err();
        assert_eq!(
            err.to_string(),
            "Opening a browser is not supported on plan9"
        );
    }
}
