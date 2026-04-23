use crate::args::ParsedArgs;
use crate::error::YtdError;
use crate::types::YtdConfig;

use super::open_target;

pub fn run(config: &YtdConfig, args: &ParsedArgs) -> Result<(), YtdError> {
    let target = args
        .positional
        .first()
        .ok_or_else(|| YtdError::Input("Usage: ytd url <target>".into()))?;
    let parsed = open_target::parse_target(target)?;
    println!("{}", open_target::build_url(config, &parsed));
    Ok(())
}
