use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::error::YtdError;
use crate::format::{self, OutputOptions};
use crate::input;
use crate::types::*;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

pub fn run<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs, opts: &OutputOptions) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("search") => {
            let query = args.positional.first()
                .ok_or_else(|| YtdError::Input("Usage: ytd article search <query>".into()))?;
            let project = args.flags.get("project").map(|s| s.as_str());
            let articles = client.search_articles(query, project)?;
            format::print_items(&articles, opts);
            Ok(())
        }
        Some("list") => {
            let project = args.flags.get("project")
                .ok_or_else(|| YtdError::Input("--project is required".into()))?;
            let articles = client.list_articles(project)?;
            format::print_items(&articles, opts);
            Ok(())
        }
        Some("get") => {
            let id = args.positional.first()
                .ok_or_else(|| YtdError::Input("Usage: ytd article get <id>".into()))?;
            let article = client.get_article(id)?;
            format::print_single(&article, opts);
            Ok(())
        }
        Some("create") => {
            let project = args.flags.get("project")
                .ok_or_else(|| YtdError::Input("--project is required".into()))?;
            let json = input::read_json_input(&args.flags)?;
            let summary = json.get("summary").and_then(|v| v.as_str())
                .ok_or_else(|| YtdError::Input("summary is required".into()))?;
            let content = json.get("content").and_then(|v| v.as_str());
            let input = CreateArticleInput {
                project: ProjectRef { id: String::new(), short_name: Some(project.clone()), name: None },
                summary: summary.to_string(),
                content: content.map(String::from),
            };
            let article = client.create_article(&input)?;
            println!("{}", article.id_readable.unwrap_or(article.id));
            Ok(())
        }
        Some("update") => {
            let id = args.positional.first()
                .ok_or_else(|| YtdError::Input("Usage: ytd article update <id> --json '...'".into()))?;
            let json = input::read_json_input(&args.flags)?;
            let input = UpdateArticleInput {
                summary: json.get("summary").and_then(|v| v.as_str()).map(String::from),
                content: json.get("content").and_then(|v| v.as_str()).map(String::from),
            };
            let article = client.update_article(id, &input)?;
            println!("{}", article.id_readable.unwrap_or(article.id));
            Ok(())
        }
        Some("append") => {
            let id = args.positional.first()
                .ok_or_else(|| YtdError::Input("Usage: ytd article append <id> <text>".into()))?;
            let text = args.positional.get(1..)
                .map(|s| s.join(" "))
                .filter(|s| !s.is_empty())
                .ok_or_else(|| YtdError::Input("Text is required".into()))?;
            client.append_to_article(id, &text)?;
            Ok(())
        }
        Some("comment") => {
            let id = args.positional.first()
                .ok_or_else(|| YtdError::Input("Usage: ytd article comment <id> <text>".into()))?;
            let text = args.positional.get(1..)
                .map(|s| s.join(" "))
                .filter(|s| !s.is_empty())
                .ok_or_else(|| YtdError::Input("Comment text is required".into()))?;
            client.add_article_comment(id, &text)?;
            Ok(())
        }
        Some("comments") => {
            let id = args.positional.first()
                .ok_or_else(|| YtdError::Input("Usage: ytd article comments <id>".into()))?;
            let comments = client.list_article_comments(id)?;
            format::print_items(&comments, opts);
            Ok(())
        }
        Some("attach") => {
            let id = args.positional.first()
                .ok_or_else(|| YtdError::Input("Usage: ytd article attach <id> <file>".into()))?;
            let file = args.positional.get(1)
                .ok_or_else(|| YtdError::Input("File path is required".into()))?;
            let path = Path::new(file);
            if !path.exists() {
                return Err(YtdError::Input(format!("File not found: {file}")));
            }
            client.upload_article_attachment(id, path)?;
            println!("Attached {}", path.file_name().and_then(|n| n.to_str()).unwrap_or(file));
            Ok(())
        }
        Some("attachments") => {
            let id = args.positional.first()
                .ok_or_else(|| YtdError::Input("Usage: ytd article attachments <id>".into()))?;
            let attachments = client.list_article_attachments(id)?;
            format::print_items(&attachments, opts);
            Ok(())
        }
        Some("delete") => {
            let id = args.positional.first()
                .ok_or_else(|| YtdError::Input("Usage: ytd article delete <id> [-y]".into()))?;
            if args.flags.get("y").map(|v| v == "true").unwrap_or(false) || confirm_delete("article", id)? {
                client.delete_article(id)?;
                println!("{id}");
            }
            Ok(())
        }
        _ => Err(YtdError::Input("Usage: ytd article <search|list|get|create|update|append|comment|comments|attach|attachments|delete>".into())),
    }
}

fn confirm_delete(entity_type: &str, id: &str) -> Result<bool, YtdError> {
    if !io::stdin().is_terminal() {
        return Ok(true);
    }
    print!("Delete {entity_type} {id}? [y/N] ");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim().eq_ignore_ascii_case("y"))
}
