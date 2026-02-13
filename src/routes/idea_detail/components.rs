use leptos::prelude::*;
use leptos_meta::Title;

use crate::auth::UserSession;
use crate::models::{CommentWithAuthor, Idea};

mod card;
mod comments;

use card::IdeaDetailCard;
use comments::CommentsSection;

#[component]
pub(super) fn IdeaDetailLoaded(
    idea: Idea,
    idea_resource: Resource<Result<Idea, ServerFnError>>,
    comments_resource: Resource<Result<Vec<CommentWithAuthor>, ServerFnError>>,
    user_resource: Resource<Result<Option<UserSession>, ServerFnError>>,
    has_voted: RwSignal<bool>,
) -> impl IntoView {
    let page_title = if idea.title.is_empty() {
        format!("Idea #{} — UAB IT Idea Board", idea.id)
    } else {
        format!("{} — UAB IT Idea Board", idea.title.clone())
    };
    let idea_id = idea.id;
    let idea_comments_enabled = idea.comments_enabled;

    view! {
        <Title text=page_title/>
        <IdeaDetailCard
            idea=idea
            idea_resource=idea_resource
            user_resource=user_resource
            has_voted=has_voted
        />
        <CommentsSection
            idea_id=idea_id
            idea_comments_enabled=idea_comments_enabled
            comments_resource=comments_resource
            user_resource=user_resource
        />
    }
}
