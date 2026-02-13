use leptos::ev::SubmitEvent;
use leptos::prelude::*;

use crate::auth::UserSession;
use crate::models::CommentWithAuthor;
use crate::routes::async_helpers::{
    spawn_server_action_refetch_resource, spawn_server_action_with_error,
};
use crate::routes::view_helpers::format_relative_time;

use super::super::{create_comment, delete_comment_mod, toggle_comment_pin, update_comment_mod};

#[component]
pub(super) fn CommentsSection(
    idea_id: i32,
    idea_comments_enabled: bool,
    comments_resource: Resource<Result<Vec<CommentWithAuthor>, ServerFnError>>,
    user_resource: Resource<Result<Option<UserSession>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <div class="comments-section">
            <h2 class="comments-heading">"Comments"</h2>

            <Show
                when=move || idea_comments_enabled && matches!(user_resource.get(), Some(Ok(Some(_))))
                fallback=move || {
                    if !idea_comments_enabled {
                        view! {
                            <p class="comments-locked-notice">"Comments are locked on this idea."</p>
                        }
                            .into_any()
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
                        <CommentForm idea_id=idea_id comments_resource=comments_resource />
                    </div>
                </article>
            </Show>

            <Suspense fallback=move || view! { <p class="loading">"Loading comments…"</p> }>
                {move || {
                    comments_resource.get().map(|result| match result {
                        Ok(comments) if comments.is_empty() => {
                            view! {
                                <p class="no-comments">"No comments yet. Be the first to comment!"</p>
                            }
                                .into_any()
                        }
                        Ok(comments) => {
                            view! {
                                <div class="comment-list">
                                    <For
                                        each=move || comments.clone()
                                        key=|cwa| cwa.comment.id
                                        children=move |cwa: CommentWithAuthor| {
                                            view! {
                                                <CommentItem
                                                    cwa=cwa
                                                    comments_resource=comments_resource
                                                    user_resource=user_resource
                                                />
                                            }
                                        }
                                    />
                                </div>
                            }
                                .into_any()
                        }
                        Err(_) => view! {
                            <p class="error-state">"Failed to load comments."</p>
                        }
                            .into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn CommentItem(
    cwa: CommentWithAuthor,
    comments_resource: Resource<Result<Vec<CommentWithAuthor>, ServerFnError>>,
    user_resource: Resource<Result<Option<UserSession>, ServerFnError>>,
) -> impl IntoView {
    let time = format_relative_time(&cwa.comment.created_at);
    let comment_id = cwa.comment.id;
    let comment_is_pinned = cwa.comment.is_pinned;
    let is_editing = RwSignal::new(false);
    let edit_content = RwSignal::new(cwa.comment.content.clone());
    let edit_error = RwSignal::new(Option::<String>::None);
    let comment_content_value = StoredValue::new(cwa.comment.content.clone());
    let author_name = cwa.author_name;

    view! {
        <div class="comment-item" class:pinned=comment_is_pinned>
            <Show when=move || comment_is_pinned>
                <span class="pinned-badge">"Pinned"</span>
            </Show>
            <Show
                when=move || is_editing.get()
                fallback=move || {
                    view! { <p class="comment-text">{move || comment_content_value.get_value()}</p> }
                        .into_any()
                }
            >
                <form
                    on:submit=move |ev| {
                        ev.prevent_default();
                        let content_value = edit_content.get();
                        edit_error.set(None);
                        spawn_server_action_with_error(
                            update_comment_mod(comment_id, content_value),
                            move |_| {
                                comments_resource.refetch();
                                is_editing.set(false);
                            },
                            edit_error,
                        );
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
                        bind:value=edit_content
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
                <span class="comment-author">{author_name}</span>
                <span class="comment-time">{time}</span>
                <Suspense fallback=|| ()>
                    {move || user_resource.get().map(|ur| match ur {
                        Ok(Some(user)) if user.is_moderator() => {
                            view! {
                                <Show when=move || !is_editing.get()>
                                    <button
                                        type="button"
                                        class="btn-pin"
                                        on:click=move |_| {
                                            spawn_server_action_refetch_resource(
                                                toggle_comment_pin(comment_id),
                                                comments_resource,
                                            );
                                        }
                                    >
                                        {move || if comment_is_pinned { "Unpin" } else { "Pin" }}
                                    </button>
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
                                            spawn_server_action_refetch_resource(
                                                delete_comment_mod(comment_id),
                                                comments_resource,
                                            );
                                        }
                                    >
                                        "Delete"
                                    </button>
                                </Show>
                            }
                                .into_any()
                        }
                        _ => view! {}.into_any(),
                    })}
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
        spawn_server_action_refetch_resource(create_comment(idea_id, content_clone), comments_resource);
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
                    bind:value=content
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
