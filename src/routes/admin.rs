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
