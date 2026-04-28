use crate::error::YtdError;
use crate::types::*;
use serde::de::DeserializeOwned;
use std::path::Path;

// --- Transport trait ---

pub trait HttpTransport {
    fn get(&self, url: &str, token: &str) -> Result<String, YtdError>;
    fn get_bytes(&self, url: &str, token: &str) -> Result<Vec<u8>, YtdError>;
    fn post(&self, url: &str, token: &str, body: &str) -> Result<String, YtdError>;
    fn post_multipart(
        &self,
        url: &str,
        token: &str,
        file_path: &Path,
        file_name: &str,
    ) -> Result<String, YtdError>;
    fn delete(&self, url: &str, token: &str) -> Result<(), YtdError>;
}

// --- ureq implementation ---

pub struct UreqTransport;

impl HttpTransport for UreqTransport {
    fn get(&self, url: &str, token: &str) -> Result<String, YtdError> {
        let response = ureq::get(url)
            .header("Authorization", &format!("Bearer {token}"))
            .header("Accept", "application/json")
            .config()
            .http_status_as_error(false)
            .build()
            .call()
            .map_err(|e| YtdError::Http(e.to_string()))?;
        read_response(response)
    }

    fn get_bytes(&self, url: &str, token: &str) -> Result<Vec<u8>, YtdError> {
        let mut request = ureq::get(url).header("Cache-Control", "no-cache");
        if !token.is_empty() {
            request = request.header("Authorization", &format!("Bearer {token}"));
        }
        let response = request
            .config()
            .http_status_as_error(false)
            .build()
            .call()
            .map_err(|e| YtdError::Http(e.to_string()))?;
        read_response_bytes(response)
    }

    fn post(&self, url: &str, token: &str, body: &str) -> Result<String, YtdError> {
        let response = ureq::post(url)
            .header("Authorization", &format!("Bearer {token}"))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .config()
            .http_status_as_error(false)
            .build()
            .send(body.as_bytes())
            .map_err(|e| YtdError::Http(e.to_string()))?;
        read_response(response)
    }

    fn post_multipart(
        &self,
        url: &str,
        token: &str,
        file_path: &Path,
        _file_name: &str,
    ) -> Result<String, YtdError> {
        let file_bytes = std::fs::read(file_path)?;
        let name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
        let mime = mime_from_extension(file_path);

        // Build multipart body manually
        let boundary = format!(
            "----ytd{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );

        let mut body = Vec::new();
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"file\"; filename=\"{name}\"\r\n")
                .as_bytes(),
        );
        body.extend_from_slice(format!("Content-Type: {mime}\r\n\r\n").as_bytes());
        body.extend_from_slice(&file_bytes);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        let response = ureq::post(url)
            .header("Authorization", &format!("Bearer {token}"))
            .header("Accept", "application/json")
            .header(
                "Content-Type",
                &format!("multipart/form-data; boundary={boundary}"),
            )
            .config()
            .http_status_as_error(false)
            .build()
            .send(&body[..])
            .map_err(|e| YtdError::Http(e.to_string()))?;
        read_response(response)
    }

    fn delete(&self, url: &str, token: &str) -> Result<(), YtdError> {
        let response = ureq::delete(url)
            .header("Authorization", &format!("Bearer {token}"))
            .config()
            .http_status_as_error(false)
            .build()
            .call()
            .map_err(|e| YtdError::Http(e.to_string()))?;
        read_response(response)?;
        Ok(())
    }
}

fn read_response(mut response: ureq::http::Response<ureq::Body>) -> Result<String, YtdError> {
    let status = response.status().as_u16();
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| YtdError::Http(e.to_string()))?;

    if status >= 400 {
        return Err(YtdError::from_api(status, extract_api_detail(&body)));
    }

    Ok(body)
}

fn read_response_bytes(
    mut response: ureq::http::Response<ureq::Body>,
) -> Result<Vec<u8>, YtdError> {
    let status = response.status().as_u16();
    let bytes = response
        .body_mut()
        .read_to_vec()
        .map_err(|e| YtdError::Http(e.to_string()))?;

    if status >= 400 {
        let detail = String::from_utf8_lossy(&bytes).to_string();
        return Err(YtdError::from_api(status, extract_api_detail(&detail)));
    }

    Ok(bytes)
}

fn extract_api_detail(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return "Request failed".into();
    }

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
        for key in ["error_description", "error", "message", "value"] {
            if let Some(detail) = value.get(key).and_then(|v| v.as_str()) {
                return detail.to_string();
            }
        }
    }

    trimmed.to_string()
}

fn mime_from_extension(path: &Path) -> &str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("txt") => "text/plain",
        Some("pdf") => "application/pdf",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("zip") => "application/zip",
        _ => "application/octet-stream",
    }
}

// --- YtClient ---

pub struct YtClient<T: HttpTransport> {
    base_url: String,
    token: String,
    transport: T,
    verbose: bool,
}

const ATTACHMENT_FIELDS: &str =
    "id,name,url,size,mimeType,created,author(id,login,fullName),comment(id)";
const COMMENT_ATTACHMENT_FIELDS: &str =
    "id,text,created,updated,author(id,login,fullName),attachments(id,name,url,size,mimeType,created,author(id,login,fullName),comment(id))";

impl<T: HttpTransport> YtClient<T> {
    pub fn new(config: YtdConfig, transport: T) -> Self {
        let base_url = config.url.trim_end_matches('/').to_string() + "/api";
        Self {
            base_url,
            token: config.token,
            transport,
            verbose: false,
        }
    }

    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    fn url(&self, path: &str, params: &[(&str, &str)]) -> String {
        let mut url = format!("{}{}", self.base_url, path);
        if !params.is_empty() {
            url.push('?');
            for (i, (k, v)) in params.iter().enumerate() {
                if i > 0 {
                    url.push('&');
                }
                url.push_str(&format!("{}={}", urlenc(k), urlenc(v)));
            }
        }
        url
    }

    fn log_request(&self, method: &str, url: &str, body: Option<&str>) {
        if !self.verbose {
            return;
        }
        eprintln!(">> {method} {url}");
        if let Some(b) = body {
            eprintln!(">> {b}");
        }
    }

    fn log_response(&self, body: &str) {
        if !self.verbose {
            return;
        }
        if body.len() <= 200 {
            eprintln!("<< {body}");
        } else {
            eprintln!("<< {}… ({} bytes)", &body[..200], body.len());
        }
    }

    fn get<R: DeserializeOwned>(&self, path: &str, params: &[(&str, &str)]) -> Result<R, YtdError> {
        let url = self.url(path, params);
        self.log_request("GET", &url, None);
        let body = self.transport.get(&url, &self.token)?;
        self.log_response(&body);
        serde_json::from_str(&body).map_err(YtdError::from)
    }

    fn post<R: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
        params: &[(&str, &str)],
    ) -> Result<R, YtdError> {
        let url = self.url(path, params);
        let json = serde_json::to_string(body)?;
        self.log_request("POST", &url, Some(&json));
        let resp = self.transport.post(&url, &self.token, &json)?;
        self.log_response(&resp);
        serde_json::from_str(&resp).map_err(YtdError::from)
    }

    fn post_no_response(
        &self,
        path: &str,
        body: &impl serde::Serialize,
        params: &[(&str, &str)],
    ) -> Result<(), YtdError> {
        let url = self.url(path, params);
        let json = serde_json::to_string(body)?;
        self.log_request("POST", &url, Some(&json));
        self.transport.post(&url, &self.token, &json)?;
        Ok(())
    }

    fn delete(&self, path: &str) -> Result<(), YtdError> {
        let url = self.url(path, &[]);
        self.log_request("DELETE", &url, None);
        self.transport.delete(&url, &self.token)
    }

    fn upload(
        &self,
        path: &str,
        file_path: &Path,
        params: &[(&str, &str)],
    ) -> Result<String, YtdError> {
        let url = self.url(path, params);
        let name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
        self.log_request("POST multipart", &url, Some(name));
        let resp = self
            .transport
            .post_multipart(&url, &self.token, file_path, name)?;
        self.log_response(&resp);
        Ok(resp)
    }

    pub fn download_attachment_file(&self, file_url: &str) -> Result<Vec<u8>, YtdError> {
        let url = self.absolute_file_url(file_url);
        self.log_request("GET bytes", &url, None);
        self.transport.get_bytes(&url, "")
    }

    fn absolute_file_url(&self, file_url: &str) -> String {
        if file_url.starts_with("http://") || file_url.starts_with("https://") {
            return file_url.to_string();
        }
        let root = self.base_url.trim_end_matches("/api");
        if file_url.starts_with('/') {
            format!("{root}{file_url}")
        } else {
            format!("{root}/{file_url}")
        }
    }

    fn is_project_database_id(project_ref: &str) -> bool {
        let mut parts = project_ref.split('-');
        matches!(
            (parts.next(), parts.next(), parts.next()),
            (Some(left), Some(right), None)
                if !left.is_empty()
                    && !right.is_empty()
                    && left.chars().all(|c| c.is_ascii_digit())
                    && right.chars().all(|c| c.is_ascii_digit())
        )
    }

    pub fn resolve_project_id(&self, project_ref: &str) -> Result<String, YtdError> {
        if Self::is_project_database_id(project_ref) {
            return Ok(project_ref.to_string());
        }

        match self.get_project(project_ref) {
            Ok(project) => return Ok(project.id),
            Err(YtdError::Http(_)) | Err(YtdError::Api { .. }) => {}
            Err(err) => return Err(err),
        }

        let projects = self.list_projects()?;

        if let Some(project) = projects.iter().find(|p| p.id == project_ref) {
            return Ok(project.id.clone());
        }

        if let Some(project) = projects.iter().find(|p| p.short_name == project_ref) {
            return Ok(project.id.clone());
        }

        if let Some(project) = projects
            .iter()
            .find(|p| p.short_name.eq_ignore_ascii_case(project_ref))
        {
            return Ok(project.id.clone());
        }

        Err(YtdError::Input(format!("Project not found: {project_ref}")))
    }

    pub fn resolve_project(&self, project_ref: &str) -> Result<Project, YtdError> {
        if Self::is_project_database_id(project_ref) {
            return self.get_project(project_ref);
        }

        match self.get_project(project_ref) {
            Ok(project) => return Ok(project),
            Err(err @ YtdError::PermissionDenied(_)) => return Err(err),
            Err(_) => {}
        }

        let projects = self.list_projects()?;

        if let Some(project) = projects.iter().find(|p| p.id == project_ref) {
            return Ok(project.clone());
        }

        if let Some(project) = projects.iter().find(|p| p.short_name == project_ref) {
            return Ok(project.clone());
        }

        if let Some(project) = projects
            .iter()
            .find(|p| p.short_name.eq_ignore_ascii_case(project_ref))
        {
            return Ok(project.clone());
        }

        Err(YtdError::Input(format!("Project not found: {project_ref}")))
    }

    fn article_matches_query(article: &Article, query: &str) -> bool {
        let query = query.to_lowercase();
        if query.is_empty() {
            return true;
        }

        article
            .id_readable
            .as_deref()
            .map(|s| s.to_lowercase().contains(&query))
            .unwrap_or(false)
            || article
                .summary
                .as_deref()
                .map(|s| s.to_lowercase().contains(&query))
                .unwrap_or(false)
            || article
                .content
                .as_deref()
                .map(|s| s.to_lowercase().contains(&query))
                .unwrap_or(false)
    }

    // --- Users ---

    pub fn get_me(&self) -> Result<User, YtdError> {
        self.get(
            "/users/me",
            &[("fields", "id,login,fullName,email,banned,guest")],
        )
    }

    pub fn list_users(&self) -> Result<Vec<User>, YtdError> {
        self.get(
            "/users",
            &[
                ("fields", "id,login,fullName,email,banned,guest"),
                ("$top", "500"),
            ],
        )
    }

    pub fn get_user(&self, id: &str) -> Result<User, YtdError> {
        self.get(
            &format!("/users/{id}"),
            &[("fields", "id,login,fullName,email,banned,guest")],
        )
    }

    pub fn resolve_user(&self, value: &str) -> Result<User, YtdError> {
        if value.trim().is_empty() {
            return Err(YtdError::Input("User ID or login is required".into()));
        }

        match self.get_user(value) {
            Ok(user) => return Ok(user),
            Err(YtdError::Http(_)) | Err(YtdError::Api { .. }) => {}
            Err(err) => return Err(err),
        }

        let needle = value.to_ascii_lowercase();
        let matches: Vec<User> = self
            .list_users()?
            .into_iter()
            .filter(|user| {
                user.id == value
                    || user.login.eq_ignore_ascii_case(value)
                    || user
                        .full_name
                        .as_deref()
                        .map(|name| name.to_ascii_lowercase() == needle)
                        .unwrap_or(false)
            })
            .collect();

        match matches.len() {
            0 => Err(YtdError::Input(format!("User not found: {value}"))),
            1 => Ok(matches.into_iter().next().unwrap()),
            _ => {
                let candidates = matches
                    .iter()
                    .map(|user| {
                        let display = user.full_name.as_deref().unwrap_or(&user.login);
                        format!("{} ({}, {})", display, user.login, user.id)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                Err(YtdError::Input(format!(
                    "User reference is ambiguous: {value}. Matches: {candidates}"
                )))
            }
        }
    }

    // --- Projects ---

    pub fn list_projects(&self) -> Result<Vec<Project>, YtdError> {
        self.get(
            "/admin/projects",
            &[
                ("fields", "id,name,shortName,archived,description"),
                ("$top", "500"),
            ],
        )
    }

    pub fn list_groups(&self) -> Result<Vec<UserGroup>, YtdError> {
        self.get(
            "/groups",
            &[("fields", "id,name,usersCount"), ("$top", "500")],
        )
    }

    pub fn get_project(&self, id: &str) -> Result<Project, YtdError> {
        self.get(
            &format!("/admin/projects/{id}"),
            &[("fields", "id,name,shortName,archived,description")],
        )
    }

    // --- Articles ---

    pub fn search_articles(
        &self,
        query: &str,
        project: Option<&str>,
    ) -> Result<Vec<Article>, YtdError> {
        let mut articles: Vec<Article> = match project {
            Some(project_ref) => {
                let project_id = self.resolve_project_id(project_ref)?;
                self.get(
                    &format!("/admin/projects/{project_id}/articles"),
                    &[
                        (
                            "fields",
                            "id,idReadable,summary,content,updated,project(id,shortName,name)",
                        ),
                        ("$top", "500"),
                    ],
                )?
            }
            None => self.get(
                "/articles",
                &[
                    (
                        "fields",
                        "id,idReadable,summary,content,updated,project(id,shortName,name)",
                    ),
                    ("$top", "500"),
                ],
            )?,
        };
        articles.retain(|article| Self::article_matches_query(article, query));
        for article in &mut articles {
            article.content = None;
        }
        Ok(articles)
    }

    pub fn list_articles(&self, project: &str) -> Result<Vec<Article>, YtdError> {
        let project_id = self.resolve_project_id(project)?;
        self.get(
            &format!("/admin/projects/{project_id}/articles"),
            &[
                (
                    "fields",
                    "id,idReadable,summary,updated,project(id,shortName,name)",
                ),
                ("$top", "500"),
            ],
        )
    }

    pub fn get_article(&self, id: &str) -> Result<Article, YtdError> {
        self.get(&format!("/articles/{id}"), &[
            ("fields", "id,idReadable,summary,content,created,updated,reporter(id,login,fullName),project(id,shortName,name),visibility($type,permittedGroups(id,name)),parentArticle(id,idReadable,summary)"),
        ])
    }

    pub fn create_article(&self, input: &CreateArticleInput) -> Result<Article, YtdError> {
        self.post("/articles", input, &[("fields", "id,idReadable")])
    }

    pub fn update_article(
        &self,
        id: &str,
        input: &UpdateArticleInput,
    ) -> Result<Article, YtdError> {
        self.post(
            &format!("/articles/{id}"),
            input,
            &[("fields", "id,idReadable")],
        )
    }

    pub fn append_to_article(&self, id: &str, text: &str) -> Result<(), YtdError> {
        let article = self.get_article(id)?;
        let current = article.content.unwrap_or_default();
        let new_content = format!("{current}{text}");
        let input = UpdateArticleInput {
            summary: None,
            content: Some(new_content),
            visibility: None,
            parent_article: None,
        };
        self.update_article(id, &input)?;
        Ok(())
    }

    pub fn delete_article(&self, id: &str) -> Result<(), YtdError> {
        self.delete(&format!("/articles/{id}"))
    }

    // --- Article Comments ---

    pub fn list_article_comments(&self, article_id: &str) -> Result<Vec<ArticleComment>, YtdError> {
        self.get(
            &format!("/articles/{article_id}/comments"),
            &[
                (
                    "fields",
                    "id,text,created,updated,author(id,login,fullName)",
                ),
                ("$top", "500"),
            ],
        )
    }

    pub fn get_article_comment_with_attachments(
        &self,
        article_id: &str,
        comment_id: &str,
    ) -> Result<ArticleComment, YtdError> {
        self.get(
            &format!("/articles/{article_id}/comments/{comment_id}"),
            &[("fields", COMMENT_ATTACHMENT_FIELDS)],
        )
    }

    pub fn upload_article_comment_attachment(
        &self,
        article_id: &str,
        comment_id: &str,
        file_path: &Path,
    ) -> Result<(), YtdError> {
        self.upload(
            &format!("/articles/{article_id}/comments/{comment_id}/attachments"),
            file_path,
            &[],
        )?;
        Ok(())
    }

    pub fn add_article_comment(
        &self,
        article_id: &str,
        text: &str,
        visibility: Option<LimitedVisibilityInput>,
    ) -> Result<ArticleComment, YtdError> {
        let input = CommentInput {
            text: text.to_string(),
            visibility,
        };
        self.post(
            &format!("/articles/{article_id}/comments"),
            &input,
            &[(
                "fields",
                "id,text,created,updated,author(id,login,fullName),visibility($type,permittedGroups(id,name))",
            )],
        )
    }

    pub fn get_article_comment(
        &self,
        article_id: &str,
        comment_id: &str,
    ) -> Result<ArticleComment, YtdError> {
        self.get(
            &format!("/articles/{article_id}/comments/{comment_id}"),
            &[(
                "fields",
                "id,text,created,updated,author(id,login,fullName),visibility($type,permittedGroups(id,name))",
            )],
        )
    }

    pub fn update_article_comment(
        &self,
        article_id: &str,
        comment_id: &str,
        text: &str,
        visibility: Option<LimitedVisibilityInput>,
    ) -> Result<ArticleComment, YtdError> {
        let input = CommentInput {
            text: text.to_string(),
            visibility,
        };
        self.post(
            &format!("/articles/{article_id}/comments/{comment_id}"),
            &input,
            &[(
                "fields",
                "id,text,created,updated,author(id,login,fullName),visibility($type,permittedGroups(id,name))",
            )],
        )
    }

    pub fn delete_article_comment(
        &self,
        article_id: &str,
        comment_id: &str,
    ) -> Result<(), YtdError> {
        self.delete(&format!("/articles/{article_id}/comments/{comment_id}"))
    }

    // --- Article Attachments ---

    pub fn list_article_attachments(&self, article_id: &str) -> Result<Vec<Attachment>, YtdError> {
        self.get(
            &format!("/articles/{article_id}/attachments"),
            &[("fields", ATTACHMENT_FIELDS), ("$top", "500")],
        )
    }

    pub fn get_article_attachment(
        &self,
        article_id: &str,
        attachment_id: &str,
    ) -> Result<Attachment, YtdError> {
        self.get(
            &format!("/articles/{article_id}/attachments/{attachment_id}"),
            &[("fields", ATTACHMENT_FIELDS)],
        )
    }

    pub fn upload_article_attachment(
        &self,
        article_id: &str,
        file_path: &Path,
    ) -> Result<(), YtdError> {
        self.upload(
            &format!("/articles/{article_id}/attachments"),
            file_path,
            &[],
        )?;
        Ok(())
    }

    pub fn delete_article_attachment(
        &self,
        article_id: &str,
        attachment_id: &str,
    ) -> Result<(), YtdError> {
        self.delete(&format!(
            "/articles/{article_id}/attachments/{attachment_id}"
        ))
    }

    // --- Issues ---

    pub fn search_issues(
        &self,
        query: &str,
        project: Option<&str>,
    ) -> Result<Vec<Issue>, YtdError> {
        let q = match project {
            Some(p) => format!("project: {{{p}}} {query}"),
            None => query.to_string(),
        };
        self.get(
            "/issues",
            &[
                (
                    "fields",
                    "id,idReadable,summary,created,updated,resolved,project(id,shortName,name),customFields(id,name,$type,value(id,name,login,fullName,minutes,presentation,$type))",
                ),
                ("query", &q),
                ("$top", "100"),
            ],
        )
    }

    pub fn list_issues(&self, project: &str) -> Result<Vec<Issue>, YtdError> {
        self.get(
            "/issues",
            &[
                (
                    "fields",
                    "id,idReadable,summary,created,updated,resolved,project(id,shortName,name),customFields(id,name,$type,value(id,name,login,fullName,minutes,presentation,$type))",
                ),
                ("query", &format!("project: {{{project}}}")),
                ("$top", "500"),
            ],
        )
    }

    pub fn get_issue(&self, id: &str) -> Result<Issue, YtdError> {
        self.get(&format!("/issues/{id}"), &[
            ("fields", "id,idReadable,summary,description,created,updated,resolved,reporter(id,login,fullName),project(id,shortName,name),visibility($type,permittedGroups(id,name)),tags(id,name),comments(id,text,created,updated,author(id,login,fullName),visibility($type,permittedGroups(id,name))),customFields(id,name,$type,value(id,name,login,fullName,minutes,presentation,$type))"),
        ])
    }

    pub fn create_issue(&self, input: &CreateIssueInput) -> Result<Issue, YtdError> {
        self.post("/issues", input, &[("fields", "id,idReadable")])
    }

    pub fn update_issue(&self, id: &str, input: &UpdateIssueInput) -> Result<Issue, YtdError> {
        self.post(
            &format!("/issues/{id}"),
            input,
            &[("fields", "id,idReadable")],
        )
    }

    pub fn delete_issue(&self, id: &str) -> Result<(), YtdError> {
        self.delete(&format!("/issues/{id}"))
    }

    // --- Issue Comments ---

    pub fn add_comment(
        &self,
        issue_id: &str,
        text: &str,
        visibility: Option<LimitedVisibilityInput>,
    ) -> Result<IssueComment, YtdError> {
        let input = CommentInput {
            text: text.to_string(),
            visibility,
        };
        self.post(
            &format!("/issues/{issue_id}/comments"),
            &input,
            &[(
                "fields",
                "id,text,created,updated,author(id,login,fullName),visibility($type,permittedGroups(id,name))",
            )],
        )
    }

    pub fn list_issue_comments(&self, issue_id: &str) -> Result<Vec<IssueComment>, YtdError> {
        self.get(
            &format!("/issues/{issue_id}/comments"),
            &[
                (
                    "fields",
                    "id,text,created,updated,author(id,login,fullName),visibility($type,permittedGroups(id,name))",
                ),
                ("$top", "500"),
            ],
        )
    }

    pub fn get_issue_comment_with_attachments(
        &self,
        issue_id: &str,
        comment_id: &str,
    ) -> Result<IssueComment, YtdError> {
        self.get(
            &format!("/issues/{issue_id}/comments/{comment_id}"),
            &[("fields", COMMENT_ATTACHMENT_FIELDS)],
        )
    }

    pub fn upload_issue_comment_attachment(
        &self,
        issue_id: &str,
        comment_id: &str,
        file_path: &Path,
    ) -> Result<(), YtdError> {
        self.upload(
            &format!("/issues/{issue_id}/comments/{comment_id}/attachments"),
            file_path,
            &[],
        )?;
        Ok(())
    }

    pub fn get_issue_comment(
        &self,
        issue_id: &str,
        comment_id: &str,
    ) -> Result<IssueComment, YtdError> {
        self.get(
            &format!("/issues/{issue_id}/comments/{comment_id}"),
            &[(
                "fields",
                "id,text,created,updated,author(id,login,fullName),visibility($type,permittedGroups(id,name))",
            )],
        )
    }

    pub fn update_issue_comment(
        &self,
        issue_id: &str,
        comment_id: &str,
        text: &str,
        visibility: Option<LimitedVisibilityInput>,
    ) -> Result<IssueComment, YtdError> {
        let input = CommentInput {
            text: text.to_string(),
            visibility,
        };
        self.post(
            &format!("/issues/{issue_id}/comments/{comment_id}"),
            &input,
            &[(
                "fields",
                "id,text,created,updated,author(id,login,fullName),visibility($type,permittedGroups(id,name))",
            )],
        )
    }

    pub fn delete_issue_comment(&self, issue_id: &str, comment_id: &str) -> Result<(), YtdError> {
        self.delete(&format!("/issues/{issue_id}/comments/{comment_id}"))
    }

    // --- Tags ---

    pub fn list_tags(&self) -> Result<Vec<Tag>, YtdError> {
        self.get("/tags", &[("fields", "id,name"), ("$top", "500")])
    }

    pub fn add_issue_tag(&self, issue_id: &str, tag: &Tag) -> Result<(), YtdError> {
        self.post_no_response(&format!("/issues/{issue_id}/tags"), tag, &[])
    }

    pub fn remove_issue_tag(&self, issue_id: &str, tag_id: &str) -> Result<(), YtdError> {
        self.delete(&format!("/issues/{issue_id}/tags/{tag_id}"))
    }

    // --- Issue Links ---

    pub fn list_issue_links(&self, issue_id: &str) -> Result<Vec<IssueLink>, YtdError> {
        self.get(&format!("/issues/{issue_id}/links"), &[
            ("fields", "id,direction,linkType(id,name,sourceToTarget,targetToSource),issues(id,idReadable,summary,updated,resolved,project(id,shortName,name),customFields(id,name,$type,value(id,name,login,fullName,minutes,presentation,$type)))"),
            ("$top", "500"),
        ])
    }

    pub fn apply_command(&self, issue_id: &str, command: &str) -> Result<(), YtdError> {
        let body = serde_json::json!({
            "query": command,
            "issues": [{"idReadable": issue_id}]
        });
        self.post_no_response("/commands", &body, &[])
    }

    // --- Attachments ---

    pub fn list_attachments(&self, issue_id: &str) -> Result<Vec<Attachment>, YtdError> {
        self.get(
            &format!("/issues/{issue_id}/attachments"),
            &[("fields", ATTACHMENT_FIELDS), ("$top", "500")],
        )
    }

    pub fn get_issue_attachment(
        &self,
        issue_id: &str,
        attachment_id: &str,
    ) -> Result<Attachment, YtdError> {
        self.get(
            &format!("/issues/{issue_id}/attachments/{attachment_id}"),
            &[("fields", ATTACHMENT_FIELDS)],
        )
    }

    pub fn upload_attachment(&self, issue_id: &str, file_path: &Path) -> Result<(), YtdError> {
        self.upload(&format!("/issues/{issue_id}/attachments"), file_path, &[])?;
        Ok(())
    }

    pub fn delete_issue_attachment(
        &self,
        issue_id: &str,
        attachment_id: &str,
    ) -> Result<(), YtdError> {
        self.delete(&format!("/issues/{issue_id}/attachments/{attachment_id}"))
    }

    // --- Time Tracking ---

    pub fn list_work_items(&self, issue_id: &str) -> Result<Vec<WorkItem>, YtdError> {
        self.get(&format!("/issues/{issue_id}/timeTracking/workItems"), &[
            ("fields", "id,duration(minutes,presentation),date,text,author(id,login,fullName),type(id,name)"),
            ("$top", "500"),
        ])
    }

    pub fn add_work_item(
        &self,
        issue_id: &str,
        input: &CreateWorkItemInput,
    ) -> Result<WorkItem, YtdError> {
        self.post(
            &format!("/issues/{issue_id}/timeTracking/workItems"),
            input,
            &[("fields", "id,duration(minutes,presentation),date,text")],
        )
    }

    // --- Custom Fields ---

    pub fn set_custom_field(
        &self,
        issue_id: &str,
        body: &serde_json::Value,
    ) -> Result<(), YtdError> {
        let url = self.url(&format!("/issues/{issue_id}"), &[("fields", "id")]);
        let json = serde_json::to_string(body)?;
        self.transport.post(&url, &self.token, &json)?;
        Ok(())
    }

    // --- Saved Searches ---

    pub fn list_saved_queries(&self) -> Result<Vec<SavedQuery>, YtdError> {
        self.get(
            "/savedQueries",
            &[("fields", "id,name,query"), ("$top", "500")],
        )
    }

    // --- Activities ---

    pub fn list_activities(
        &self,
        issue_id: &str,
        categories: Option<&str>,
    ) -> Result<Vec<ActivityItem>, YtdError> {
        let cats = categories.unwrap_or(
            "IssueCreatedCategory,CustomFieldCategory,CommentCategory,AttachmentCategory,LinkCategory,SummaryCategory,DescriptionCategory,TagsCategory,SprintCategory"
        );
        self.get(&format!("/issues/{issue_id}/activities"), &[
            ("fields", "id,timestamp,author(id,login,fullName),field(presentation,id),added,removed,category(id)"),
            ("categories", cats),
            ("$top", "500"),
        ])
    }

    // --- Agile Boards ---

    const AGILE_LIST_FIELDS: &'static str = "id,name,owner(id,login,fullName),projects(id,shortName,name),currentSprint(id,name,goal,start,finish,archived,isDefault,unresolvedIssuesCount),sprints(id,name,start,finish,archived)";
    const AGILE_DETAIL_FIELDS: &'static str = "id,name,owner(id,login,fullName),projects(id,shortName,name),currentSprint(id,name,goal,start,finish,archived,isDefault,unresolvedIssuesCount),sprints(id,name,goal,start,finish,archived,isDefault,unresolvedIssuesCount),orphansAtTheTop,hideOrphansSwimlane,estimationField(id,name),originalEstimationField(id,name)";
    const SPRINT_FIELDS: &'static str =
        "id,name,goal,start,finish,archived,isDefault,unresolvedIssuesCount";
    const SPRINT_ISSUES_FIELDS: &'static str = "id,name,issues(id,idReadable,summary,created,updated,resolved,project(id,shortName,name),customFields(id,name,$type,value(id,name,login,fullName,minutes,presentation,$type)))";
    const ISSUE_REF_FIELDS: &'static str = "id,idReadable,summary";
    const ISSUE_SPRINT_FIELDS: &'static str =
        "id,name,goal,start,finish,archived,isDefault,unresolvedIssuesCount,agile(id,name)";

    pub fn list_agiles(&self) -> Result<Vec<Agile>, YtdError> {
        self.get(
            "/agiles",
            &[("fields", Self::AGILE_LIST_FIELDS), ("$top", "500")],
        )
    }

    pub fn get_agile(&self, id: &str) -> Result<Agile, YtdError> {
        self.get(
            &format!("/agiles/{id}"),
            &[("fields", Self::AGILE_DETAIL_FIELDS)],
        )
    }

    pub fn create_agile(
        &self,
        template: Option<&str>,
        body: &serde_json::Value,
    ) -> Result<Agile, YtdError> {
        let mut params = vec![("fields", Self::AGILE_DETAIL_FIELDS)];
        if let Some(template) = template {
            params.push(("template", template));
        }
        let url = self.url("/agiles", &params);
        let json = serde_json::to_string(body)?;
        let response = self.transport.post(&url, &self.token, &json)?;
        serde_json::from_str(&response).map_err(YtdError::from)
    }

    pub fn update_agile(&self, id: &str, body: &serde_json::Value) -> Result<Agile, YtdError> {
        let url = self.url(
            &format!("/agiles/{id}"),
            &[("fields", Self::AGILE_DETAIL_FIELDS)],
        );
        let json = serde_json::to_string(body)?;
        let response = self.transport.post(&url, &self.token, &json)?;
        serde_json::from_str(&response).map_err(YtdError::from)
    }

    pub fn delete_agile(&self, id: &str) -> Result<(), YtdError> {
        let url = self.url(&format!("/agiles/{id}"), &[]);
        self.transport.delete(&url, &self.token)
    }

    pub fn get_sprint(&self, agile_id: &str, sprint_id: &str) -> Result<Sprint, YtdError> {
        self.get(
            &format!("/agiles/{agile_id}/sprints/{sprint_id}"),
            &[("fields", Self::SPRINT_FIELDS)],
        )
    }

    pub fn create_sprint(
        &self,
        agile_id: &str,
        body: &serde_json::Value,
    ) -> Result<Sprint, YtdError> {
        let url = self.url(
            &format!("/agiles/{agile_id}/sprints"),
            &[("fields", Self::SPRINT_FIELDS)],
        );
        let json = serde_json::to_string(body)?;
        let response = self.transport.post(&url, &self.token, &json)?;
        serde_json::from_str(&response).map_err(YtdError::from)
    }

    pub fn update_sprint(
        &self,
        agile_id: &str,
        sprint_id: &str,
        body: &serde_json::Value,
    ) -> Result<Sprint, YtdError> {
        let url = self.url(
            &format!("/agiles/{agile_id}/sprints/{sprint_id}"),
            &[("fields", Self::SPRINT_FIELDS)],
        );
        let json = serde_json::to_string(body)?;
        let response = self.transport.post(&url, &self.token, &json)?;
        serde_json::from_str(&response).map_err(YtdError::from)
    }

    pub fn delete_sprint(&self, agile_id: &str, sprint_id: &str) -> Result<(), YtdError> {
        let url = self.url(&format!("/agiles/{agile_id}/sprints/{sprint_id}"), &[]);
        self.transport.delete(&url, &self.token)
    }

    pub fn list_issue_sprints(&self, issue_id: &str) -> Result<Vec<Sprint>, YtdError> {
        self.get(
            &format!("/issues/{issue_id}/sprints"),
            &[("fields", Self::ISSUE_SPRINT_FIELDS), ("$top", "500")],
        )
    }

    pub fn get_issue_ref(&self, issue_id: &str) -> Result<Issue, YtdError> {
        self.get(
            &format!("/issues/{issue_id}"),
            &[("fields", Self::ISSUE_REF_FIELDS)],
        )
    }

    pub fn list_sprint_issues(
        &self,
        agile_id: &str,
        sprint_id: &str,
    ) -> Result<Vec<Issue>, YtdError> {
        let sprint: Sprint = self.get(
            &format!("/agiles/{agile_id}/sprints/{sprint_id}"),
            &[("fields", Self::SPRINT_ISSUES_FIELDS)],
        )?;
        Ok(sprint.issues)
    }

    pub fn add_issue_to_sprint(
        &self,
        agile_id: &str,
        sprint_id: &str,
        issue_id: &str,
    ) -> Result<Issue, YtdError> {
        let issue = self.get_issue_ref(issue_id)?;
        let body = serde_json::json!({
            "id": issue.id,
            "$type": "Issue",
        });
        self.post(
            &format!("/agiles/{agile_id}/sprints/{sprint_id}/issues"),
            &body,
            &[("fields", Self::ISSUE_REF_FIELDS)],
        )
    }

    pub fn remove_issue_from_sprint(
        &self,
        agile_id: &str,
        sprint_id: &str,
        issue_id: &str,
    ) -> Result<Issue, YtdError> {
        let issue = self.get_issue_ref(issue_id)?;
        self.delete(&format!(
            "/agiles/{agile_id}/sprints/{sprint_id}/issues/{}",
            issue.id
        ))?;
        Ok(issue)
    }
}

fn urlenc(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            ' ' => result.push_str("%20"),
            _ => {
                for b in c.to_string().as_bytes() {
                    result.push_str(&format!("%{b:02X}"));
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct CapturedRequest {
        method: String,
        url: String,
        body: Option<String>,
    }

    struct MockTransport {
        responses: RefCell<Vec<String>>,
        requests: RefCell<Vec<CapturedRequest>>,
    }

    impl MockTransport {
        fn new(responses: Vec<&str>) -> Self {
            Self {
                responses: RefCell::new(responses.into_iter().rev().map(String::from).collect()),
                requests: RefCell::new(vec![]),
            }
        }

        fn request(&self, index: usize) -> CapturedRequest {
            self.requests.borrow()[index].clone()
        }
    }

    impl HttpTransport for MockTransport {
        fn get(&self, url: &str, _token: &str) -> Result<String, YtdError> {
            self.requests.borrow_mut().push(CapturedRequest {
                method: "GET".into(),
                url: url.into(),
                body: None,
            });
            self.responses
                .borrow_mut()
                .pop()
                .ok_or_else(|| YtdError::Http("No more mock responses".into()))
        }
        fn get_bytes(&self, url: &str, _token: &str) -> Result<Vec<u8>, YtdError> {
            self.requests.borrow_mut().push(CapturedRequest {
                method: "GET bytes".into(),
                url: url.into(),
                body: None,
            });
            let response = self
                .responses
                .borrow_mut()
                .pop()
                .ok_or_else(|| YtdError::Http("No more mock responses".into()))?;
            Ok(response.into_bytes())
        }
        fn post(&self, url: &str, _token: &str, body: &str) -> Result<String, YtdError> {
            self.requests.borrow_mut().push(CapturedRequest {
                method: "POST".into(),
                url: url.into(),
                body: Some(body.into()),
            });
            self.responses
                .borrow_mut()
                .pop()
                .ok_or_else(|| YtdError::Http("No more mock responses".into()))
        }
        fn post_multipart(
            &self,
            url: &str,
            _token: &str,
            _file: &Path,
            _name: &str,
        ) -> Result<String, YtdError> {
            self.requests.borrow_mut().push(CapturedRequest {
                method: "POST multipart".into(),
                url: url.into(),
                body: None,
            });
            self.responses
                .borrow_mut()
                .pop()
                .ok_or_else(|| YtdError::Http("No more mock responses".into()))
        }
        fn delete(&self, url: &str, _token: &str) -> Result<(), YtdError> {
            self.requests.borrow_mut().push(CapturedRequest {
                method: "DELETE".into(),
                url: url.into(),
                body: None,
            });
            self.responses.borrow_mut().pop();
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

    #[test]
    fn get_me() {
        let client = test_client(vec![
            r#"{"id":"1","login":"admin","fullName":"Admin","email":"a@b.com"}"#,
        ]);
        let user = client.get_me().unwrap();
        assert_eq!(user.login, "admin");
    }

    #[test]
    fn list_users_requests_user_fields() {
        let client = test_client(vec![
            r#"[{"id":"1-51","login":"alice","fullName":"Alice","email":"a@example.com","banned":false,"guest":false}]"#,
        ]);

        let users = client.list_users().unwrap();

        assert_eq!(users[0].id, "1-51");
        assert_eq!(users[0].banned, Some(false));
        let request = client.transport.request(0);
        assert_eq!(request.method, "GET");
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/users?fields=id%2Clogin%2CfullName%2Cemail%2Cbanned%2Cguest&%24top=500"
        );
    }

    #[test]
    fn get_user_requests_user_id() {
        let client = test_client(vec![
            r#"{"id":"1-51","login":"alice","fullName":"Alice","email":null,"banned":false,"guest":false}"#,
        ]);

        let user = client.get_user("1-51").unwrap();

        assert_eq!(user.login, "alice");
        let request = client.transport.request(0);
        assert_eq!(request.method, "GET");
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/users/1-51?fields=id%2Clogin%2CfullName%2Cemail%2Cbanned%2Cguest"
        );
    }

    #[test]
    fn list_projects() {
        let client = test_client(vec![
            r#"[{"id":"1","name":"Test","shortName":"TEST","archived":false,"description":null}]"#,
        ]);
        let projects = client.list_projects().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].short_name, "TEST");
    }

    #[test]
    fn resolve_project_returns_direct_database_id_project() {
        let client = test_client(vec![
            r#"{"id":"0-96","name":"Developer Workflow Platform","shortName":"DWP","archived":false,"description":null}"#,
        ]);

        let project = client.resolve_project("0-96").unwrap();

        assert_eq!(project.id, "0-96");
        assert_eq!(project.short_name, "DWP");
        let request = client.transport.request(0);
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/admin/projects/0-96?fields=id%2Cname%2CshortName%2Carchived%2Cdescription"
        );
    }

    #[test]
    fn resolve_project_resolves_exact_short_name() {
        let client = test_client(vec![
            r#"{"id":"0-96","name":"Developer Workflow Platform","shortName":"DWP","archived":false,"description":null}"#,
        ]);

        let project = client.resolve_project("DWP").unwrap();

        assert_eq!(project.id, "0-96");
        assert_eq!(project.short_name, "DWP");
    }

    #[test]
    fn resolve_project_resolves_case_insensitive_short_name_from_list() {
        let client = test_client(vec![
            "not json",
            r#"[{"id":"0-96","name":"Developer Workflow Platform","shortName":"DWP","archived":false,"description":null}]"#,
        ]);

        let project = client.resolve_project("dwp").unwrap();

        assert_eq!(project.id, "0-96");
        assert_eq!(project.short_name, "DWP");
    }

    #[test]
    fn resolve_project_reports_missing_project() {
        let client = test_client(vec!["not json", r#"[]"#]);

        let err = client.resolve_project("NOPE").unwrap_err();

        assert_eq!(err.to_string(), "Project not found: NOPE");
    }

    #[test]
    fn create_agile_with_template_posts_body() {
        let client = test_client(vec![
            r#"{"id":"108-4","name":"Board","projects":[{"id":"0-96","shortName":"DWP","name":"DW Playground"}],"sprints":[]}"#,
        ]);
        let body = serde_json::json!({"name":"Board","projects":[{"id":"0-96"}]});

        let agile = client.create_agile(Some("scrum"), &body).unwrap();

        assert_eq!(agile.id, "108-4");
        let request = client.transport.request(0);
        assert_eq!(request.method, "POST");
        assert!(request
            .url
            .starts_with("https://test.youtrack.cloud/api/agiles?fields="));
        assert!(request.url.contains("&template=scrum"));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&request.body.unwrap()).unwrap(),
            body
        );
    }

    #[test]
    fn create_agile_without_template_omits_template_param() {
        let client = test_client(vec![
            r#"{"id":"108-4","name":"Board","projects":[{"id":"0-96","shortName":"DWP","name":"DW Playground"}],"sprints":[]}"#,
        ]);

        client
            .create_agile(
                None,
                &serde_json::json!({"name":"Board","projects":[{"id":"0-96"}]}),
            )
            .unwrap();

        let request = client.transport.request(0);
        assert_eq!(request.method, "POST");
        assert!(request
            .url
            .starts_with("https://test.youtrack.cloud/api/agiles?fields="));
        assert!(!request.url.contains("template="));
    }

    #[test]
    fn update_agile_posts_to_board_id() {
        let client = test_client(vec![
            r#"{"id":"108-4","name":"Renamed","projects":[{"id":"0-96","shortName":"DWP","name":"DW Playground"}],"sprints":[]}"#,
        ]);
        let body = serde_json::json!({"name":"Renamed"});

        let agile = client.update_agile("108-4", &body).unwrap();

        assert_eq!(agile.id, "108-4");
        let request = client.transport.request(0);
        assert_eq!(request.method, "POST");
        assert!(request
            .url
            .starts_with("https://test.youtrack.cloud/api/agiles/108-4?fields="));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&request.body.unwrap()).unwrap(),
            body
        );
    }

    #[test]
    fn delete_agile_uses_board_id() {
        let client = test_client(vec![""]);

        client.delete_agile("108-4").unwrap();

        let request = client.transport.request(0);
        assert_eq!(request.method, "DELETE");
        assert_eq!(request.url, "https://test.youtrack.cloud/api/agiles/108-4");
    }

    #[test]
    fn get_sprint_gets_board_sprint_id() {
        let client = test_client(vec![r#"{"id":"113-6","name":"Sprint 1"}"#]);

        let sprint = client.get_sprint("108-4", "113-6").unwrap();

        assert_eq!(sprint.id, "113-6");
        let request = client.transport.request(0);
        assert_eq!(request.method, "GET");
        assert!(request
            .url
            .starts_with("https://test.youtrack.cloud/api/agiles/108-4/sprints/113-6?fields="));
    }

    #[test]
    fn create_sprint_posts_body() {
        let client = test_client(vec![r#"{"id":"113-6","name":"Sprint 1"}"#]);
        let body = serde_json::json!({"name":"Sprint 1"});

        let sprint = client.create_sprint("108-4", &body).unwrap();

        assert_eq!(sprint.id, "113-6");
        let request = client.transport.request(0);
        assert_eq!(request.method, "POST");
        assert!(request
            .url
            .starts_with("https://test.youtrack.cloud/api/agiles/108-4/sprints?fields="));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&request.body.unwrap()).unwrap(),
            body
        );
    }

    #[test]
    fn update_sprint_posts_body() {
        let client = test_client(vec![r#"{"id":"113-6","name":"Renamed"}"#]);
        let body = serde_json::json!({"name":"Renamed"});

        let sprint = client.update_sprint("108-4", "113-6", &body).unwrap();

        assert_eq!(sprint.id, "113-6");
        let request = client.transport.request(0);
        assert_eq!(request.method, "POST");
        assert!(request
            .url
            .starts_with("https://test.youtrack.cloud/api/agiles/108-4/sprints/113-6?fields="));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&request.body.unwrap()).unwrap(),
            body
        );
    }

    #[test]
    fn delete_sprint_uses_board_and_sprint_id() {
        let client = test_client(vec![""]);

        client.delete_sprint("108-4", "113-6").unwrap();

        let request = client.transport.request(0);
        assert_eq!(request.method, "DELETE");
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/agiles/108-4/sprints/113-6"
        );
    }

    #[test]
    fn list_issue_sprints_gets_issue_sprints_with_agile() {
        let client = test_client(vec![
            r#"[{"id":"113-6","name":"Sprint 1","agile":{"id":"108-4","name":"Board"}}]"#,
        ]);

        let sprints = client.list_issue_sprints("DWP-1").unwrap();

        assert_eq!(sprints[0].id, "113-6");
        let request = client.transport.request(0);
        assert_eq!(request.method, "GET");
        assert!(request
            .url
            .starts_with("https://test.youtrack.cloud/api/issues/DWP-1/sprints?fields="));
        assert!(request.url.contains("agile%28id%2Cname%29"));
        assert!(request.url.contains("%24top=500"));
    }

    #[test]
    fn list_sprint_issues_gets_issues_from_board_sprint() {
        let client = test_client(vec![
            r#"{"id":"113-6","name":"Sprint 1","issues":[{"id":"2-1","idReadable":"DWP-1","summary":"Issue"}]}"#,
        ]);

        let issues = client.list_sprint_issues("108-4", "113-6").unwrap();

        assert_eq!(issues[0].id, "2-1");
        assert_eq!(issues[0].id_readable.as_deref(), Some("DWP-1"));
        let request = client.transport.request(0);
        assert_eq!(request.method, "GET");
        assert!(request
            .url
            .starts_with("https://test.youtrack.cloud/api/agiles/108-4/sprints/113-6?fields="));
        assert!(request
            .url
            .contains("issues%28id%2CidReadable%2Csummary%2Ccreated%2Cupdated%2Cresolved"));
        assert!(request.url.contains("customFields%28id%2Cname%2C%24type"));
    }

    #[test]
    fn add_issue_to_sprint_resolves_issue_database_id() {
        let client = test_client(vec![
            r#"{"id":"2-1","idReadable":"DWP-1","summary":"Issue"}"#,
            r#"{"id":"2-1","idReadable":"DWP-1","summary":"Issue"}"#,
        ]);

        let issue = client
            .add_issue_to_sprint("108-4", "113-6", "DWP-1")
            .unwrap();

        assert_eq!(issue.id, "2-1");
        let lookup = client.transport.request(0);
        assert_eq!(lookup.method, "GET");
        assert!(lookup
            .url
            .starts_with("https://test.youtrack.cloud/api/issues/DWP-1?fields="));
        let add = client.transport.request(1);
        assert_eq!(add.method, "POST");
        assert!(add.url.starts_with(
            "https://test.youtrack.cloud/api/agiles/108-4/sprints/113-6/issues?fields="
        ));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&add.body.unwrap()).unwrap(),
            serde_json::json!({"id":"2-1","$type":"Issue"})
        );
    }

    #[test]
    fn remove_issue_from_sprint_resolves_issue_database_id() {
        let client = test_client(vec![
            r#"{"id":"2-1","idReadable":"DWP-1","summary":"Issue"}"#,
        ]);

        let issue = client
            .remove_issue_from_sprint("108-4", "113-6", "DWP-1")
            .unwrap();

        assert_eq!(issue.id, "2-1");
        let lookup = client.transport.request(0);
        assert_eq!(lookup.method, "GET");
        assert!(lookup
            .url
            .starts_with("https://test.youtrack.cloud/api/issues/DWP-1?fields="));
        let delete = client.transport.request(1);
        assert_eq!(delete.method, "DELETE");
        assert_eq!(
            delete.url,
            "https://test.youtrack.cloud/api/agiles/108-4/sprints/113-6/issues/2-1"
        );
    }

    #[test]
    fn list_groups_includes_users_count_fields() {
        let client = test_client(vec![r#"[{"id":"3-1","name":"Admins","usersCount":2}]"#]);

        let groups = client.list_groups().unwrap();

        assert_eq!(groups[0].users_count, Some(2));
        let request = client.transport.request(0);
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/groups?fields=id%2Cname%2CusersCount&%24top=500"
        );
    }

    #[test]
    fn create_issue() {
        let client = test_client(vec![r#"{"id":"2-1","idReadable":"TEST-1"}"#]);
        let input = CreateIssueInput {
            project: ProjectRef {
                id: "1".into(),
                short_name: None,
                name: None,
            },
            summary: "Test".into(),
            description: None,
            visibility: None,
        };
        let issue = client.create_issue(&input).unwrap();
        assert_eq!(issue.id_readable.as_deref(), Some("TEST-1"));
    }

    #[test]
    fn create_issue_serializes_visibility_payload() {
        let client = test_client(vec![r#"{"id":"2-1","idReadable":"TEST-1"}"#]);
        let input = CreateIssueInput {
            project: ProjectRef {
                id: "1".into(),
                short_name: None,
                name: None,
            },
            summary: "Restricted".into(),
            description: Some("Secret".into()),
            visibility: Some(LimitedVisibilityInput {
                visibility_type: "LimitedVisibility",
                permitted_groups: vec![UserGroupInput { id: "3-7".into() }],
            }),
        };

        client.create_issue(&input).unwrap();

        let request = client.transport.request(0);
        assert_eq!(request.method, "POST");
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/issues?fields=id%2CidReadable"
        );
        let body = request.body.expect("missing request body");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&body).unwrap(),
            serde_json::json!({
                "project": {"id": "1"},
                "summary": "Restricted",
                "description": "Secret",
                "visibility": {
                    "$type": "LimitedVisibility",
                    "permittedGroups": [{"id": "3-7"}]
                }
            })
        );
    }

    #[test]
    fn update_issue_serializes_visibility_clear_payload() {
        let client = test_client(vec![r#"{"id":"2-1","idReadable":"TEST-1"}"#]);
        let input = UpdateIssueInput {
            summary: None,
            description: Some("Visible again".into()),
            visibility: Some(LimitedVisibilityInput {
                visibility_type: "LimitedVisibility",
                permitted_groups: vec![],
            }),
        };

        client.update_issue("TEST-1", &input).unwrap();

        let request = client.transport.request(0);
        assert_eq!(request.method, "POST");
        let body = request.body.expect("missing request body");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&body).unwrap(),
            serde_json::json!({
                "description": "Visible again",
                "visibility": {
                    "$type": "LimitedVisibility",
                    "permittedGroups": []
                }
            })
        );
    }

    #[test]
    fn create_article_serializes_visibility_payload() {
        let client = test_client(vec![r#"{"id":"109-1","idReadable":"KB-1"}"#]);
        let input = CreateArticleInput {
            project: ProjectRef {
                id: "0-1".into(),
                short_name: None,
                name: None,
            },
            summary: "Runbook".into(),
            content: Some("Internal".into()),
            visibility: Some(LimitedVisibilityInput {
                visibility_type: "LimitedVisibility",
                permitted_groups: vec![UserGroupInput { id: "3-8".into() }],
            }),
            parent_article: None,
        };

        client.create_article(&input).unwrap();

        let request = client.transport.request(0);
        assert_eq!(request.method, "POST");
        let body = request.body.expect("missing request body");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&body).unwrap(),
            serde_json::json!({
                "project": {"id": "0-1"},
                "summary": "Runbook",
                "content": "Internal",
                "visibility": {
                    "$type": "LimitedVisibility",
                    "permittedGroups": [{"id": "3-8"}]
                }
            })
        );
    }

    #[test]
    fn get_article_requests_parent_article_fields() {
        let client = test_client(vec![
            r#"{"id":"109-813","idReadable":"GRA-A-32","summary":"Child","parentArticle":{"id":"109-812","idReadable":"GRA-A-31","summary":"Parent"}}"#,
        ]);

        let article = client.get_article("GRA-A-32").unwrap();

        assert_eq!(article.parent_article.unwrap().id, "109-812");
        let request = client.transport.request(0);
        assert_eq!(request.method, "GET");
        assert!(request
            .url
            .contains("parentArticle%28id%2CidReadable%2Csummary%29"));
    }

    #[test]
    fn create_article_serializes_parent_article_internal_id() {
        let client = test_client(vec![r#"{"id":"109-813","idReadable":"GRA-A-32"}"#]);
        let input = CreateArticleInput {
            project: ProjectRef {
                id: "0-1".into(),
                short_name: None,
                name: None,
            },
            summary: "Child".into(),
            content: None,
            visibility: None,
            parent_article: Some(ArticleParentInput {
                id: "109-812".into(),
            }),
        };

        client.create_article(&input).unwrap();

        let request = client.transport.request(0);
        let body = request.body.expect("missing request body");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&body).unwrap(),
            serde_json::json!({
                "project": {"id": "0-1"},
                "summary": "Child",
                "parentArticle": {"id": "109-812"}
            })
        );
    }

    #[test]
    fn update_article_serializes_visibility_clear_payload() {
        let client = test_client(vec![r#"{"id":"109-1","idReadable":"KB-1"}"#]);
        let input = UpdateArticleInput {
            summary: Some("Runbook".into()),
            content: None,
            visibility: Some(LimitedVisibilityInput {
                visibility_type: "LimitedVisibility",
                permitted_groups: vec![],
            }),
            parent_article: None,
        };

        client.update_article("KB-1", &input).unwrap();

        let request = client.transport.request(0);
        assert_eq!(request.method, "POST");
        let body = request.body.expect("missing request body");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&body).unwrap(),
            serde_json::json!({
                "summary": "Runbook",
                "visibility": {
                    "$type": "LimitedVisibility",
                    "permittedGroups": []
                }
            })
        );
    }

    #[test]
    fn update_article_serializes_parent_article_null() {
        let client = test_client(vec![r#"{"id":"109-813","idReadable":"GRA-A-32"}"#]);
        let input = UpdateArticleInput {
            summary: None,
            content: None,
            visibility: None,
            parent_article: Some(None),
        };

        client.update_article("GRA-A-32", &input).unwrap();

        let request = client.transport.request(0);
        let body = request.body.expect("missing request body");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&body).unwrap(),
            serde_json::json!({
                "parentArticle": null
            })
        );
    }

    #[test]
    fn search_issues_with_project() {
        let client = test_client(vec![r#"[]"#]);
        let issues = client.search_issues("bug", Some("TEST")).unwrap();
        assert!(issues.is_empty());
        let request = client.transport.request(0);
        assert!(request.url.contains("customFields%28id%2Cname%2C%24type"));
        assert!(request.url.contains("project%3A%20%7BTEST%7D%20bug"));
    }

    #[test]
    fn list_issues_requests_custom_fields_for_text_output() {
        let client = test_client(vec![r#"[]"#]);

        let issues = client.list_issues("TEST").unwrap();

        assert!(issues.is_empty());
        let request = client.transport.request(0);
        assert!(request.url.contains("customFields%28id%2Cname%2C%24type"));
        assert!(request.url.contains("project%3A%20%7BTEST%7D"));
    }

    #[test]
    fn list_issue_links_requests_enriched_linked_issues() {
        let client = test_client(vec![r#"[]"#]);

        let links = client.list_issue_links("DWP-12").unwrap();

        assert!(links.is_empty());
        let request = client.transport.request(0);
        assert!(request
            .url
            .contains("issues%28id%2CidReadable%2Csummary%2Cupdated%2Cresolved"));
        assert!(request.url.contains("customFields%28id%2Cname%2C%24type"));
    }

    #[test]
    fn list_issue_comments_uses_issue_comments_endpoint() {
        let client = test_client(vec![
            r#"[{"id":"4-17","text":"Hi","created":1,"updated":2}]"#,
        ]);

        let comments = client.list_issue_comments("DWP-12").unwrap();

        assert_eq!(comments[0].id, "4-17");
        let request = client.transport.request(0);
        assert_eq!(request.method, "GET");
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/issues/DWP-12/comments?fields=id%2Ctext%2Ccreated%2Cupdated%2Cauthor%28id%2Clogin%2CfullName%29%2Cvisibility%28%24type%2CpermittedGroups%28id%2Cname%29%29&%24top=500"
        );
    }

    #[test]
    fn update_issue_comment_without_visibility_posts_only_text() {
        let client = test_client(vec![
            r#"{"id":"4-17","text":"Updated","created":1,"updated":2}"#,
        ]);

        client
            .update_issue_comment("DWP-12", "4-17", "Updated", None)
            .unwrap();

        let request = client.transport.request(0);
        assert_eq!(request.method, "POST");
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/issues/DWP-12/comments/4-17?fields=id%2Ctext%2Ccreated%2Cupdated%2Cauthor%28id%2Clogin%2CfullName%29%2Cvisibility%28%24type%2CpermittedGroups%28id%2Cname%29%29"
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&request.body.unwrap()).unwrap(),
            serde_json::json!({"text": "Updated"})
        );
    }

    #[test]
    fn add_issue_comment_serializes_visibility_payload() {
        let client = test_client(vec![r#"{"id":"4-17","text":"Hi"}"#]);

        client
            .add_comment(
                "DWP-12",
                "Hi",
                Some(LimitedVisibilityInput {
                    visibility_type: "LimitedVisibility",
                    permitted_groups: vec![UserGroupInput { id: "3-7".into() }],
                }),
            )
            .unwrap();

        let request = client.transport.request(0);
        assert_eq!(request.method, "POST");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&request.body.unwrap()).unwrap(),
            serde_json::json!({
                "text": "Hi",
                "visibility": {
                    "$type": "LimitedVisibility",
                    "permittedGroups": [{"id": "3-7"}]
                }
            })
        );
    }

    #[test]
    fn add_article_comment_serializes_visibility_payload() {
        let client = test_client(vec![r#"{"id":"251-0","text":"Hi"}"#]);

        client
            .add_article_comment(
                "DWP-A-1",
                "Hi",
                Some(LimitedVisibilityInput {
                    visibility_type: "LimitedVisibility",
                    permitted_groups: vec![UserGroupInput { id: "3-7".into() }],
                }),
            )
            .unwrap();

        let request = client.transport.request(0);
        assert_eq!(request.method, "POST");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&request.body.unwrap()).unwrap(),
            serde_json::json!({
                "text": "Hi",
                "visibility": {
                    "$type": "LimitedVisibility",
                    "permittedGroups": [{"id": "3-7"}]
                }
            })
        );
    }

    #[test]
    fn update_issue_comment_serializes_visibility_payload() {
        let client = test_client(vec![
            r#"{"id":"4-17","text":"Updated","created":1,"updated":2}"#,
        ]);

        client
            .update_issue_comment(
                "DWP-12",
                "4-17",
                "Updated",
                Some(LimitedVisibilityInput {
                    visibility_type: "LimitedVisibility",
                    permitted_groups: vec![UserGroupInput { id: "3-7".into() }],
                }),
            )
            .unwrap();

        let request = client.transport.request(0);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&request.body.unwrap()).unwrap(),
            serde_json::json!({
                "text": "Updated",
                "visibility": {
                    "$type": "LimitedVisibility",
                    "permittedGroups": [{"id": "3-7"}]
                }
            })
        );
    }

    #[test]
    fn update_article_comment_serializes_visibility_clear_payload() {
        let client = test_client(vec![
            r#"{"id":"251-0","text":"Updated","created":1,"updated":2}"#,
        ]);

        client
            .update_article_comment(
                "DWP-A-1",
                "251-0",
                "Updated",
                Some(LimitedVisibilityInput {
                    visibility_type: "LimitedVisibility",
                    permitted_groups: vec![],
                }),
            )
            .unwrap();

        let request = client.transport.request(0);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&request.body.unwrap()).unwrap(),
            serde_json::json!({
                "text": "Updated",
                "visibility": {
                    "$type": "LimitedVisibility",
                    "permittedGroups": []
                }
            })
        );
    }

    #[test]
    fn delete_article_comment_uses_article_comment_endpoint() {
        let client = test_client(vec![r#""#]);

        client.delete_article_comment("DWP-A-1", "251-0").unwrap();

        let request = client.transport.request(0);
        assert_eq!(request.method, "DELETE");
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/articles/DWP-A-1/comments/251-0"
        );
    }

    #[test]
    fn get_article_comment_uses_article_comment_endpoint() {
        let client = test_client(vec![
            r#"{"id":"251-0","text":"Hi","created":1,"updated":2}"#,
        ]);

        let comment = client.get_article_comment("DWP-A-1", "251-0").unwrap();

        assert_eq!(comment.id, "251-0");
        let request = client.transport.request(0);
        assert_eq!(request.method, "GET");
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/articles/DWP-A-1/comments/251-0?fields=id%2Ctext%2Ccreated%2Cupdated%2Cauthor%28id%2Clogin%2CfullName%29%2Cvisibility%28%24type%2CpermittedGroups%28id%2Cname%29%29"
        );
    }

    #[test]
    fn get_issue_attachment_uses_issue_attachment_endpoint() {
        let client = test_client(vec![
            r#"{"id":"8-2897","name":"log.txt","comment":{"id":"4-17"}}"#,
        ]);

        let attachment = client.get_issue_attachment("DWP-12", "8-2897").unwrap();

        assert_eq!(attachment.id, "8-2897");
        assert_eq!(
            attachment.comment.as_ref().map(|c| c.id.as_str()),
            Some("4-17")
        );
        let request = client.transport.request(0);
        assert_eq!(request.method, "GET");
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/issues/DWP-12/attachments/8-2897?fields=id%2Cname%2Curl%2Csize%2CmimeType%2Ccreated%2Cauthor%28id%2Clogin%2CfullName%29%2Ccomment%28id%29"
        );
    }

    #[test]
    fn get_article_attachment_uses_article_attachment_endpoint() {
        let client = test_client(vec![r#"{"id":"237-3","name":"logo.png"}"#]);

        client.get_article_attachment("DWP-A-1", "237-3").unwrap();

        let request = client.transport.request(0);
        assert_eq!(request.method, "GET");
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/articles/DWP-A-1/attachments/237-3?fields=id%2Cname%2Curl%2Csize%2CmimeType%2Ccreated%2Cauthor%28id%2Clogin%2CfullName%29%2Ccomment%28id%29"
        );
    }

    #[test]
    fn delete_issue_attachment_uses_issue_attachment_endpoint() {
        let client = test_client(vec![r#""#]);

        client.delete_issue_attachment("DWP-12", "8-2897").unwrap();

        let request = client.transport.request(0);
        assert_eq!(request.method, "DELETE");
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/issues/DWP-12/attachments/8-2897"
        );
    }

    #[test]
    fn delete_article_attachment_uses_article_attachment_endpoint() {
        let client = test_client(vec![r#""#]);

        client
            .delete_article_attachment("DWP-A-1", "237-3")
            .unwrap();

        let request = client.transport.request(0);
        assert_eq!(request.method, "DELETE");
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/articles/DWP-A-1/attachments/237-3"
        );
    }

    #[test]
    fn get_issue_comment_with_attachments_requests_attachment_fields() {
        let client = test_client(vec![
            r#"{"id":"4-17","text":"Hi","attachments":[{"id":"8-2897","name":"log.txt"}]}"#,
        ]);

        let comment = client
            .get_issue_comment_with_attachments("DWP-12", "4-17")
            .unwrap();

        assert_eq!(comment.attachments.len(), 1);
        let request = client.transport.request(0);
        assert_eq!(request.method, "GET");
        assert!(request.url.contains("/api/issues/DWP-12/comments/4-17?"));
        assert!(request.url.contains("attachments%28id%2Cname%2Curl%2Csize%2CmimeType%2Ccreated%2Cauthor%28id%2Clogin%2CfullName%29%2Ccomment%28id%29%29"));
    }

    #[test]
    fn upload_issue_comment_attachment_uses_issue_comment_attachment_endpoint() {
        let client = test_client(vec![r#""#]);
        let temp = tempfile::NamedTempFile::new().unwrap();

        client
            .upload_issue_comment_attachment("DWP-12", "4-17", temp.path())
            .unwrap();

        let request = client.transport.request(0);
        assert_eq!(request.method, "POST multipart");
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/issues/DWP-12/comments/4-17/attachments"
        );
    }

    #[test]
    fn upload_article_comment_attachment_uses_article_comment_attachment_endpoint() {
        let client = test_client(vec![r#""#]);
        let temp = tempfile::NamedTempFile::new().unwrap();

        client
            .upload_article_comment_attachment("DWP-A-1", "187-66", temp.path())
            .unwrap();

        let request = client.transport.request(0);
        assert_eq!(request.method, "POST multipart");
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/articles/DWP-A-1/comments/187-66/attachments"
        );
    }

    #[test]
    fn download_attachment_file_resolves_relative_urls() {
        let client = test_client(vec!["file bytes"]);

        let bytes = client
            .download_attachment_file("/api/files/8-2897?sign=fake")
            .unwrap();

        assert_eq!(bytes, b"file bytes");
        let request = client.transport.request(0);
        assert_eq!(request.method, "GET bytes");
        assert_eq!(
            request.url,
            "https://test.youtrack.cloud/api/files/8-2897?sign=fake"
        );
    }

    #[test]
    fn base_url_strips_trailing_slash() {
        let client = YtClient::new(
            YtdConfig {
                url: "https://test.youtrack.cloud/".into(),
                token: "t".into(),
            },
            MockTransport::new(vec![]),
        );
        assert_eq!(client.base_url, "https://test.youtrack.cloud/api");
    }

    #[test]
    fn url_encoding() {
        assert_eq!(urlenc("hello world"), "hello%20world");
        assert_eq!(urlenc("a=b&c"), "a%3Db%26c");
    }

    #[test]
    fn detects_project_database_ids() {
        assert!(YtClient::<MockTransport>::is_project_database_id("0-96"));
        assert!(!YtClient::<MockTransport>::is_project_database_id("DWP"));
        assert!(!YtClient::<MockTransport>::is_project_database_id("0-ABC"));
    }

    #[test]
    fn article_query_matches_summary_content_and_id() {
        let article = Article {
            id: "109-1".into(),
            id_readable: Some("DWP-A-1".into()),
            summary: Some("Testartikel 1".into()),
            content: Some("Technische Dokumentation".into()),
            created: None,
            updated: None,
            reporter: None,
            project: None,
            visibility: None,
            parent_article: None,
        };

        assert!(YtClient::<MockTransport>::article_matches_query(
            &article,
            "testartikel"
        ));
        assert!(YtClient::<MockTransport>::article_matches_query(
            &article,
            "DOKUMENTATION"
        ));
        assert!(YtClient::<MockTransport>::article_matches_query(
            &article, "dwp-a-1"
        ));
        assert!(!YtClient::<MockTransport>::article_matches_query(
            &article, "foobar"
        ));
    }

    #[test]
    fn list_articles_uses_project_articles_endpoint() {
        let client = test_client(vec![
            r#"{"id":"0-96","name":"DW Playground","shortName":"DWP","archived":false,"description":null}"#,
            r#"[]"#,
        ]);

        let articles = client.list_articles("DWP").unwrap();
        assert!(articles.is_empty());

        let requests = client.transport.requests.borrow();
        assert_eq!(requests[0].method, "GET");
        assert_eq!(
            requests[0].url,
            "https://test.youtrack.cloud/api/admin/projects/DWP?fields=id%2Cname%2CshortName%2Carchived%2Cdescription"
        );
        assert_eq!(requests[1].method, "GET");
        assert_eq!(
            requests[1].url,
            "https://test.youtrack.cloud/api/admin/projects/0-96/articles?fields=id%2CidReadable%2Csummary%2Cupdated%2Cproject%28id%2CshortName%2Cname%29&%24top=500"
        );
    }

    #[test]
    fn search_articles_filters_project_articles_locally() {
        let client = test_client(vec![
            r#"{"id":"0-96","name":"DW Playground","shortName":"DWP","archived":false,"description":null}"#,
            r#"[
                {"id":"109-787","idReadable":"DWP-A-1","summary":"Testartikel 1","content":"Alpha","updated":1,"project":{"id":"0-96","shortName":"DWP","name":"DW Playground"}},
                {"id":"109-788","idReadable":"DWP-A-2","summary":"Handbuch","content":"Beta","updated":2,"project":{"id":"0-96","shortName":"DWP","name":"DW Playground"}}
            ]"#,
        ]);

        let articles = client.search_articles("testartikel", Some("DWP")).unwrap();
        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0].id_readable.as_deref(), Some("DWP-A-1"));
    }

    #[test]
    fn search_articles_without_project_uses_global_articles_endpoint() {
        let client = test_client(vec![
            r#"[
                {"id":"109-787","idReadable":"DWP-A-1","summary":"Testartikel 1","content":"Alpha","updated":1,"project":{"id":"0-96","shortName":"DWP","name":"DW Playground"}}
            ]"#,
        ]);

        let articles = client.search_articles("alpha", None).unwrap();
        assert_eq!(articles.len(), 1);

        let requests = client.transport.requests.borrow();
        assert_eq!(requests[0].method, "GET");
        assert_eq!(
            requests[0].url,
            "https://test.youtrack.cloud/api/articles?fields=id%2CidReadable%2Csummary%2Ccontent%2Cupdated%2Cproject%28id%2CshortName%2Cname%29&%24top=500"
        );
    }
}
