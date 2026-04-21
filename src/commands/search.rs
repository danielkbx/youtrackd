use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::error::YtdError;
use crate::format::{self, OutputOptions};

pub fn run<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs, opts: &OutputOptions) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("list") => {
            let queries = client.list_saved_queries()?;
            // Client-side project filter (filter by query content containing project name)
            // Note: saved queries don't have a project field, so this filter checks the query text
            if let Some(_project) = args.flags.get("project") {
                // TODO: Saved queries don't reliably have project info, show all for now
            }
            format::print_items(&queries, opts);
            Ok(())
        }
        Some("run") => {
            let name_or_id = args.positional.first()
                .ok_or_else(|| YtdError::Input("Usage: ytd search run <name-or-id>".into()))?;

            let queries = client.list_saved_queries()?;

            // Find by ID first, then by name (case-insensitive)
            let query = queries.iter()
                .find(|q| q.id == *name_or_id)
                .or_else(|| queries.iter().find(|q| {
                    q.name.as_deref()
                        .map(|n| n.eq_ignore_ascii_case(name_or_id))
                        .unwrap_or(false)
                }))
                .ok_or_else(|| YtdError::Input(format!("Saved search not found: {name_or_id}")))?;

            let query_text = query.query.as_deref().unwrap_or("");
            let issues = client.search_issues(query_text, None)?;
            format::print_items(&issues, opts);
            Ok(())
        }
        _ => Err(YtdError::Input("Usage: ytd search <list|run>".into())),
    }
}
