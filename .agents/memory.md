# Project Memory

This file is for non-obvious YouTrack API behavior and durable external facts that are not already clear from the code, tests, or current documentation. Do not use it as a project history.

## YouTrack API Reference

Official REST API reference: https://www.jetbrains.com/help/youtrack/devportal/rest-api-reference.html

Useful endpoint families:

- Issues: `/api/issues`
- Issue comments: `/api/issues/{issueID}/comments`
- Articles: `/api/articles`
- Article comments: `/api/articles/{articleID}/comments`
- Projects: `/api/admin/projects`
- Current user: `/api/users/me`
- Agile boards and sprints: `/api/agiles`

## Response Size Controls

YouTrack returns large responses when fields are omitted. Use explicit `fields` query parameters for API calls.

YouTrack's default `$top` is 42. Set `$top` explicitly for list-style calls that need predictable result counts.

## Project IDs In Create Payloads

Issue creation requires the YouTrack project database ID in the payload, not only the project short name. Resolve short names before creating issues.

Article creation accepts a project in the create payload, but the article `project` field is read-only after creation.

Article parent assignment accepts `parentArticle` in create/update payloads, but the nested `id` must be the internal YouTrack article ID. The CLI accepts readable article IDs and resolves them before sending the payload.

## Article Search

`GET /api/articles` accepts a `query` parameter. Project filtering can be expressed through YouTrack search syntax instead of a separate article search endpoint.

## Parent-Scoped Comments

Specific comment get/update/delete operations require the parent issue or article path. A raw comment ID is not enough to address a comment.

## Parent-Scoped Attachments

Attachment get/delete/download operations require the parent issue or article path, even when YouTrack reports the attachment on a comment.

## Comment Attachment Upload

Verified against YouTrack Cloud on 2026-04-28 and confirmed by JetBrains support on 2026-04-27: existing issue and article comments accept multipart uploads directly on parent-scoped comment attachment endpoints:

- `POST /api/issues/{issueID}/comments/{commentID}/attachments`
- `POST /api/articles/{articleID}/comments/{commentID}/attachments`

The multipart field name `file` works. Reading the comment back with `attachments(...,comment(id))` shows the uploaded attachment associated with the target comment.

## Board-Scoped Sprints

Sprint get/update/delete operations are scoped under an Agile board. A raw sprint ID is not enough to address a sprint.

## Sprint Ticket Assignment

Adding and removing tickets from sprints uses Agile/Sprint-scoped endpoints:

- Add: `POST /api/agiles/{agileID}/sprints/{sprintID}/issues`
- Remove: `DELETE /api/agiles/{agileID}/sprints/{sprintID}/issues/{issueDatabaseID}`
- List: `GET /api/agiles/{agileID}/sprints/{sprintID}?fields=issues(...)`

The assignment payload needs the internal issue database ID. The CLI accepts readable issue IDs and resolves them before calling the sprint assignment API.
