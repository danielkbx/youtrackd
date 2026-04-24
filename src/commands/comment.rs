use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::commands::visibility;
use crate::error::YtdError;
use crate::format::{self, OutputOptions};
use crate::types::{
    article_attachment_output, article_comment_output, issue_attachment_output,
    issue_comment_output, parse_comment_id, AttachmentOutput, CommentParentType, ParsedCommentId,
};
use std::io::{self, BufRead, IsTerminal, Write};

pub fn run<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("get") => cmd_get(client, args, opts),
        Some("update") => cmd_update(client, args),
        Some("delete") => cmd_delete(client, args),
        Some("attachments") => cmd_attachments(client, args, opts),
        _ => Err(YtdError::Input(
            "Usage: ytd comment <get|update|delete|attachments>".into(),
        )),
    }
}

fn require_comment_id(args: &ParsedArgs, usage: &str) -> Result<ParsedCommentId, YtdError> {
    let id = args
        .positional
        .first()
        .ok_or_else(|| YtdError::Input(usage.into()))?;
    parse_comment_id(id)
}

fn cmd_get<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let id = require_comment_id(args, "Usage: ytd comment get <comment-id>")?;
    match id.parent_type {
        CommentParentType::Ticket => {
            let comment = client.get_issue_comment(&id.parent_id, &id.comment_id)?;
            let output = issue_comment_output(&id.parent_id, comment);
            format::print_single(&output, opts);
        }
        CommentParentType::Article => {
            let comment = client.get_article_comment(&id.parent_id, &id.comment_id)?;
            let output = article_comment_output(&id.parent_id, comment);
            format::print_single(&output, opts);
        }
    }
    Ok(())
}

fn cmd_update<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_comment_id(args, "Usage: ytd comment update <comment-id> <text>")?;
    let text = args
        .positional
        .get(1..)
        .map(|s| s.join(" "))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| YtdError::Input("Comment text is required".into()))?;
    let visibility = visibility::build_comment_update_visibility_input(client, args)?;

    match id.parent_type {
        CommentParentType::Ticket => {
            client.update_issue_comment(&id.parent_id, &id.comment_id, &text, visibility)?;
        }
        CommentParentType::Article => {
            client.update_article_comment(&id.parent_id, &id.comment_id, &text, visibility)?;
        }
    }

    println!("{}", args.positional[0]);
    Ok(())
}

fn cmd_delete<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_comment_id(args, "Usage: ytd comment delete <comment-id> [-y]")?;
    let encoded = args.positional[0].as_str();
    if args.flags.get("y").map(|v| v == "true").unwrap_or(false) || confirm_delete(encoded)? {
        match id.parent_type {
            CommentParentType::Ticket => {
                client.delete_issue_comment(&id.parent_id, &id.comment_id)?;
            }
            CommentParentType::Article => {
                client.delete_article_comment(&id.parent_id, &id.comment_id)?;
            }
        }
        println!("{encoded}");
    }
    Ok(())
}

fn cmd_attachments<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let id = require_comment_id(args, "Usage: ytd comment attachments <comment-id>")?;
    let attachments: Vec<AttachmentOutput> = match id.parent_type {
        CommentParentType::Ticket => client
            .get_issue_comment_with_attachments(&id.parent_id, &id.comment_id)?
            .attachments
            .into_iter()
            .map(|attachment| issue_attachment_output(&id.parent_id, attachment))
            .collect(),
        CommentParentType::Article => client
            .get_article_comment_with_attachments(&id.parent_id, &id.comment_id)?
            .attachments
            .into_iter()
            .map(|attachment| article_attachment_output(&id.parent_id, attachment))
            .collect(),
    };
    format::print_items(&attachments, opts);
    Ok(())
}

fn confirm_delete(id: &str) -> Result<bool, YtdError> {
    if !io::stdin().is_terminal() {
        return Ok(true);
    }
    print!("Delete comment {id}? [y/N] ");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim().eq_ignore_ascii_case("y"))
}
