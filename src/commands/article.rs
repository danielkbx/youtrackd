use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::commands;
use crate::commands::visibility;
use crate::error::YtdError;
use crate::format::{self, OutputOptions};
use crate::input;
use crate::types::*;
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
        _ => Err(YtdError::Input("Usage: ytd article <search|list|get|create|update|move|append|comment|comments|attach|attachments|delete>".into())),
    }
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
