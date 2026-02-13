use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;
use crate::auth::{get_user, UserSession};
use crate::models::{IdeaWithAuthor, User};
#[cfg(feature = "ssr")]
use crate::models::{Idea, Flag};

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
pub async fn mark_idea_off_topic_action(idea_id: i32, is_off_topic: bool) -> Result<(), ServerFnError> {
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

#[server]
pub async fn export_ideas_csv() -> Result<String, ServerFnError> {
    use crate::auth::require_admin;
    require_admin().await?;

    #[cfg(feature = "ssr")]
    {
        build_ideas_csv()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to export ideas CSV: {}", e)))
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("CSV export is only available on the server"))
    }
}

#[server]
pub async fn export_comments_csv() -> Result<String, ServerFnError> {
    use crate::auth::require_admin;
    require_admin().await?;

    #[cfg(feature = "ssr")]
    {
        build_comments_csv()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to export comments CSV: {}", e)))
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("CSV export is only available on the server"))
    }
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
    let escaped = value.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}

#[cfg(feature = "ssr")]
async fn build_ideas_csv() -> Result<String, sqlx::Error> {
    use sqlx::Row;

    let rows = sqlx::query(
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
    .fetch_all(crate::database::get_db())
    .await?;

    let mut csv = String::from(
        "id,user_id,author_name,author_email,title,content,tags,stage,is_public,is_off_topic,comments_enabled,vote_count,pinned_at,created_at\n",
    );

    for row in rows {
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

        csv.push_str(&format!(
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
        ));
    }

    Ok(csv)
}

#[cfg(feature = "ssr")]
async fn build_comments_csv() -> Result<String, sqlx::Error> {
    use sqlx::Row;

    let rows = sqlx::query(
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
    .fetch_all(crate::database::get_db())
    .await?;

    let mut csv = String::from(
        "id,idea_id,idea_title,user_id,author_name,author_email,content,is_pinned,is_deleted,created_at\n",
    );

    for row in rows {
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

        csv.push_str(&format!(
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
        ));
    }

    Ok(csv)
}
