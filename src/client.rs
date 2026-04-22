use crate::error::YtdError;
use crate::types::*;
use serde::de::DeserializeOwned;
use std::path::Path;

// --- Transport trait ---

pub trait HttpTransport {
    fn get(&self, url: &str, token: &str) -> Result<String, YtdError>;
    fn post(&self, url: &str, token: &str, body: &str) -> Result<String, YtdError>;
    fn post_multipart(&self, url: &str, token: &str, file_path: &Path, file_name: &str) -> Result<String, YtdError>;
    fn delete(&self, url: &str, token: &str) -> Result<(), YtdError>;
}

// --- ureq implementation ---

pub struct UreqTransport;

impl HttpTransport for UreqTransport {
    fn get(&self, url: &str, token: &str) -> Result<String, YtdError> {
        let mut resp = ureq::get(url)
            .header("Authorization", &format!("Bearer {token}"))
            .header("Accept", "application/json")
            .call()
            .map_err(|e| YtdError::Http(e.to_string()))?;
        let body = resp.body_mut().read_to_string().map_err(|e| YtdError::Http(e.to_string()))?;
        Ok(body)
    }

    fn post(&self, url: &str, token: &str, body: &str) -> Result<String, YtdError> {
        let mut resp = ureq::post(url)
            .header("Authorization", &format!("Bearer {token}"))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .send(body.as_bytes())
            .map_err(|e| YtdError::Http(e.to_string()))?;
        let resp_body = resp.body_mut().read_to_string().map_err(|e| YtdError::Http(e.to_string()))?;
        Ok(resp_body)
    }

    fn post_multipart(&self, url: &str, token: &str, file_path: &Path, _file_name: &str) -> Result<String, YtdError> {
        let file_bytes = std::fs::read(file_path)?;
        let name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
        let mime = mime_from_extension(file_path);

        // Build multipart body manually
        let boundary = format!("----ytd{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis());

        let mut body = Vec::new();
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(format!("Content-Disposition: form-data; name=\"file\"; filename=\"{name}\"\r\n").as_bytes());
        body.extend_from_slice(format!("Content-Type: {mime}\r\n\r\n").as_bytes());
        body.extend_from_slice(&file_bytes);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        let mut resp = ureq::post(url)
            .header("Authorization", &format!("Bearer {token}"))
            .header("Accept", "application/json")
            .header("Content-Type", &format!("multipart/form-data; boundary={boundary}"))
            .send(&body[..])
            .map_err(|e| YtdError::Http(e.to_string()))?;
        let resp_body = resp.body_mut().read_to_string().map_err(|e| YtdError::Http(e.to_string()))?;
        Ok(resp_body)
    }

    fn delete(&self, url: &str, token: &str) -> Result<(), YtdError> {
        ureq::delete(url)
            .header("Authorization", &format!("Bearer {token}"))
            .call()
            .map_err(|e| YtdError::Http(e.to_string()))?;
        Ok(())
    }
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
                if i > 0 { url.push('&'); }
                url.push_str(&format!("{}={}", urlenc(k), urlenc(v)));
            }
        }
        url
    }

    fn log_request(&self, method: &str, url: &str, body: Option<&str>) {
        if !self.verbose { return; }
        eprintln!(">> {method} {url}");
        if let Some(b) = body {
            eprintln!(">> {b}");
        }
    }

    fn log_response(&self, body: &str) {
        if !self.verbose { return; }
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

    fn post<R: DeserializeOwned>(&self, path: &str, body: &impl serde::Serialize, params: &[(&str, &str)]) -> Result<R, YtdError> {
        let url = self.url(path, params);
        let json = serde_json::to_string(body)?;
        self.log_request("POST", &url, Some(&json));
        let resp = self.transport.post(&url, &self.token, &json)?;
        self.log_response(&resp);
        serde_json::from_str(&resp).map_err(YtdError::from)
    }

    fn post_no_response(&self, path: &str, body: &impl serde::Serialize, params: &[(&str, &str)]) -> Result<(), YtdError> {
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

    fn upload(&self, path: &str, file_path: &Path, params: &[(&str, &str)]) -> Result<String, YtdError> {
        let url = self.url(path, params);
        let name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
        self.log_request("POST multipart", &url, Some(name));
        let resp = self.transport.post_multipart(&url, &self.token, file_path, name)?;
        self.log_response(&resp);
        Ok(resp)
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

    fn resolve_project_id(&self, project_ref: &str) -> Result<String, YtdError> {
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

        if let Some(project) = projects.iter().find(|p| p.short_name.eq_ignore_ascii_case(project_ref)) {
            return Ok(project.id.clone());
        }

        Err(YtdError::Input(format!("Project not found: {project_ref}")))
    }

    fn article_matches_query(article: &Article, query: &str) -> bool {
        let query = query.to_lowercase();
        if query.is_empty() {
            return true;
        }

        article.id_readable.as_deref().map(|s| s.to_lowercase().contains(&query)).unwrap_or(false)
            || article.summary.as_deref().map(|s| s.to_lowercase().contains(&query)).unwrap_or(false)
            || article.content.as_deref().map(|s| s.to_lowercase().contains(&query)).unwrap_or(false)
    }

    // --- Users ---

    pub fn get_me(&self) -> Result<User, YtdError> {
        self.get("/users/me", &[("fields", "id,login,fullName,email")])
    }

    // --- Projects ---

    pub fn list_projects(&self) -> Result<Vec<Project>, YtdError> {
        self.get("/admin/projects", &[
            ("fields", "id,name,shortName,archived,description"),
            ("$top", "500"),
        ])
    }

    pub fn get_project(&self, id: &str) -> Result<Project, YtdError> {
        self.get(&format!("/admin/projects/{id}"), &[
            ("fields", "id,name,shortName,archived,description"),
        ])
    }

    // --- Articles ---

    pub fn search_articles(&self, query: &str, project: Option<&str>) -> Result<Vec<Article>, YtdError> {
        let mut articles: Vec<Article> = match project {
            Some(project_ref) => {
                let project_id = self.resolve_project_id(project_ref)?;
                self.get(&format!("/admin/projects/{project_id}/articles"), &[
                    ("fields", "id,idReadable,summary,content,updated,project(id,shortName,name)"),
                    ("$top", "500"),
                ])?
            }
            None => self.get("/articles", &[
                ("fields", "id,idReadable,summary,content,updated,project(id,shortName,name)"),
                ("$top", "500"),
            ])?,
        };
        articles.retain(|article| Self::article_matches_query(article, query));
        for article in &mut articles {
            article.content = None;
        }
        Ok(articles)
    }

    pub fn list_articles(&self, project: &str) -> Result<Vec<Article>, YtdError> {
        let project_id = self.resolve_project_id(project)?;
        self.get(&format!("/admin/projects/{project_id}/articles"), &[
            ("fields", "id,idReadable,summary,updated,project(id,shortName,name)"),
            ("$top", "500"),
        ])
    }

    pub fn get_article(&self, id: &str) -> Result<Article, YtdError> {
        self.get(&format!("/articles/{id}"), &[
            ("fields", "id,idReadable,summary,content,created,updated,reporter(id,login,fullName),project(id,shortName,name)"),
        ])
    }

    pub fn create_article(&self, input: &CreateArticleInput) -> Result<Article, YtdError> {
        self.post("/articles", input, &[("fields", "id,idReadable")])
    }

    pub fn update_article(&self, id: &str, input: &UpdateArticleInput) -> Result<Article, YtdError> {
        self.post(&format!("/articles/{id}"), input, &[("fields", "id,idReadable")])
    }

    pub fn append_to_article(&self, id: &str, text: &str) -> Result<(), YtdError> {
        let article = self.get_article(id)?;
        let current = article.content.unwrap_or_default();
        let new_content = format!("{current}{text}");
        let input = UpdateArticleInput {
            summary: None,
            content: Some(new_content),
        };
        self.update_article(id, &input)?;
        Ok(())
    }

    pub fn delete_article(&self, id: &str) -> Result<(), YtdError> {
        self.delete(&format!("/articles/{id}"))
    }

    // --- Article Comments ---

    pub fn list_article_comments(&self, article_id: &str) -> Result<Vec<ArticleComment>, YtdError> {
        self.get(&format!("/articles/{article_id}/comments"), &[
            ("fields", "id,text,created,updated,author(id,login,fullName)"),
            ("$top", "500"),
        ])
    }

    pub fn add_article_comment(&self, article_id: &str, text: &str) -> Result<ArticleComment, YtdError> {
        let input = CommentInput { text: text.to_string() };
        self.post(&format!("/articles/{article_id}/comments"), &input, &[
            ("fields", "id,text,created,author(id,login,fullName)"),
        ])
    }

    // --- Article Attachments ---

    pub fn list_article_attachments(&self, article_id: &str) -> Result<Vec<Attachment>, YtdError> {
        self.get(&format!("/articles/{article_id}/attachments"), &[
            ("fields", "id,name,url,size,mimeType,created,author(id,login,fullName)"),
            ("$top", "500"),
        ])
    }

    pub fn upload_article_attachment(&self, article_id: &str, file_path: &Path) -> Result<(), YtdError> {
        self.upload(&format!("/articles/{article_id}/attachments"), file_path, &[])?;
        Ok(())
    }

    // --- Issues ---

    pub fn search_issues(&self, query: &str, project: Option<&str>) -> Result<Vec<Issue>, YtdError> {
        let q = match project {
            Some(p) => format!("project: {{{p}}} {query}"),
            None => query.to_string(),
        };
        self.get("/issues", &[
            ("fields", "id,idReadable,summary,created,updated,resolved,project(id,shortName,name)"),
            ("query", &q),
            ("$top", "100"),
        ])
    }

    pub fn list_issues(&self, project: &str) -> Result<Vec<Issue>, YtdError> {
        self.get("/issues", &[
            ("fields", "id,idReadable,summary,created,updated,resolved,project(id,shortName,name)"),
            ("query", &format!("project: {{{project}}}")),
            ("$top", "500"),
        ])
    }

    pub fn get_issue(&self, id: &str) -> Result<Issue, YtdError> {
        self.get(&format!("/issues/{id}"), &[
            ("fields", "id,idReadable,summary,description,created,updated,resolved,reporter(id,login,fullName),project(id,shortName,name),tags(id,name),comments(id,text,created,author(id,login,fullName)),customFields(id,name,$type,value(id,name,login,fullName,minutes,presentation,$type))"),
        ])
    }

    pub fn create_issue(&self, input: &CreateIssueInput) -> Result<Issue, YtdError> {
        self.post("/issues", input, &[("fields", "id,idReadable")])
    }

    pub fn update_issue(&self, id: &str, input: &UpdateIssueInput) -> Result<Issue, YtdError> {
        self.post(&format!("/issues/{id}"), input, &[("fields", "id,idReadable")])
    }

    pub fn delete_issue(&self, id: &str) -> Result<(), YtdError> {
        self.delete(&format!("/issues/{id}"))
    }

    // --- Issue Comments ---

    pub fn add_comment(&self, issue_id: &str, text: &str) -> Result<IssueComment, YtdError> {
        let input = CommentInput { text: text.to_string() };
        self.post(&format!("/issues/{issue_id}/comments"), &input, &[
            ("fields", "id,text,created,author(id,login,fullName)"),
        ])
    }

    // --- Tags ---

    pub fn list_tags(&self) -> Result<Vec<Tag>, YtdError> {
        self.get("/tags", &[
            ("fields", "id,name"),
            ("$top", "500"),
        ])
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
            ("fields", "id,direction,linkType(id,name,sourceToTarget,targetToSource),issues(id,idReadable,summary)"),
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
        self.get(&format!("/issues/{issue_id}/attachments"), &[
            ("fields", "id,name,url,size,mimeType,created,author(id,login,fullName)"),
            ("$top", "500"),
        ])
    }

    pub fn upload_attachment(&self, issue_id: &str, file_path: &Path) -> Result<(), YtdError> {
        self.upload(&format!("/issues/{issue_id}/attachments"), file_path, &[])?;
        Ok(())
    }

    // --- Time Tracking ---

    pub fn list_work_items(&self, issue_id: &str) -> Result<Vec<WorkItem>, YtdError> {
        self.get(&format!("/issues/{issue_id}/timeTracking/workItems"), &[
            ("fields", "id,duration(minutes,presentation),date,text,author(id,login,fullName),type(id,name)"),
            ("$top", "500"),
        ])
    }

    pub fn add_work_item(&self, issue_id: &str, input: &CreateWorkItemInput) -> Result<WorkItem, YtdError> {
        self.post(&format!("/issues/{issue_id}/timeTracking/workItems"), input, &[
            ("fields", "id,duration(minutes,presentation),date,text"),
        ])
    }

    // --- Custom Fields ---

    pub fn set_custom_field(&self, issue_id: &str, body: &serde_json::Value) -> Result<(), YtdError> {
        let url = self.url(&format!("/issues/{issue_id}"), &[("fields", "id")]);
        let json = serde_json::to_string(body)?;
        self.transport.post(&url, &self.token, &json)?;
        Ok(())
    }

    // --- Saved Searches ---

    pub fn list_saved_queries(&self) -> Result<Vec<SavedQuery>, YtdError> {
        self.get("/savedQueries", &[
            ("fields", "id,name,query"),
            ("$top", "500"),
        ])
    }

    // --- Activities ---

    pub fn list_activities(&self, issue_id: &str, categories: Option<&str>) -> Result<Vec<ActivityItem>, YtdError> {
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

    pub fn list_agiles(&self) -> Result<Vec<Agile>, YtdError> {
        self.get("/agiles", &[
            ("fields", "id,name,projects(id,shortName,name)"),
            ("$top", "500"),
        ])
    }

    pub fn get_agile(&self, id: &str) -> Result<Agile, YtdError> {
        self.get(&format!("/agiles/{id}"), &[
            ("fields", "id,name,projects(id,shortName,name),sprints(id,name,start,finish,archived)"),
        ])
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

    struct MockTransport {
        responses: RefCell<Vec<String>>,
        requests: RefCell<Vec<String>>,
    }

    impl MockTransport {
        fn new(responses: Vec<&str>) -> Self {
            Self {
                responses: RefCell::new(responses.into_iter().rev().map(String::from).collect()),
                requests: RefCell::new(vec![]),
            }
        }
    }

    impl HttpTransport for MockTransport {
        fn get(&self, url: &str, _token: &str) -> Result<String, YtdError> {
            self.requests.borrow_mut().push(format!("GET {url}"));
            self.responses.borrow_mut().pop()
                .ok_or_else(|| YtdError::Http("No more mock responses".into()))
        }
        fn post(&self, url: &str, _token: &str, _body: &str) -> Result<String, YtdError> {
            self.requests.borrow_mut().push(format!("POST {url}"));
            self.responses.borrow_mut().pop()
                .ok_or_else(|| YtdError::Http("No more mock responses".into()))
        }
        fn post_multipart(&self, url: &str, _token: &str, _file: &Path, _name: &str) -> Result<String, YtdError> {
            self.requests.borrow_mut().push(format!("POST multipart {url}"));
            self.responses.borrow_mut().pop()
                .ok_or_else(|| YtdError::Http("No more mock responses".into()))
        }
        fn delete(&self, url: &str, _token: &str) -> Result<(), YtdError> {
            self.requests.borrow_mut().push(format!("DELETE {url}"));
            self.responses.borrow_mut().pop();
            Ok(())
        }
    }

    fn test_client(responses: Vec<&str>) -> YtClient<MockTransport> {
        YtClient::new(
            YtdConfig { url: "https://test.youtrack.cloud".into(), token: "perm:test".into() },
            MockTransport::new(responses),
        )
    }

    #[test]
    fn get_me() {
        let client = test_client(vec![r#"{"id":"1","login":"admin","fullName":"Admin","email":"a@b.com"}"#]);
        let user = client.get_me().unwrap();
        assert_eq!(user.login, "admin");
    }

    #[test]
    fn list_projects() {
        let client = test_client(vec![r#"[{"id":"1","name":"Test","shortName":"TEST","archived":false,"description":null}]"#]);
        let projects = client.list_projects().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].short_name, "TEST");
    }

    #[test]
    fn create_issue() {
        let client = test_client(vec![r#"{"id":"2-1","idReadable":"TEST-1"}"#]);
        let input = CreateIssueInput {
            project: ProjectRef { id: "1".into(), short_name: None, name: None },
            summary: "Test".into(),
            description: None,
        };
        let issue = client.create_issue(&input).unwrap();
        assert_eq!(issue.id_readable.as_deref(), Some("TEST-1"));
    }

    #[test]
    fn search_issues_with_project() {
        let client = test_client(vec![r#"[]"#]);
        let issues = client.search_issues("bug", Some("TEST")).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn base_url_strips_trailing_slash() {
        let client = YtClient::new(
            YtdConfig { url: "https://test.youtrack.cloud/".into(), token: "t".into() },
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
        };

        assert!(YtClient::<MockTransport>::article_matches_query(&article, "testartikel"));
        assert!(YtClient::<MockTransport>::article_matches_query(&article, "DOKUMENTATION"));
        assert!(YtClient::<MockTransport>::article_matches_query(&article, "dwp-a-1"));
        assert!(!YtClient::<MockTransport>::article_matches_query(&article, "foobar"));
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
        assert_eq!(requests[0], "GET https://test.youtrack.cloud/api/admin/projects/DWP?fields=id%2Cname%2CshortName%2Carchived%2Cdescription");
        assert_eq!(requests[1], "GET https://test.youtrack.cloud/api/admin/projects/0-96/articles?fields=id%2CidReadable%2Csummary%2Cupdated%2Cproject%28id%2CshortName%2Cname%29&%24top=500");
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
        assert_eq!(requests[0], "GET https://test.youtrack.cloud/api/articles?fields=id%2CidReadable%2Csummary%2Ccontent%2Cupdated%2Cproject%28id%2CshortName%2Cname%29&%24top=500");
    }
}
