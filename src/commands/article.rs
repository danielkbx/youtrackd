use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::commands;
use crate::commands::visibility;
use crate::error::YtdError;
use crate::format::{self, OutputOptions};
use crate::input;
use crate::types::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

pub fn run<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("search") => {
            let query = args.positional.first()
                .ok_or_else(|| YtdError::Input("Usage: ytd article search <query>".into()))?;
            let project = args.flags.get("project").map(|s| s.as_str());
            let articles = client.search_articles(query, project)?;
            let outputs: Vec<ArticleOutput> = articles.iter().cloned().map(article_output).collect();
            format::print_raw_or_processed_items(&articles, &outputs, opts)?;
            Ok(())
        }
        Some("list") => {
            let project = args.flags.get("project")
                .ok_or_else(|| YtdError::Input("--project is required".into()))?;
            let articles = client.list_articles(project)?;
            let outputs: Vec<ArticleOutput> = articles.iter().cloned().map(article_output).collect();
            format::print_raw_or_processed_items(&articles, &outputs, opts)?;
            Ok(())
        }
        Some("get") => {
            let id = args.positional.first()
                .ok_or_else(|| YtdError::Input("Usage: ytd article get <id> [--no-comments]".into()))?;
            let article = client.get_article(id)?;
            if matches!(opts.format, format::Format::Raw) {
                let mut value = serde_json::to_value(&article)?;
                remove_comments_if_requested(&mut value, args);
                format::print_value(&value, opts);
            } else {
                let mut value = serde_json::to_value(article_output(article))?;
                remove_comments_if_requested(&mut value, args);
                format::print_value(&value, opts);
            }
            Ok(())
        }
        Some("create") => {
            let json = input::read_json_input(&args.flags)?;
            let input = build_create_article_input(client, args, &json)?;
            let article = client.create_article(&input)?;
            println!("{}", article.id_readable.unwrap_or(article.id));
            Ok(())
        }
        Some("update") => {
            let id = args.positional.first().ok_or_else(|| {
                YtdError::Input("Usage: ytd article update <id> --json '...'".into())
            })?;
            let json = input::read_json_input(&args.flags)?;
            let input = build_update_article_input(client, args, &json)?;
            let article = client.update_article(id, &input)?;
            println!("{}", article.id_readable.unwrap_or(article.id));
            Ok(())
        }
        Some("move") => {
            let id = args.positional.first().ok_or_else(|| {
                YtdError::Input("Usage: ytd article move <id> <parent-id|none>".into())
            })?;
            let parent = args
                .positional
                .get(1)
                .ok_or_else(|| YtdError::Input("Parent article ID or none is required".into()))?;
            if args.positional.len() > 2 {
                return Err(YtdError::Input(
                    "Usage: ytd article move <id> <parent-id|none>".into(),
                ));
            }
            let input = build_move_article_input(client, parent)?;
            let article = client.update_article(id, &input)?;
            println!("{}", article.id_readable.unwrap_or(article.id));
            Ok(())
        }
        Some("dump") => cmd_dump(client, args),
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
            let visibility = visibility::build_create_visibility_input(client, args)?;
            client.add_article_comment(id, &text, visibility)?;
            Ok(())
        }
        Some("comments") => {
            let id = args.positional.first()
                .ok_or_else(|| YtdError::Input("Usage: ytd article comments <id>".into()))?;
            let comments = client.list_article_comments(id)?;
            let outputs: Vec<CommentOutput> = comments
                .iter()
                .cloned()
                .map(|comment| article_comment_output(id, comment))
                .collect();
            format::print_raw_or_processed_items(&comments, &outputs, opts)?;
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
            let outputs: Vec<AttachmentOutput> = attachments
                .iter()
                .cloned()
                .map(|attachment| article_attachment_output(id, attachment))
                .collect();
            format::print_raw_or_processed_items(&attachments, &outputs, opts)?;
            Ok(())
        }
        Some("delete") => {
            let id = args.positional.first()
                .ok_or_else(|| YtdError::Input("Usage: ytd article delete <id> [-y]".into()))?;
            if commands::confirm_delete(
                "article",
                id,
                args.flags.get("y").map(|v| v == "true").unwrap_or(false),
            )? {
                client.delete_article(id)?;
                println!("{id}");
            }
            Ok(())
        }
        _ => Err(YtdError::Input("Usage: ytd article <search|list|get|create|update|move|dump|append|comment|comments|attach|attachments|delete>".into())),
    }
}

fn cmd_dump<T: HttpTransport>(client: &YtClient<T>, args: &ParsedArgs) -> Result<(), YtdError> {
    let project = args
        .flags
        .get("project")
        .ok_or_else(|| YtdError::Input("Usage: ytd article dump --project <id> <dir>".into()))?;
    let dir = args
        .positional
        .first()
        .ok_or_else(|| YtdError::Input("Usage: ytd article dump --project <id> <dir>".into()))?;
    if args.positional.len() > 1 {
        return Err(YtdError::Input(
            "Usage: ytd article dump --project <id> <dir>".into(),
        ));
    }

    let root = Path::new(dir);
    fs::create_dir_all(root)?;

    let summaries = client.list_articles(project)?;
    let mut articles = Vec::with_capacity(summaries.len());
    for article in summaries {
        let id = article.id_readable.as_deref().unwrap_or(&article.id);
        articles.push(client.get_article(id)?);
    }

    let dumped = dump_articles(root, articles)?;
    println!("{dumped}");
    Ok(())
}

fn remove_comments_if_requested(value: &mut serde_json::Value, args: &ParsedArgs) {
    if !args
        .flags
        .get("no-comments")
        .map(|value| value == "true")
        .unwrap_or(false)
    {
        return;
    }
    if let Some(obj) = value.as_object_mut() {
        obj.remove("comments");
    }
}

fn build_create_article_input<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    json: &serde_json::Value,
) -> Result<CreateArticleInput, YtdError> {
    validate_article_json_fields(json, &["content", "parentArticle", "summary"])?;
    let project = args
        .flags
        .get("project")
        .ok_or_else(|| YtdError::Input("--project is required".into()))?;
    let summary = json
        .get("summary")
        .and_then(|v| v.as_str())
        .ok_or_else(|| YtdError::Input("summary is required".into()))?;

    Ok(CreateArticleInput {
        project: ProjectRef {
            id: String::new(),
            short_name: Some(project.clone()),
            name: None,
        },
        summary: summary.to_string(),
        content: json
            .get("content")
            .and_then(|v| v.as_str())
            .map(String::from),
        visibility: visibility::build_create_visibility_input(client, args)?,
        parent_article: match json.get("parentArticle") {
            Some(value) if !value.is_null() => Some(build_parent_article_input(client, value)?),
            _ => None,
        },
    })
}

fn build_update_article_input<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    json: &serde_json::Value,
) -> Result<UpdateArticleInput, YtdError> {
    validate_article_json_fields(json, &["content", "parentArticle", "summary"])?;
    let summary = json
        .get("summary")
        .and_then(|v| v.as_str())
        .map(String::from);
    let content = json
        .get("content")
        .and_then(|v| v.as_str())
        .map(String::from);
    let visibility = visibility::build_explicit_update_visibility_input(client, args)?;
    let parent_article = match json.get("parentArticle") {
        Some(value) if value.is_null() => Some(None),
        Some(value) => Some(Some(build_parent_article_input(client, value)?)),
        None => None,
    };

    if summary.is_none() && content.is_none() && visibility.is_none() && parent_article.is_none() {
        return Err(YtdError::Input(
            "At least one update field is required. Use JSON fields or explicit visibility flags."
                .into(),
        ));
    }

    Ok(UpdateArticleInput {
        summary,
        content,
        visibility,
        parent_article,
    })
}

fn build_move_article_input<T: HttpTransport>(
    client: &YtClient<T>,
    parent: &str,
) -> Result<UpdateArticleInput, YtdError> {
    let parent_article = if parent == "none" {
        Some(None)
    } else {
        Some(Some(build_parent_article_input(
            client,
            &serde_json::json!({ "id": parent }),
        )?))
    };

    Ok(UpdateArticleInput {
        summary: None,
        content: None,
        visibility: None,
        parent_article,
    })
}

fn validate_article_json_fields(
    json: &serde_json::Value,
    allowed: &[&str],
) -> Result<(), YtdError> {
    let obj = json
        .as_object()
        .ok_or_else(|| YtdError::Input("JSON input must be an object".into()))?;
    let mut unknown: Vec<&str> = obj
        .keys()
        .map(String::as_str)
        .filter(|key| !allowed.contains(key))
        .collect();
    unknown.sort_unstable();
    if unknown.is_empty() {
        return Ok(());
    }
    Err(YtdError::Input(format!(
        "Unknown article JSON field{}: {}. Allowed fields: {}",
        if unknown.len() == 1 { "" } else { "s" },
        unknown.join(", "),
        allowed.join(", ")
    )))
}

fn build_parent_article_input<T: HttpTransport>(
    client: &YtClient<T>,
    value: &serde_json::Value,
) -> Result<ArticleParentInput, YtdError> {
    let obj = value
        .as_object()
        .ok_or_else(|| YtdError::Input("parentArticle must be an object with id".into()))?;

    if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
        if id.trim().is_empty() {
            return Err(YtdError::Input("parentArticle.id must not be empty".into()));
        }
        let article = client.get_article(id)?;
        return Ok(ArticleParentInput { id: article.id });
    }

    if let Some(yt_id) = obj.get("ytId").and_then(|v| v.as_str()) {
        if yt_id.trim().is_empty() {
            return Err(YtdError::Input(
                "parentArticle.ytId must not be empty".into(),
            ));
        }
        return Ok(ArticleParentInput {
            id: yt_id.to_string(),
        });
    }

    Err(YtdError::Input(
        "parentArticle must include id or ytId".into(),
    ))
}

#[derive(Debug, Clone)]
struct ArticleDumpItem {
    id: String,
    yt_id: String,
    summary: String,
    content: String,
    updated: Option<u64>,
    parent_id: Option<String>,
    parent_yt_id: Option<String>,
    parent_summary: Option<String>,
}

impl From<Article> for ArticleDumpItem {
    fn from(article: Article) -> Self {
        let parent = article.parent_article.map(article_ref_output);
        Self {
            id: article.id_readable.unwrap_or_else(|| article.id.clone()),
            yt_id: article.id,
            summary: article.summary.unwrap_or_else(|| "Untitled".into()),
            content: article.content.unwrap_or_default(),
            updated: article.updated,
            parent_id: parent.as_ref().map(|parent| parent.id.clone()),
            parent_yt_id: parent.as_ref().map(|parent| parent.yt_id.clone()),
            parent_summary: parent.and_then(|parent| parent.summary),
        }
    }
}

fn dump_articles(root: &Path, articles: Vec<Article>) -> Result<usize, YtdError> {
    let items: Vec<ArticleDumpItem> = articles.into_iter().map(ArticleDumpItem::from).collect();
    let by_id: HashMap<String, ArticleDumpItem> = items
        .iter()
        .cloned()
        .map(|article| (article.id.clone(), article))
        .collect();
    let by_yt_id: HashMap<String, String> = items
        .iter()
        .map(|article| (article.yt_id.clone(), article.id.clone()))
        .collect();

    let mut used_paths = HashSet::new();
    for article in &items {
        let path = article_dump_path(article, root, &by_id, &by_yt_id, &mut vec![]);
        let unique = unique_markdown_path(path, &mut used_paths);
        if let Some(parent) = unique.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(&unique)?;
        file.write_all(render_article_dump(article).as_bytes())?;
    }

    Ok(items.len())
}

fn article_dump_path(
    article: &ArticleDumpItem,
    root: &Path,
    by_id: &HashMap<String, ArticleDumpItem>,
    by_yt_id: &HashMap<String, String>,
    stack: &mut Vec<String>,
) -> PathBuf {
    let mut path = if stack.contains(&article.id) {
        root.to_path_buf()
    } else if let Some(parent_id) = resolve_dump_parent_id(article, by_id, by_yt_id) {
        if let Some(parent) = by_id.get(parent_id) {
            stack.push(article.id.clone());
            let mut parent_path = article_dump_path(parent, root, by_id, by_yt_id, stack);
            stack.pop();
            parent_path.set_extension("");
            parent_path
        } else {
            root.to_path_buf()
        }
    } else {
        root.to_path_buf()
    };

    path.push(article_dump_stem(article));
    path.set_extension("md");
    path
}

fn resolve_dump_parent_id<'a>(
    article: &'a ArticleDumpItem,
    by_id: &'a HashMap<String, ArticleDumpItem>,
    by_yt_id: &'a HashMap<String, String>,
) -> Option<&'a String> {
    if let Some(parent_id) = &article.parent_id {
        if by_id.contains_key(parent_id) {
            return Some(parent_id);
        }
    }
    article
        .parent_yt_id
        .as_ref()
        .and_then(|id| by_yt_id.get(id))
}

fn article_dump_stem(article: &ArticleDumpItem) -> String {
    sanitize_path_segment(&format!("{} - {}", article.id, article.summary))
}

fn sanitize_path_segment(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut last_was_space = false;
    for ch in value.chars() {
        let replacement = match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            ch if ch.is_control() => '-',
            ch => ch,
        };

        if replacement.is_whitespace() {
            if !last_was_space {
                out.push(' ');
            }
            last_was_space = true;
        } else {
            out.push(replacement);
            last_was_space = false;
        }
    }

    let trimmed = out.trim().trim_matches('.').to_string();
    if trimmed.is_empty() {
        "untitled".into()
    } else {
        trimmed.chars().take(120).collect()
    }
}

fn unique_markdown_path(path: PathBuf, used: &mut HashSet<PathBuf>) -> PathBuf {
    if used.insert(path.clone()) {
        return path;
    }

    let parent = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("article");

    for index in 2.. {
        let candidate = parent.join(format!("{stem} ({index}).md"));
        if used.insert(candidate.clone()) {
            return candidate;
        }
    }
    unreachable!()
}

fn render_article_dump(article: &ArticleDumpItem) -> String {
    let mut out = String::new();
    out.push_str("# ");
    out.push_str(&article.summary);
    out.push_str("\n\n<!--\n");
    out.push_str(&format!("id: {}\n", article.id));
    out.push_str(&format!("ytId: {}\n", article.yt_id));
    if let Some(parent) = &article.parent_id {
        out.push_str(&format!("parent: {parent}\n"));
    }
    if let Some(parent_yt_id) = &article.parent_yt_id {
        out.push_str(&format!("parentYtId: {parent_yt_id}\n"));
    }
    if let Some(parent_summary) = &article.parent_summary {
        out.push_str(&format!("parentSummary: {parent_summary}\n"));
    }
    if let Some(updated) = article.updated {
        out.push_str(&format!("updated: {updated}\n"));
    }
    out.push_str("-->\n\n");
    out.push_str(article.content.trim());
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::ParsedArgs;
    use crate::config::TEST_ENV_LOCK;
    use crate::types::YtdConfig;
    use std::cell::RefCell;
    use std::path::Path;

    struct MockTransport {
        responses: RefCell<Vec<String>>,
    }

    impl MockTransport {
        fn new(responses: Vec<&str>) -> Self {
            Self {
                responses: RefCell::new(responses.into_iter().rev().map(String::from).collect()),
            }
        }
    }

    impl HttpTransport for MockTransport {
        fn get(&self, _url: &str, _token: &str) -> Result<String, YtdError> {
            self.responses
                .borrow_mut()
                .pop()
                .ok_or_else(|| YtdError::Http("No more mock responses".into()))
        }

        fn get_bytes(&self, _url: &str, _token: &str) -> Result<Vec<u8>, YtdError> {
            Err(YtdError::Http("unused".into()))
        }

        fn post(&self, _url: &str, _token: &str, _body: &str) -> Result<String, YtdError> {
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

        fn delete(&self, _url: &str, _token: &str) -> Result<(), YtdError> {
            Ok(())
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

    fn clear_env() {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("YTD_CONFIG");
        std::env::remove_var("YTD_VISIBILITY_GROUP");
    }

    #[test]
    fn no_comments_flag_removes_article_comments_from_output_value() {
        let mut value = serde_json::json!({
            "id": "6-1",
            "idReadable": "DWP-A-1",
            "summary": "Runbook",
            "comments": [{"id": "251-0", "text": "Internal note"}]
        });
        let args = ParsedArgs {
            resource: Some("article".into()),
            action: Some("get".into()),
            positional: vec!["DWP-A-1".into()],
            flags: [("no-comments".to_string(), "true".to_string())]
                .into_iter()
                .collect(),
        };

        remove_comments_if_requested(&mut value, &args);

        assert!(value.get("comments").is_none());
    }

    #[test]
    fn article_comments_remain_by_default() {
        let mut value = serde_json::json!({
            "id": "6-1",
            "idReadable": "DWP-A-1",
            "summary": "Runbook",
            "comments": [{"id": "251-0", "text": "Internal note"}]
        });
        let args = ParsedArgs {
            resource: Some("article".into()),
            action: Some("get".into()),
            positional: vec!["DWP-A-1".into()],
            flags: Default::default(),
        };

        remove_comments_if_requested(&mut value, &args);

        assert!(value.get("comments").is_some());
    }

    #[test]
    fn sanitize_path_segment_replaces_unsafe_characters() {
        assert_eq!(
            sanitize_path_segment(" DWP-A-1: Root/Article? "),
            "DWP-A-1- Root-Article-"
        );
        assert_eq!(sanitize_path_segment("..."), "untitled");
    }

    #[test]
    fn dump_articles_preserves_parent_hierarchy() {
        let dir = tempfile::tempdir().unwrap();
        let articles = vec![
            Article {
                id: "109-1".into(),
                id_readable: Some("DWP-A-1".into()),
                summary: Some("Root".into()),
                content: Some("Root body".into()),
                created: None,
                updated: Some(1),
                reporter: None,
                project: None,
                visibility: None,
                parent_article: None,
            },
            Article {
                id: "109-2".into(),
                id_readable: Some("DWP-A-2".into()),
                summary: Some("Child".into()),
                content: Some("Child body".into()),
                created: None,
                updated: Some(2),
                reporter: None,
                project: None,
                visibility: None,
                parent_article: Some(ArticleRef {
                    id: "109-1".into(),
                    id_readable: Some("DWP-A-1".into()),
                    summary: Some("Root".into()),
                }),
            },
        ];

        let count = dump_articles(dir.path(), articles).unwrap();

        assert_eq!(count, 2);
        let root_file = dir.path().join("DWP-A-1 - Root.md");
        let child_file = dir.path().join("DWP-A-1 - Root").join("DWP-A-2 - Child.md");
        assert!(root_file.exists(), "missing {}", root_file.display());
        assert!(child_file.exists(), "missing {}", child_file.display());
        let child = std::fs::read_to_string(child_file).unwrap();
        assert!(child.contains("# Child"));
        assert!(child.contains("parent: DWP-A-1"));
        assert!(child.contains("Child body"));
    }

    #[test]
    fn dump_articles_disambiguates_duplicate_paths() {
        let dir = tempfile::tempdir().unwrap();
        let articles = vec![
            Article {
                id: "109-1".into(),
                id_readable: Some("DWP-A-1".into()),
                summary: Some("Same".into()),
                content: None,
                created: None,
                updated: None,
                reporter: None,
                project: None,
                visibility: None,
                parent_article: None,
            },
            Article {
                id: "109-2".into(),
                id_readable: Some("DWP-A-1".into()),
                summary: Some("Same".into()),
                content: None,
                created: None,
                updated: None,
                reporter: None,
                project: None,
                visibility: None,
                parent_article: None,
            },
        ];

        dump_articles(dir.path(), articles).unwrap();

        assert!(dir.path().join("DWP-A-1 - Same.md").exists());
        assert!(dir.path().join("DWP-A-1 - Same (2).md").exists());
    }

    #[test]
    fn build_create_article_input_uses_env_visibility_group() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        std::env::set_var("YTD_VISIBILITY_GROUP", "Docs Team");

        let args = ParsedArgs {
            resource: Some("article".into()),
            action: Some("create".into()),
            positional: vec![],
            flags: [("project".to_string(), "DOC".to_string())]
                .into_iter()
                .collect(),
        };
        let json = serde_json::json!({
            "summary": "Restricted article",
            "content": "content"
        });

        let client = test_client(vec![r#"[{"id":"3-8","name":"Docs Team"}]"#]);
        let input = build_create_article_input(&client, &args, &json).unwrap();
        let visibility = input.visibility.expect("visibility should be set");
        assert_eq!(visibility.permitted_groups.len(), 1);
        assert_eq!(visibility.permitted_groups[0].id, "3-8");

        clear_env();
    }

    #[test]
    fn build_update_article_input_clears_visibility_with_flag() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        std::env::set_var("YTD_VISIBILITY_GROUP", "Docs Team");

        let args = ParsedArgs {
            resource: Some("article".into()),
            action: Some("update".into()),
            positional: vec!["DOC-A-1".into()],
            flags: [("no-visibility-group".to_string(), "true".to_string())]
                .into_iter()
                .collect(),
        };
        let json = serde_json::json!({});

        let client = test_client(vec![]);
        let input = build_update_article_input(&client, &args, &json).unwrap();
        let visibility = input
            .visibility
            .expect("visibility clear payload should be set");
        assert!(visibility.permitted_groups.is_empty());

        clear_env();
    }

    #[test]
    fn build_update_article_input_ignores_env_visibility_without_flag() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        std::env::set_var("YTD_VISIBILITY_GROUP", "Docs Team");

        let args = ParsedArgs {
            resource: Some("article".into()),
            action: Some("update".into()),
            positional: vec!["DOC-A-1".into()],
            flags: Default::default(),
        };
        let json = serde_json::json!({"content": "Updated"});

        let client = test_client(vec![]);
        let input = build_update_article_input(&client, &args, &json).unwrap();
        assert_eq!(input.content.as_deref(), Some("Updated"));
        assert!(input.visibility.is_none());

        clear_env();
    }

    #[test]
    fn build_create_article_input_resolves_parent_article_readable_id() {
        let args = ParsedArgs {
            resource: Some("article".into()),
            action: Some("create".into()),
            positional: vec![],
            flags: [
                ("project".to_string(), "DOC".to_string()),
                ("no-visibility-group".to_string(), "true".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        let json = serde_json::json!({
            "summary": "Child",
            "parentArticle": {"id": "DOC-A-1"}
        });
        let client = test_client(vec![
            r#"{"id":"109-812","idReadable":"DOC-A-1","summary":"Parent"}"#,
        ]);

        let input = build_create_article_input(&client, &args, &json).unwrap();

        let value = serde_json::to_value(input).unwrap();
        assert_eq!(value["parentArticle"], serde_json::json!({"id": "109-812"}));
    }

    #[test]
    fn build_update_article_input_resolves_parent_article_readable_id() {
        let args = ParsedArgs {
            resource: Some("article".into()),
            action: Some("update".into()),
            positional: vec!["DOC-A-2".into()],
            flags: Default::default(),
        };
        let json = serde_json::json!({
            "parentArticle": {"id": "DOC-A-1"}
        });
        let client = test_client(vec![
            r#"{"id":"109-812","idReadable":"DOC-A-1","summary":"Parent"}"#,
        ]);

        let input = build_update_article_input(&client, &args, &json).unwrap();

        let value = serde_json::to_value(input).unwrap();
        assert_eq!(value["parentArticle"], serde_json::json!({"id": "109-812"}));
    }

    #[test]
    fn build_update_article_input_clears_parent_article_with_null() {
        let args = ParsedArgs {
            resource: Some("article".into()),
            action: Some("update".into()),
            positional: vec!["DOC-A-2".into()],
            flags: Default::default(),
        };
        let json = serde_json::json!({"parentArticle": null});

        let client = test_client(vec![]);
        let input = build_update_article_input(&client, &args, &json).unwrap();
        let value = serde_json::to_value(input).unwrap();

        assert!(value.get("parentArticle").unwrap().is_null());
    }

    #[test]
    fn build_move_article_input_resolves_parent_article_readable_id() {
        let client = test_client(vec![
            r#"{"id":"109-812","idReadable":"DOC-A-1","summary":"Parent"}"#,
        ]);

        let input = build_move_article_input(&client, "DOC-A-1").unwrap();
        let value = serde_json::to_value(input).unwrap();

        assert_eq!(value["parentArticle"], serde_json::json!({"id": "109-812"}));
    }

    #[test]
    fn build_move_article_input_clears_parent_with_none() {
        let client = test_client(vec![]);

        let input = build_move_article_input(&client, "none").unwrap();
        let value = serde_json::to_value(input).unwrap();

        assert!(value.get("parentArticle").unwrap().is_null());
    }

    #[test]
    fn build_create_article_input_rejects_unknown_json_fields() {
        let args = ParsedArgs {
            resource: Some("article".into()),
            action: Some("create".into()),
            positional: vec![],
            flags: [("project".to_string(), "DOC".to_string())]
                .into_iter()
                .collect(),
        };
        let json = serde_json::json!({
            "summary": "Child",
            "foo": "bar"
        });

        let client = test_client(vec![]);
        let err = build_create_article_input(&client, &args, &json).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Unknown article JSON field: foo. Allowed fields: content, parentArticle, summary"
        );
    }

    #[test]
    fn build_update_article_input_rejects_unknown_json_fields() {
        let args = ParsedArgs {
            resource: Some("article".into()),
            action: Some("update".into()),
            positional: vec!["DOC-A-2".into()],
            flags: Default::default(),
        };
        let json = serde_json::json!({
            "content": "Updated",
            "foo": "bar"
        });

        let client = test_client(vec![]);
        let err = build_update_article_input(&client, &args, &json).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Unknown article JSON field: foo. Allowed fields: content, parentArticle, summary"
        );
    }

    #[test]
    fn build_parent_article_input_rejects_invalid_shapes() {
        let client = test_client(vec![]);

        let err = build_parent_article_input(&client, &serde_json::json!("DOC-A-1")).unwrap_err();
        assert_eq!(err.to_string(), "parentArticle must be an object with id");

        let err = build_parent_article_input(&client, &serde_json::json!({"id": ""})).unwrap_err();
        assert_eq!(err.to_string(), "parentArticle.id must not be empty");

        let err = build_parent_article_input(&client, &serde_json::json!({})).unwrap_err();
        assert_eq!(err.to_string(), "parentArticle must include id or ytId");
    }

    #[test]
    fn build_update_article_input_rejects_empty_update_without_explicit_visibility() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        std::env::set_var("YTD_VISIBILITY_GROUP", "Docs Team");

        let args = ParsedArgs {
            resource: Some("article".into()),
            action: Some("update".into()),
            positional: vec!["DOC-A-1".into()],
            flags: Default::default(),
        };
        let json = serde_json::json!({});

        let client = test_client(vec![]);
        let err = build_update_article_input(&client, &args, &json).unwrap_err();
        assert_eq!(
            err.to_string(),
            "At least one update field is required. Use JSON fields or explicit visibility flags."
        );

        clear_env();
    }

    #[test]
    fn build_create_article_input_fails_for_unknown_visibility_group() {
        let _lock = TEST_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        clear_env();
        std::env::set_var("YTD_VISIBILITY_GROUP", "Missing Team");

        let args = ParsedArgs {
            resource: Some("article".into()),
            action: Some("create".into()),
            positional: vec![],
            flags: [("project".to_string(), "DOC".to_string())]
                .into_iter()
                .collect(),
        };
        let json = serde_json::json!({
            "summary": "Restricted article"
        });
        let client = test_client(vec![r#"[{"id":"3-8","name":"Docs Team"}]"#]);

        let err = build_create_article_input(&client, &args, &json).unwrap_err();
        assert_eq!(err.to_string(), "Visibility group not found: Missing Team");

        clear_env();
    }
}
