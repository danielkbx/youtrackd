use crate::config;
use crate::error::YtdError;

pub fn run() -> Result<(), YtdError> {
    config::clear_config()?;
    println!("Logged out.");
    Ok(())
}
