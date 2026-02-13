use std::cmp::Ordering;
use std::collections::HashMap;
use leptos::prelude::*;
use crate::models::{Idea, IdeaWithAuthor};
use crate::routes::error_helpers::server_fn_error_with_log;
use crate::routes::validation_helpers::validate_idea_title_and_content;

mod components;
use components::IdeasBoard;

// ============================================================================
// SERVER FUNCTIONS
// ============================================================================

#[server]
pub async fn create_idea_auth(title: String, content: String, tags: String) -> Result<Idea, ServerFnError> {
    use crate::auth::require_auth;
    let user = require_auth().await?;

    validate_idea_title_and_content(&title, &content)?;

    Idea::create(user.id, title.trim().to_string(), content.trim().to_string(), tags.trim().to_string())
        .await
        .map_err(|e| server_fn_error_with_log("Failed to create idea", e, "Failed to create idea"))
}

#[server]
pub async fn get_ideas_with_authors() -> Result<Vec<IdeaWithAuthor>, ServerFnError> {
    Idea::get_all()
        .await
        .map_err(|e| server_fn_error_with_log("Failed to fetch ideas", e, "Failed to fetch ideas"))
}

#[server]
pub async fn toggle_vote(idea_id: i32) -> Result<bool, ServerFnError> {
    use crate::auth::require_auth;
    use crate::models::Vote;

    let user = require_auth().await?;
    Vote::toggle(user.id, idea_id)
        .await
        .map_err(|e| server_fn_error_with_log("Failed to toggle vote", e, "Failed to toggle vote"))
}

#[server]
pub async fn check_user_votes() -> Result<Vec<i32>, ServerFnError> {
    use crate::auth::require_auth;
    use crate::models::Vote;

    let user = require_auth().await?;
    Vote::get_voted_ideas(user.id)
        .await
        .map_err(|e| server_fn_error_with_log("Failed to check votes", e, "Failed to check votes"))
}

#[server]
pub async fn get_idea_statistics() -> Result<(i64, i64), ServerFnError> {
    Idea::get_statistics()
        .await
        .map_err(|e| {
            server_fn_error_with_log("Failed to fetch statistics", e, "Failed to fetch statistics")
        })
}

#[server]
pub async fn get_comment_counts() -> Result<HashMap<i32, i64>, ServerFnError> {
    use crate::models::Comment;
    Comment::count_all_grouped()
        .await
        .map(|counts| counts.into_iter().collect())
        .map_err(|e| {
            server_fn_error_with_log("Failed to fetch comment counts", e, "Failed to fetch comment counts")
        })
}

#[server]
pub async fn flag_idea_server(idea_id: i32) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::models::Flag;

    let user = require_auth().await?;
    Flag::create(user.id, "idea", idea_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to flag idea: {}", e)))
}

// ============================================================================
// COMPONENTS
// ============================================================================

#[derive(Clone, Copy, PartialEq)]
pub(super) enum SortMode {
    Popular,
    Recent,
}

/// Main Idea Board page
#[component]
pub fn IdeasPage() -> impl IntoView {
    view! { <IdeasBoard/> }
}

fn compare_pinned_first(a: &IdeaWithAuthor, b: &IdeaWithAuthor) -> Ordering {
    // Keep pinned ideas ahead of unpinned ideas in every sort mode.
    b.idea
        .is_pinned()
        .cmp(&a.idea.is_pinned())
        .then_with(|| b.idea.pinned_at.cmp(&a.idea.pinned_at))
}

pub(super) fn sort_ideas(ideas: &mut [IdeaWithAuthor], mode: SortMode) {
    ideas.sort_by(|a, b| match mode {
        SortMode::Popular => compare_pinned_first(a, b)
            .then_with(|| b.idea.vote_count.cmp(&a.idea.vote_count))
            .then_with(|| b.idea.created_at.cmp(&a.idea.created_at)),
        SortMode::Recent => compare_pinned_first(a, b)
            .then_with(|| b.idea.created_at.cmp(&a.idea.created_at)),
    });
}
