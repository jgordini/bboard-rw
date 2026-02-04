use leptos::prelude::*;
use leptos::ev::SubmitEvent;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;
use crate::models::{Idea, Comment};
use leptos_shadcn_button::Button;
use leptos_shadcn_card::{Card, CardHeader, CardTitle, CardContent};
use leptos_shadcn_badge::{Badge, BadgeVariant};
use leptos_shadcn_label::Label;

#[server]
pub async fn get_idea(id: i32) -> Result<Idea, ServerFnError> {
    Idea::get_by_id(id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch idea {}: {:?}", id, e);
            ServerFnError::new("Idea not found")
        })
}

#[server]
pub async fn get_comments(idea_id: i32) -> Result<Vec<Comment>, ServerFnError> {
    Comment::get_by_idea_id(idea_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch comments: {:?}", e);
            ServerFnError::new("Failed to fetch comments")
        })
}

#[server]
pub async fn create_comment(idea_id: i32, content: String) -> Result<Comment, ServerFnError> {
    if content.trim().is_empty() {
        return Err(ServerFnError::new("Comment cannot be empty"));
    }
    if content.len() > 500 {
        return Err(ServerFnError::new("Comment cannot exceed 500 characters"));
    }
    if crate::profanity::contains_profanity(&content) {
        return Err(ServerFnError::new("Your comment contains inappropriate language. Please revise and try again."));
    }
    Comment::create(idea_id, content.trim().to_string())
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

    view! {
        <div class="detail-page">
            <div class="container page">
                <a href="/" class="back-link">"← Back to all ideas"</a>

                <Suspense fallback=move || view! { <p class="loading">"Loading..."</p> }>
                    {move || {
                        idea_resource.get().map(|result| {
                            match result {
                                Ok(idea) => {
                                    let idea_id_val = idea.id;
                                    let title = format!("Idea #{} — UAB IT Idea Board", idea.id);
                                    let relative_time = format_relative_time(&idea.created_at);
                                    view! {
                                        <Title text=title/>
                                        <Card class="detail-card">
                                            <CardContent class="detail-card-body">
                                                <div class="detail-vote-box">
                                                    <span class="detail-vote-arrow">"▲"</span>
                                                    <span class="detail-vote-count">{idea.vote_count}</span>
                                                    <span class="detail-vote-label">"votes"</span>
                                                </div>
                                                <div class="detail-idea-content">
                                                    <p class="detail-idea-text">{idea.content}</p>
                                                    <Badge variant=BadgeVariant::Outline class="detail-time">
                                                        {format!("submitted {}", relative_time)}
                                                    </Badge>
                                                </div>
                                            </CardContent>
                                        </Card>

                                        <div class="comments-section">
                                            <h2 class="comments-heading">"Comments"</h2>

                                            <Card class="sidebar-card comment-form-card">
                                                <CardHeader class="sidebar-card-header">
                                                    <CardTitle class="sidebar-card-title">"Add a Comment"</CardTitle>
                                                </CardHeader>
                                                <CardContent class="sidebar-card-body">
                                                    <CommentForm idea_id=idea_id_val comments_resource />
                                                </CardContent>
                                            </Card>

                                            <Suspense fallback=move || view! { <p class="loading">"Loading comments..."</p> }>
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
                                                                                key=|comment| comment.id
                                                                                children=move |comment: Comment| {
                                                                                    let time = format_relative_time(&comment.created_at);
                                                                                    view! {
                                                                                        <div class="comment-item">
                                                                                            <p class="comment-text">{comment.content}</p>
                                                                                            <span class="comment-time">{time}</span>
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
    comments_resource: Resource<Result<Vec<Comment>, ServerFnError>>,
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
                <Label class="form-label">"Your Comment"</Label>
                <textarea
                    class="idea-textarea"
                    class:warning=is_warning
                    class:error=is_error
                    placeholder="Add a comment (max 500 characters)"
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
                <Button
                    class="submit-btn"
                    disabled=Signal::derive(move || content.get().trim().is_empty())
                >
                    "Post Comment"
                </Button>
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
