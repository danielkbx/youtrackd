use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::commands::ticket;
use crate::error::YtdError;
use crate::format::{self, OutputOptions};
use crate::types::SavedQuery;

pub fn run<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("list") => {
            let mut queries = client.list_saved_queries()?;
            if let Some(project) = args.flags.get("project") {
                queries.retain(|query| saved_query_matches_project(query, project));
            }
            format::print_items(&queries, opts);
            Ok(())
        }
        Some("run") => {
            let name_or_id = args
                .positional
                .first()
                .ok_or_else(|| YtdError::Input("Usage: ytd search run <name-or-id>".into()))?;

            let queries = client.list_saved_queries()?;

            // Find by ID first, then by name (case-insensitive)
            let query = queries
                .iter()
                .find(|q| q.id == *name_or_id)
                .or_else(|| {
                    queries.iter().find(|q| {
                        q.name
                            .as_deref()
                            .map(|n| n.eq_ignore_ascii_case(name_or_id))
                            .unwrap_or(false)
                    })
                })
                .ok_or_else(|| YtdError::Input(format!("Saved search not found: {name_or_id}")))?;

            let query_text = query.query.as_deref().unwrap_or("");
            let issues = client.search_issues(query_text, None)?;
            ticket::print_issue_items(&issues, opts);
            Ok(())
        }
        _ => Err(YtdError::Input("Usage: ytd search <list|run>".into())),
    }
}

fn saved_query_matches_project(query: &SavedQuery, project: &str) -> bool {
    let Some(query_text) = query.query.as_deref() else {
        return false;
    };
    let project = project.trim();
    if project.is_empty() {
        return false;
    }

    let query_lower = query_text.to_lowercase();
    let project_lower = project.to_lowercase();
    let compact_query = compact_query_text(&query_lower);
    let compact_project = compact_query_text(&project_lower);

    for prefix in ["project:", "in:"] {
        if compact_query.contains(&format!("{prefix}{compact_project}")) {
            return true;
        }
    }

    query_tokens(&query_lower)
        .iter()
        .any(|token| token == &project_lower)
}

fn compact_query_text(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '{' && *ch != '}')
        .collect()
}

fn query_tokens(value: &str) -> Vec<String> {
    value
        .split(|ch: char| {
            !(ch.is_alphanumeric() || ch == '-' || ch == '_' || ch == '.' || ch == ' ')
        })
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved(query: Option<&str>) -> SavedQuery {
        SavedQuery {
            id: "1".into(),
            name: Some("Saved".into()),
            query: query.map(String::from),
        }
    }

    #[test]
    fn saved_query_project_filter_matches_project_prefix() {
        assert!(saved_query_matches_project(
            &saved(Some("project: DWP State: Open")),
            "DWP"
        ));
        assert!(saved_query_matches_project(
            &saved(Some("PROJECT:{DW Playground} State: Open")),
            "DW Playground"
        ));
    }

    #[test]
    fn saved_query_project_filter_matches_in_prefix() {
        assert!(saved_query_matches_project(
            &saved(Some("in:DWP #Unresolved")),
            "dwp"
        ));
        assert!(saved_query_matches_project(
            &saved(Some("in: {DW Playground}")),
            "DW Playground"
        ));
    }

    #[test]
    fn saved_query_project_filter_excludes_missing_queries() {
        assert!(!saved_query_matches_project(&saved(None), "DWP"));
        assert!(!saved_query_matches_project(
            &saved(Some("State: Open")),
            "DWP"
        ));
    }
}
