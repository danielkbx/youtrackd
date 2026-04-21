use crate::error::YtdError;
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, IsTerminal, Read};

/// Read JSON input from --json flag or stdin. Stdin takes precedence.
pub fn read_json_input(flags: &HashMap<String, String>) -> Result<Value, YtdError> {
    // Check stdin first (if not a TTY)
    if !io::stdin().is_terminal() {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        let buf = buf.trim();
        if !buf.is_empty() {
            return serde_json::from_str(buf).map_err(YtdError::from);
        }
    }

    // Then --json flag
    if let Some(json_str) = flags.get("json") {
        return serde_json::from_str(json_str).map_err(YtdError::from);
    }

    Err(YtdError::Input("No JSON input provided. Use --json '{...}' or pipe via stdin.".into()))
}
