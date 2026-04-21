use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::error::YtdError;
use crate::format::{self, OutputOptions};

pub fn run<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs, opts: &OutputOptions) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("list") => {
            let mut tags = client.list_tags()?;

            // Client-side project filter
            if let Some(project) = args.flags.get("project") {
                // Fetch issues for the project to get used tags
                let issues = client.list_issues(project)?;
                let used_tag_names: std::collections::HashSet<String> = issues.iter()
                    .flat_map(|i| i.tags.iter())
                    .map(|t| t.name.to_lowercase())
                    .collect();
                tags.retain(|t| used_tag_names.contains(&t.name.to_lowercase()));
            }

            format::print_items(&tags, opts);
            Ok(())
        }
        _ => Err(YtdError::Input("Usage: ytd tag list [--project <id>]".into())),
    }
}
