use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

use crate::auth::{UserSession, get_user};
#[cfg(feature = "ssr")]
use crate::models::{Flag, Idea};
use crate::models::{IdeaWithAuthor, User};
#[cfg(feature = "ssr")]
use async_stream::try_stream;
#[cfg(feature = "ssr")]
use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
};
#[cfg(feature = "ssr")]
use axum_extra::extract::CookieJar;
#[cfg(feature = "ssr")]
use futures_util::TryStreamExt;

mod components;
use components::AdminDashboard;

// ============================================================================
// SERVER FUNCTIONS
// ============================================================================

#[server]
pub async fn get_admin_stats() -> Result<AdminStats, ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    let (total_ideas, total_votes) = Idea::get_statistics()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch statistics: {}", e)))?;

    let total_users = User::get_all()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to count users: {}", e)))?
        .len() as i64;

    let flagged_items = Flag::get_flagged_items()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get flagged items: {}", e)))?
        .len() as i64;

    Ok(AdminStats {
        total_ideas,
        total_votes,
        total_users,
        flagged_items,
    })
}

#[server]
pub async fn get_flagged_content() -> Result<Vec<FlaggedItemDetail>, ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    let flagged_items = Flag::get_flagged_items()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get flagged items: {}", e)))?;

    let mut details = Vec::new();
    for item in flagged_items {
        let content = if item.target_type == "idea" {
            Idea::get_with_author(item.target_id)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to get idea: {}", e)))?
                .map(|iwa| format!("{}: {}", iwa.idea.title, iwa.idea.content))
        } else {
            // For comments, just get the content
            Some(format!("Comment ID: {}", item.target_id))
        };

        if let Some(content_text) = content {
            details.push(FlaggedItemDetail {
                target_type: item.target_type,
                target_id: item.target_id,
                flag_count: item.flag_count,
                content_preview: content_text.chars().take(200).collect(),
            });
        }
    }

    Ok(details)
}

#[server]
pub async fn clear_flags_action(target_type: String, target_id: i32) -> Result<(), ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    Flag::clear_flags(&target_type, target_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to clear flags: {}", e)))?;

    Ok(())
}

#[server]
pub async fn mark_idea_off_topic_action(
    idea_id: i32,
    is_off_topic: bool,
) -> Result<(), ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    Idea::mark_off_topic(idea_id, is_off_topic)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to mark as off-topic: {}", e)))?;

    // Clear flags when marking off-topic
    if is_off_topic {
        Flag::clear_flags("idea", idea_id)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to clear flags: {}", e)))?;
    }

    Ok(())
}

#[server]
pub async fn delete_idea_action(idea_id: i32) -> Result<(), ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    Idea::delete(idea_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete idea: {}", e)))?;

    Ok(())
}

#[server]
pub async fn update_idea_stage_action(idea_id: i32, stage: String) -> Result<(), ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    Idea::update_stage(idea_id, stage)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update stage: {}", e)))?;

    Ok(())
}

#[server]
pub async fn toggle_idea_pin_action(idea_id: i32) -> Result<bool, ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    Idea::toggle_pin(idea_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to toggle pin: {}", e)))
}

#[server]
pub async fn get_off_topic_ideas() -> Result<Vec<IdeaWithAuthor>, ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    Idea::get_off_topic()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get off-topic ideas: {}", e)))
}

#[server]
pub async fn get_all_users_admin() -> Result<Vec<User>, ServerFnError> {
    use crate::auth::require_admin;
    require_admin().await?;

    User::get_all()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get users: {}", e)))
}

#[server]
pub async fn update_user_role_action(user_id: i32, role: i16) -> Result<(), ServerFnError> {
    use crate::auth::require_admin;
    require_admin().await?;

    User::update_role(user_id, role)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update role: {}", e)))
}

#[server]
pub async fn delete_user_action(user_id: i32) -> Result<(), ServerFnError> {
    use crate::auth::require_admin;
    require_admin().await?;

    User::delete(user_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete user: {}", e)))
}

#[cfg(feature = "ssr")]
pub async fn admin_export_ideas_csv(jar: CookieJar) -> Result<impl IntoResponse, StatusCode> {
    require_admin_cookie_jar(&jar)?;
    Ok((csv_download_headers("ideas_export.csv")?, ideas_csv_body()))
}

#[cfg(feature = "ssr")]
pub async fn admin_export_comments_csv(jar: CookieJar) -> Result<impl IntoResponse, StatusCode> {
    require_admin_cookie_jar(&jar)?;
    Ok((
        csv_download_headers("comments_export.csv")?,
        comments_csv_body(),
    ))
}

// ============================================================================
// TYPES
// ============================================================================

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AdminStats {
    pub total_ideas: i64,
    pub total_votes: i64,
    pub total_users: i64,
    pub flagged_items: i64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FlaggedItemDetail {
    pub target_type: String,
    pub target_id: i32,
    pub flag_count: i64,
    pub content_preview: String,
}

// ============================================================================
// COMPONENTS
// ============================================================================

#[component]
pub fn AdminPage() -> impl IntoView {
    let user_resource = Resource::new(|| (), |_| async { get_user().await });

    view! {
        <Title text="Admin Dashboard - UAB IT Idea Board"/>
        <Suspense fallback=|| view! { <p>"Loading…"</p> }>
            {move || user_resource.get().map(|user_result: Result<Option<UserSession>, ServerFnError>| {
                match user_result {
                    Ok(Some(user)) if user.is_moderator() => {
                        view! { <AdminDashboard user=user /> }.into_any()
                    }
                    Ok(Some(_)) => {
                        view! {
                            <div class="container page">
                                <p class="error">"Access denied. Moderator privileges required."</p>
                                <A href="/">"Back to Home"</A>
                            </div>
                        }.into_any()
                    }
                    Ok(None) => {
                        view! {
                            <div class="container page">
                                <p class="error">"Please log in to access the admin dashboard."</p>
                                <A href="/login">"Go to Login"</A>
                            </div>
                        }.into_any()
                    }
                    Err(_) => {
                        view! {
                            <div class="container page">
                                <p class="error">"Failed to load user session."</p>
                            </div>
                        }.into_any()
                    }
                }
            })}
        </Suspense>
    }
}

// ============================================================================
// HELPERS
// ============================================================================

fn role_name(role: i16) -> &'static str {
    match role {
        2 => "Admin",
        1 => "Moderator",
        _ => "User",
    }
}

#[cfg(feature = "ssr")]
fn csv_escape(value: &str) -> String {
    let trimmed = value.trim_start_matches([' ', '\t']);
    let mut sanitized = value.to_string();
    if matches!(trimmed.chars().next(), Some('=' | '+' | '-' | '@')) {
        sanitized.insert(0, '\'');
    }
    let escaped = sanitized.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}

#[cfg(feature = "ssr")]
fn csv_download_headers(filename: &str) -> Result<HeaderMap, StatusCode> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    let content_disposition =
        HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")).map_err(|e| {
            tracing::error!("Failed to build content disposition header: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    headers.insert(header::CONTENT_DISPOSITION, content_disposition);
    Ok(headers)
}

#[cfg(feature = "ssr")]
fn require_admin_cookie_jar(jar: &CookieJar) -> Result<(), StatusCode> {
    let Some(cookie) = jar.get("user_session") else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let session: UserSession = serde_json::from_str(cookie.value()).map_err(|e| {
        tracing::warn!("Rejected admin CSV export due to invalid session cookie: {e}");
        StatusCode::UNAUTHORIZED
    })?;
    if session.is_admin() {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

#[cfg(feature = "ssr")]
fn ideas_csv_body() -> Body {
    use sqlx::Row;

    let mut rows = sqlx::query(
        r#"
        SELECT
            i.id,
            i.user_id,
            u.name AS author_name,
            u.email AS author_email,
            i.title,
            i.content,
            i.tags,
            i.stage,
            i.is_public,
            i.is_off_topic,
            i.comments_enabled,
            i.vote_count,
            i.pinned_at,
            i.created_at
        FROM ideas i
        INNER JOIN users u ON i.user_id = u.id
        ORDER BY i.created_at DESC
        "#,
    )
    .fetch(crate::database::get_db());

    let stream = try_stream! {
        yield Bytes::from_static(
            b"id,user_id,author_name,author_email,title,content,tags,stage,is_public,is_off_topic,comments_enabled,vote_count,pinned_at,created_at\n",
        );

        while let Some(row) = rows.try_next().await? {
            let id: i32 = row.try_get("id")?;
            let user_id: i32 = row.try_get("user_id")?;
            let author_name: String = row.try_get("author_name")?;
            let author_email: String = row.try_get("author_email")?;
            let title: String = row.try_get("title")?;
            let content: String = row.try_get("content")?;
            let tags: String = row.try_get("tags")?;
            let stage: String = row.try_get("stage")?;
            let is_public: bool = row.try_get("is_public")?;
            let is_off_topic: bool = row.try_get("is_off_topic")?;
            let comments_enabled: bool = row.try_get("comments_enabled")?;
            let vote_count: i32 = row.try_get("vote_count")?;
            let pinned_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("pinned_at")?;
            let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at")?;

            let line = format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                id,
                user_id,
                csv_escape(&author_name),
                csv_escape(&author_email),
                csv_escape(&title),
                csv_escape(&content),
                csv_escape(&tags),
                csv_escape(&stage),
                is_public,
                is_off_topic,
                comments_enabled,
                vote_count,
                csv_escape(&pinned_at.map(|x| x.to_rfc3339()).unwrap_or_default()),
                csv_escape(&created_at.to_rfc3339()),
            );
            yield Bytes::from(line);
        }
    };
    Body::from_stream(stream.map_err(|e: sqlx::Error| {
        tracing::error!("Ideas CSV stream failed: {e}");
        std::io::Error::other(e)
    }))
}

#[cfg(feature = "ssr")]
fn comments_csv_body() -> Body {
    use sqlx::Row;

    let mut rows = sqlx::query(
        r#"
        SELECT
            c.id,
            c.idea_id,
            i.title AS idea_title,
            c.user_id,
            u.name AS author_name,
            u.email AS author_email,
            c.content,
            c.is_pinned,
            c.is_deleted,
            c.created_at
        FROM comments c
        INNER JOIN users u ON c.user_id = u.id
        INNER JOIN ideas i ON c.idea_id = i.id
        ORDER BY c.created_at DESC
        "#,
    )
    .fetch(crate::database::get_db());

    let stream = try_stream! {
        yield Bytes::from_static(
            b"id,idea_id,idea_title,user_id,author_name,author_email,content,is_pinned,is_deleted,created_at\n",
        );

        while let Some(row) = rows.try_next().await? {
            let id: i32 = row.try_get("id")?;
            let idea_id: i32 = row.try_get("idea_id")?;
            let idea_title: String = row.try_get("idea_title")?;
            let user_id: i32 = row.try_get("user_id")?;
            let author_name: String = row.try_get("author_name")?;
            let author_email: String = row.try_get("author_email")?;
            let content: String = row.try_get("content")?;
            let is_pinned: bool = row.try_get("is_pinned")?;
            let is_deleted: bool = row.try_get("is_deleted")?;
            let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at")?;

            let line = format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                id,
                idea_id,
                csv_escape(&idea_title),
                user_id,
                csv_escape(&author_name),
                csv_escape(&author_email),
                csv_escape(&content),
                is_pinned,
                is_deleted,
                csv_escape(&created_at.to_rfc3339()),
            );
            yield Bytes::from(line);
        }
    };
    Body::from_stream(stream.map_err(|e: sqlx::Error| {
        tracing::error!("Comments CSV stream failed: {e}");
        std::io::Error::other(e)
    }))
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::{csv_escape, require_admin_cookie_jar};
    use crate::auth::UserSession;
    use axum::http::StatusCode;
    use axum_extra::extract::{CookieJar, cookie::Cookie};

    #[test]
    fn csv_escape_quotes_and_escapes_embedded_quotes() {
        assert_eq!(csv_escape("he said \"hello\""), "\"he said \"\"hello\"\"\"");
    }

    #[test]
    fn csv_escape_prefixes_dangerous_formula_values() {
        assert_eq!(csv_escape("=1+1"), "\"'=1+1\"");
        assert_eq!(csv_escape(" +SUM(A1:A2)"), "\"' +SUM(A1:A2)\"");
        assert_eq!(csv_escape("@cmd"), "\"'@cmd\"");
    }

    #[test]
    fn require_admin_cookie_jar_denies_missing_cookie() {
        let jar = CookieJar::new();
        assert_eq!(
            require_admin_cookie_jar(&jar),
            Err(StatusCode::UNAUTHORIZED)
        );
    }

    #[test]
    fn require_admin_cookie_jar_denies_non_admin() {
        let session = UserSession {
            id: 1,
            email: "mod@example.com".to_string(),
            name: "Mod".to_string(),
            role: 1,
        };
        let session_json =
            serde_json::to_string(&session).expect("session serialization should succeed");
        let jar = CookieJar::new().add(Cookie::new("user_session", session_json));
        assert_eq!(require_admin_cookie_jar(&jar), Err(StatusCode::FORBIDDEN));
    }

    #[test]
    fn require_admin_cookie_jar_allows_admin() {
        let session = UserSession {
            id: 2,
            email: "admin@example.com".to_string(),
            name: "Admin".to_string(),
            role: 2,
        };
        let session_json =
            serde_json::to_string(&session).expect("session serialization should succeed");
        let jar = CookieJar::new().add(Cookie::new("user_session", session_json));
        assert_eq!(require_admin_cookie_jar(&jar), Ok(()));
    }
}
