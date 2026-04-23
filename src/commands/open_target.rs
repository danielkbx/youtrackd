use crate::error::YtdError;
use crate::types::YtdConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenTarget {
    Issue(String),
    Article(String),
    Project(String),
    KnowledgeBase(String),
}

pub fn parse_target(input: &str) -> Result<OpenTarget, YtdError> {
    let value = input.trim();
    if value.is_empty() {
        return Err(YtdError::Input("target cannot be empty".into()));
    }

    if let Some((project, article_no)) = value.rsplit_once("-A-") {
        if is_valid_project_key(project) && is_numeric(article_no) {
            return Ok(OpenTarget::Article(value.to_string()));
        }
        return Err(YtdError::Input(format!(
            "Unsupported target format: {value}"
        )));
    }

    if let Some(project) = value.strip_suffix("-A") {
        if is_valid_project_key(project) {
            return Ok(OpenTarget::KnowledgeBase(project.to_string()));
        }
        return Err(YtdError::Input(format!(
            "Unsupported target format: {value}"
        )));
    }

    if let Some((project, issue_no)) = value.rsplit_once('-') {
        if is_valid_project_key(project) && is_numeric(issue_no) {
            return Ok(OpenTarget::Issue(value.to_string()));
        }
        return Err(YtdError::Input(format!(
            "Unsupported target format: {value}"
        )));
    }

    if is_valid_project_key(value) {
        return Ok(OpenTarget::Project(value.to_string()));
    }

    Err(YtdError::Input(format!(
        "Unsupported target format: {value}"
    )))
}

pub fn build_url(config: &YtdConfig, target: &OpenTarget) -> String {
    let base = config.url.trim_end_matches('/');
    match target {
        OpenTarget::Issue(id) => format!("{base}/issue/{id}"),
        OpenTarget::Article(id) => {
            let project =
                article_project(id).expect("article targets are validated before URL build");
            format!("{base}/projects/{project}/articles/{id}")
        }
        OpenTarget::Project(id) => format!("{base}/projects/{id}"),
        OpenTarget::KnowledgeBase(project) => format!("{base}/articles/{project}"),
    }
}

fn is_valid_project_key(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
}

fn is_numeric(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit())
}

fn article_project(value: &str) -> Option<&str> {
    value.rsplit_once("-A-").map(|(project, _)| project)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(url: &str) -> YtdConfig {
        YtdConfig {
            url: url.into(),
            token: "perm:test".into(),
        }
    }

    #[test]
    fn parses_issue_target() {
        assert_eq!(
            parse_target("DWP-12").unwrap(),
            OpenTarget::Issue("DWP-12".into())
        );
    }

    #[test]
    fn parses_article_target() {
        assert_eq!(
            parse_target("DWP-A-12").unwrap(),
            OpenTarget::Article("DWP-A-12".into())
        );
    }

    #[test]
    fn parses_project_target() {
        assert_eq!(
            parse_target("DWP").unwrap(),
            OpenTarget::Project("DWP".into())
        );
    }

    #[test]
    fn parses_knowledge_base_target() {
        assert_eq!(
            parse_target("DWP-A").unwrap(),
            OpenTarget::KnowledgeBase("DWP".into())
        );
    }

    #[test]
    fn rejects_invalid_targets() {
        for target in ["", "DWP-A-", "DWP-foo", "-12"] {
            assert!(parse_target(target).is_err(), "{target} should fail");
        }
    }

    #[test]
    fn builds_urls_for_all_target_types() {
        let config = cfg("https://example.youtrack.cloud");
        assert_eq!(
            build_url(&config, &OpenTarget::Issue("DWP-12".into())),
            "https://example.youtrack.cloud/issue/DWP-12"
        );
        assert_eq!(
            build_url(&config, &OpenTarget::Article("DWP-A-12".into())),
            "https://example.youtrack.cloud/projects/DWP/articles/DWP-A-12"
        );
        assert_eq!(
            build_url(&config, &OpenTarget::Project("DWP".into())),
            "https://example.youtrack.cloud/projects/DWP"
        );
        assert_eq!(
            build_url(&config, &OpenTarget::KnowledgeBase("DWP".into())),
            "https://example.youtrack.cloud/articles/DWP"
        );
    }

    #[test]
    fn trims_trailing_slash_from_base_url() {
        let config = cfg("https://example.youtrack.cloud/");
        assert_eq!(
            build_url(&config, &OpenTarget::Issue("DWP-12".into())),
            "https://example.youtrack.cloud/issue/DWP-12"
        );
    }
}
