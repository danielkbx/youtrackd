use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::commands;
use crate::commands::visibility;
use crate::error::YtdError;
use crate::format::{self, OutputOptions};
use crate::types::{
    article_attachment_output, article_comment_output, issue_attachment_output,
    issue_comment_output, parse_comment_id, AttachmentOutput, CommentParentType, ParsedCommentId,
};
use std::path::Path;

pub fn run<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("get") => cmd_get(client, args, opts),
        Some("update") => cmd_update(client, args),
        Some("attach") => cmd_attach(client, args),
        Some("delete") => cmd_delete(client, args),
        Some("attachments") => cmd_attachments(client, args, opts),
        _ => Err(YtdError::Input(
            "Usage: ytd comment <get|update|attach|attachments|delete>".into(),
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
            let output = issue_comment_output(&id.parent_id, comment.clone());
            format::print_raw_or_processed(&comment, &output, opts)?;
        }
        CommentParentType::Article => {
            let comment = client.get_article_comment(&id.parent_id, &id.comment_id)?;
            let output = article_comment_output(&id.parent_id, comment.clone());
            format::print_raw_or_processed(&comment, &output, opts)?;
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

fn cmd_attach<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_comment_id(args, "Usage: ytd comment attach <comment-id> <file>")?;
    let file = args
        .positional
        .get(1)
        .ok_or_else(|| YtdError::Input("File path is required".into()))?;
    let path = Path::new(file);
    if !path.exists() {
        return Err(YtdError::Input(format!("File not found: {file}")));
    }

    match id.parent_type {
        CommentParentType::Ticket => {
            client.upload_issue_comment_attachment(&id.parent_id, &id.comment_id, path)?;
        }
        CommentParentType::Article => {
            client.upload_article_comment_attachment(&id.parent_id, &id.comment_id, path)?;
        }
    }

    println!(
        "Attached {}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or(file)
    );
    Ok(())
}

fn cmd_delete<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_comment_id(args, "Usage: ytd comment delete <comment-id> [-y]")?;
    let encoded = args.positional[0].as_str();
    if commands::confirm_delete(
        "comment",
        encoded,
        args.flags.get("y").map(|v| v == "true").unwrap_or(false),
    )? {
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
    let attachments = match id.parent_type {
        CommentParentType::Ticket => {
            client
                .get_issue_comment_with_attachments(&id.parent_id, &id.comment_id)?
                .attachments
        }
        CommentParentType::Article => {
            client
                .get_article_comment_with_attachments(&id.parent_id, &id.comment_id)?
                .attachments
        }
    };
    let outputs: Vec<AttachmentOutput> = match id.parent_type {
        CommentParentType::Ticket => attachments
            .iter()
            .cloned()
            .map(|attachment| issue_attachment_output(&id.parent_id, attachment))
            .collect(),
        CommentParentType::Article => attachments
            .iter()
            .cloned()
            .map(|attachment| article_attachment_output(&id.parent_id, attachment))
            .collect(),
    };
    format::print_raw_or_processed_items(&attachments, &outputs, opts)?;
    Ok(())
}
