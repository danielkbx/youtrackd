use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::duration;
use crate::error::YtdError;
use crate::format::{self, OutputOptions};
use crate::input;
use crate::types::*;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

pub fn run<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs, opts: &OutputOptions) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("search") => cmd_search(client, args, opts),
        Some("list") => cmd_list(client, args, opts),
        Some("get") => cmd_get(client, args, opts),
        Some("create") => cmd_create(client, args),
        Some("update") => cmd_update(client, args),
        Some("comment") => cmd_comment(client, args),
        Some("tag") => cmd_tag(client, args),
        Some("untag") => cmd_untag(client, args),
        Some("link") => cmd_link(client, args),
        Some("links") => cmd_links(client, args, opts),
        Some("attach") => cmd_attach(client, args),
        Some("attachments") => cmd_attachments(client, args, opts),
        Some("log") => cmd_log(client, args),
        Some("worklog") => cmd_worklog(client, args, opts),
        Some("set") => cmd_set(client, args),
        Some("fields") => cmd_fields(client, args, opts),
        Some("history") => cmd_history(client, args, opts),
        Some("delete") => cmd_delete(client, args),
        _ => Err(YtdError::Input("Usage: ytd ticket <search|list|get|create|update|comment|tag|untag|link|links|attach|attachments|log|worklog|set|fields|history|delete>".into())),
    }
}

fn require_id(args: &ParsedArgs) -> Result<&str, YtdError> {
    args.positional.first().map(|s| s.as_str())
        .ok_or_else(|| YtdError::Input("Ticket ID is required".into()))
}

fn cmd_search<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs, opts: &OutputOptions) -> Result<(), YtdError> {
    let query = args.positional.first()
        .ok_or_else(|| YtdError::Input("Usage: ytd ticket search <query>".into()))?;
    let project = args.flags.get("project").map(|s| s.as_str());
    let issues = client.search_issues(query, project)?;
    format::print_items(&issues, opts);
    Ok(())
}

fn cmd_list<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs, opts: &OutputOptions) -> Result<(), YtdError> {
    let project = args.flags.get("project")
        .ok_or_else(|| YtdError::Input("--project is required".into()))?;
    let issues = client.list_issues(project)?;
    format::print_items(&issues, opts);
    Ok(())
}

fn cmd_get<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs, opts: &OutputOptions) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let issue = client.get_issue(id)?;
    format::print_single(&issue, opts);
    Ok(())
}

fn cmd_create<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let project = args.flags.get("project")
        .ok_or_else(|| YtdError::Input("--project is required".into()))?;
    let json = input::read_json_input(&args.flags)?;
    let summary = json.get("summary").and_then(|v| v.as_str())
        .ok_or_else(|| YtdError::Input("summary is required".into()))?;
    let description = json.get("description").and_then(|v| v.as_str());
    let input = CreateIssueInput {
        project: ProjectRef { id: String::new(), short_name: Some(project.clone()), name: None },
        summary: summary.to_string(),
        description: description.map(String::from),
    };
    let issue = client.create_issue(&input)?;
    println!("{}", issue.id_readable.unwrap_or(issue.id));
    Ok(())
}

fn cmd_update<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let json = input::read_json_input(&args.flags)?;
    let input = UpdateIssueInput {
        summary: json.get("summary").and_then(|v| v.as_str()).map(String::from),
        description: json.get("description").and_then(|v| v.as_str()).map(String::from),
    };
    let issue = client.update_issue(id, &input)?;
    println!("{}", issue.id_readable.unwrap_or(issue.id));
    Ok(())
}

fn cmd_comment<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let text = args.positional.get(1..)
        .map(|s| s.join(" "))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| YtdError::Input("Comment text is required".into()))?;
    client.add_comment(id, &text)?;
    Ok(())
}

fn cmd_tag<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let tag_name = args.positional.get(1)
        .ok_or_else(|| YtdError::Input("Tag name is required".into()))?;

    // Find the tag by name to get its ID
    let tags = client.list_tags()?;
    let tag = tags.iter().find(|t| t.name.eq_ignore_ascii_case(tag_name))
        .ok_or_else(|| YtdError::Input(format!("Tag not found: {tag_name}")))?;

    client.add_issue_tag(id, tag)?;
    Ok(())
}

fn cmd_untag<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let tag_name = args.positional.get(1)
        .ok_or_else(|| YtdError::Input("Tag name is required".into()))?;

    // Get issue to find the tag ID
    let issue = client.get_issue(id)?;
    let tag = issue.tags.iter().find(|t| t.name.eq_ignore_ascii_case(tag_name))
        .ok_or_else(|| YtdError::Input(format!("Tag not found on issue: {tag_name}")))?;

    let tag_id = tag.id.as_ref()
        .ok_or_else(|| YtdError::Input("Tag has no ID".into()))?;
    client.remove_issue_tag(id, tag_id)?;
    Ok(())
}

fn cmd_link<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let target = args.positional.get(1)
        .ok_or_else(|| YtdError::Input("Target ticket ID is required".into()))?;
    let link_type = args.flags.get("type").map(|s| s.as_str()).unwrap_or("relates to");

    let command = format!("{link_type} {target}");
    client.apply_command(id, &command)?;
    Ok(())
}

fn cmd_links<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs, opts: &OutputOptions) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let links = client.list_issue_links(id)?;
    if matches!(opts.format, format::Format::Text) {
        print!("{}", render_issue_links_text(&links));
    } else {
        format::print_items(&links, opts);
    }
    Ok(())
}

fn render_issue_links_text(links: &[IssueLink]) -> String {
    let populated: Vec<&IssueLink> = links.iter()
        .filter(|link| link.issues.as_ref().map(|issues| !issues.is_empty()).unwrap_or(false))
        .collect();

    if populated.is_empty() {
        return "No linked tickets.\n".to_string();
    }

    let mut out = String::new();

    for (idx, link) in populated.iter().enumerate() {
        if idx > 0 {
            out.push('\n');
        }

        out.push_str("linkType: ");
        out.push_str(link.link_type.as_ref().and_then(|lt| lt.name.as_deref()).unwrap_or("Unknown"));
        out.push('\n');
        out.push_str("direction: ");
        out.push_str(link.direction.as_deref().unwrap_or("Unknown"));
        out.push('\n');
        out.push_str("issues:\n");

        if let Some(issues) = &link.issues {
            for issue in issues {
                let identifier = issue.id_readable.as_deref().unwrap_or(&issue.id);
                match issue.summary.as_deref() {
                    Some(summary) if !summary.is_empty() => {
                        out.push_str("- ");
                        out.push_str(identifier);
                        out.push_str(": ");
                        out.push_str(summary);
                        out.push('\n');
                    }
                    _ => {
                        out.push_str("- ");
                        out.push_str(identifier);
                        out.push('\n');
                    }
                }
            }
        }
    }

    out
}

fn cmd_attach<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let file = args.positional.get(1)
        .ok_or_else(|| YtdError::Input("File path is required".into()))?;
    let path = Path::new(file);
    if !path.exists() {
        return Err(YtdError::Input(format!("File not found: {file}")));
    }
    client.upload_attachment(id, path)?;
    println!("Attached {}", path.file_name().and_then(|n| n.to_str()).unwrap_or(file));
    Ok(())
}

fn cmd_attachments<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs, opts: &OutputOptions) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let attachments = client.list_attachments(id)?;
    format::print_items(&attachments, opts);
    Ok(())
}

fn cmd_log<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let dur_str = args.positional.get(1)
        .ok_or_else(|| YtdError::Input("Duration is required (e.g. 30m, 1h, 2h30m)".into()))?;
    let minutes = duration::parse_duration(dur_str)?;
    let text = args.positional.get(2..).map(|s| s.join(" ")).filter(|s| !s.is_empty());

    let date = args.flags.get("date").and_then(|d| {
        // Parse YYYY-MM-DD to epoch ms
        parse_date_to_epoch_ms(d)
    });

    let work_type = args.flags.get("type").map(|t| WorkType {
        id: None,
        name: Some(t.clone()),
    });

    let input = CreateWorkItemInput {
        duration: WorkItemDuration { minutes: Some(minutes), presentation: None },
        text,
        date,
        work_type,
    };
    client.add_work_item(id, &input)?;
    Ok(())
}

fn cmd_worklog<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs, opts: &OutputOptions) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let items = client.list_work_items(id)?;
    format::print_items(&items, opts);
    Ok(())
}

fn cmd_set<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let field_name = args.positional.get(1)
        .ok_or_else(|| YtdError::Input("Field name is required".into()))?;
    let value_str = args.positional.get(2..)
        .map(|s| s.join(" "))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| YtdError::Input("Value is required".into()))?;

    // Get issue to read existing custom field $types
    let issue = client.get_issue(id)?;

    // Find the $type from the issue's existing custom fields
    let cf_type = issue.custom_fields.iter()
        .find(|f| f.name.as_deref().map(|n| n.eq_ignore_ascii_case(field_name)).unwrap_or(false))
        .and_then(|f| f.field_type.as_deref())
        .unwrap_or("");

    let value = build_field_value(cf_type, &value_str);

    let body = serde_json::json!({
        "customFields": [{
            "$type": cf_type,
            "name": field_name,
            "value": value
        }]
    });
    client.set_custom_field(id, &body)?;
    Ok(())
}

fn build_field_value(cf_type: &str, value: &str) -> serde_json::Value {
    // cf_type is the $type of the issue custom field, e.g. "SingleEnumIssueCustomField"
    if cf_type.contains("User") || cf_type.contains("Owned") {
        serde_json::json!({"login": value})
    } else if cf_type.contains("Period") {
        if let Ok(mins) = duration::parse_duration(value) {
            serde_json::json!({"minutes": mins})
        } else {
            serde_json::json!(value)
        }
    } else if cf_type.contains("State") {
        serde_json::json!({"$type": "StateBundleElement", "name": value})
    } else if cf_type.contains("Version") || cf_type.contains("Build") {
        serde_json::json!({"$type": "VersionBundleElement", "name": value})
    } else if cf_type.contains("Enum") {
        serde_json::json!({"$type": "EnumBundleElement", "name": value})
    } else if let Ok(n) = value.parse::<i64>() {
        serde_json::json!(n)
    } else if let Ok(f) = value.parse::<f64>() {
        serde_json::json!(f)
    } else {
        serde_json::json!(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_issue(id: &str, id_readable: Option<&str>, summary: Option<&str>) -> Issue {
        Issue {
            id: id.to_string(),
            id_readable: id_readable.map(String::from),
            summary: summary.map(String::from),
            description: None,
            created: None,
            updated: None,
            resolved: None,
            reporter: None,
            project: None,
            tags: vec![],
            comments: vec![],
            custom_fields: vec![],
        }
    }

    #[test]
    fn render_issue_links_text_with_populated_issues() {
        let links = vec![
            IssueLink {
                id: Some("105-0".into()),
                direction: Some("BOTH".into()),
                link_type: Some(IssueLinkType {
                    id: Some("105-0".into()),
                    name: Some("Relates".into()),
                    source_to_target: Some("relates to".into()),
                    target_to_source: Some(String::new()),
                }),
                issues: Some(vec![sample_issue("2-14252", Some("DWP-14"), Some("Link Target"))]),
            }
        ];

        let rendered = render_issue_links_text(&links);
        assert!(rendered.contains("linkType: Relates"));
        assert!(rendered.contains("direction: BOTH"));
        assert!(rendered.contains("- DWP-14: Link Target"));
    }

    #[test]
    fn render_issue_links_text_handles_missing_summaries() {
        let links = vec![
            IssueLink {
                id: Some("105-0".into()),
                direction: Some("BOTH".into()),
                link_type: Some(IssueLinkType {
                    id: Some("105-0".into()),
                    name: Some("Relates".into()),
                    source_to_target: None,
                    target_to_source: None,
                }),
                issues: Some(vec![sample_issue("2-14252", Some("DWP-14"), None)]),
            }
        ];

        let rendered = render_issue_links_text(&links);
        assert!(rendered.contains("- DWP-14\n"));
    }

    #[test]
    fn render_issue_links_text_with_no_populated_links() {
        let links = vec![
            IssueLink {
                id: Some("105-0".into()),
                direction: Some("BOTH".into()),
                link_type: Some(IssueLinkType {
                    id: Some("105-0".into()),
                    name: Some("Relates".into()),
                    source_to_target: None,
                    target_to_source: None,
                }),
                issues: Some(vec![]),
            }
        ];

        let rendered = render_issue_links_text(&links);
        assert_eq!(rendered, "No linked tickets.\n");
    }
}

fn cmd_fields<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs, opts: &OutputOptions) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let issue = client.get_issue(id)?;
    format::print_items(&issue.custom_fields, opts);
    Ok(())
}

fn cmd_history<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs, opts: &OutputOptions) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let category = args.flags.get("category").map(|s| s.as_str());
    let activities = client.list_activities(id, category)?;
    format::print_items(&activities, opts);
    Ok(())
}

fn cmd_delete<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    if args.flags.get("y").map(|v| v == "true").unwrap_or(false) || confirm_delete(id)? {
        client.delete_issue(id)?;
        println!("{id}");
    }
    Ok(())
}

fn confirm_delete(id: &str) -> Result<bool, YtdError> {
    if !io::stdin().is_terminal() {
        return Ok(true);
    }
    print!("Delete ticket {id}? [y/N] ");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim().eq_ignore_ascii_case("y"))
}

fn parse_date_to_epoch_ms(date: &str) -> Option<u64> {
    // Parse YYYY-MM-DD
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 { return None; }
    let y: i64 = parts[0].parse().ok()?;
    let m: i64 = parts[1].parse().ok()?;
    let d: i64 = parts[2].parse().ok()?;

    // Days from epoch using civil date algorithm
    let m2 = if m <= 2 { m + 9 } else { m - 3 };
    let y2 = if m <= 2 { y - 1 } else { y };
    let era = if y2 >= 0 { y2 } else { y2 - 399 } / 400;
    let yoe = y2 - era * 400;
    let doy = (153 * m2 + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;

    Some((days as u64) * 86400 * 1000)
}
