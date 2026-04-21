use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtdConfig {
    pub url: String,
    pub token: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleComment {
    pub id: String,
    pub text: Option<String>,
    pub created: Option<u64>,
    pub updated: Option<u64>,
    pub author: Option<User>,
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
    pub author: Option<User>,
}

// --- Tags ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: Option<String>,
    pub name: String,
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
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArticleInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIssueInput {
    pub project: ProjectRef,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIssueInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
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
}
