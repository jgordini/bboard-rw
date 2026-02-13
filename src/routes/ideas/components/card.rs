use std::collections::HashMap;

use leptos::prelude::*;

use crate::auth::UserSession;
use crate::models::IdeaWithAuthor;
use crate::routes::async_helpers::spawn_server_action_ok;
use crate::routes::view_helpers::{format_relative_time, is_user_logged_in, stage_badge_color};

use super::super::toggle_vote;

#[component]
pub(super) fn IdeaCard(
    idea_with_author: IdeaWithAuthor,
    rank: usize,
    user_resource: Resource<Result<Option<UserSession>, ServerFnError>>,
    voted_ideas: RwSignal<Vec<i32>>,
    ideas_resource: Resource<Result<Vec<IdeaWithAuthor>, ServerFnError>>,
    comment_counts_resource: Resource<Result<HashMap<i32, i64>, ServerFnError>>,
) -> impl IntoView {
    let idea_id = idea_with_author.idea.id;
    let vote_count = idea_with_author.idea.vote_count;
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

    let handle_vote = move |_| {
        if !is_logged_in() {
            return;
        }
        spawn_server_action_ok(toggle_vote(idea_id), move |now_voted| {
            voted_ideas.update(|v| {
                if now_voted {
                    if !v.contains(&idea_id) {
                        v.push(idea_id);
                    }
                } else {
                    v.retain(|&id| id != idea_id);
                }
            });
            ideas_resource.refetch();
        });
    };

    let relative_time = format_relative_time(&created_at);
    let stage_color = stage_badge_color(&stage);

    view! {
        <div class="digg-item" class:pinned=is_pinned>
            <div class="digg-rank">{rank}</div>
            <div class="digg-vote-box" class:voted=has_voted>
                <span class="digg-arrow" aria-hidden="true">"▲"</span>
                <span class="digg-count">{vote_count}</span>
                <button
                    class="digg-btn"
                    disabled=move || !is_logged_in()
                    on:click=handle_vote
                    title=move || if !is_logged_in() { "Login to vote" } else if has_voted() { "Click to remove vote" } else { "Vote for this idea" }
                >
                    {move || if has_voted() { "unvote" } else { "vote" }}
                </button>
            </div>
            <a class="digg-content" href=format!("/ideas/{}", idea_id)>
                {move || {
                    if is_pinned {
                        view! { <span class="pinned-badge">"Pinned"</span> }.into_any()
                    } else {
                        view! {}.into_any()
                    }
                }}
                <h3 class="digg-title">{title}</h3>
                <p class="digg-text">{content}</p>
                <div class="digg-meta">
                    <span class=format!("stage-badge stage-{}", stage_color)>{stage.clone()}</span>
                    <span class="author-name">"by " {author_name}</span>
                    <span class="digg-time">{format!("submitted {}", relative_time)}</span>
                    <span class="digg-comments-badge">
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
