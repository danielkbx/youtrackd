use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::error::YtdError;
use crate::format::{self, OutputOptions};
use crate::types::{
    article_attachment_output, issue_attachment_output, parse_attachment_id, AttachmentOutput,
    AttachmentParentType, ParsedAttachmentId,
};
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

pub fn run<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("get") => cmd_get(client, args, opts),
        Some("delete") => cmd_delete(client, args),
        Some("download") => cmd_download(client, args),
        _ => Err(YtdError::Input(
            "Usage: ytd attachment <get|delete|download>".into(),
        )),
    }
}

fn require_attachment_id(args: &ParsedArgs, usage: &str) -> Result<ParsedAttachmentId, YtdError> {
    let id = args
        .positional
        .first()
        .ok_or_else(|| YtdError::Input(usage.into()))?;
    parse_attachment_id(id)
}

fn cmd_get<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let id = require_attachment_id(args, "Usage: ytd attachment get <attachment-id>")?;
    let output = get_attachment_output(client, &id)?;
    format::print_single(&output, opts);
    Ok(())
}

fn cmd_delete<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_attachment_id(args, "Usage: ytd attachment delete <attachment-id> [-y]")?;
    let encoded = args.positional[0].as_str();
    if args.flags.get("y").map(|v| v == "true").unwrap_or(false) || confirm_delete(encoded)? {
        match id.parent_type {
            AttachmentParentType::Ticket => {
                client.delete_issue_attachment(&id.parent_id, &id.attachment_id)?;
            }
            AttachmentParentType::Article => {
                client.delete_article_attachment(&id.parent_id, &id.attachment_id)?;
            }
        }
        println!("{encoded}");
    }
    Ok(())
}

fn cmd_download<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let id = require_attachment_id(
        args,
        "Usage: ytd attachment download <attachment-id> [--output <path>]",
    )?;
    let output = get_attachment_output(client, &id)?;
    let url = output
        .url
        .as_deref()
        .ok_or_else(|| YtdError::Input("Attachment has no download URL".into()))?;
    let path = resolve_output_path(args.flags.get("output").map(String::as_str), &output);
    let bytes = client.download_attachment_file(url)?;
    std::fs::write(&path, bytes)?;
    println!("Downloaded {}", path.display());
    Ok(())
}

fn get_attachment_output<T: HttpTransport>(
    client: &YtClient<T>,
    id: &ParsedAttachmentId,
) -> Result<AttachmentOutput, YtdError> {
    match id.parent_type {
        AttachmentParentType::Ticket => {
            let attachment = client.get_issue_attachment(&id.parent_id, &id.attachment_id)?;
            Ok(issue_attachment_output(&id.parent_id, attachment))
        }
        AttachmentParentType::Article => {
            let attachment = client.get_article_attachment(&id.parent_id, &id.attachment_id)?;
            Ok(article_attachment_output(&id.parent_id, attachment))
        }
    }
}

fn resolve_output_path(output: Option<&str>, attachment: &AttachmentOutput) -> PathBuf {
    let file_name = attachment_file_name(attachment);
    match output {
        Some(value) => {
            let path = Path::new(value);
            if path.is_dir() {
                path.join(file_name)
            } else {
                path.to_path_buf()
            }
        }
        None => PathBuf::from(file_name),
    }
}

fn attachment_file_name(attachment: &AttachmentOutput) -> String {
    attachment
        .name
        .as_deref()
        .and_then(|name| Path::new(name).file_name())
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(String::from)
        .unwrap_or_else(|| format!("attachment-{}", attachment.yt_id))
}

fn confirm_delete(id: &str) -> Result<bool, YtdError> {
    if !io::stdin().is_terminal() {
        return Ok(true);
    }
    print!("Delete attachment {id}? [y/N] ");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim().eq_ignore_ascii_case("y"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_path_uses_attachment_name_by_default() {
        let attachment = AttachmentOutput {
            id: "DWP-1:8-1".into(),
            yt_id: "8-1".into(),
            parent_type: "ticket".into(),
            parent_id: "DWP-1".into(),
            comment_id: None,
            name: Some("notes.txt".into()),
            url: None,
            size: None,
            mime_type: None,
            created: None,
            author: None,
        };

        assert_eq!(
            resolve_output_path(None, &attachment),
            PathBuf::from("notes.txt")
        );
    }

    #[test]
    fn output_path_uses_fallback_name() {
        let attachment = AttachmentOutput {
            id: "DWP-1:8-1".into(),
            yt_id: "8-1".into(),
            parent_type: "ticket".into(),
            parent_id: "DWP-1".into(),
            comment_id: None,
            name: None,
            url: None,
            size: None,
            mime_type: None,
            created: None,
            author: None,
        };

        assert_eq!(
            resolve_output_path(None, &attachment),
            PathBuf::from("attachment-8-1")
        );
    }
}
