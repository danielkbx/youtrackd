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
            let projects = client.list_projects()?;
            format::print_items(&projects, opts);
            Ok(())
        }
        Some("get") => {
            let id = args
                .positional
                .first()
                .ok_or_else(|| YtdError::Input("Usage: ytd project get <shortName>".into()))?;
            let project = client.get_project(id)?;
            format::print_single(&project, opts);
            Ok(())
        }
        _ => Err(YtdError::Input("Usage: ytd project <list|get>".into())),
    }
}
