use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::config;
use crate::error::YtdError;
use crate::format::{self, OutputOptions};
use crate::input;
use crate::types::*;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

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
            let json = input::read_json_input(&args.flags)?;
            let input = build_create_article_input(client, args, &json)?;
            let article = client.create_article(&input)?;
            println!("{}", article.id_readable.unwrap_or(article.id));
            Ok(())
        }
        Some("update") => {
            let id = args.positional.first()
                .ok_or_else(|| YtdError::Input("Usage: ytd article update <id> --json '...'".into()))?;
            let json = input::read_json_input(&args.flags)?;
            let input = build_update_article_input(client, args, &json)?;
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
            let comments: Vec<CommentOutput> = comments
                .into_iter()
                .map(|comment| article_comment_output(id, comment))
                .collect();
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

fn build_create_article_input<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    json: &serde_json::Value,
) -> Result<CreateArticleInput, YtdError> {
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
        visibility: build_visibility_input(client, args, false)?,
    })
}

fn build_update_article_input<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    json: &serde_json::Value,
) -> Result<UpdateArticleInput, YtdError> {
    Ok(UpdateArticleInput {
        summary: json
            .get("summary")
            .and_then(|v| v.as_str())
            .map(String::from),
        content: json
            .get("content")
            .and_then(|v| v.as_str())
            .map(String::from),
        visibility: build_visibility_input(client, args, true)?,
    })
}

fn build_visibility_input<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    is_update: bool,
) -> Result<Option<LimitedVisibilityInput>, YtdError> {
    match config::resolve_visibility_group(
        args.flags.get("visibility-group").map(|s| s.as_str()),
        args.flags.contains_key("no-visibility-group"),
    )? {
        ResolvedVisibilityGroup::Group(group) => Ok(Some(LimitedVisibilityInput {
            visibility_type: "LimitedVisibility",
            permitted_groups: vec![UserGroupInput {
                id: resolve_group_id(client, &group)?,
            }],
        })),
        ResolvedVisibilityGroup::Clear if is_update => Ok(Some(LimitedVisibilityInput {
            visibility_type: "LimitedVisibility",
            permitted_groups: vec![],
        })),
        ResolvedVisibilityGroup::Clear | ResolvedVisibilityGroup::None => Ok(None),
    }
}

fn resolve_group_id<T: HttpTransport>(
    client: &YtClient<T>,
    group_name: &str,
) -> Result<String, YtdError> {
    let groups = client.list_groups()?;

    if let Some(group) = groups.iter().find(|group| group.name == group_name) {
        return Ok(group.id.clone());
    }

    if let Some(group) = groups
        .iter()
        .find(|group| group.name.eq_ignore_ascii_case(group_name))
    {
        return Ok(group.id.clone());
    }

    Err(YtdError::Input(format!(
        "Visibility group not found: {group_name}"
    )))
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
