use leptos::prelude::*;
use leptos::ev::SubmitEvent;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;
use crate::auth::get_user;
use crate::models::{Idea, CommentWithAuthor, Comment};

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
    let user_resource = Resource::new(|| (), |_| async move { get_user().await });
    let flagged = RwSignal::new(false);

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
                                    let page_title = if idea.title.is_empty() {
                                        format!("Idea #{} — UAB IT Idea Board", idea.id)
                                    } else {
                                        format!("{} — UAB IT Idea Board", idea.title)
                                    };
                                    let relative_time = format_relative_time(&idea.created_at);
                                    let tags_str = idea.tags.clone();
                                    view! {
                                        <Title text=page_title/>
                                        <article class="detail-card">
                                            <div class="detail-card-body">
                                                <div class="detail-vote-box">
                                                    <span class="detail-vote-arrow">"▲"</span>
                                                    <span class="detail-vote-count">{idea.vote_count}</span>
                                                    <span class="detail-vote-label">"votes"</span>
                                                </div>
                                                <div class="detail-idea-content">
                                                    <Show when={
                                                        let t = idea.title.clone();
                                                        move || !t.is_empty()
                                                    }>
                                                        <h1 class="detail-idea-title">{idea.title.clone()}</h1>
                                                    </Show>
                                                    <p class="detail-idea-text">{idea.content.clone()}</p>
                                                    {move || {
                                                        let tag_list: Vec<String> = tags_str
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
                                                    <div class="detail-meta-row">
                                                        <span class="detail-time">
                                                            {format!("submitted {}", relative_time)}
                                                        </span>
                                                        <Suspense fallback=|| ()>
                                                            {move || user_resource.get().map(|ur| match ur {
                                                                Ok(Some(user)) => {
                                                                    let is_mod = user.is_moderator();
                                                                    view! {
                                                                        <div class="detail-card-actions">
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

                                            <Show when=move || matches!(user_resource.get(), Some(Ok(Some(_))))>
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
                                                                                    view! {
                                                                                        <div class="comment-item">
                                                                                            <p class="comment-text">{cwa.comment.content}</p>
                                                                                            <span class="comment-meta">
                                                                                                <span class="comment-author">{cwa.author_name}</span>
                                                                                                " - "
                                                                                                <span class="comment-time">{time}</span>
                                                                                            </span>
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
