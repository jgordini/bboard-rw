use std::collections::HashMap;

use leptos::prelude::*;

use crate::auth::UserSession;
use crate::models::IdeaWithAuthor;
use crate::routes::async_helpers::spawn_server_action;
use crate::routes::view_helpers::{format_relative_time, is_user_logged_in, stage_badge_color};

use super::super::toggle_vote;

#[component]
pub(super) fn IdeaCard(
    idea_with_author: IdeaWithAuthor,
    rank: usize,
    user_resource: Resource<Result<Option<UserSession>, ServerFnError>>,
    voted_ideas: RwSignal<Vec<i32>>,
    comment_counts_resource: Resource<Result<HashMap<i32, i64>, ServerFnError>>,
) -> impl IntoView {
    let idea_id = idea_with_author.idea.id;
    let vote_count = RwSignal::new(idea_with_author.idea.vote_count);
    let title = idea_with_author.idea.title.clone();
    let content = idea_with_author.idea.content.clone();
    let stage = idea_with_author.idea.stage.clone();
    let is_pinned = idea_with_author.idea.is_pinned();
    let created_at = idea_with_author.idea.created_at;
    let author_name = idea_with_author.author_name.clone();

    let has_voted = move || voted_ideas.get().contains(&idea_id);
    let is_logged_in = move || is_user_logged_in(&user_resource);

    let comment_count = move || {
        comment_counts_resource
            .get()
            .and_then(|r| r.ok())
            .and_then(|counts| counts.get(&idea_id).copied())
            .unwrap_or(0)
    };

    let is_toggling = RwSignal::new(false);

    let handle_vote = move |_| {
        if !is_logged_in() || is_toggling.get() {
            return;
        }
        is_toggling.set(true);
        // Optimistic update: adjust count immediately
        let was_voted = voted_ideas.get().contains(&idea_id);
        if was_voted {
            vote_count.update(|c| *c -= 1);
        } else {
            vote_count.update(|c| *c += 1);
        }
        voted_ideas.update(|v| {
            if was_voted {
                v.retain(|&id| id != idea_id);
            } else if !v.contains(&idea_id) {
                v.push(idea_id);
            }
        });
        spawn_server_action(
            toggle_vote(idea_id),
            move |now_voted| {
                is_toggling.set(false);
                // Reconcile with server truth
                let locally_voted = voted_ideas.get().contains(&idea_id);
                if now_voted != locally_voted {
                    voted_ideas.update(|v| {
                        if now_voted {
                            if !v.contains(&idea_id) {
                                v.push(idea_id);
                            }
                        } else {
                            v.retain(|&id| id != idea_id);
                        }
                    });
                    // Fix count if local state diverged
                    if now_voted {
                        vote_count.update(|c| *c += 1);
                    } else {
                        vote_count.update(|c| *c -= 1);
                    }
                }
            },
            move |_err| {
                is_toggling.set(false);
                // Rollback on failure
                if was_voted {
                    vote_count.update(|c| *c += 1);
                    voted_ideas.update(|v| {
                        if !v.contains(&idea_id) {
                            v.push(idea_id);
                        }
                    });
                } else {
                    vote_count.update(|c| *c -= 1);
                    voted_ideas.update(|v| v.retain(|&id| id != idea_id));
                }
            },
        );
    };

    let relative_time = format_relative_time(&created_at);
    let stage_color = stage_badge_color(&stage);

    view! {
        <div class="spark-item" class:pinned=is_pinned class:voted=has_voted>
            <div class="spark-rank">{rank}</div>
            <div class="spark-vote-box" class:voted=has_voted>
                <span class="spark-arrow" aria-hidden="true">"▲"</span>
                <span class="spark-count">{vote_count}</span>
                <button
                    class="spark-btn btn"
                    disabled=move || !is_logged_in() || is_toggling.get()
                    on:click=handle_vote
                    title=move || if !is_logged_in() { "Login to spark" } else if has_voted() { "Remove spark" } else { "Spark this idea" }
                >
                    {move || if has_voted() { "sparked" } else { "spark" }}
                </button>
            </div>
            <a class="spark-content" href=format!("/ideas/{}", idea_id)>
                {move || {
                    if is_pinned {
                        view! { <span class="pinned-badge">"Pinned"</span> }.into_any()
                    } else {
                        ().into_any()
                    }
                }}
                <h3 class="spark-title">{title}</h3>
                <p class="spark-text">{content}</p>
                <div class="spark-meta">
                    <span class=format!("stage-badge stage-{}", stage_color)>{stage.clone()}</span>
                    <span class="author-name">"by " {author_name}</span>
                    <span class="spark-time">{format!("submitted {}", relative_time)}</span>
                    <span class="spark-comments-badge">
                        {move || {
                            let count = comment_count();
                            if count == 1 {
                                "1 comment".to_string()
                            } else {
                                format!("{} comments", count)
                            }
                        }}
                    </span>
                </div>
            </a>
        </div>
    }
}
