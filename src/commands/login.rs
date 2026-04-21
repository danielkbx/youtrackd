use crate::args::ParsedArgs;
use crate::client::{UreqTransport, YtClient};
use crate::config;
use crate::error::YtdError;
use crate::types::YtdConfig;
use std::io::{self, BufRead, Write};

pub fn run(_args: &ParsedArgs) -> Result<(), YtdError> {
    let url = prompt("YouTrack URL: ")?;
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err(YtdError::Input("URL must start with https:// or http://".into()));
    }

    let token = prompt("Permanent token: ")?;
    if token.is_empty() {
        return Err(YtdError::Input("Token cannot be empty".into()));
    }

    let cfg = YtdConfig { url, token };

    // Validate credentials
    let transport = UreqTransport;
    let client = YtClient::new(cfg.clone(), transport);
    let user = client.get_me().map_err(|_| YtdError::Input("Could not authenticate. Check URL and token.".into()))?;

    config::save_config(&cfg)?;

    println!("Logged in as {} ({})", user.full_name.unwrap_or_default(), user.login);
    Ok(())
}

fn prompt(label: &str) -> Result<String, YtdError> {
    print!("{label}");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim().to_string())
}
