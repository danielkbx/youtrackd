use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::error::YtdError;
use crate::format::{self, OutputOptions};

pub fn run<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs, opts: &OutputOptions) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("list") => {
            let mut agiles = client.list_agiles()?;

            // Client-side project filter
            if let Some(project) = args.flags.get("project") {
                agiles.retain(|a| {
                    a.projects.iter().any(|p| {
                        p.short_name.as_deref().map(|s| s.eq_ignore_ascii_case(project)).unwrap_or(false)
                            || p.id == *project
                    })
                });
            }

            format::print_items(&agiles, opts);
            Ok(())
        }
        Some("get") => {
            let id = args.positional.first()
                .ok_or_else(|| YtdError::Input("Usage: ytd board get <id>".into()))?;
            let agile = client.get_agile(id)?;
            format::print_single(&agile, opts);
            Ok(())
        }
        _ => Err(YtdError::Input("Usage: ytd board <list|get>".into())),
    }
}
