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
    #[serde(default)]
    pub attachments: Vec<Attachment>,
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
    #[serde(default)]
    pub attachments: Vec<Attachment>,
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
    pub comment: Option<CommentRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentRef {
    pub id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentParentType {
    Ticket,
    Article,
}

impl AttachmentParentType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ticket => "ticket",
            Self::Article => "article",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAttachmentId {
    pub parent_type: AttachmentParentType,
    pub parent_id: String,
    pub attachment_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentOutput {
    pub id: String,
    pub yt_id: String,
    pub parent_type: String,
    pub parent_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_id: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub size: Option<u64>,
    pub mime_type: Option<String>,
    pub created: Option<u64>,
    pub author: Option<User>,
}

pub fn encode_attachment_id(parent_id: &str, attachment_id: &str) -> String {
    format!("{parent_id}:{attachment_id}")
}

pub fn parse_attachment_id(value: &str) -> Result<ParsedAttachmentId, YtdError> {
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(invalid_attachment_id());
    }

    let parent_type = infer_attachment_parent_type(parts[0])?;

    Ok(ParsedAttachmentId {
        parent_type,
        parent_id: parts[0].to_string(),
        attachment_id: parts[1].to_string(),
    })
}

fn infer_attachment_parent_type(parent_id: &str) -> Result<AttachmentParentType, YtdError> {
    let parts: Vec<&str> = parent_id.split('-').collect();
    if parts.len() == 3
        && !parts[0].is_empty()
        && parts[1] == "A"
        && parts[2].parse::<u64>().is_ok()
    {
        return Ok(AttachmentParentType::Article);
    }
    if parts.len() == 2 && !parts[0].is_empty() && parts[1].parse::<u64>().is_ok() {
        return Ok(AttachmentParentType::Ticket);
    }
    Err(invalid_attachment_id())
}

pub fn issue_attachment_output(issue_id: &str, attachment: Attachment) -> AttachmentOutput {
    attachment_output(AttachmentParentType::Ticket, issue_id, attachment)
}

pub fn article_attachment_output(article_id: &str, attachment: Attachment) -> AttachmentOutput {
    attachment_output(AttachmentParentType::Article, article_id, attachment)
}

fn attachment_output(
    parent_type: AttachmentParentType,
    parent_id: &str,
    attachment: Attachment,
) -> AttachmentOutput {
    let comment_id = attachment
        .comment
        .as_ref()
        .map(|comment| encode_comment_id(parent_id, &comment.id));
    AttachmentOutput {
        id: encode_attachment_id(parent_id, &attachment.id),
        yt_id: attachment.id,
        parent_type: parent_type.as_str().to_string(),
        parent_id: parent_id.to_string(),
        comment_id,
        name: attachment.name,
        url: attachment.url,
        size: attachment.size,
        mime_type: attachment.mime_type,
        created: attachment.created,
        author: attachment.author,
    }
}

fn invalid_attachment_id() -> YtdError {
    YtdError::Input(
        "Invalid attachment ID. Expected <ticket-id>:<attachment-id> or <article-id>:<attachment-id>"
            .into(),
    )
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
    pub owner: Option<User>,
    #[serde(default)]
    pub projects: Vec<ProjectRef>,
    #[serde(default)]
    pub sprints: Vec<Sprint>,
    pub current_sprint: Option<Sprint>,
    pub orphans_at_the_top: Option<bool>,
    pub hide_orphans_swimlane: Option<bool>,
    pub estimation_field: Option<serde_json::Value>,
    pub original_estimation_field: Option<serde_json::Value>,
    pub column_settings: Option<serde_json::Value>,
    pub swimlane_settings: Option<serde_json::Value>,
    pub sprints_settings: Option<serde_json::Value>,
    pub color_coding: Option<serde_json::Value>,
    pub status: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sprint {
    pub id: String,
    pub name: Option<String>,
    pub agile: Option<AgileRef>,
    #[serde(default)]
    pub issues: Vec<Issue>,
    pub goal: Option<String>,
    pub start: Option<u64>,
    pub finish: Option<u64>,
    pub archived: Option<bool>,
    pub is_default: Option<bool>,
    pub unresolved_issues_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgileRef {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSprintId {
    pub board_id: String,
    pub sprint_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SprintOutput {
    pub id: String,
    pub yt_id: String,
    pub board_id: String,
    pub name: Option<String>,
    pub agile: Option<AgileRef>,
    pub goal: Option<String>,
    pub start: Option<u64>,
    pub finish: Option<u64>,
    pub archived: Option<bool>,
    pub is_default: Option<bool>,
    pub unresolved_issues_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentSprintOutput {
    pub id: String,
    pub yt_id: String,
    pub board_id: String,
    pub board_name: Option<String>,
    pub projects: Vec<ProjectRef>,
    pub name: Option<String>,
    pub goal: Option<String>,
    pub start: Option<u64>,
    pub finish: Option<u64>,
    pub archived: Option<bool>,
    pub is_default: Option<bool>,
    pub unresolved_issues_count: Option<i64>,
}

pub fn encode_sprint_id(board_id: &str, sprint_id: &str) -> String {
    format!("{board_id}:{sprint_id}")
}

pub fn parse_sprint_id(value: &str) -> Result<ParsedSprintId, YtdError> {
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(invalid_sprint_id());
    }
    if parts[1] == "current" {
        return Err(YtdError::Input(
            "`current` is not a sprint-id. Use `ytd sprint current` instead.".into(),
        ));
    }

    Ok(ParsedSprintId {
        board_id: parts[0].to_string(),
        sprint_id: parts[1].to_string(),
    })
}

fn invalid_sprint_id() -> YtdError {
    YtdError::Input(
        "Invalid sprint-id. Use the id returned by ytd sprint commands or ytd ticket sprints."
            .into(),
    )
}

pub fn sprint_output(board_id: &str, sprint: Sprint) -> SprintOutput {
    SprintOutput {
        id: encode_sprint_id(board_id, &sprint.id),
        yt_id: sprint.id,
        board_id: board_id.to_string(),
        name: sprint.name,
        agile: sprint.agile,
        goal: sprint.goal,
        start: sprint.start,
        finish: sprint.finish,
        archived: sprint.archived,
        is_default: sprint.is_default,
        unresolved_issues_count: sprint.unresolved_issues_count,
    }
}

pub fn sprint_output_from_agile(sprint: Sprint) -> Result<SprintOutput, YtdError> {
    let board_id = sprint
        .agile
        .as_ref()
        .map(|agile| agile.id.clone())
        .ok_or_else(|| YtdError::Input("Sprint response is missing agile.id".into()))?;
    Ok(sprint_output(&board_id, sprint))
}

pub fn current_sprint_output(board: &Agile) -> Option<CurrentSprintOutput> {
    let sprint = board.current_sprint.as_ref()?;
    Some(CurrentSprintOutput {
        id: encode_sprint_id(&board.id, &sprint.id),
        yt_id: sprint.id.clone(),
        board_id: board.id.clone(),
        board_name: board.name.clone(),
        projects: board.projects.clone(),
        name: sprint.name.clone(),
        goal: sprint.goal.clone(),
        start: sprint.start,
        finish: sprint.finish,
        archived: sprint.archived,
        is_default: sprint.is_default,
        unresolved_issues_count: sprint.unresolved_issues_count,
    })
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

    #[test]
    fn encodes_ticket_attachment_id() {
        assert_eq!(encode_attachment_id("DWP-12", "8-2897"), "DWP-12:8-2897");
    }

    #[test]
    fn parses_ticket_attachment_id() {
        let parsed = parse_attachment_id("DWP-12:8-2897").unwrap();
        assert_eq!(parsed.parent_type, AttachmentParentType::Ticket);
        assert_eq!(parsed.parent_id, "DWP-12");
        assert_eq!(parsed.attachment_id, "8-2897");
    }

    #[test]
    fn parses_article_attachment_id() {
        let parsed = parse_attachment_id("DWP-A-3:237-3").unwrap();
        assert_eq!(parsed.parent_type, AttachmentParentType::Article);
        assert_eq!(parsed.parent_id, "DWP-A-3");
        assert_eq!(parsed.attachment_id, "237-3");
    }

    #[test]
    fn rejects_invalid_attachment_ids() {
        for value in [
            "issue:DWP-12:8-2897",
            "DWP-A-3",
            "DWP-A-B:237-3",
            "DWP-ABC:8-2897",
            "DWP-12:8-2897:extra",
        ] {
            assert!(parse_attachment_id(value).is_err(), "{value} should fail");
        }
    }

    #[test]
    fn attachment_output_preserves_raw_id_and_encodes_comment_id() {
        let output = issue_attachment_output(
            "DWP-12",
            Attachment {
                id: "8-2897".into(),
                name: Some("log.txt".into()),
                url: None,
                size: Some(12),
                mime_type: Some("text/plain".into()),
                created: Some(1),
                author: None,
                comment: Some(CommentRef { id: "4-17".into() }),
            },
        );

        assert_eq!(output.id, "DWP-12:8-2897");
        assert_eq!(output.yt_id, "8-2897");
        assert_eq!(output.parent_type, "ticket");
        assert_eq!(output.parent_id, "DWP-12");
        assert_eq!(output.comment_id.as_deref(), Some("DWP-12:4-17"));
    }

    #[test]
    fn encodes_sprint_id() {
        assert_eq!(encode_sprint_id("108-4", "113-6"), "108-4:113-6");
    }

    #[test]
    fn parses_sprint_id() {
        let parsed = parse_sprint_id("108-4:113-6").unwrap();
        assert_eq!(parsed.board_id, "108-4");
        assert_eq!(parsed.sprint_id, "113-6");
    }

    #[test]
    fn rejects_invalid_sprint_ids() {
        for value in ["108-4", ":113-6", "108-4:", "108-4:113-6:extra"] {
            assert!(parse_sprint_id(value).is_err(), "{value} should fail");
        }
    }

    #[test]
    fn rejects_current_as_sprint_id() {
        let err = parse_sprint_id("108-4:current").unwrap_err();
        assert!(err.to_string().contains("sprint current"));
    }

    #[test]
    fn sprint_output_preserves_raw_id() {
        let output = sprint_output(
            "108-4",
            Sprint {
                id: "113-6".into(),
                name: Some("Sprint 1".into()),
                agile: None,
                issues: vec![],
                goal: None,
                start: Some(1),
                finish: Some(2),
                archived: Some(false),
                is_default: Some(true),
                unresolved_issues_count: Some(3),
            },
        );

        assert_eq!(output.id, "108-4:113-6");
        assert_eq!(output.yt_id, "113-6");
        assert_eq!(output.board_id, "108-4");
    }

    #[test]
    fn sprint_output_from_agile_requires_agile_id() {
        let sprint = Sprint {
            id: "113-6".into(),
            name: None,
            agile: None,
            issues: vec![],
            goal: None,
            start: None,
            finish: None,
            archived: None,
            is_default: None,
            unresolved_issues_count: None,
        };

        assert!(sprint_output_from_agile(sprint).is_err());
    }

    #[test]
    fn current_sprint_output_includes_board_context() {
        let board = Agile {
            id: "108-4".into(),
            name: Some("Board".into()),
            owner: None,
            projects: vec![ProjectRef {
                id: "0-96".into(),
                short_name: Some("DWP".into()),
                name: Some("DW Playground".into()),
            }],
            sprints: vec![],
            current_sprint: Some(Sprint {
                id: "113-6".into(),
                name: Some("Sprint 1".into()),
                agile: None,
                issues: vec![],
                goal: None,
                start: None,
                finish: None,
                archived: None,
                is_default: None,
                unresolved_issues_count: None,
            }),
            orphans_at_the_top: None,
            hide_orphans_swimlane: None,
            estimation_field: None,
            original_estimation_field: None,
            column_settings: None,
            swimlane_settings: None,
            sprints_settings: None,
            color_coding: None,
            status: None,
        };

        let output = current_sprint_output(&board).unwrap();
        assert_eq!(output.id, "108-4:113-6");
        assert_eq!(output.board_name.as_deref(), Some("Board"));
        assert_eq!(output.projects[0].short_name.as_deref(), Some("DWP"));
    }
}
