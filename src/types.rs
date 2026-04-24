use crate::error::YtdError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtdConfig {
    pub url: String,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedVisibilityGroup {
    Group(String),
    Clear,
    None,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility_group: Option<String>,
}

impl StoredConfig {
    pub fn is_empty(&self) -> bool {
        self.url.is_none() && self.token.is_none() && self.visibility_group.is_none()
    }
}

// --- Users ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub login: String,
    pub full_name: Option<String>,
    pub email: Option<String>,
}

// --- Projects ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub archived: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRef {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// --- Articles ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    pub id: String,
    #[serde(default)]
    pub id_readable: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub created: Option<u64>,
    pub updated: Option<u64>,
    pub reporter: Option<User>,
    pub project: Option<ProjectRef>,
    pub visibility: Option<LimitedVisibility>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleComment {
    pub id: String,
    pub text: Option<String>,
    pub created: Option<u64>,
    pub updated: Option<u64>,
    pub author: Option<User>,
    pub visibility: Option<LimitedVisibility>,
}

// --- Issues ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub id: String,
    #[serde(default)]
    pub id_readable: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub created: Option<u64>,
    pub updated: Option<u64>,
    pub resolved: Option<u64>,
    pub reporter: Option<User>,
    pub project: Option<ProjectRef>,
    pub visibility: Option<LimitedVisibility>,
    #[serde(default)]
    pub tags: Vec<Tag>,
    #[serde(default)]
    pub comments: Vec<IssueComment>,
    #[serde(default, rename = "customFields")]
    pub custom_fields: Vec<CustomField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueComment {
    pub id: String,
    pub text: Option<String>,
    pub created: Option<u64>,
    pub updated: Option<u64>,
    pub author: Option<User>,
    pub visibility: Option<LimitedVisibility>,
}

// --- Comments ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentParentType {
    Ticket,
    Article,
}

impl CommentParentType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ticket => "ticket",
            Self::Article => "article",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCommentId {
    pub parent_type: CommentParentType,
    pub parent_id: String,
    pub comment_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentOutput {
    pub id: String,
    pub yt_id: String,
    pub parent_type: String,
    pub parent_id: String,
    pub text: Option<String>,
    pub created: Option<u64>,
    pub updated: Option<u64>,
    pub author: Option<User>,
    pub visibility: Option<LimitedVisibility>,
}

pub fn encode_comment_id(parent_id: &str, comment_id: &str) -> String {
    format!("{parent_id}:{comment_id}")
}

pub fn parse_comment_id(value: &str) -> Result<ParsedCommentId, YtdError> {
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(invalid_comment_id());
    }

    let parent_type = infer_comment_parent_type(parts[0])?;

    Ok(ParsedCommentId {
        parent_type,
        parent_id: parts[0].to_string(),
        comment_id: parts[1].to_string(),
    })
}

fn infer_comment_parent_type(parent_id: &str) -> Result<CommentParentType, YtdError> {
    let parts: Vec<&str> = parent_id.split('-').collect();
    if parts.len() == 3
        && !parts[0].is_empty()
        && parts[1] == "A"
        && parts[2].parse::<u64>().is_ok()
    {
        return Ok(CommentParentType::Article);
    }
    if parts.len() == 2 && !parts[0].is_empty() && parts[1].parse::<u64>().is_ok() {
        return Ok(CommentParentType::Ticket);
    }
    Err(invalid_comment_id())
}

pub fn issue_comment_output(issue_id: &str, comment: IssueComment) -> CommentOutput {
    comment_output(CommentParentType::Ticket, issue_id, comment)
}

pub fn article_comment_output(article_id: &str, comment: ArticleComment) -> CommentOutput {
    comment_output(CommentParentType::Article, article_id, comment)
}

fn comment_output<C>(parent_type: CommentParentType, parent_id: &str, comment: C) -> CommentOutput
where
    C: Into<CommentParts>,
{
    let comment = comment.into();
    CommentOutput {
        id: encode_comment_id(parent_id, &comment.id),
        yt_id: comment.id,
        parent_type: parent_type.as_str().to_string(),
        parent_id: parent_id.to_string(),
        text: comment.text,
        created: comment.created,
        updated: comment.updated,
        author: comment.author,
        visibility: comment.visibility,
    }
}

struct CommentParts {
    id: String,
    text: Option<String>,
    created: Option<u64>,
    updated: Option<u64>,
    author: Option<User>,
    visibility: Option<LimitedVisibility>,
}

impl From<IssueComment> for CommentParts {
    fn from(comment: IssueComment) -> Self {
        Self {
            id: comment.id,
            text: comment.text,
            created: comment.created,
            updated: comment.updated,
            author: comment.author,
            visibility: comment.visibility,
        }
    }
}

impl From<ArticleComment> for CommentParts {
    fn from(comment: ArticleComment) -> Self {
        Self {
            id: comment.id,
            text: comment.text,
            created: comment.created,
            updated: comment.updated,
            author: comment.author,
            visibility: comment.visibility,
        }
    }
}

fn invalid_comment_id() -> YtdError {
    YtdError::Input(
        "Invalid comment ID. Expected <ticket-id>:<comment-id> or <article-id>:<comment-id>".into(),
    )
}

// --- Tags ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: Option<String>,
    pub name: String,
}

// --- Groups ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserGroup {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub users_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitedVisibility {
    #[serde(rename = "$type")]
    pub visibility_type: Option<String>,
    #[serde(default)]
    pub permitted_groups: Vec<UserGroup>,
}

// --- Issue Links ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueLink {
    pub id: Option<String>,
    pub direction: Option<String>,
    pub link_type: Option<IssueLinkType>,
    pub issues: Option<Vec<Issue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueLinkType {
    pub id: Option<String>,
    pub name: Option<String>,
    pub source_to_target: Option<String>,
    pub target_to_source: Option<String>,
}

// --- Attachments ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: String,
    pub name: Option<String>,
    pub url: Option<String>,
    pub size: Option<u64>,
    pub mime_type: Option<String>,
    pub created: Option<u64>,
    pub author: Option<User>,
}

// --- Time Tracking ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItem {
    pub id: String,
    pub duration: Option<WorkItemDuration>,
    pub date: Option<u64>,
    pub text: Option<String>,
    pub author: Option<User>,
    #[serde(rename = "type")]
    pub work_type: Option<WorkType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItemDuration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minutes: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// --- Custom Fields ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomField {
    #[serde(default)]
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "$type")]
    pub field_type: Option<String>,
    pub value: Option<serde_json::Value>,
}

// --- Saved Searches ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedQuery {
    pub id: String,
    pub name: Option<String>,
    pub query: Option<String>,
}

// --- Activities ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityItem {
    pub id: Option<String>,
    pub timestamp: Option<u64>,
    pub author: Option<User>,
    pub target: Option<serde_json::Value>,
    pub field: Option<ActivityField>,
    pub added: Option<serde_json::Value>,
    pub removed: Option<serde_json::Value>,
    pub category: Option<ActivityCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityField {
    pub presentation: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityCategory {
    pub id: Option<String>,
}

// --- Agile Boards ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agile {
    pub id: String,
    pub name: Option<String>,
    #[serde(default)]
    pub projects: Vec<ProjectRef>,
    #[serde(default)]
    pub sprints: Vec<Sprint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sprint {
    pub id: String,
    pub name: Option<String>,
    pub start: Option<u64>,
    pub finish: Option<u64>,
    pub archived: Option<bool>,
}

// --- Input structs ---

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateArticleInput {
    pub project: ProjectRef,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<LimitedVisibilityInput>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArticleInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<LimitedVisibilityInput>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIssueInput {
    pub project: ProjectRef,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<LimitedVisibilityInput>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIssueInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<LimitedVisibilityInput>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitedVisibilityInput {
    #[serde(rename = "$type")]
    pub visibility_type: &'static str,
    pub permitted_groups: Vec<UserGroupInput>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserGroupInput {
    pub id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkItemInput {
    pub duration: WorkItemDuration,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub work_type: Option<WorkType>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentInput {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<LimitedVisibilityInput>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_ticket_comment_id() {
        assert_eq!(encode_comment_id("DWP-12", "4-17"), "DWP-12:4-17");
    }

    #[test]
    fn parses_ticket_comment_id() {
        let parsed = parse_comment_id("DWP-12:4-17").unwrap();
        assert_eq!(parsed.parent_type, CommentParentType::Ticket);
        assert_eq!(parsed.parent_id, "DWP-12");
        assert_eq!(parsed.comment_id, "4-17");
    }

    #[test]
    fn parses_article_comment_id() {
        let parsed = parse_comment_id("DWP-A-3:251-0").unwrap();
        assert_eq!(parsed.parent_type, CommentParentType::Article);
        assert_eq!(parsed.parent_id, "DWP-A-3");
        assert_eq!(parsed.comment_id, "251-0");
    }

    #[test]
    fn rejects_invalid_comment_ids() {
        for value in [
            "issue:DWP-12:4-17",
            "ticket::4-17",
            "ticket:DWP-12:",
            "DWP-A-3",
            "DWP-A-B:251-0",
            "DWP-ABC:4-17",
            "DWP-12:4-17:extra",
        ] {
            assert!(parse_comment_id(value).is_err(), "{value} should fail");
        }
    }
}
