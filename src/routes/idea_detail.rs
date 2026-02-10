use leptos::prelude::*;
use leptos::ev::SubmitEvent;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;
use crate::auth::{get_user, AuthRefresh};
use crate::models::{Idea, CommentWithAuthor, Comment};
use crate::routes::ideas::{toggle_vote, check_user_votes};

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

    if content.trim().is_empty() {
        return Err(ServerFnError::new("Comment cannot be empty"));
    }
    if content.len() > 500 {
        return Err(ServerFnError::new("Comment cannot exceed 500 characters"));
    }
    if crate::profanity::contains_profanity(&content) {
        return Err(ServerFnError::new("Your comment contains inappropriate language. Please revise and try again."));
    }
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

    if title.trim().is_empty() {
        return Err(ServerFnError::new("Idea title cannot be empty"));
    }
    if title.len() > 100 {
        return Err(ServerFnError::new("Idea title cannot exceed 100 characters"));
    }
    if content.trim().is_empty() {
        return Err(ServerFnError::new("Idea description cannot be empty"));
    }
    if content.len() > 500 {
        return Err(ServerFnError::new("Idea description cannot exceed 500 characters"));
    }
    if tags.len() > 200 {
        return Err(ServerFnError::new("Tags cannot exceed 200 characters"));
    }
    if crate::profanity::contains_profanity(&title) || crate::profanity::contains_profanity(&content) {
        return Err(ServerFnError::new("Your submission contains inappropriate language. Please revise and try again."));
    }

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

    if content.trim().is_empty() {
        return Err(ServerFnError::new("Comment cannot be empty"));
    }
    if content.len() > 500 {
        return Err(ServerFnError::new("Comment cannot exceed 500 characters"));
    }
    if crate::profanity::contains_profanity(&content) {
        return Err(ServerFnError::new("Your comment contains inappropriate language. Please revise and try again."));
    }

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
    let flagged = RwSignal::new(false);
    let stage_updating = RwSignal::new(false);
    let idea_editing = RwSignal::new(false);
    let idea_edit_error = RwSignal::new(Option::<String>::None);
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
                                Ok(idea) => {
                                    let idea_id_val = idea.id;
                                    let idea_pinned = idea.is_pinned();
                                    let idea_comments_enabled = idea.comments_enabled;
                                    let idea_vote_count = idea.vote_count;
                                    let idea_title = idea.title.clone();
                                    let idea_content = idea.content.clone();
                                    let page_title = if idea_title.is_empty() {
                                        format!("Idea #{} — UAB IT Idea Board", idea.id)
                                    } else {
                                        format!("{} — UAB IT Idea Board", idea_title)
                                    };
                                    let relative_time = format_relative_time(&idea.created_at);
                                    let stage = idea.stage.clone();
                                    let stage_color = stage_badge_color(&stage).to_string();
                                    let tags_str = idea.tags.clone();
                                    let edit_title = RwSignal::new(idea_title.clone());
                                    let edit_content = RwSignal::new(idea_content.clone());
                                    let edit_tags = RwSignal::new(tags_str.clone());
                                    let idea_title_value = StoredValue::new(idea_title.clone());
                                    let idea_content_value = StoredValue::new(idea_content.clone());
                                    let tags_str_value = StoredValue::new(tags_str.clone());
                                    view! {
                                        <Title text=page_title/>
                                        <article class="detail-card">
                                            <div class="detail-card-body">
                                                <div class="detail-vote-box" class:voted=move || has_voted.get()>
                                                    <span class="detail-vote-arrow">"▲"</span>
                                                    <span class="detail-vote-count">{idea_vote_count}</span>
                                                    <Suspense fallback=|| view! { <span class="detail-vote-label">"votes"</span> }>
                                                        {move || user_resource.get().map(|ur| match ur {
                                                            Ok(Some(_)) => {
                                                                view! {
                                                                    <button
                                                                        class="detail-vote-btn"
                                                                        on:click=move |_| {
                                                                            let id = idea_id_val;
                                                                            leptos::task::spawn_local(async move {
                                                                                if let Ok(now_voted) = toggle_vote(id).await {
                                                                                    has_voted.set(now_voted);
                                                                                    idea_resource.refetch();
                                                                                }
                                                                            });
                                                                        }
                                                                        title=move || if has_voted.get() { "Click to remove vote" } else { "Vote for this idea" }
                                                                    >
                                                                        {move || if has_voted.get() { "unvote" } else { "vote" }}
                                                                    </button>
                                                                }.into_any()
                                                            }
                                                            _ => view! { <span class="detail-vote-label">"votes"</span> }.into_any(),
                                                        })}
                                                    </Suspense>
                                                </div>
                                                <div class="detail-idea-content">
                                                    <Show
                                                        when=move || idea_editing.get()
                                                        fallback=move || {
                                                            view! {
                                                                <Show when=move || !idea_title_value.get_value().is_empty()>
                                                                    <h1 class="detail-idea-title">{move || idea_title_value.get_value()}</h1>
                                                                </Show>
                                                                <p class="detail-idea-text">{move || idea_content_value.get_value()}</p>
                                                            }.into_any()
                                                        }
                                                    >
                                                        <form
                                                            on:submit=move |ev| {
                                                                ev.prevent_default();
                                                                let title_value = edit_title.get();
                                                                let content_value = edit_content.get();
                                                                let tags_value = edit_tags.get();
                                                                idea_edit_error.set(None);
                                                                let id = idea_id_val;
                                                                leptos::task::spawn_local(async move {
                                                                    match update_idea_content_mod(id, title_value, content_value, tags_value).await {
                                                                        Ok(()) => {
                                                                            idea_resource.refetch();
                                                                            idea_editing.set(false);
                                                                        }
                                                                        Err(e) => {
                                                                            idea_edit_error.set(Some(e.to_string()));
                                                                        }
                                                                    }
                                                                });
                                                            }
                                                        >
                                                            <Show when=move || idea_edit_error.get().is_some()>
                                                                <div class="dialog-alert dialog-alert-error" role="alert" aria-live="polite" aria-atomic="true">
                                                                    {move || idea_edit_error.get().unwrap_or_default()}
                                                                </div>
                                                            </Show>
                                                            <div class="form-group">
                                                                <label class="form-label" for="idea-edit-title">"Title"</label>
                                                                <input
                                                                    id="idea-edit-title"
                                                                    class="dialog-input"
                                                                    type="text"
                                                                    maxlength=100
                                                                    prop:value=move || edit_title.get()
                                                                    on:input=move |ev| edit_title.set(event_target_value(&ev))
                                                                />
                                                            </div>
                                                            <div class="form-group">
                                                                <label class="form-label" for="idea-edit-content">"Description"</label>
                                                                <textarea
                                                                    id="idea-edit-content"
                                                                    class="dialog-textarea"
                                                                    maxlength=500
                                                                    prop:value=move || edit_content.get()
                                                                    on:input=move |ev| edit_content.set(event_target_value(&ev))
                                                                />
                                                            </div>
                                                            <div class="form-group">
                                                                <label class="form-label" for="idea-edit-tags">"Tags (comma-separated)"</label>
                                                                <input
                                                                    id="idea-edit-tags"
                                                                    class="dialog-input"
                                                                    type="text"
                                                                    maxlength=200
                                                                    placeholder="e.g., security, performance, ui"
                                                                    prop:value=move || edit_tags.get()
                                                                    on:input=move |ev| edit_tags.set(event_target_value(&ev))
                                                                />
                                                            </div>
                                                            <div class="dialog-footer">
                                                                <button type="submit" class="submit-btn">"Save"</button>
                                                            </div>
                                                        </form>
                                                    </Show>
                                                    <Show when=move || !idea_editing.get()>
                                                        {move || {
                                                            let tag_list: Vec<String> = tags_str_value.get_value()
                                                                .split(',')
                                                                .map(|s| s.trim())
                                                                .filter(|s| !s.is_empty())
                                                                .map(String::from)
                                                                .collect();
                                                            if tag_list.is_empty() {
                                                                view! {}.into_any()
                                                            } else {
                                                                view! {
                                                                    <div class="detail-tags">
                                                                        <For
                                                                            each=move || tag_list.clone()
                                                                            key=|t| t.clone()
                                                                            children=move |tag: String| {
                                                                                view! { <span class="detail-tag">{tag}</span> }
                                                                            }
                                                                        />
                                                                    </div>
                                                                }.into_any()
                                                            }
                                                        }}
                                                    </Show>
                                                    <div class="detail-meta-row">
                                                        <div class="detail-meta-info">
                                                            <Suspense fallback=|| ()>
                                                                {move || user_resource.get().map(|ur| match ur {
                                                                    Ok(Some(user)) if user.is_moderator() => {
                                                                        let current_stage = stage.clone();
                                                                        let current_stage_for_value = current_stage.clone();
                                                                        let stage_color_class = stage_color.clone();
                                                                        view! {
                                                                            <label class="sr-only" for="idea-stage">"Stage"</label>
                                                                            <select
                                                                                id="idea-stage"
                                                                                class=format!("stage-select stage-badge stage-{}", stage_color_class)
                                                                                prop:value=move || current_stage_for_value.clone()
                                                                                disabled=move || stage_updating.get()
                                                                                on:change=move |ev| {
                                                                                    let new_stage = event_target_value(&ev);
                                                                                    if new_stage == current_stage {
                                                                                        return;
                                                                                    }
                                                                                    stage_updating.set(true);
                                                                                    let id = idea_id_val;
                                                                                    leptos::task::spawn_local(async move {
                                                                                        if crate::routes::admin::update_idea_stage_action(id, new_stage).await.is_ok() {
                                                                                            idea_resource.refetch();
                                                                                        }
                                                                                        stage_updating.set(false);
                                                                                    });
                                                                                }
                                                                            >
                                                                                <option value="Ideate">"Ideate"</option>
                                                                                <option value="Review">"Review"</option>
                                                                                <option value="In Progress">"In Progress"</option>
                                                                                <option value="Completed">"Completed"</option>
                                                                            </select>
                                                                        }.into_any()
                                                                    }
                                                                    _ => {
                                                                        let stage_color_class = stage_color.clone();
                                                                        view! {
                                                                            <span class=format!("stage-badge stage-{}", stage_color_class)>{stage.clone()}</span>
                                                                        }.into_any()
                                                                    }
                                                                })}
                                                            </Suspense>
                                                            <span class="detail-time">
                                                                {format!("submitted {}", relative_time)}
                                                            </span>
                                                        </div>
                                                        <Suspense fallback=|| ()>
                                                            {move || user_resource.get().map(|ur| match ur {
                                                                Ok(Some(user)) => {
                                                                    let is_mod = user.is_moderator();
                                                                    view! {
                                                                        <div class="detail-card-actions">
                                                                            <Show when=move || is_mod>
                                                                                <button
                                                                                    type="button"
                                                                                    class="btn-edit"
                                                                                    on:click=move |_| {
                                                                                        idea_edit_error.set(None);
                                                                                        idea_editing.set(!idea_editing.get());
                                                                                    }
                                                                                >
                                                                                    {move || if idea_editing.get() { "Cancel Edit" } else { "Edit" }}
                                                                                </button>
                                                                            </Show>
                                                                            <button
                                                                                type="button"
                                                                                class="btn-flag"
                                                                                disabled=move || flagged.get()
                                                                                on:click=move |_| {
                                                                                    let id = idea_id_val;
                                                                                    flagged.set(true);
                                                                                    leptos::task::spawn_local(async move {
                                                                                        let _ = crate::routes::ideas::flag_idea_server(id).await;
                                                                                    });
                                                                                }
                                                                            >
                                                                                {move || if flagged.get() { "Flagged" } else { "Flag" }}
                                                                            </button>
                                                                            <Show when=move || is_mod>
                                                                                <button
                                                                                    type="button"
                                                                                    class="btn-pin"
                                                                                    on:click=move |_| {
                                                                                        let id = idea_id_val;
                                                                                        leptos::task::spawn_local(async move {
                                                                                            if crate::routes::admin::toggle_idea_pin_action(id).await.is_ok() {
                                                                                                idea_resource.refetch();
                                                                                            }
                                                                                        });
                                                                                    }
                                                                                >
                                                                                    {move || if idea_pinned { "Unpin" } else { "Pin" }}
                                                                                </button>
                                                                                <button
                                                                                    type="button"
                                                                                    class="btn-toggle-comments"
                                                                                    on:click=move |_| {
                                                                                        let id = idea_id_val;
                                                                                        leptos::task::spawn_local(async move {
                                                                                            if toggle_idea_comments(id).await.is_ok() {
                                                                                                idea_resource.refetch();
                                                                                            }
                                                                                        });
                                                                                    }
                                                                                >
                                                                                    {move || if idea_comments_enabled { "Lock Comments" } else { "Unlock Comments" }}
                                                                                </button>
                                                                            </Show>
                                                                        </div>
                                                                    }.into_any()
                                                                }
                                                                _ => view! {}.into_any(),
                                                            })}
                                                        </Suspense>
                                                    </div>
                                                </div>
                                            </div>
                                        </article>

                                        <div class="comments-section">
                                            <h2 class="comments-heading">"Comments"</h2>

                                            <Show
                                                when=move || idea_comments_enabled && matches!(user_resource.get(), Some(Ok(Some(_))))
                                                fallback=move || {
                                                    if !idea_comments_enabled {
                                                        view! {
                                                            <p class="comments-locked-notice">"Comments are locked on this idea."</p>
                                                        }.into_any()
                                                    } else {
                                                        view! {}.into_any()
                                                    }
                                                }
                                            >
                                                <article class="sidebar-card comment-form-card">
                                                    <header class="sidebar-card-header">
                                                        <h3 class="sidebar-card-title">"Add a Comment"</h3>
                                                    </header>
                                                    <div class="sidebar-card-body">
                                                        <CommentForm idea_id=idea_id_val comments_resource />
                                                    </div>
                                                </article>
                                            </Show>

                                            <Suspense fallback=move || view! { <p class="loading">"Loading comments…"</p> }>
                                                {move || {
                                                    comments_resource.get().map(|result| {
                                                        match result {
                                                            Ok(comments) => {
                                                                if comments.is_empty() {
                                                                    view! {
                                                                        <p class="no-comments">"No comments yet. Be the first to comment!"</p>
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <div class="comment-list">
                                                                            <For
                                                                                each=move || comments.clone()
                                                                                key=|cwa| cwa.comment.id
                                                                                children=move |cwa: CommentWithAuthor| {
                                                                                    let time = format_relative_time(&cwa.comment.created_at);
                                                                                    let is_editing = RwSignal::new(false);
                                                                                    let edit_content = RwSignal::new(cwa.comment.content.clone());
                                                                                    let edit_error = RwSignal::new(Option::<String>::None);
                                                                                    let comment_content_value = StoredValue::new(cwa.comment.content.clone());
                                                                                    view! {
                                                                                <div class="comment-item">
                                                                                            <Show
                                                                                                when=move || is_editing.get()
                                                                                                fallback=move || {
                                                                                                    view! { <p class="comment-text">{move || comment_content_value.get_value()}</p> }.into_any()
                                                                                                }
                                                                                            >
                                                                                                <form
                                                                                                    on:submit=move |ev| {
                                                                                                        ev.prevent_default();
                                                                                                        let content_value = edit_content.get();
                                                                                                        edit_error.set(None);
                                                                                                        let comment_id = cwa.comment.id;
                                                                                                        leptos::task::spawn_local(async move {
                                                                                                            match update_comment_mod(comment_id, content_value).await {
                                                                                                                Ok(()) => {
                                                                                                                    comments_resource.refetch();
                                                                                                                    is_editing.set(false);
                                                                                                                }
                                                                                                                Err(e) => {
                                                                                                                    edit_error.set(Some(e.to_string()));
                                                                                                                }
                                                                                                            }
                                                                                                        });
                                                                                                    }
                                                                                                >
                                                                                                    <Show when=move || edit_error.get().is_some()>
                                                                                                        <div class="dialog-alert dialog-alert-error" role="alert" aria-live="polite" aria-atomic="true">
                                                                                                            {move || edit_error.get().unwrap_or_default()}
                                                                                                        </div>
                                                                                                    </Show>
                                                                                                    <textarea
                                                                                                        class="dialog-textarea"
                                                                                                        maxlength=500
                                                                                                        prop:value=move || edit_content.get()
                                                                                                        on:input=move |ev| edit_content.set(event_target_value(&ev))
                                                                                                    />
                                                                                                    <div class="dialog-footer">
                                                                                                        <button
                                                                                                            type="button"
                                                                                                            class="btn-cancel"
                                                                                                            on:click=move |_| {
                                                                                                                edit_content.set(comment_content_value.get_value());
                                                                                                                edit_error.set(None);
                                                                                                                is_editing.set(false);
                                                                                                            }
                                                                                                        >
                                                                                                            "Cancel Edit"
                                                                                                        </button>
                                                                                                        <button type="submit" class="submit-btn">"Save"</button>
                                                                                                    </div>
                                                                                                </form>
                                                                                            </Show>
                                                                                            <div class="comment-meta">
                                                                                                <span class="comment-author">{cwa.author_name}</span>
                                                                                                <span class="comment-time">{time}</span>
                                                                                                <Suspense fallback=|| ()>
                                                                                                    {move || user_resource.get().map(|ur| match ur {
                                                                                                        Ok(Some(user)) if user.is_moderator() => {
                                                                                                            let comment_id = cwa.comment.id;
                                                                                                            view! {
                                                                                                                <Show when=move || !is_editing.get()>
                                                                                                                    <button
                                                                                                                        type="button"
                                                                                                                        class="btn-edit"
                                                                                                                        on:click=move |_| {
                                                                                                                            edit_error.set(None);
                                                                                                                            is_editing.set(true);
                                                                                                                        }
                                                                                                                    >
                                                                                                                        "Edit"
                                                                                                                    </button>
                                                                                                                    <button
                                                                                                                        type="button"
                                                                                                                        class="btn-delete"
                                                                                                                        on:click=move |_| {
                                                                                                                            leptos::task::spawn_local(async move {
                                                                                                                                if delete_comment_mod(comment_id).await.is_ok() {
                                                                                                                                    comments_resource.refetch();
                                                                                                                                }
                                                                                                                            });
                                                                                                                        }
                                                                                                                    >
                                                                                                                        "Delete"
                                                                                                                    </button>
                                                                                                                </Show>
                                                                                                            }.into_any()
                                                                                                        }
                                                                                                        _ => view! {}.into_any(),
                                                                                                    })}
                                                                                                </Suspense>
                                                                                            </div>
                                                                                        </div>
                                                                                    }
                                                                                }
                                                                            />
                                                                        </div>
                                                                    }.into_any()
                                                                }
                                                            }
                                                            Err(_) => view! {
                                                                <p class="error-state">"Failed to load comments."</p>
                                                            }.into_any()
                                                        }
                                                    })
                                                }}
                                            </Suspense>
                                        </div>
                                    }.into_any()
                                }
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

#[component]
fn CommentForm(
    idea_id: i32,
    comments_resource: Resource<Result<Vec<CommentWithAuthor>, ServerFnError>>,
) -> impl IntoView {
    let content = RwSignal::new(String::new());
    let max_chars: usize = 500;
    let char_count = move || content.get().len();
    let is_warning = move || char_count() >= (max_chars as f64 * 0.9) as usize;
    let is_error = move || char_count() >= max_chars;

    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let content_value = content.get();
        if content_value.trim().is_empty() || content_value.len() > max_chars {
            return;
        }
        let content_clone = content_value.clone();
        leptos::task::spawn_local(async move {
            if create_comment(idea_id, content_clone).await.is_ok() {
                comments_resource.refetch();
            }
        });
        content.set(String::new());
    };

    view! {
        <form on:submit=handle_submit>
            <div class="form-group">
                <label class="form-label" for="comment-content">"Your Comment"</label>
                <textarea
                    id="comment-content"
                    class="dialog-textarea"
                    placeholder="Add a comment (max 500 characters)…"
                    maxlength=max_chars
                    prop:value=move || content.get()
                    on:input=move |ev| {
                        content.set(event_target_value(&ev));
                    }
                />
            </div>
            <div class="form-footer">
                <span class="char-counter" class:warning=is_warning class:error=is_error>
                    {move || format!("{}/{}", char_count(), max_chars)}
                </span>
                <button
                    type="submit"
                    class="submit-btn"
                    disabled=move || content.get().trim().is_empty()
                >
                    "Post Comment"
                </button>
            </div>
        </form>
    }
}

fn format_relative_time(dt: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now - *dt;

    if duration.num_seconds() < 60 {
        format!("{} seconds ago", duration.num_seconds())
    } else if duration.num_minutes() < 60 {
        format!("{} minutes ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_days() < 30 {
        format!("{} days ago", duration.num_days())
    } else {
        dt.format("%Y-%m-%d").to_string()
    }
}

fn stage_badge_color(stage: &str) -> &str {
    match stage {
        "Ideate" => "ideate",
        "Review" => "review",
        "In Progress" => "progress",
        "Completed" => "completed",
        _ => "default",
    }
}
