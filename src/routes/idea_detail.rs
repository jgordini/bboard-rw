use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;
use crate::auth::{get_user, AuthRefresh};
use crate::models::{Idea, CommentWithAuthor, Comment};
use crate::routes::ideas::check_user_votes;
use crate::routes::validation_helpers::{
    validate_comment_content, validate_idea_tags, validate_idea_title_and_content,
};

mod components;
use components::IdeaDetailLoaded;

#[server]
pub async fn get_idea(id: i32) -> Result<Idea, ServerFnError> {
    Idea::get_by_id(id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch idea {}: {:?}", id, e);
            ServerFnError::new("Idea not found")
        })?
        .ok_or_else(|| ServerFnError::new("Idea not found"))
}

#[server]
pub async fn get_comments(idea_id: i32) -> Result<Vec<CommentWithAuthor>, ServerFnError> {
    Comment::get_by_idea_id(idea_id, false)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch comments: {:?}", e);
            ServerFnError::new("Failed to fetch comments")
        })
}

#[server]
pub async fn create_comment(idea_id: i32, content: String) -> Result<Comment, ServerFnError> {
    use crate::auth::require_auth;
    let user = require_auth().await?;

    // Check if comments are enabled on this idea
    let idea = Idea::get_by_id(idea_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch idea: {:?}", e);
            ServerFnError::new("Failed to fetch idea")
        })?
        .ok_or_else(|| ServerFnError::new("Idea not found"))?;

    if !idea.comments_enabled {
        return Err(ServerFnError::new("Comments are locked on this idea"));
    }

    validate_comment_content(&content)?;
    Comment::create(user.id, idea_id, content.trim().to_string())
        .await
        .map_err(|e| {
            tracing::error!("Failed to create comment: {:?}", e);
            ServerFnError::new("Failed to create comment")
        })
}

#[server]
pub async fn update_idea_content_mod(idea_id: i32, title: String, content: String, tags: String) -> Result<(), ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    validate_idea_title_and_content(&title, &content)?;
    validate_idea_tags(&tags)?;

    let updated = Idea::update_content_mod(idea_id, title.trim().to_string(), content.trim().to_string(), tags.trim().to_string())
        .await
        .map_err(|e| {
            tracing::error!("Failed to update idea: {:?}", e);
            ServerFnError::new("Failed to update idea")
        })?;

    if !updated {
        return Err(ServerFnError::new("Idea not found"));
    }

    Ok(())
}

#[server]
pub async fn update_comment_mod(comment_id: i32, content: String) -> Result<(), ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    validate_comment_content(&content)?;

    let updated = Comment::update_content_mod(comment_id, content.trim().to_string())
        .await
        .map_err(|e| {
            tracing::error!("Failed to update comment: {:?}", e);
            ServerFnError::new("Failed to update comment")
        })?;

    if !updated {
        return Err(ServerFnError::new("Comment not found"));
    }

    Ok(())
}

#[server]
pub async fn delete_comment_mod(comment_id: i32) -> Result<(), ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    Comment::soft_delete(comment_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete comment: {:?}", e);
            ServerFnError::new("Failed to delete comment")
        })
}

#[server]
pub async fn toggle_comment_pin(comment_id: i32) -> Result<bool, ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    Comment::toggle_pin(comment_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to toggle comment pin: {:?}", e);
            ServerFnError::new("Failed to toggle comment pin")
        })
}

#[server]
pub async fn toggle_idea_comments(idea_id: i32) -> Result<bool, ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    Idea::toggle_comments(idea_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to toggle comments: {:?}", e);
            ServerFnError::new("Failed to toggle comments")
        })
}

/// Individual idea detail page with comments
#[component]
pub fn IdeaDetailPage() -> impl IntoView {
    let params = use_params_map();
    let idea_id = move || {
        params.read().get("id")
            .and_then(|id| id.parse::<i32>().ok())
            .unwrap_or(0)
    };

    let idea_resource = Resource::new(idea_id, |id| async move { get_idea(id).await });
    let comments_resource = Resource::new(idea_id, |id| async move { get_comments(id).await });
    let auth_refresh = expect_context::<AuthRefresh>().0;
    let user_resource = Resource::new(
        move || auth_refresh.get(),
        move |_| async move { get_user().await },
    );
    let has_voted = RwSignal::new(false);

    // Load user's vote status for this idea
    Effect::new(move |_| {
        if let Some(Ok(Some(_user))) = user_resource.get() {
            let current_idea_id = idea_id();
            leptos::task::spawn_local(async move {
                if let Ok(voted_ids) = check_user_votes().await {
                    has_voted.set(voted_ids.contains(&current_idea_id));
                }
            });
        }
    });

    view! {
        <div class="detail-page">
            <div class="container page">
                <a href="/" class="back-link">"← Back to all ideas"</a>

                <Suspense fallback=move || view! { <p class="loading">"Loading…"</p> }>
                    {move || {
                        idea_resource.get().map(|result| {
                            match result {
                                Ok(idea) => view! {
                                    <IdeaDetailLoaded
                                        idea=idea
                                        idea_resource=idea_resource
                                        comments_resource=comments_resource
                                        user_resource=user_resource
                                        has_voted=has_voted
                                    />
                                }.into_any(),
                                Err(_) => view! {
                                    <Title text="Not Found — UAB IT Idea Board"/>
                                    <div class="error-state">
                                        <p>"Idea not found."</p>
                                        <a href="/" class="back-link">"Return to idea board"</a>
                                    </div>
                                }.into_any()
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
