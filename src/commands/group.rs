use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::error::YtdError;
use crate::format::{self, OutputOptions};

pub fn run<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("list") => {
            let groups = client.list_groups()?;
            format::print_items(&groups, opts);
            Ok(())
        }
        _ => Err(YtdError::Input("Usage: ytd group list".into())),
    }
}
