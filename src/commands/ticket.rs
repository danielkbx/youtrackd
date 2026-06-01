use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::commands;
use crate::commands::visibility;
use crate::duration;
use crate::error::YtdError;
use crate::format::{self, OutputOptions};
use crate::input;
use crate::types::*;
use std::path::Path;

const TICKET_JSON_FIELDS: &[&str] = &["customFields", "description", "summary", "tags"];

pub fn run<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("search") => cmd_search(client, args, opts),
        Some("list") => cmd_list(client, args, opts),
        Some("get") => cmd_get(client, args, opts),
        Some("create") => cmd_create(client, args),
        Some("update") => cmd_update(client, args),
        Some("comment") => cmd_comment(client, args),
        Some("comments") => cmd_comments(client, args, opts),
        Some("tag") => cmd_tag(client, args),
        Some("untag") => cmd_untag(client, args),
        Some("link") => cmd_link(client, args),
        Some("unlink") => cmd_unlink(client, args),
        Some("links") => cmd_links(client, args, opts),
        Some("link-types") => cmd_link_types(client, opts),
        Some("attach") => cmd_attach(client, args),
        Some("attachments") => cmd_attachments(client, args, opts),
        Some("log") => cmd_log(client, args),
        Some("worklog") => cmd_worklog(client, args, opts),
        Some("status") => cmd_status(client, args, opts),
        Some("set") => cmd_set(client, args),
        Some("fields") => cmd_fields(client, args, opts),
        Some("history") => cmd_history(client, args, opts),
        Some("sprints") => cmd_sprints(client, args, opts),
        Some("delete") => cmd_delete(client, args),
        _ => Err(YtdError::Input("Usage: ytd ticket <search|list|get|create|update|comment|comments|tag|untag|link|unlink|links|link-types|attach|attachments|log|worklog|status|set|fields|history|sprints|delete>".into())),
    }
}

fn require_id(args: &ParsedArgs) -> Result<&str, YtdError> {
    args.positional
        .first()
        .map(|s| s.as_str())
        .ok_or_else(|| YtdError::Input("Ticket ID is required".into()))
}

fn cmd_search<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let query = args
        .positional
        .first()
        .ok_or_else(|| YtdError::Input("Usage: ytd ticket search <query>".into()))?;
    let project = args.flags.get("project").map(|s| s.as_str());
    let issues = client.search_issues(query, project)?;
    print_issue_items(&issues, opts);
    Ok(())
}

fn cmd_list<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let project = args
        .flags
        .get("project")
        .ok_or_else(|| YtdError::Input("--project is required".into()))?;
    let issues = client.list_issues(project)?;
    print_issue_items(&issues, opts);
    Ok(())
}

fn cmd_get<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let issue = client.get_issue(id)?;
    let include_comments = !args
        .flags
        .get("no-comments")
        .map(|value| value == "true")
        .unwrap_or(false);
    if matches!(opts.format, format::Format::Text) {
        print!(
            "{}",
            render_issue_detail_text(&issue, id, opts, include_comments)
        );
        return Ok(());
    }

    let mut value = if matches!(opts.format, format::Format::Raw) {
        serde_json::to_value(&issue)?
    } else {
        serde_json::to_value(issue_output(issue))?
    };
    if matches!(opts.format, format::Format::Raw) {
        if !include_comments {
            if let Some(obj) = value.as_object_mut() {
                obj.remove("comments");
            }
        }
    } else if include_comments {
        normalize_comment_array(&mut value, "comments", CommentParentType::Ticket, id);
    } else if let Some(obj) = value.as_object_mut() {
        obj.remove("comments");
    }
    format::print_value(&value, opts);
    Ok(())
}

fn cmd_sprints<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let sprints = client.list_issue_sprints(id)?;
    let outputs = sprints
        .iter()
        .cloned()
        .map(sprint_output_from_agile)
        .collect::<Result<Vec<_>, _>>()?;
    format::print_raw_or_processed_items(&sprints, &outputs, opts)?;
    Ok(())
}

fn cmd_create<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let json = input::read_json_input(&args.flags)?;
    let input = build_create_issue_input(client, args, &json)?;
    let issue = client.create_issue(&input)?;
    println!("{}", issue.id_readable.unwrap_or(issue.id));
    Ok(())
}

fn cmd_update<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let json = input::read_json_input(&args.flags)?;
    let input = build_update_issue_input(client, args, &json)?;
    let issue = client.update_issue(id, &input)?;
    println!("{}", issue.id_readable.unwrap_or(issue.id));
    Ok(())
}

fn build_create_issue_input<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    json: &serde_json::Value,
) -> Result<CreateIssueInput, YtdError> {
    validate_ticket_json_fields(json)?;
    let project = args
        .flags
        .get("project")
        .ok_or_else(|| YtdError::Input("--project is required".into()))?;
    let summary = json
        .get("summary")
        .and_then(|v| v.as_str())
        .ok_or_else(|| YtdError::Input("summary is required".into()))?;

    Ok(CreateIssueInput {
        project: ProjectRef {
            id: String::new(),
            short_name: Some(project.clone()),
            name: None,
        },
        summary: summary.to_string(),
        description: json
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from),
        custom_fields: json.get("customFields").cloned(),
        tags: json.get("tags").cloned(),
        visibility: visibility::build_create_visibility_input(client, args)?,
    })
}

fn build_update_issue_input<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    json: &serde_json::Value,
) -> Result<UpdateIssueInput, YtdError> {
    validate_ticket_json_fields(json)?;
    let summary = json
        .get("summary")
        .and_then(|v| v.as_str())
        .map(String::from);
    let description = json
        .get("description")
        .and_then(|v| v.as_str())
        .map(String::from);
    let custom_fields = json.get("customFields").cloned();
    let tags = json.get("tags").cloned();
    let visibility = visibility::build_explicit_update_visibility_input(client, args)?;

    if summary.is_none()
        && description.is_none()
        && custom_fields.is_none()
        && tags.is_none()
        && visibility.is_none()
    {
        return Err(YtdError::Input(
            "At least one update field is required. Use JSON fields or explicit visibility flags."
                .into(),
        ));
    }

    Ok(UpdateIssueInput {
        summary,
        description,
        custom_fields,
        tags,
        visibility,
    })
}

fn validate_ticket_json_fields(json: &serde_json::Value) -> Result<(), YtdError> {
    let obj = json
        .as_object()
        .ok_or_else(|| YtdError::Input("JSON input must be an object".into()))?;
    let mut unknown: Vec<&str> = obj
        .keys()
        .map(String::as_str)
        .filter(|key| !TICKET_JSON_FIELDS.contains(key))
        .collect();
    unknown.sort_unstable();
    if unknown.is_empty() {
        return Ok(());
    }
    Err(YtdError::Input(format!(
        "Unknown ticket JSON field{}: {}. Allowed fields: {}",
        if unknown.len() == 1 { "" } else { "s" },
        unknown.join(", "),
        TICKET_JSON_FIELDS.join(", ")
    )))
}

fn cmd_comment<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let text = args
        .positional
        .get(1..)
        .map(|s| s.join(" "))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| YtdError::Input("Comment text is required".into()))?;
    let visibility = visibility::build_create_visibility_input(client, args)?;
    client.add_comment(id, &text, visibility)?;
    Ok(())
}

fn cmd_comments<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let comments = client.list_issue_comments(id)?;
    let outputs: Vec<CommentOutput> = comments
        .iter()
        .cloned()
        .map(|comment| issue_comment_output(id, comment))
        .collect();
    format::print_raw_or_processed_items(&comments, &outputs, opts)?;
    Ok(())
}

fn cmd_tag<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let tag_name = args
        .positional
        .get(1)
        .ok_or_else(|| YtdError::Input("Tag name is required".into()))?;

    // Find the tag by name to get its ID
    let tags = client.list_tags()?;
    let tag = tags
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case(tag_name))
        .ok_or_else(|| YtdError::Input(format!("Tag not found: {tag_name}")))?;

    client.add_issue_tag(id, tag)?;
    Ok(())
}

fn cmd_untag<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let tag_name = args
        .positional
        .get(1)
        .ok_or_else(|| YtdError::Input("Tag name is required".into()))?;

    // Get issue to find the tag ID
    let issue = client.get_issue(id)?;
    let tag = issue
        .tags
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case(tag_name))
        .ok_or_else(|| YtdError::Input(format!("Tag not found on issue: {tag_name}")))?;

    let tag_id = tag
        .id
        .as_ref()
        .ok_or_else(|| YtdError::Input("Tag has no ID".into()))?;
    client.remove_issue_tag(id, tag_id)?;
    Ok(())
}

fn cmd_link<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let target = args
        .positional
        .get(1)
        .ok_or_else(|| YtdError::Input("Target ticket ID is required".into()))?;
    let requested_type = args
        .flags
        .get("type")
        .map(|s| s.as_str())
        .unwrap_or("Relates");
    let requested_direction = args.flags.get("direction").map(|s| s.as_str());
    if requested_direction.is_some() && !args.flags.contains_key("type") {
        return Err(YtdError::Input(
            "--direction requires --type for ticket link".into(),
        ));
    }

    let link_types = client.list_issue_link_types()?;
    let resolved = resolve_link_type(&link_types, requested_type, requested_direction)?;
    let target_issue = client.get_issue(target)?;
    client.add_issue_link(id, &resolved.link_id, &target_issue.id)?;
    Ok(())
}

fn cmd_unlink<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let target = args
        .positional
        .get(1)
        .ok_or_else(|| YtdError::Input("Target ticket ID is required".into()))?;
    let requested_type = args
        .flags
        .get("type")
        .map(|s| s.as_str())
        .unwrap_or("Relates");
    let requested_direction = args.flags.get("direction").map(|s| s.as_str());
    if requested_direction.is_some() && !args.flags.contains_key("type") {
        return Err(YtdError::Input(
            "--direction requires --type for ticket unlink".into(),
        ));
    }

    let links = client.list_issue_links(id)?;
    let link_types = client.list_issue_link_types()?;
    let resolved = resolve_link_type(&link_types, requested_type, requested_direction)?;
    let (link_id, target_database_id) = find_issue_link_to_unlink(&links, id, target, &resolved)?;
    client.delete_issue_link(id, link_id, target_database_id)?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinkDirection {
    Inward,
    Outward,
}

impl LinkDirection {
    fn parse(value: &str) -> Result<Self, YtdError> {
        match normalize_link_value(value).as_str() {
            "inward" => Ok(Self::Inward),
            "outward" => Ok(Self::Outward),
            _ => Err(YtdError::Input(format!(
                "Invalid direction: {value}. Expected one of: inward, outward"
            ))),
        }
    }

    fn link_suffix(self) -> &'static str {
        match self {
            Self::Inward => "t",
            Self::Outward => "s",
        }
    }

    fn youtrack_direction(self) -> &'static str {
        match self {
            Self::Inward => "INWARD",
            Self::Outward => "OUTWARD",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Inward => "inward",
            Self::Outward => "outward",
        }
    }
}

#[derive(Debug, Clone)]
struct ResolvedLinkType {
    name: String,
    link_id: String,
    direction: Option<LinkDirection>,
    phrase: String,
}

#[derive(Debug, Clone, Copy)]
enum LinkTypeMatchKind {
    Name,
    SourceToTarget,
    TargetToSource,
}

fn resolve_link_type(
    link_types: &[IssueLinkType],
    requested_type: &str,
    requested_direction: Option<&str>,
) -> Result<ResolvedLinkType, YtdError> {
    let requested = normalize_link_value(requested_type);
    if requested.is_empty() {
        return Err(YtdError::Input("Link type must not be empty".into()));
    }
    let direction = requested_direction.map(LinkDirection::parse).transpose()?;

    let mut matches = Vec::new();
    for link_type in link_types {
        if matches_link_value(link_type.name.as_deref(), &requested) {
            matches.push((link_type, LinkTypeMatchKind::Name));
        }
        if matches_link_value(link_type.source_to_target.as_deref(), &requested) {
            matches.push((link_type, LinkTypeMatchKind::SourceToTarget));
        }
        if matches_link_value(link_type.target_to_source.as_deref(), &requested) {
            matches.push((link_type, LinkTypeMatchKind::TargetToSource));
        }
    }

    if matches.is_empty() {
        return Err(unknown_link_type_error(requested_type, link_types));
    }

    let first_id = matches[0].0.id.as_deref();
    let first_kind = std::mem::discriminant(&matches[0].1);
    let ambiguous = matches.iter().skip(1).any(|(link_type, kind)| {
        link_type.id.as_deref() != first_id || std::mem::discriminant(kind) != first_kind
    });
    if ambiguous {
        return Err(YtdError::Input(format!(
            "Ambiguous link type: {requested_type}. Run `ytd ticket link-types` to list valid link types."
        )));
    }

    resolve_link_type_match(matches[0].0, matches[0].1, direction)
}

fn resolve_link_type_match(
    link_type: &IssueLinkType,
    kind: LinkTypeMatchKind,
    requested_direction: Option<LinkDirection>,
) -> Result<ResolvedLinkType, YtdError> {
    let id = link_type
        .id
        .as_deref()
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(|| YtdError::Input("Issue link type has no ID".into()))?;
    let name = link_type.name.clone().unwrap_or_else(|| id.to_string());
    let directed = link_type.directed.unwrap_or_else(|| {
        link_type
            .target_to_source
            .as_deref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    });

    match kind {
        LinkTypeMatchKind::Name => {
            if !directed {
                if requested_direction.is_some() {
                    return Err(YtdError::Input(format!(
                        "Link type `{name}` is not directed; omit --direction"
                    )));
                }
                let phrase = first_non_empty(&[
                    link_type.source_to_target.as_deref(),
                    link_type.target_to_source.as_deref(),
                    link_type.name.as_deref(),
                ])
                .ok_or_else(|| {
                    YtdError::Input(format!("Issue link type `{name}` has no usable phrase"))
                })?;
                return Ok(ResolvedLinkType {
                    name,
                    link_id: id.to_string(),
                    direction: None,
                    phrase: phrase.to_string(),
                });
            }

            let direction = requested_direction.unwrap_or(LinkDirection::Inward);
            let phrase = phrase_for_direction(link_type, direction, &name)?;
            Ok(ResolvedLinkType {
                name,
                link_id: format!("{id}{}", direction.link_suffix()),
                direction: Some(direction),
                phrase: phrase.to_string(),
            })
        }
        LinkTypeMatchKind::SourceToTarget => resolve_link_phrase_match(
            id,
            name,
            directed,
            LinkDirection::Outward,
            link_type.source_to_target.as_deref(),
            requested_direction,
        ),
        LinkTypeMatchKind::TargetToSource => resolve_link_phrase_match(
            id,
            name,
            directed,
            LinkDirection::Inward,
            link_type.target_to_source.as_deref(),
            requested_direction,
        ),
    }
}

fn resolve_link_phrase_match(
    id: &str,
    name: String,
    directed: bool,
    phrase_direction: LinkDirection,
    phrase: Option<&str>,
    requested_direction: Option<LinkDirection>,
) -> Result<ResolvedLinkType, YtdError> {
    if !directed && requested_direction.is_some() {
        return Err(YtdError::Input(format!(
            "Link type `{name}` is not directed; omit --direction"
        )));
    }

    if let Some(requested_direction) = requested_direction {
        if requested_direction != phrase_direction {
            return Err(YtdError::Input(format!(
                "Link phrase `{}` implies {} direction; remove --direction or use --direction {}",
                phrase.unwrap_or(""),
                phrase_direction.label(),
                phrase_direction.label()
            )));
        }
    }

    let phrase = phrase
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| YtdError::Input(format!("Issue link type `{name}` has no usable phrase")))?;
    let direction = if directed {
        Some(phrase_direction)
    } else {
        None
    };
    let link_id = if directed {
        format!("{id}{}", phrase_direction.link_suffix())
    } else {
        id.to_string()
    };

    Ok(ResolvedLinkType {
        name,
        link_id,
        direction,
        phrase: phrase.to_string(),
    })
}

fn phrase_for_direction<'a>(
    link_type: &'a IssueLinkType,
    direction: LinkDirection,
    name: &str,
) -> Result<&'a str, YtdError> {
    let phrase = match direction {
        LinkDirection::Inward => link_type.target_to_source.as_deref(),
        LinkDirection::Outward => link_type.source_to_target.as_deref(),
    };
    phrase
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            YtdError::Input(format!(
                "Issue link type `{name}` has no {} phrase",
                direction.label()
            ))
        })
}

fn first_non_empty<'a>(values: &[Option<&'a str>]) -> Option<&'a str> {
    values
        .iter()
        .flatten()
        .copied()
        .find(|value| !value.trim().is_empty())
}

fn matches_link_value(value: Option<&str>, requested: &str) -> bool {
    value
        .map(|value| normalize_link_value(value) == requested)
        .unwrap_or(false)
}

fn normalize_link_value(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn unknown_link_type_error(requested_type: &str, link_types: &[IssueLinkType]) -> YtdError {
    let examples = link_type_examples(link_types);
    let mut message = format!(
        "Unknown link type: {requested_type}\nRun `ytd ticket link-types` to list valid link types."
    );
    if !examples.is_empty() {
        message.push_str("\nAvailable examples: ");
        message.push_str(&examples.join(", "));
    }
    YtdError::Input(message)
}

fn link_type_examples(link_types: &[IssueLinkType]) -> Vec<String> {
    let mut examples = Vec::new();
    for link_type in link_types {
        for value in [
            link_type.name.as_deref(),
            link_type.source_to_target.as_deref(),
            link_type.target_to_source.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if value.trim().is_empty() {
                continue;
            }
            if !examples
                .iter()
                .any(|existing: &String| existing.eq_ignore_ascii_case(value))
            {
                examples.push(value.to_string());
            }
            if examples.len() >= 8 {
                return examples;
            }
        }
    }
    examples
}

fn find_issue_link_to_unlink<'a>(
    links: &'a [IssueLink],
    source: &str,
    target: &str,
    link_type: &ResolvedLinkType,
) -> Result<(&'a str, &'a str), YtdError> {
    let link = links
        .iter()
        .find(|link| {
            link_type_matches(link, link_type)
                && link
                    .issues
                    .as_ref()
                    .map(|issues| {
                        issues
                            .iter()
                            .any(|issue| linked_issue_matches(issue, target))
                    })
                    .unwrap_or(false)
        })
        .ok_or_else(|| {
            YtdError::Input(format!(
                "Link not found: {source} -> {target} (type: {}, phrase: {})",
                link_type.name, link_type.phrase
            ))
        })?;

    let link_id = link.id.as_deref().ok_or_else(|| {
        YtdError::Input(format!("Issue link has no ID for type: {}", link_type.name))
    })?;
    let target_database_id = link
        .issues
        .as_ref()
        .and_then(|issues| {
            issues
                .iter()
                .find(|issue| linked_issue_matches(issue, target))
        })
        .map(|issue| issue.id.as_str())
        .filter(|id| !id.is_empty())
        .ok_or_else(|| YtdError::Input(format!("Linked issue has no database ID: {target}")))?;

    Ok((link_id, target_database_id))
}

fn link_type_matches(link: &IssueLink, requested: &ResolvedLinkType) -> bool {
    let Some(link_type) = link.link_type.as_ref() else {
        return false;
    };
    let type_matches = link_type
        .id
        .as_deref()
        .map(|id| {
            requested.link_id == id
                || requested.link_id == format!("{id}s")
                || requested.link_id == format!("{id}t")
        })
        .unwrap_or(false)
        || link_type
            .name
            .as_deref()
            .map(|name| name.eq_ignore_ascii_case(&requested.name))
            .unwrap_or(false);

    if !type_matches {
        return false;
    }

    requested
        .direction
        .map(|direction| {
            link.direction
                .as_deref()
                .map(|value| value.eq_ignore_ascii_case(direction.youtrack_direction()))
                .unwrap_or(false)
        })
        .unwrap_or(true)
}

fn linked_issue_matches(issue: &Issue, target: &str) -> bool {
    issue.id.eq_ignore_ascii_case(target)
        || issue
            .id_readable
            .as_deref()
            .map(|id| id.eq_ignore_ascii_case(target))
            .unwrap_or(false)
}

fn cmd_links<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let links = client.list_issue_links(id)?;
    if matches!(opts.format, format::Format::Text) {
        print!("{}", render_issue_links_text(&links));
    } else {
        let outputs: Vec<IssueLinkOutput> = links.iter().cloned().map(issue_link_output).collect();
        format::print_raw_or_processed_items(&links, &outputs, opts)?;
    }
    Ok(())
}

fn cmd_link_types<T: HttpTransport>(
    client: &YtClient<T>,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    if matches!(opts.format, format::Format::Md) {
        return Err(YtdError::Input(
            "Format md is not supported for ticket link-types".into(),
        ));
    }

    let link_types = client.list_issue_link_types()?;
    if matches!(opts.format, format::Format::Text) {
        print!("{}", render_issue_link_types_text(&link_types));
    } else {
        let outputs: Vec<IssueLinkTypeOutput> = link_types
            .iter()
            .cloned()
            .map(issue_link_type_output)
            .collect();
        format::print_raw_or_processed_items(&link_types, &outputs, opts)?;
    }
    Ok(())
}

fn render_issue_link_types_text(link_types: &[IssueLinkType]) -> String {
    let mut rows = vec![(
        "Name".to_string(),
        "Direction".to_string(),
        "Phrase".to_string(),
        "Link ID".to_string(),
    )];

    for link_type in link_types {
        let name = link_type.name.as_deref().unwrap_or("-");
        let id = link_type.id.as_deref().unwrap_or("-");
        let directed = link_type.directed.unwrap_or_else(|| {
            link_type
                .target_to_source
                .as_deref()
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
        });

        if directed {
            rows.push((
                name.to_string(),
                "inward".to_string(),
                link_type
                    .target_to_source
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or("-")
                    .to_string(),
                format!("{id}t"),
            ));
            rows.push((
                name.to_string(),
                "outward".to_string(),
                link_type
                    .source_to_target
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or("-")
                    .to_string(),
                format!("{id}s"),
            ));
        } else {
            rows.push((
                name.to_string(),
                "both".to_string(),
                first_non_empty(&[
                    link_type.source_to_target.as_deref(),
                    link_type.target_to_source.as_deref(),
                    link_type.name.as_deref(),
                ])
                .unwrap_or("-")
                .to_string(),
                id.to_string(),
            ));
        }
    }

    if rows.len() == 1 {
        return "No link types found.\n".into();
    }

    let name_width = rows.iter().map(|row| row.0.len()).max().unwrap_or(4);
    let direction_width = rows.iter().map(|row| row.1.len()).max().unwrap_or(9);
    let phrase_width = rows.iter().map(|row| row.2.len()).max().unwrap_or(6);
    let mut out = String::new();
    for (name, direction, phrase, link_id) in rows {
        out.push_str(&format!(
            "{name:<name_width$}  {direction:<direction_width$}  {phrase:<phrase_width$}  {link_id}\n"
        ));
    }
    out
}

fn render_issue_links_text(links: &[IssueLink]) -> String {
    let populated: Vec<&IssueLink> = links
        .iter()
        .filter(|link| {
            link.issues
                .as_ref()
                .map(|issues| !issues.is_empty())
                .unwrap_or(false)
        })
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
        out.push_str(
            link.link_type
                .as_ref()
                .and_then(|lt| lt.name.as_deref())
                .unwrap_or("Unknown"),
        );
        out.push('\n');
        out.push_str("direction: ");
        out.push_str(link.direction.as_deref().unwrap_or("Unknown"));
        out.push('\n');
        out.push_str("issues:\n");

        if let Some(issues) = &link.issues {
            for issue in issues {
                out.push_str("- ");
                out.push_str(&render_issue_inline(issue));
                out.push('\n');
            }
        }
    }

    out
}

pub(crate) fn print_issue_items(issues: &[Issue], opts: &OutputOptions) {
    if matches!(opts.format, format::Format::Text) {
        print!("{}", render_issue_list_text(issues, opts));
    } else {
        let outputs: Vec<IssueOutput> = issues.iter().cloned().map(issue_output).collect();
        format::print_raw_or_processed_items(issues, &outputs, opts)
            .expect("issue output is serializable");
    }
}

fn render_issue_list_text(issues: &[Issue], opts: &OutputOptions) -> String {
    let mut out = String::new();

    for (idx, issue) in issues.iter().enumerate() {
        if idx > 0 {
            out.push('\n');
        }

        out.push_str(&render_issue_list_item(issue, opts));
    }

    out
}

fn render_issue_list_item(issue: &Issue, opts: &OutputOptions) -> String {
    let mut out = String::new();
    out.push_str(&issue_title(issue, opts.no_meta));
    out.push('\n');

    let mut fields = Vec::new();
    if !opts.no_meta {
        if let Some(project) = issue_project(issue) {
            fields.push(("Project".to_string(), project));
        }
    }

    fields.extend(important_custom_fields(issue));

    if !has_important_field(issue, &["state", "status", "zustand"]) {
        if let Some(resolved) = issue.resolved {
            fields.push(("Resolved".to_string(), timestamp_or_yes(resolved)));
        } else {
            fields.push(("Resolved".to_string(), "no".to_string()));
        }
    }

    if !opts.no_meta {
        if let Some(updated) = issue.updated {
            fields.push(("Updated".to_string(), format_timestamp(updated)));
        }
    }

    for (label, value) in fields {
        out.push_str("  ");
        out.push_str(&label);
        out.push_str(": ");
        out.push_str(&value);
        out.push('\n');
    }

    out
}

fn render_issue_inline(issue: &Issue) -> String {
    let mut parts = Vec::new();
    parts.push(issue_identifier(issue).to_string());

    if let Some(summary) = issue.summary.as_deref().filter(|s| !s.trim().is_empty()) {
        parts.push(summary.to_string());
    }

    for (label, value) in important_custom_fields(issue) {
        if matches!(
            label.as_str(),
            "State" | "Status" | "Assignee" | "Priority" | "Type"
        ) {
            parts.push(format!("{label}: {value}"));
        }
    }

    parts.join(" | ")
}

fn render_issue_detail_text(
    issue: &Issue,
    parent_id: &str,
    opts: &OutputOptions,
    include_comments: bool,
) -> String {
    let mut out = String::new();
    out.push_str(&issue_title(issue, opts.no_meta));
    out.push('\n');
    out.push('\n');

    push_section(&mut out, "Status");
    let mut status_rows = Vec::new();
    if !opts.no_meta {
        if let Some(project) = issue_project(issue) {
            status_rows.push(("Project".to_string(), project));
        }
    }
    status_rows.extend(important_custom_fields(issue));
    if !opts.no_meta {
        status_rows.push((
            "Resolved".to_string(),
            issue
                .resolved
                .map(timestamp_or_yes)
                .unwrap_or_else(|| "no".into()),
        ));
    }
    if !issue.tags.is_empty() {
        status_rows.push((
            "Tags".to_string(),
            issue
                .tags
                .iter()
                .map(|tag| tag.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }
    push_rows(&mut out, status_rows);

    if !issue.custom_fields.is_empty() {
        out.push('\n');
        push_section(&mut out, "Custom Fields");
        push_rows(&mut out, all_custom_fields(issue));
    }

    if !opts.no_meta {
        out.push('\n');
        push_section(&mut out, "Metadata");
        let mut rows = vec![("id".to_string(), issue_identifier(issue).to_string())];
        rows.push(("ytId".to_string(), issue.id.clone()));
        if let Some(created) = issue.created {
            rows.push(("Created".to_string(), format_timestamp(created)));
        }
        if let Some(updated) = issue.updated {
            rows.push(("Updated".to_string(), format_timestamp(updated)));
        }
        if let Some(reporter) = &issue.reporter {
            rows.push(("Reporter".to_string(), format_user(reporter)));
        }
        if let Some(visibility) = &issue.visibility {
            if let Some(visibility) = format_visibility(visibility) {
                rows.push(("Visibility".to_string(), visibility));
            }
        }
        push_rows(&mut out, rows);
    }

    if let Some(description) = issue
        .description
        .as_deref()
        .filter(|description| !description.trim().is_empty())
    {
        out.push('\n');
        let description = format::markdown_to_text(description);
        out.push_str(&description);
        out.push('\n');
    }

    if include_comments && !issue.comments.is_empty() {
        out.push('\n');
        push_section(&mut out, "Comments");
        for comment in &issue.comments {
            let id = if opts.no_meta {
                String::new()
            } else {
                encode_comment_id(parent_id, &comment.id)
            };
            let author = comment
                .author
                .as_ref()
                .map(format_user)
                .unwrap_or_else(|| "Unknown".into());
            let created = if opts.no_meta {
                None
            } else {
                comment.created.map(format_timestamp)
            };
            let text = comment
                .text
                .as_deref()
                .map(format::markdown_to_text)
                .unwrap_or_default();

            out.push_str("  ");
            if !id.is_empty() {
                out.push_str(&id);
                out.push_str(" | ");
            }
            out.push_str(&author);
            if let Some(created) = created {
                out.push_str(" | ");
                out.push_str(&created);
            }
            out.push('\n');
            push_indented_block(&mut out, &text);
        }
    }

    out
}

fn issue_title(issue: &Issue, no_meta: bool) -> String {
    let summary = issue.summary.as_deref().unwrap_or("").trim();
    if no_meta {
        if summary.is_empty() {
            return "(no summary)".to_string();
        }
        return summary.to_string();
    }

    let identifier = issue_identifier(issue);
    if summary.is_empty() {
        identifier.to_string()
    } else {
        format!("{identifier} | {summary}")
    }
}

fn issue_identifier(issue: &Issue) -> &str {
    issue.id_readable.as_deref().unwrap_or(&issue.id)
}

fn issue_project(issue: &Issue) -> Option<String> {
    issue.project.as_ref().and_then(|project| {
        project
            .short_name
            .clone()
            .or_else(|| project.name.clone())
            .filter(|value| !value.trim().is_empty())
    })
}

fn important_custom_fields(issue: &Issue) -> Vec<(String, String)> {
    let mut rows = Vec::new();
    for field in &issue.custom_fields {
        let Some(name) = field.name.as_deref() else {
            continue;
        };
        let Some(label) = important_field_label(name) else {
            continue;
        };
        let Some(value) = field.value.as_ref().and_then(format_custom_field_value) else {
            continue;
        };
        rows.push((label.to_string(), value));
    }
    rows
}

fn all_custom_fields(issue: &Issue) -> Vec<(String, String)> {
    issue
        .custom_fields
        .iter()
        .filter_map(|field| {
            let name = field.name.as_deref()?;
            let value = field.value.as_ref().and_then(format_custom_field_value)?;
            Some((name.to_string(), value))
        })
        .collect()
}

fn important_field_label(name: &str) -> Option<&'static str> {
    let normalized = normalize_field_name(name);
    match normalized.as_str() {
        "state" | "status" | "zustand" => Some("State"),
        "assignee" | "assignees" | "bearbeiter" | "zustandig" | "zustaendig" => Some("Assignee"),
        "priority" | "prioritat" | "prioritaet" => Some("Priority"),
        "type" | "typ" => Some("Type"),
        "estimation" | "estimate" | "schatzung" | "schaetzung" => Some("Estimation"),
        "spent time" | "aufwand" => Some("Spent time"),
        _ => None,
    }
}

fn has_important_field(issue: &Issue, names: &[&str]) -> bool {
    issue.custom_fields.iter().any(|field| {
        field
            .name
            .as_deref()
            .map(normalize_field_name)
            .map(|name| names.contains(&name.as_str()))
            .unwrap_or(false)
            && field
                .value
                .as_ref()
                .and_then(format_custom_field_value)
                .is_some()
    })
}

fn normalize_field_name(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .replace('ä', "a")
        .replace('ö', "o")
        .replace('ü', "u")
        .replace('ß', "ss")
}

fn format_custom_field_value(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::Bool(value) => Some(if *value { "yes" } else { "no" }.to_string()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        serde_json::Value::String(value) => non_empty(value),
        serde_json::Value::Array(values) => {
            let parts = values
                .iter()
                .filter_map(format_custom_field_value)
                .collect::<Vec<_>>();
            if parts.is_empty() {
                None
            } else {
                Some(parts.join(", "))
            }
        }
        serde_json::Value::Object(map) => {
            if let Some(value) = map.get("presentation").and_then(|value| value.as_str()) {
                return non_empty(value);
            }
            if let Some(minutes) = map.get("minutes").and_then(|value| value.as_u64()) {
                return Some(format_minutes(minutes));
            }
            if let Some(full_name) = map.get("fullName").and_then(|value| value.as_str()) {
                if let Some(login) = map.get("login").and_then(|value| value.as_str()) {
                    return Some(format!("{full_name} ({login})"));
                }
                return non_empty(full_name);
            }
            if let Some(login) = map.get("login").and_then(|value| value.as_str()) {
                return non_empty(login);
            }
            if let Some(name) = map.get("name").and_then(|value| value.as_str()) {
                return non_empty(name);
            }
            serde_json::to_string(value)
                .ok()
                .and_then(|value| non_empty(&value))
        }
    }
}

fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn format_minutes(minutes: u64) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    match (hours, mins) {
        (0, mins) => format!("{mins}m"),
        (hours, 0) => format!("{hours}h"),
        (hours, mins) => format!("{hours}h{mins}m"),
    }
}

fn format_user(user: &User) -> String {
    match user.full_name.as_deref() {
        Some(full_name) if !full_name.trim().is_empty() => format!("{full_name} ({})", user.login),
        _ => user.login.clone(),
    }
}

fn format_visibility(visibility: &LimitedVisibility) -> Option<String> {
    if visibility.permitted_groups.is_empty() {
        return None;
    }
    Some(
        visibility
            .permitted_groups
            .iter()
            .map(|group| group.name.as_str())
            .collect::<Vec<_>>()
            .join(", "),
    )
}

fn format_timestamp(ms: u64) -> String {
    let secs = (ms / 1000) as i64;
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let (year, month, day) = days_to_date(days_since_epoch);
    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;

    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02} UTC")
}

fn days_to_date(mut days: i64) -> (i64, i64, i64) {
    days += 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn timestamp_or_yes(ms: u64) -> String {
    format!("yes ({})", format_timestamp(ms))
}

fn push_section(out: &mut String, title: &str) {
    out.push_str(title);
    out.push('\n');
}

fn push_rows(out: &mut String, rows: Vec<(String, String)>) {
    for (label, value) in rows {
        if value.trim().is_empty() {
            continue;
        }
        out.push_str("  ");
        out.push_str(&label);
        out.push_str(": ");
        out.push_str(&value);
        out.push('\n');
    }
}

fn push_indented_block(out: &mut String, text: &str) {
    if text.trim().is_empty() {
        out.push_str("  \n");
        return;
    }
    for line in text.lines() {
        out.push_str("  ");
        out.push_str(line);
        out.push('\n');
    }
}

fn cmd_attach<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let file = args
        .positional
        .get(1)
        .ok_or_else(|| YtdError::Input("File path is required".into()))?;
    let path = Path::new(file);
    if !path.exists() {
        return Err(YtdError::Input(format!("File not found: {file}")));
    }
    client.upload_attachment(id, path)?;
    println!(
        "Attached {}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or(file)
    );
    Ok(())
}

fn cmd_attachments<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let attachments = client.list_attachments(id)?;
    let outputs: Vec<AttachmentOutput> = attachments
        .iter()
        .cloned()
        .map(|attachment| issue_attachment_output(id, attachment))
        .collect();
    format::print_raw_or_processed_items(&attachments, &outputs, opts)?;
    Ok(())
}

fn cmd_log<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let dur_str = args
        .positional
        .get(1)
        .ok_or_else(|| YtdError::Input("Duration is required (e.g. 30m, 1h, 2h30m)".into()))?;
    let minutes = duration::parse_duration(dur_str)?;
    let text = args
        .positional
        .get(2..)
        .map(|s| s.join(" "))
        .filter(|s| !s.is_empty());

    let date = args.flags.get("date").and_then(|d| {
        // Parse YYYY-MM-DD to epoch ms
        parse_date_to_epoch_ms(d)
    });

    let work_type = args.flags.get("type").map(|t| WorkType {
        id: None,
        name: Some(t.clone()),
    });

    let input = CreateWorkItemInput {
        duration: WorkItemDuration {
            minutes: Some(minutes),
            presentation: None,
        },
        text,
        date,
        work_type,
    };
    client.add_work_item(id, &input)?;
    Ok(())
}

fn cmd_worklog<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let items = client.list_work_items(id)?;
    format::print_items(&items, opts);
    Ok(())
}

#[derive(Debug, Clone)]
struct ResolvedStatusField {
    name: String,
    values: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusValueOutput {
    name: String,
    current: bool,
}

fn cmd_status<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    if matches!(opts.format, format::Format::Md) {
        return Err(YtdError::Input(
            "Format md is not supported for ticket status".into(),
        ));
    }

    let id = require_id(args)?;
    let status = args
        .positional
        .get(1..)
        .map(|parts| parts.join(" "))
        .filter(|value| !value.trim().is_empty());
    let issue = client.get_issue(id)?;
    let status_field = resolve_issue_status_field(client, &issue)?;

    match status {
        Some(status) => {
            let canonical = canonical_status_value(&status_field, &status)?;
            let body = serde_json::json!({
                "customFields": [{
                    "$type": "StateIssueCustomField",
                    "name": status_field.name,
                    "value": {
                        "$type": "StateBundleElement",
                        "name": canonical
                    }
                }]
            });
            client.set_custom_field(id, &body)?;
        }
        None => print_status_values(&issue, &status_field, opts)?,
    }

    Ok(())
}

fn resolve_issue_status_field<T: HttpTransport>(
    client: &YtClient<T>,
    issue: &Issue,
) -> Result<ResolvedStatusField, YtdError> {
    let project_id = issue
        .project
        .as_ref()
        .map(|project| project.id.as_str())
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(|| YtdError::Input("Ticket has no project in the YouTrack response".into()))?;
    let fields = client.list_project_custom_fields(project_id)?;
    resolve_status_field(issue, &fields)
}

fn resolve_status_field(
    issue: &Issue,
    fields: &[ProjectCustomField],
) -> Result<ResolvedStatusField, YtdError> {
    let candidates: Vec<&ProjectCustomField> = fields
        .iter()
        .filter(|field| is_state_project_field(field))
        .collect();
    if candidates.is_empty() {
        return Err(YtdError::Input(
            "Project has no State/Status custom field. Use ytd ticket fields <id> or ytd ticket set <id> <field> <value>."
                .into(),
        ));
    }

    let selected = if candidates.len() == 1 {
        candidates[0]
    } else if let Some(field) = candidates.iter().copied().find(|field| {
        project_field_name(field)
            .as_deref()
            .map(|name| issue_has_state_field(issue, name))
            .unwrap_or(false)
    }) {
        field
    } else {
        let names = candidates
            .iter()
            .filter_map(|field| project_field_name(field))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(YtdError::Input(format!(
            "Multiple State/Status custom fields found: {names}. Use ytd ticket set <id> <field> <value>."
        )));
    };

    let name = project_field_name(selected).ok_or_else(|| {
        YtdError::Input("State/Status custom field has no name in the YouTrack response".into())
    })?;
    let values = selected
        .bundle
        .as_ref()
        .map(|bundle| bundle.values.clone())
        .unwrap_or_default();
    if values.is_empty() {
        return Err(YtdError::Input(format!(
            "Cannot validate statuses for project field {name}. Use ytd ticket set <id> <field> <value>."
        )));
    }

    Ok(ResolvedStatusField { name, values })
}

fn is_state_project_field(field: &ProjectCustomField) -> bool {
    field.field_type.as_deref() == Some("StateProjectCustomField")
        || field
            .field
            .as_ref()
            .and_then(|prototype| prototype.field_type.as_ref())
            .and_then(|field_type| field_type.value_type.as_deref())
            == Some("state")
}

fn project_field_name(field: &ProjectCustomField) -> Option<String> {
    field
        .field
        .as_ref()
        .and_then(|prototype| prototype.name.clone())
        .or_else(|| field.name.clone())
        .filter(|name| !name.trim().is_empty())
}

fn issue_has_state_field(issue: &Issue, name: &str) -> bool {
    issue.custom_fields.iter().any(|field| {
        field
            .name
            .as_deref()
            .map(|field_name| field_name.eq_ignore_ascii_case(name))
            .unwrap_or(false)
            && field
                .field_type
                .as_deref()
                .map(|field_type| field_type.contains("State"))
                .unwrap_or(true)
    })
}

fn canonical_status_value(
    status_field: &ResolvedStatusField,
    input: &str,
) -> Result<String, YtdError> {
    let matches = status_field
        .values
        .iter()
        .filter_map(status_value_name)
        .filter(|name| name.eq_ignore_ascii_case(input.trim()))
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [name] => Ok((*name).to_string()),
        [] => Err(YtdError::Input(format!(
            "Unknown status: {}. Valid statuses: {}",
            input,
            valid_status_names(status_field).join(", ")
        ))),
        _ => Err(YtdError::Input(format!(
            "Ambiguous status: {}. Valid statuses: {}",
            input,
            valid_status_names(status_field).join(", ")
        ))),
    }
}

fn status_value_name(value: &serde_json::Value) -> Option<&str> {
    value.get("name").and_then(|name| name.as_str())
}

fn valid_status_names(status_field: &ResolvedStatusField) -> Vec<String> {
    status_field
        .values
        .iter()
        .filter_map(status_value_name)
        .map(String::from)
        .collect()
}

fn current_status_name(issue: &Issue, status_field: &ResolvedStatusField) -> Option<String> {
    issue
        .custom_fields
        .iter()
        .find(|field| {
            field
                .name
                .as_deref()
                .map(|name| name.eq_ignore_ascii_case(&status_field.name))
                .unwrap_or(false)
        })
        .and_then(|field| field.value.as_ref())
        .and_then(status_value_name)
        .map(String::from)
}

fn status_value_outputs(
    issue: &Issue,
    status_field: &ResolvedStatusField,
) -> Vec<StatusValueOutput> {
    let current = current_status_name(issue, status_field);
    status_field
        .values
        .iter()
        .filter_map(status_value_name)
        .map(|name| StatusValueOutput {
            name: name.to_string(),
            current: current
                .as_deref()
                .map(|current| current.eq_ignore_ascii_case(name))
                .unwrap_or(false),
        })
        .collect()
}

fn print_status_values(
    issue: &Issue,
    status_field: &ResolvedStatusField,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    if matches!(opts.format, format::Format::Raw) {
        format::print_value(&serde_json::Value::Array(status_field.values.clone()), opts);
        return Ok(());
    }

    let outputs = status_value_outputs(issue, status_field);
    if matches!(opts.format, format::Format::Text) {
        for value in outputs {
            if value.current {
                println!("* {}", value.name);
            } else {
                println!("  {}", value.name);
            }
        }
    } else {
        format::print_value(&serde_json::to_value(outputs)?, opts);
    }
    Ok(())
}

fn cmd_set<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let field_name = args
        .positional
        .get(1)
        .ok_or_else(|| YtdError::Input("Field name is required".into()))?;
    let value_str = args
        .positional
        .get(2..)
        .map(|s| s.join(" "))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| YtdError::Input("Value is required".into()))?;

    // Get issue to read existing custom field $types
    let issue = client.get_issue(id)?;

    // Find the $type from the issue's existing custom fields
    let cf_type = issue
        .custom_fields
        .iter()
        .find(|f| {
            f.name
                .as_deref()
                .map(|n| n.eq_ignore_ascii_case(field_name))
                .unwrap_or(false)
        })
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
    use crate::config::TEST_ENV_LOCK;
    use crate::types::YtdConfig;
    use std::cell::RefCell;
    use std::path::Path;
    use std::rc::Rc;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct CapturedRequest {
        method: String,
        url: String,
        body: Option<String>,
    }

    struct MockTransport {
        responses: RefCell<Vec<String>>,
        requests: RefCell<Vec<CapturedRequest>>,
    }

    impl MockTransport {
        fn new(responses: Vec<&str>) -> Self {
            Self {
                responses: RefCell::new(responses.into_iter().rev().map(String::from).collect()),
                requests: RefCell::new(vec![]),
            }
        }

        fn request(&self, index: usize) -> CapturedRequest {
            self.requests.borrow()[index].clone()
        }

        fn request_count(&self) -> usize {
            self.requests.borrow().len()
        }
    }

    impl HttpTransport for MockTransport {
        fn get(&self, url: &str, _token: &str) -> Result<String, YtdError> {
            self.requests.borrow_mut().push(CapturedRequest {
                method: "GET".into(),
                url: url.into(),
                body: None,
            });
            self.responses
                .borrow_mut()
                .pop()
                .ok_or_else(|| YtdError::Http("No more mock responses".into()))
        }

        fn get_bytes(&self, _url: &str, _token: &str) -> Result<Vec<u8>, YtdError> {
            Err(YtdError::Http("unused".into()))
        }

        fn post(&self, url: &str, _token: &str, body: &str) -> Result<String, YtdError> {
            self.requests.borrow_mut().push(CapturedRequest {
                method: "POST".into(),
                url: url.into(),
                body: Some(body.into()),
            });
            self.responses
                .borrow_mut()
                .pop()
                .ok_or_else(|| YtdError::Http("No more mock responses".into()))
        }

        fn post_multipart(
            &self,
            _url: &str,
            _token: &str,
            _file_path: &Path,
            _file_name: &str,
        ) -> Result<String, YtdError> {
            Err(YtdError::Http("unused".into()))
        }

        fn delete(&self, url: &str, _token: &str) -> Result<(), YtdError> {
            self.requests.borrow_mut().push(CapturedRequest {
                method: "DELETE".into(),
                url: url.into(),
                body: None,
            });
            Ok(())
        }
    }

    impl HttpTransport for Rc<MockTransport> {
        fn get(&self, url: &str, token: &str) -> Result<String, YtdError> {
            self.as_ref().get(url, token)
        }

        fn get_bytes(&self, url: &str, token: &str) -> Result<Vec<u8>, YtdError> {
            self.as_ref().get_bytes(url, token)
        }

        fn post(&self, url: &str, token: &str, body: &str) -> Result<String, YtdError> {
            self.as_ref().post(url, token, body)
        }

        fn post_multipart(
            &self,
            url: &str,
            token: &str,
            file_path: &Path,
            file_name: &str,
        ) -> Result<String, YtdError> {
            self.as_ref()
                .post_multipart(url, token, file_path, file_name)
        }

        fn delete(&self, url: &str, token: &str) -> Result<(), YtdError> {
            self.as_ref().delete(url, token)
        }
    }

    fn test_client(responses: Vec<&str>) -> YtClient<MockTransport> {
        YtClient::new(
            YtdConfig {
                url: "https://test.youtrack.cloud".into(),
                token: "perm:test".into(),
            },
            MockTransport::new(responses),
        )
    }

    fn test_client_with_transport(
        responses: Vec<&str>,
    ) -> (YtClient<Rc<MockTransport>>, Rc<MockTransport>) {
        let transport = Rc::new(MockTransport::new(responses));
        let client = YtClient::new(
            YtdConfig {
                url: "https://test.youtrack.cloud".into(),
                token: "perm:test".into(),
            },
            transport.clone(),
        );
        (client, transport)
    }

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
            visibility: None,
            tags: vec![],
            comments: vec![],
            custom_fields: vec![],
        }
    }

    fn custom_field(name: &str, value: serde_json::Value) -> CustomField {
        CustomField {
            id: None,
            name: Some(name.into()),
            field_type: None,
            value: Some(value),
        }
    }

    fn state_custom_field(name: &str, value: &str) -> CustomField {
        CustomField {
            id: None,
            name: Some(name.into()),
            field_type: Some("StateIssueCustomField".into()),
            value: Some(serde_json::json!({
                "$type": "StateBundleElement",
                "name": value
            })),
        }
    }

    fn issue_with_project(current_field_name: &str, current_status: &str) -> Issue {
        Issue {
            id: "2-12".into(),
            id_readable: Some("DWP-12".into()),
            summary: Some("Ticket".into()),
            description: None,
            created: None,
            updated: None,
            resolved: None,
            reporter: None,
            project: Some(ProjectRef {
                id: "0-96".into(),
                short_name: Some("DWP".into()),
                name: Some("DW Playground".into()),
            }),
            visibility: None,
            tags: vec![],
            comments: vec![],
            custom_fields: vec![state_custom_field(current_field_name, current_status)],
        }
    }

    fn issue_with_project_json(current_field_name: &str, current_status: &str) -> String {
        serde_json::to_string(&issue_with_project(current_field_name, current_status)).unwrap()
    }

    fn project_status_fields_json(field_name: &str, values: &[&str]) -> String {
        let values = values
            .iter()
            .map(|value| {
                serde_json::json!({
                    "id": format!("s-{value}"),
                    "name": value,
                    "$type": "StateBundleElement"
                })
            })
            .collect::<Vec<_>>();
        serde_json::json!([{
            "id": "92-1",
            "field": {
                "id": "58-1",
                "name": field_name,
                "fieldType": {
                    "id": "state[1]",
                    "valueType": "state",
                    "isMultiValue": false
                }
            },
            "canBeEmpty": false,
            "bundle": { "values": values },
            "$type": "StateProjectCustomField"
        }])
        .to_string()
    }

    fn parsed_status_args(status: &[&str], format: Option<&str>) -> ParsedArgs {
        let mut positional = vec!["DWP-12".to_string()];
        positional.extend(status.iter().map(|part| (*part).to_string()));
        let mut flags = std::collections::HashMap::new();
        if let Some(format) = format {
            flags.insert("format".into(), format.into());
        }
        ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("status".into()),
            positional,
            flags,
        }
    }

    fn standard_link_types() -> Vec<IssueLinkType> {
        vec![
            IssueLinkType {
                id: Some("80-0".into()),
                name: Some("Relates".into()),
                directed: Some(false),
                aggregation: Some(false),
                read_only: Some(false),
                source_to_target: Some("relates to".into()),
                target_to_source: Some(String::new()),
            },
            IssueLinkType {
                id: Some("80-1".into()),
                name: Some("Depend".into()),
                directed: Some(true),
                aggregation: Some(false),
                read_only: Some(false),
                source_to_target: Some("is required for".into()),
                target_to_source: Some("depends on".into()),
            },
            IssueLinkType {
                id: Some("80-3".into()),
                name: Some("Subtask".into()),
                directed: Some(true),
                aggregation: Some(true),
                read_only: Some(true),
                source_to_target: Some("parent for".into()),
                target_to_source: Some("subtask of".into()),
            },
        ]
    }

    fn standard_link_types_json() -> &'static str {
        r#"[
            {"id":"80-0","name":"Relates","directed":false,"aggregation":false,"readOnly":false,"sourceToTarget":"relates to","targetToSource":""},
            {"id":"80-1","name":"Depend","directed":true,"aggregation":false,"readOnly":false,"sourceToTarget":"is required for","targetToSource":"depends on"},
            {"id":"80-3","name":"Subtask","directed":true,"aggregation":true,"readOnly":true,"sourceToTarget":"parent for","targetToSource":"subtask of"}
        ]"#
    }

    fn resolved_test_link_type(
        name: &str,
        link_id: &str,
        direction: Option<LinkDirection>,
        phrase: &str,
    ) -> ResolvedLinkType {
        ResolvedLinkType {
            name: name.into(),
            link_id: link_id.into(),
            direction,
            phrase: phrase.into(),
        }
    }

    fn list_opts() -> OutputOptions {
        OutputOptions {
            format: format::Format::Text,
            no_meta: false,
        }
    }

    #[test]
    fn status_value_outputs_mark_current_status() {
        let issue = issue_with_project("State", "In Progress");
        let fields: Vec<ProjectCustomField> = serde_json::from_str(&project_status_fields_json(
            "State",
            &["Open", "In Progress", "Done"],
        ))
        .unwrap();
        let status_field = resolve_status_field(&issue, &fields).unwrap();

        let outputs = status_value_outputs(&issue, &status_field);

        assert_eq!(outputs.len(), 3);
        assert_eq!(outputs[1].name, "In Progress");
        assert!(outputs[1].current);
        assert!(!outputs[0].current);
    }

    #[test]
    fn status_value_outputs_serialize_for_json() {
        let issue = issue_with_project("State", "Done");
        let fields: Vec<ProjectCustomField> =
            serde_json::from_str(&project_status_fields_json("State", &["Open", "Done"])).unwrap();
        let status_field = resolve_status_field(&issue, &fields).unwrap();

        let value = serde_json::to_value(status_value_outputs(&issue, &status_field)).unwrap();

        assert_eq!(value[1]["name"], "Done");
        assert_eq!(value[1]["current"], true);
    }

    #[test]
    fn cmd_status_without_value_lists_without_mutating() {
        let issue_json = issue_with_project_json("State", "Open");
        let fields_json = project_status_fields_json("State", &["Open", "Done"]);
        let (client, transport) = test_client_with_transport(vec![&issue_json, &fields_json]);
        let args = parsed_status_args(&[], Some("json"));
        let opts = OutputOptions {
            format: format::Format::Json,
            no_meta: false,
        };

        cmd_status(&client, &args, &opts).unwrap();

        assert_eq!(transport.request_count(), 2);
        assert!(transport.request(0).url.contains("/api/issues/DWP-12?"));
        assert!(transport
            .request(1)
            .url
            .contains("/api/admin/projects/0-96/customFields?"));
    }

    #[test]
    fn raw_status_values_preserve_bundle_shape() {
        let issue = issue_with_project("State", "Open");
        let fields: Vec<ProjectCustomField> =
            serde_json::from_str(&project_status_fields_json("State", &["Open", "Done"])).unwrap();
        let status_field = resolve_status_field(&issue, &fields).unwrap();

        assert_eq!(status_field.values[0]["name"], "Open");
        assert_eq!(status_field.values[0]["$type"], "StateBundleElement");
    }

    #[test]
    fn cmd_status_rejects_md_format() {
        let (client, transport) = test_client_with_transport(vec![]);
        let args = parsed_status_args(&[], Some("md"));
        let opts = OutputOptions {
            format: format::Format::Md,
            no_meta: false,
        };

        let err = cmd_status(&client, &args, &opts).unwrap_err();

        assert_eq!(
            err.to_string(),
            "Format md is not supported for ticket status"
        );
        assert_eq!(transport.request_count(), 0);
    }

    #[test]
    fn cmd_status_sets_canonical_project_state() {
        let issue_json = issue_with_project_json("State", "Open");
        let fields_json = project_status_fields_json("State", &["Open", "In Progress", "Done"]);
        let (client, transport) =
            test_client_with_transport(vec![&issue_json, &fields_json, r#"{}"#]);
        let args = parsed_status_args(&["done"], None);

        cmd_status(&client, &args, &list_opts()).unwrap();

        assert_eq!(transport.request_count(), 3);
        let request = transport.request(2);
        assert_eq!(request.method, "POST");
        let body: serde_json::Value = serde_json::from_str(&request.body.unwrap()).unwrap();
        assert_eq!(body["customFields"][0]["name"], "State");
        assert_eq!(body["customFields"][0]["$type"], "StateIssueCustomField");
        assert_eq!(body["customFields"][0]["value"]["name"], "Done");
        assert_eq!(
            body["customFields"][0]["value"]["$type"],
            "StateBundleElement"
        );
    }

    #[test]
    fn cmd_status_accepts_multi_word_status() {
        let issue_json = issue_with_project_json("State", "Open");
        let fields_json = project_status_fields_json("State", &["Open", "In Progress", "Done"]);
        let (client, transport) =
            test_client_with_transport(vec![&issue_json, &fields_json, r#"{}"#]);
        let args = parsed_status_args(&["In", "Progress"], None);

        cmd_status(&client, &args, &list_opts()).unwrap();

        let request = transport.request(2);
        let body: serde_json::Value = serde_json::from_str(&request.body.unwrap()).unwrap();
        assert_eq!(body["customFields"][0]["value"]["name"], "In Progress");
    }

    #[test]
    fn cmd_status_rejects_unknown_status_before_post() {
        let issue_json = issue_with_project_json("State", "Open");
        let fields_json = project_status_fields_json("State", &["Open", "In Progress", "Done"]);
        let (client, transport) = test_client_with_transport(vec![&issue_json, &fields_json]);
        let args = parsed_status_args(&["Closed"], None);

        let err = cmd_status(&client, &args, &list_opts()).unwrap_err();

        assert!(err.to_string().contains("Unknown status: Closed"));
        assert!(err.to_string().contains("Open, In Progress, Done"));
        assert_eq!(transport.request_count(), 2);
    }

    #[test]
    fn resolve_status_field_rejects_missing_state_field() {
        let issue = issue_with_project("State", "Open");
        let fields: Vec<ProjectCustomField> = serde_json::from_value(serde_json::json!([{
            "id": "92-2",
            "field": {
                "id": "58-2",
                "name": "Priority",
                "fieldType": {
                    "id": "enum[1]",
                    "valueType": "enum",
                    "isMultiValue": false
                }
            },
            "$type": "EnumProjectCustomField",
            "bundle": { "values": [{ "name": "Major" }] }
        }]))
        .unwrap();

        let err = resolve_status_field(&issue, &fields).unwrap_err();

        assert!(err.to_string().contains("Project has no State/Status"));
    }

    #[test]
    fn resolve_status_field_uses_non_state_field_name() {
        let issue = issue_with_project("Status", "Open");
        let fields: Vec<ProjectCustomField> =
            serde_json::from_str(&project_status_fields_json("Status", &["Open", "Done"])).unwrap();

        let status_field = resolve_status_field(&issue, &fields).unwrap();

        assert_eq!(status_field.name, "Status");
    }

    fn clear_env() {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("YTD_CONFIG");
        std::env::remove_var("YTD_VISIBILITY_GROUP");
    }

    #[test]
    fn resolve_link_type_matches_name_default_inward() {
        let resolved = resolve_link_type(&standard_link_types(), "Subtask", None).unwrap();

        assert_eq!(resolved.name, "Subtask");
        assert_eq!(resolved.link_id, "80-3t");
        assert_eq!(resolved.direction, Some(LinkDirection::Inward));
        assert_eq!(resolved.phrase, "subtask of");
    }

    #[test]
    fn resolve_link_type_matches_name_outward() {
        let resolved =
            resolve_link_type(&standard_link_types(), "Subtask", Some("outward")).unwrap();

        assert_eq!(resolved.link_id, "80-3s");
        assert_eq!(resolved.direction, Some(LinkDirection::Outward));
        assert_eq!(resolved.phrase, "parent for");
    }

    #[test]
    fn resolve_link_type_matches_undirected_name() {
        let resolved = resolve_link_type(&standard_link_types(), "Relates", None).unwrap();

        assert_eq!(resolved.link_id, "80-0");
        assert_eq!(resolved.direction, None);
        assert_eq!(resolved.phrase, "relates to");
    }

    #[test]
    fn resolve_link_type_rejects_direction_for_undirected() {
        let error =
            resolve_link_type(&standard_link_types(), "Relates", Some("inward")).unwrap_err();

        assert!(error.to_string().contains("not directed"));
    }

    #[test]
    fn resolve_link_type_rejects_direction_for_undirected_phrase() {
        let error =
            resolve_link_type(&standard_link_types(), "relates to", Some("outward")).unwrap_err();

        assert!(error.to_string().contains("not directed"));
    }

    #[test]
    fn resolve_link_type_matches_legacy_phrase() {
        let inward = resolve_link_type(&standard_link_types(), "subtask of", None).unwrap();
        let outward = resolve_link_type(&standard_link_types(), "parent for", None).unwrap();

        assert_eq!(inward.link_id, "80-3t");
        assert_eq!(inward.direction, Some(LinkDirection::Inward));
        assert_eq!(outward.link_id, "80-3s");
        assert_eq!(outward.direction, Some(LinkDirection::Outward));
    }

    #[test]
    fn resolve_link_type_rejects_conflicting_phrase_direction() {
        let error =
            resolve_link_type(&standard_link_types(), "subtask of", Some("outward")).unwrap_err();

        assert!(error.to_string().contains("implies inward direction"));
    }

    #[test]
    fn resolve_link_type_rejects_unknown_type_with_helpful_message() {
        let error = resolve_link_type(&standard_link_types(), "Blocks", None).unwrap_err();

        assert!(error.to_string().contains("Unknown link type: Blocks"));
        assert!(error.to_string().contains("ytd ticket link-types"));
        assert!(error.to_string().contains("Subtask"));
    }

    #[test]
    fn render_issue_link_types_text_lists_direction_rows() {
        let rendered = render_issue_link_types_text(&standard_link_types());

        assert!(rendered.contains("Name"));
        assert!(rendered.contains("Relates"));
        assert!(rendered.contains("both"));
        assert!(rendered.contains("Subtask"));
        assert!(rendered.contains("inward"));
        assert!(rendered.contains("subtask of"));
        assert!(rendered.contains("80-3t"));
        assert!(rendered.contains("outward"));
        assert!(rendered.contains("parent for"));
        assert!(rendered.contains("80-3s"));
    }

    #[test]
    fn render_issue_links_text_with_populated_issues() {
        let links = vec![IssueLink {
            id: Some("105-0".into()),
            direction: Some("BOTH".into()),
            link_type: Some(IssueLinkType {
                id: Some("105-0".into()),
                name: Some("Relates".into()),
                source_to_target: Some("relates to".into()),
                target_to_source: Some(String::new()),
                ..Default::default()
            }),
            issues: Some(vec![sample_issue(
                "2-14252",
                Some("DWP-14"),
                Some("Link Target"),
            )]),
        }];

        let rendered = render_issue_links_text(&links);
        assert!(rendered.contains("linkType: Relates"));
        assert!(rendered.contains("direction: BOTH"));
        assert!(rendered.contains("- DWP-14 | Link Target"));
    }

    #[test]
    fn render_issue_links_text_enriches_linked_issues() {
        let mut issue = sample_issue("2-14252", Some("DWP-14"), Some("Link Target"));
        issue.custom_fields = vec![
            custom_field("State", serde_json::json!({"name": "Open"})),
            custom_field(
                "Assignee",
                serde_json::json!({"fullName": "Daniel Wetzel", "login": "daniel"}),
            ),
        ];
        let links = vec![IssueLink {
            id: Some("105-0".into()),
            direction: Some("BOTH".into()),
            link_type: Some(IssueLinkType {
                id: Some("105-0".into()),
                name: Some("Relates".into()),
                source_to_target: Some("relates to".into()),
                target_to_source: Some(String::new()),
                ..Default::default()
            }),
            issues: Some(vec![issue]),
        }];

        let rendered = render_issue_links_text(&links);

        assert!(rendered
            .contains("- DWP-14 | Link Target | State: Open | Assignee: Daniel Wetzel (daniel)"));
    }

    #[test]
    fn render_issue_list_text_prints_compact_ticket_fields() {
        let mut issue = sample_issue("2-14252", Some("DWP-14"), Some("Fix login"));
        issue.project = Some(ProjectRef {
            id: "0-1".into(),
            short_name: Some("DWP".into()),
            name: Some("Demo".into()),
        });
        issue.updated = Some(1_772_000_400_000);
        issue.custom_fields = vec![
            custom_field("State", serde_json::json!({"name": "Open"})),
            custom_field(
                "Assignee",
                serde_json::json!({"fullName": "Daniel Wetzel", "login": "daniel"}),
            ),
            custom_field("Priority", serde_json::json!({"name": "Major"})),
        ];

        let rendered = render_issue_list_text(&[issue], &list_opts());

        assert!(rendered.contains("DWP-14 | Fix login"));
        assert!(rendered.contains("  Project: DWP"));
        assert!(rendered.contains("  State: Open"));
        assert!(rendered.contains("  Assignee: Daniel Wetzel (daniel)"));
        assert!(rendered.contains("  Priority: Major"));
        assert!(rendered.contains("  Updated: 2026-02-25 06:20 UTC"));
    }

    #[test]
    fn render_issue_list_text_recognizes_german_field_aliases() {
        let mut issue = sample_issue("2-14252", Some("DWP-14"), Some("Fix login"));
        issue.custom_fields = vec![
            custom_field("Zustand", serde_json::json!({"name": "Offen"})),
            custom_field("Priorität", serde_json::json!({"name": "Hoch"})),
            custom_field("Schätzung", serde_json::json!({"minutes": 150})),
        ];

        let rendered = render_issue_list_text(&[issue], &list_opts());

        assert!(rendered.contains("  State: Offen"));
        assert!(rendered.contains("  Priority: Hoch"));
        assert!(rendered.contains("  Estimation: 2h30m"));
        assert!(!rendered.contains("Resolved: no"));
    }

    #[test]
    fn render_issue_list_text_no_meta_hides_metadata_but_keeps_work_fields() {
        let mut issue = sample_issue("2-14252", Some("DWP-14"), Some("Fix login"));
        issue.project = Some(ProjectRef {
            id: "0-1".into(),
            short_name: Some("DWP".into()),
            name: None,
        });
        issue.updated = Some(1_772_000_400_000);
        issue.custom_fields = vec![custom_field(
            "Priority",
            serde_json::json!({"name": "Major"}),
        )];
        let opts = OutputOptions {
            format: format::Format::Text,
            no_meta: true,
        };

        let rendered = render_issue_list_text(&[issue], &opts);

        assert!(rendered.starts_with("Fix login\n"));
        assert!(rendered.contains("  Priority: Major"));
        assert!(!rendered.contains("DWP-14"));
        assert!(!rendered.contains("Project:"));
        assert!(!rendered.contains("Updated:"));
    }

    #[test]
    fn render_issue_detail_text_prints_all_custom_fields_and_comments() {
        let mut issue = sample_issue("2-14252", Some("DWP-14"), Some("Fix login"));
        issue.description = Some("## Context\n\nLine **one**\nLine two".into());
        issue.created = Some(1_772_000_000_000);
        issue.updated = Some(1_772_000_400_000);
        issue.reporter = Some(User {
            id: "1-1".into(),
            login: "alice".into(),
            full_name: Some("Alice Example".into()),
            email: None,
            banned: None,
            guest: None,
        });
        issue.tags = vec![Tag {
            id: Some("tag-1".into()),
            name: "backend".into(),
        }];
        issue.custom_fields = vec![
            custom_field("State", serde_json::json!({"name": "Open"})),
            custom_field(
                "Subsystem",
                serde_json::json!([{"name": "Auth"}, {"name": "API"}]),
            ),
            custom_field("Spent time", serde_json::json!({"presentation": "1h 30m"})),
        ];
        issue.comments = vec![IssueComment {
            id: "4-17".into(),
            text: Some("Looks **reproducible**".into()),
            created: Some(1_772_000_500_000),
            updated: None,
            author: Some(User {
                id: "1-2".into(),
                login: "bob".into(),
                full_name: Some("Bob Example".into()),
                email: None,
                banned: None,
                guest: None,
            }),
            visibility: None,
            attachments: vec![],
        }];

        let rendered = render_issue_detail_text(&issue, "DWP-14", &list_opts(), true);

        assert!(rendered.contains("DWP-14 | Fix login"));
        assert!(rendered.contains("\nContext\n\nLine one\nLine two\n"));
        assert!(rendered.contains("Custom Fields\n  State: Open\n  Subsystem: Auth, API"));
        assert!(rendered.contains("Comments\n  DWP-14:4-17 | Bob Example (bob)"));
        assert!(rendered.contains("  Looks reproducible"));
        assert!(rendered.contains("Metadata\n  id: DWP-14\n  ytId: 2-14252"));
        assert!(!rendered.contains("Description\n"));

        let metadata = rendered.find("Metadata").unwrap();
        let description = rendered.find("Context").unwrap();
        let comments = rendered.find("Comments").unwrap();
        assert!(metadata < description);
        assert!(description < comments);
    }

    #[test]
    fn render_issue_detail_text_can_omit_comments() {
        let mut issue = sample_issue("2-14252", Some("DWP-14"), Some("Fix login"));
        issue.comments = vec![IssueComment {
            id: "4-17".into(),
            text: Some("Looks reproducible".into()),
            created: None,
            updated: None,
            author: None,
            visibility: None,
            attachments: vec![],
        }];

        let rendered = render_issue_detail_text(&issue, "DWP-14", &list_opts(), false);

        assert!(!rendered.contains("Comments"));
        assert!(!rendered.contains("Looks reproducible"));
        assert!(!rendered.contains("DWP-14:4-17"));
    }

    #[test]
    fn render_issue_detail_text_hides_unlimited_visibility() {
        let mut issue = sample_issue("2-14252", Some("DWP-14"), Some("Fix login"));
        issue.visibility = Some(LimitedVisibility {
            visibility_type: Some("UnlimitedVisibility".into()),
            permitted_groups: vec![],
        });

        let rendered = render_issue_detail_text(&issue, "DWP-14", &list_opts(), true);

        assert!(!rendered.contains("Visibility:"));
        assert!(!rendered.contains("UnlimitedVisibility"));
    }

    #[test]
    fn render_issue_detail_text_prints_limited_visibility_groups() {
        let mut issue = sample_issue("2-14252", Some("DWP-14"), Some("Fix login"));
        issue.visibility = Some(LimitedVisibility {
            visibility_type: Some("LimitedVisibility".into()),
            permitted_groups: vec![UserGroup {
                id: "3-7".into(),
                name: "Team Alpha".into(),
                users_count: None,
            }],
        });

        let rendered = render_issue_detail_text(&issue, "DWP-14", &list_opts(), true);

        assert!(rendered.contains("  Visibility: Team Alpha"));
    }

    #[test]
    fn render_issue_links_text_handles_missing_summaries() {
        let links = vec![IssueLink {
            id: Some("105-0".into()),
            direction: Some("BOTH".into()),
            link_type: Some(IssueLinkType {
                id: Some("105-0".into()),
                name: Some("Relates".into()),
                source_to_target: None,
                target_to_source: None,
                ..Default::default()
            }),
            issues: Some(vec![sample_issue("2-14252", Some("DWP-14"), None)]),
        }];

        let rendered = render_issue_links_text(&links);
        assert!(rendered.contains("- DWP-14\n"));
    }

    #[test]
    fn render_issue_links_text_with_no_populated_links() {
        let links = vec![IssueLink {
            id: Some("105-0".into()),
            direction: Some("BOTH".into()),
            link_type: Some(IssueLinkType {
                id: Some("105-0".into()),
                name: Some("Relates".into()),
                source_to_target: None,
                target_to_source: None,
                ..Default::default()
            }),
            issues: Some(vec![]),
        }];

        let rendered = render_issue_links_text(&links);
        assert_eq!(rendered, "No linked tickets.\n");
    }

    #[test]
    fn find_issue_link_to_unlink_matches_source_to_target() {
        let links = vec![IssueLink {
            id: Some("80-0".into()),
            direction: Some("OUTWARD".into()),
            link_type: Some(IssueLinkType {
                id: Some("74-0".into()),
                name: Some("Relates".into()),
                source_to_target: Some("relates to".into()),
                target_to_source: Some("relates to".into()),
                ..Default::default()
            }),
            issues: Some(vec![sample_issue("2-14", Some("DWP-14"), Some("Target"))]),
        }];

        let requested = resolved_test_link_type("Relates", "74-0", None, "relates to");
        let found = find_issue_link_to_unlink(&links, "DWP-12", "DWP-14", &requested).unwrap();

        assert_eq!(found, ("80-0", "2-14"));
    }

    #[test]
    fn find_issue_link_to_unlink_matches_target_to_source() {
        let links = vec![IssueLink {
            id: Some("80-0".into()),
            direction: Some("INWARD".into()),
            link_type: Some(IssueLinkType {
                id: Some("74-0".into()),
                name: Some("Depend".into()),
                source_to_target: Some("is required for".into()),
                target_to_source: Some("depends on".into()),
                ..Default::default()
            }),
            issues: Some(vec![sample_issue("2-14", Some("DWP-14"), Some("Target"))]),
        }];

        let requested =
            resolved_test_link_type("Depend", "74-0t", Some(LinkDirection::Inward), "depends on");
        let found = find_issue_link_to_unlink(&links, "DWP-12", "DWP-14", &requested).unwrap();

        assert_eq!(found, ("80-0", "2-14"));
    }

    #[test]
    fn find_issue_link_to_unlink_matches_link_type_name() {
        let links = vec![IssueLink {
            id: Some("80-0".into()),
            direction: Some("OUTWARD".into()),
            link_type: Some(IssueLinkType {
                id: Some("74-0".into()),
                name: Some("Relates".into()),
                source_to_target: Some("relates to".into()),
                target_to_source: Some("relates to".into()),
                ..Default::default()
            }),
            issues: Some(vec![sample_issue("2-14", Some("DWP-14"), Some("Target"))]),
        }];

        let requested = resolved_test_link_type("Relates", "74-0", None, "relates to");
        let found = find_issue_link_to_unlink(&links, "DWP-12", "DWP-14", &requested).unwrap();

        assert_eq!(found, ("80-0", "2-14"));
    }

    #[test]
    fn find_issue_link_to_unlink_accepts_raw_target_id() {
        let links = vec![IssueLink {
            id: Some("80-0".into()),
            direction: Some("OUTWARD".into()),
            link_type: Some(IssueLinkType {
                id: Some("74-0".into()),
                name: Some("Relates".into()),
                source_to_target: Some("relates to".into()),
                target_to_source: Some("relates to".into()),
                ..Default::default()
            }),
            issues: Some(vec![sample_issue("2-14", Some("DWP-14"), Some("Target"))]),
        }];

        let requested = resolved_test_link_type("Relates", "74-0", None, "relates to");
        let found = find_issue_link_to_unlink(&links, "DWP-12", "2-14", &requested).unwrap();

        assert_eq!(found, ("80-0", "2-14"));
    }

    #[test]
    fn find_issue_link_to_unlink_errors_when_missing() {
        let links = vec![IssueLink {
            id: Some("80-0".into()),
            direction: Some("OUTWARD".into()),
            link_type: Some(IssueLinkType {
                id: Some("74-0".into()),
                name: Some("Relates".into()),
                source_to_target: Some("relates to".into()),
                target_to_source: Some("relates to".into()),
                ..Default::default()
            }),
            issues: Some(vec![sample_issue("2-14", Some("DWP-14"), Some("Target"))]),
        }];

        let requested = resolved_test_link_type("Relates", "74-0", None, "relates to");
        let error = find_issue_link_to_unlink(&links, "DWP-12", "DWP-99", &requested).unwrap_err();

        assert!(error.to_string().contains("Link not found"));
    }

    #[test]
    fn find_issue_link_to_unlink_errors_without_link_id() {
        let links = vec![IssueLink {
            id: None,
            direction: Some("OUTWARD".into()),
            link_type: Some(IssueLinkType {
                id: Some("74-0".into()),
                name: Some("Relates".into()),
                source_to_target: Some("relates to".into()),
                target_to_source: Some("relates to".into()),
                ..Default::default()
            }),
            issues: Some(vec![sample_issue("2-14", Some("DWP-14"), Some("Target"))]),
        }];

        let requested = resolved_test_link_type("Relates", "74-0", None, "relates to");
        let error = find_issue_link_to_unlink(&links, "DWP-12", "DWP-14", &requested).unwrap_err();

        assert!(error.to_string().contains("Issue link has no ID"));
    }

    #[test]
    fn find_issue_link_to_unlink_errors_without_target_database_id() {
        let links = vec![IssueLink {
            id: Some("80-0".into()),
            direction: Some("OUTWARD".into()),
            link_type: Some(IssueLinkType {
                id: Some("74-0".into()),
                name: Some("Relates".into()),
                source_to_target: Some("relates to".into()),
                target_to_source: Some("relates to".into()),
                ..Default::default()
            }),
            issues: Some(vec![sample_issue("", Some("DWP-14"), Some("Target"))]),
        }];

        let requested = resolved_test_link_type("Relates", "74-0", None, "relates to");
        let error = find_issue_link_to_unlink(&links, "DWP-12", "DWP-14", &requested).unwrap_err();

        assert!(error
            .to_string()
            .contains("Linked issue has no database ID"));
    }

    #[test]
    fn cmd_link_with_name_posts_direct_issue_link() {
        let mut flags = std::collections::HashMap::new();
        flags.insert("type".into(), "Subtask".into());
        let parsed = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("link".into()),
            positional: vec!["DWP-12".into(), "DWP-14".into()],
            flags,
        };
        let (client, transport) = test_client_with_transport(vec![
            standard_link_types_json(),
            r#"{"id":"2-14","idReadable":"DWP-14","summary":"Target"}"#,
            r#"{}"#,
        ]);

        cmd_link(&client, &parsed).unwrap();

        assert_eq!(transport.request_count(), 3);
        assert!(transport.request(0).url.contains("/api/issueLinkTypes?"));
        assert!(transport.request(1).url.contains("/api/issues/DWP-14?"));
        let request = transport.request(2);
        assert_eq!(request.method, "POST");
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/issues/DWP-12/links/80-3t/issues?fields=id"
        );
        assert!(!request.url.contains("/commands"));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&request.body.unwrap()).unwrap(),
            serde_json::json!({"id":"2-14"})
        );
    }

    #[test]
    fn cmd_link_with_legacy_phrase_posts_direct_issue_link() {
        let mut flags = std::collections::HashMap::new();
        flags.insert("type".into(), "parent for".into());
        let parsed = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("link".into()),
            positional: vec!["DWP-12".into(), "DWP-14".into()],
            flags,
        };
        let (client, transport) = test_client_with_transport(vec![
            standard_link_types_json(),
            r#"{"id":"2-14","idReadable":"DWP-14","summary":"Target"}"#,
            r#"{}"#,
        ]);

        cmd_link(&client, &parsed).unwrap();

        let request = transport.request(2);
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/issues/DWP-12/links/80-3s/issues?fields=id"
        );
    }

    #[test]
    fn cmd_link_rejects_unknown_type_before_mutating() {
        let mut flags = std::collections::HashMap::new();
        flags.insert("type".into(), "Blocks".into());
        let parsed = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("link".into()),
            positional: vec!["DWP-12".into(), "DWP-14".into()],
            flags,
        };
        let (client, transport) = test_client_with_transport(vec![standard_link_types_json()]);

        let error = cmd_link(&client, &parsed).unwrap_err();

        assert!(error.to_string().contains("ytd ticket link-types"));
        assert_eq!(transport.request_count(), 1);
    }

    #[test]
    fn cmd_unlink_with_name_and_direction_deletes_matching_link() {
        let mut flags = std::collections::HashMap::new();
        flags.insert("type".into(), "Subtask".into());
        flags.insert("direction".into(), "outward".into());
        let parsed = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("unlink".into()),
            positional: vec!["DWP-12".into(), "DWP-14".into()],
            flags,
        };
        let (client, transport) = test_client_with_transport(vec![
            r#"[{"id":"80-3s","direction":"OUTWARD","linkType":{"id":"80-3","name":"Subtask","sourceToTarget":"parent for","targetToSource":"subtask of"},"issues":[{"id":"2-14","idReadable":"DWP-14","summary":"Target"}]}]"#,
            standard_link_types_json(),
        ]);

        cmd_unlink(&client, &parsed).unwrap();

        let request = transport.request(2);
        assert_eq!(request.method, "DELETE");
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/issues/DWP-12/links/80-3s/issues/2-14"
        );
    }

    #[test]
    fn cmd_unlink_with_legacy_phrase_still_works() {
        let mut flags = std::collections::HashMap::new();
        flags.insert("type".into(), "subtask of".into());
        let parsed = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("unlink".into()),
            positional: vec!["DWP-12".into(), "DWP-14".into()],
            flags,
        };
        let (client, transport) = test_client_with_transport(vec![
            r#"[{"id":"80-3t","direction":"INWARD","linkType":{"id":"80-3","name":"Subtask","sourceToTarget":"parent for","targetToSource":"subtask of"},"issues":[{"id":"2-14","idReadable":"DWP-14","summary":"Target"}]}]"#,
            standard_link_types_json(),
        ]);

        cmd_unlink(&client, &parsed).unwrap();

        let request = transport.request(2);
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/issues/DWP-12/links/80-3t/issues/2-14"
        );
    }

    #[test]
    fn cmd_unlink_requires_target_ticket_id() {
        let parsed = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("unlink".into()),
            positional: vec!["DWP-12".into()],
            flags: Default::default(),
        };
        let client = test_client(vec![]);

        let error = cmd_unlink(&client, &parsed).unwrap_err();

        assert!(error.to_string().contains("Target ticket ID is required"));
    }

    #[test]
    fn normalize_comment_array_encodes_embedded_ticket_comment_ids() {
        let mut value = serde_json::json!({
            "id": "2-1",
            "idReadable": "DWP-12",
            "comments": [
                {"id": "4-17", "text": "Hello"}
            ]
        });

        normalize_comment_array(&mut value, "comments", CommentParentType::Ticket, "DWP-12");

        let comment = &value["comments"][0];
        assert_eq!(comment["id"], "DWP-12:4-17");
        assert_eq!(comment["ytId"], "4-17");
        assert_eq!(comment["parentType"], "ticket");
        assert_eq!(comment["parentId"], "DWP-12");
    }

    #[test]
    fn normalize_activity_comment_ids_encodes_comment_category_payloads() {
        let mut value = serde_json::json!([
            {
                "id": "act-1",
                "category": {"id": "CommentCategory"},
                "added": [
                    {
                        "id": "4-17",
                        "text": "Hello",
                        "author": {"id": "1-51", "login": "wetzel"}
                    }
                ]
            }
        ]);

        normalize_activity_comment_ids(&mut value, "DWP-12");

        let comment = &value[0]["added"][0];
        assert_eq!(comment["id"], "DWP-12:4-17");
        assert_eq!(comment["ytId"], "4-17");
        assert_eq!(comment["author"]["id"], "1-51");
        assert!(comment["author"]["ytId"].is_null());
    }

    #[test]
    fn build_create_issue_input_uses_resolved_visibility_group() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();

        let args = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("create".into()),
            positional: vec![],
            flags: [
                ("project".to_string(), "DEMO".to_string()),
                ("visibility-group".to_string(), "Team Alpha".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        let json = serde_json::json!({
            "summary": "Visible issue",
            "description": "details"
        });

        let client = test_client(vec![r#"[{"id":"3-7","name":"Team Alpha"}]"#]);
        let input = build_create_issue_input(&client, &args, &json).unwrap();
        let visibility = input.visibility.expect("visibility should be set");
        assert_eq!(visibility.visibility_type, "LimitedVisibility");
        assert_eq!(visibility.permitted_groups.len(), 1);
        assert_eq!(visibility.permitted_groups[0].id, "3-7");

        clear_env();
    }

    #[test]
    fn build_update_issue_input_clears_visibility_with_flag() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        std::env::set_var("YTD_VISIBILITY_GROUP", "Env Team");

        let args = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("update".into()),
            positional: vec!["DEMO-1".into()],
            flags: [("no-visibility-group".to_string(), "true".to_string())]
                .into_iter()
                .collect(),
        };
        let json = serde_json::json!({});

        let client = test_client(vec![]);
        let input = build_update_issue_input(&client, &args, &json).unwrap();
        let visibility = input
            .visibility
            .expect("visibility clear payload should be set");
        assert_eq!(visibility.visibility_type, "LimitedVisibility");
        assert!(visibility.permitted_groups.is_empty());

        clear_env();
    }

    #[test]
    fn build_update_issue_input_ignores_env_visibility_without_flag() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        std::env::set_var("YTD_VISIBILITY_GROUP", "Env Team");

        let args = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("update".into()),
            positional: vec!["DEMO-1".into()],
            flags: Default::default(),
        };
        let json = serde_json::json!({"description": "Updated"});

        let client = test_client(vec![]);
        let input = build_update_issue_input(&client, &args, &json).unwrap();
        assert_eq!(input.description.as_deref(), Some("Updated"));
        assert!(input.visibility.is_none());

        clear_env();
    }

    #[test]
    fn build_create_issue_input_forwards_custom_fields_and_tags() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();

        let args = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("create".into()),
            positional: vec![],
            flags: [("project".to_string(), "DEMO".to_string())]
                .into_iter()
                .collect(),
        };
        let json = serde_json::json!({
            "summary": "Custom",
            "customFields": [{
                "name": "Assignee",
                "$type": "SingleUserIssueCustomField",
                "value": {"login": "jane.doe"}
            }],
            "tags": [{"name": "backend"}]
        });

        let client = test_client(vec![]);
        let input = build_create_issue_input(&client, &args, &json).unwrap();
        assert_eq!(input.custom_fields, json.get("customFields").cloned());
        assert_eq!(input.tags, json.get("tags").cloned());

        clear_env();
    }

    #[test]
    fn build_update_issue_input_allows_only_custom_fields() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();

        let args = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("update".into()),
            positional: vec!["DEMO-1".into()],
            flags: Default::default(),
        };
        let json = serde_json::json!({
            "customFields": [{
                "name": "Priority",
                "$type": "SingleEnumIssueCustomField",
                "value": {"name": "Critical"}
            }]
        });

        let client = test_client(vec![]);
        let input = build_update_issue_input(&client, &args, &json).unwrap();
        assert_eq!(input.custom_fields, json.get("customFields").cloned());
        assert!(input.summary.is_none());
        assert!(input.description.is_none());

        clear_env();
    }

    #[test]
    fn build_create_issue_input_rejects_json_project() {
        let args = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("create".into()),
            positional: vec![],
            flags: [("project".to_string(), "DEMO".to_string())]
                .into_iter()
                .collect(),
        };
        let json = serde_json::json!({
            "summary": "Custom",
            "project": {"id": "0-1"}
        });

        let client = test_client(vec![]);
        let err = build_create_issue_input(&client, &args, &json).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Unknown ticket JSON field: project. Allowed fields: customFields, description, summary, tags"
        );
    }

    #[test]
    fn build_update_issue_input_rejects_unknown_ticket_field() {
        let args = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("update".into()),
            positional: vec!["DEMO-1".into()],
            flags: Default::default(),
        };
        let json = serde_json::json!({"unknown": true});

        let client = test_client(vec![]);
        let err = build_update_issue_input(&client, &args, &json).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Unknown ticket JSON field: unknown. Allowed fields: customFields, description, summary, tags"
        );
    }

    #[test]
    fn build_update_issue_input_rejects_empty_update_without_explicit_visibility() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        std::env::set_var("YTD_VISIBILITY_GROUP", "Env Team");

        let args = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("update".into()),
            positional: vec!["DEMO-1".into()],
            flags: Default::default(),
        };
        let json = serde_json::json!({});

        let client = test_client(vec![]);
        let err = build_update_issue_input(&client, &args, &json).unwrap_err();
        assert_eq!(
            err.to_string(),
            "At least one update field is required. Use JSON fields or explicit visibility flags."
        );

        clear_env();
    }

    #[test]
    fn build_visibility_input_omits_clear_for_create() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();

        let args = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("create".into()),
            positional: vec![],
            flags: [("no-visibility-group".to_string(), "true".to_string())]
                .into_iter()
                .collect(),
        };

        let client = test_client(vec![]);
        assert!(visibility::build_create_visibility_input(&client, &args)
            .unwrap()
            .is_none());

        clear_env();
    }

    #[test]
    fn build_comment_create_visibility_uses_env_default() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        std::env::set_var("YTD_VISIBILITY_GROUP", "Team Alpha");

        let args = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("comment".into()),
            positional: vec!["DEMO-1".into(), "hello".into()],
            flags: Default::default(),
        };
        let client = test_client(vec![r#"[{"id":"3-7","name":"Team Alpha"}]"#]);

        let visibility = visibility::build_create_visibility_input(&client, &args)
            .unwrap()
            .expect("visibility should be set");
        assert_eq!(visibility.permitted_groups[0].id, "3-7");

        clear_env();
    }

    #[test]
    fn build_comment_create_visibility_no_visibility_group_omits_default() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        std::env::set_var("YTD_VISIBILITY_GROUP", "Team Alpha");

        let args = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("comment".into()),
            positional: vec!["DEMO-1".into(), "hello".into()],
            flags: [("no-visibility-group".to_string(), "true".to_string())]
                .into_iter()
                .collect(),
        };
        let client = test_client(vec![]);

        assert!(visibility::build_create_visibility_input(&client, &args)
            .unwrap()
            .is_none());

        clear_env();
    }

    #[test]
    fn build_comment_update_visibility_ignores_env_default_without_flags() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        std::env::set_var("YTD_VISIBILITY_GROUP", "Team Alpha");

        let args = ParsedArgs {
            resource: Some("comment".into()),
            action: Some("update".into()),
            positional: vec!["DEMO-1:4-17".into(), "hello".into()],
            flags: Default::default(),
        };
        let client = test_client(vec![]);

        assert!(
            visibility::build_comment_update_visibility_input(&client, &args)
                .unwrap()
                .is_none()
        );

        clear_env();
    }

    #[test]
    fn build_comment_update_visibility_sets_group_with_flag() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();

        let args = ParsedArgs {
            resource: Some("comment".into()),
            action: Some("update".into()),
            positional: vec!["DEMO-1:4-17".into(), "hello".into()],
            flags: [("visibility-group".to_string(), "Team Alpha".to_string())]
                .into_iter()
                .collect(),
        };
        let client = test_client(vec![r#"[{"id":"3-7","name":"Team Alpha"}]"#]);

        let visibility = visibility::build_comment_update_visibility_input(&client, &args)
            .unwrap()
            .expect("visibility should be set");
        assert_eq!(visibility.permitted_groups[0].id, "3-7");

        clear_env();
    }

    #[test]
    fn build_comment_update_visibility_clears_with_flag() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        std::env::set_var("YTD_VISIBILITY_GROUP", "Team Alpha");

        let args = ParsedArgs {
            resource: Some("comment".into()),
            action: Some("update".into()),
            positional: vec!["DEMO-1:4-17".into(), "hello".into()],
            flags: [("no-visibility-group".to_string(), "true".to_string())]
                .into_iter()
                .collect(),
        };
        let client = test_client(vec![]);

        let visibility = visibility::build_comment_update_visibility_input(&client, &args)
            .unwrap()
            .expect("visibility clear payload should be set");
        assert!(visibility.permitted_groups.is_empty());

        clear_env();
    }

    #[test]
    fn build_create_issue_input_fails_for_unknown_visibility_group() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();

        let args = ParsedArgs {
            resource: Some("ticket".into()),
            action: Some("create".into()),
            positional: vec![],
            flags: [
                ("project".to_string(), "DEMO".to_string()),
                ("visibility-group".to_string(), "Missing Team".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        let json = serde_json::json!({
            "summary": "Visible issue"
        });
        let client = test_client(vec![r#"[{"id":"3-7","name":"Team Alpha"}]"#]);

        let err = build_create_issue_input(&client, &args, &json).unwrap_err();
        assert_eq!(err.to_string(), "Visibility group not found: Missing Team");

        clear_env();
    }
}

fn cmd_fields<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let issue = client.get_issue(id)?;
    format::print_items(&issue.custom_fields, opts);
    Ok(())
}

fn cmd_history<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let id = require_id(args)?;
    let category = args.flags.get("category").map(|s| s.as_str());
    let activities = client.list_activities(id, category)?;
    let mut value = serde_json::to_value(&activities)?;
    if !matches!(opts.format, format::Format::Raw) {
        normalize_activity_comment_ids(&mut value, id);
    }
    format::print_value(&value, opts);
    Ok(())
}

fn normalize_comment_array(
    value: &mut serde_json::Value,
    field: &str,
    parent_type: CommentParentType,
    parent_id: &str,
) {
    let Some(comments) = value.get_mut(field).and_then(|v| v.as_array_mut()) else {
        return;
    };

    for comment in comments {
        normalize_comment_object(comment, parent_type, parent_id);
    }
}

fn normalize_activity_comment_ids(value: &mut serde_json::Value, ticket_id: &str) {
    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                normalize_activity_comment_ids(item, ticket_id);
            }
        }
        serde_json::Value::Object(map) => {
            let is_comment_activity = map
                .get("category")
                .and_then(|v| v.get("id"))
                .and_then(|v| v.as_str())
                .map(|id| id == "CommentCategory")
                .unwrap_or(false);

            if is_comment_activity {
                for field in ["added", "removed", "target"] {
                    if let Some(value) = map.get_mut(field) {
                        normalize_nested_comment_ids(value, ticket_id);
                    }
                }
            }
        }
        _ => {}
    }
}

fn normalize_nested_comment_ids(value: &mut serde_json::Value, ticket_id: &str) {
    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                normalize_nested_comment_ids(item, ticket_id);
            }
        }
        serde_json::Value::Object(_) => {
            normalize_comment_object(value, CommentParentType::Ticket, ticket_id);
            if let serde_json::Value::Object(map) = value {
                for child in map.values_mut() {
                    normalize_nested_comment_ids(child, ticket_id);
                }
            }
        }
        _ => {}
    }
}

fn normalize_comment_object(
    value: &mut serde_json::Value,
    parent_type: CommentParentType,
    parent_id: &str,
) {
    let serde_json::Value::Object(map) = value else {
        return;
    };
    let Some(raw_id) = map
        .get("id")
        .and_then(|v| v.as_str())
        .map(|id| id.to_string())
    else {
        return;
    };

    if raw_id.contains(':') {
        return;
    }
    if !looks_like_comment_object(map) {
        return;
    }

    map.insert("ytId".into(), serde_json::Value::String(raw_id.clone()));
    map.insert(
        "id".into(),
        serde_json::Value::String(encode_comment_id(parent_id, &raw_id)),
    );
    map.insert(
        "parentType".into(),
        serde_json::Value::String(parent_type.as_str().into()),
    );
    map.insert(
        "parentId".into(),
        serde_json::Value::String(parent_id.to_string()),
    );
}

fn looks_like_comment_object(map: &serde_json::Map<String, serde_json::Value>) -> bool {
    map.contains_key("text")
        || map.contains_key("created")
        || map.contains_key("updated")
        || map.contains_key("author")
}

fn cmd_delete<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_id(args)?;
    if commands::confirm_delete(
        "ticket",
        id,
        args.flags.get("y").map(|v| v == "true").unwrap_or(false),
    )? {
        client.delete_issue(id)?;
        println!("{id}");
    }
    Ok(())
}

fn parse_date_to_epoch_ms(date: &str) -> Option<u64> {
    // Parse YYYY-MM-DD
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
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
