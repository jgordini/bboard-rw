use leptos::prelude::*;

use crate::models::IdeaWithAuthor;
use crate::routes::async_helpers::spawn_server_action_refetch_resource;
use crate::routes::view_helpers::confirm_action;

use super::super::{delete_idea_action, get_off_topic_ideas, mark_idea_off_topic_action};

#[component]
pub(super) fn ModerationTab() -> impl IntoView {
    let off_topic_ideas = Resource::new(|| (), |_| async { get_off_topic_ideas().await });

    let handle_restore = move |idea_id: i32| {
        spawn_server_action_refetch_resource(
            mark_idea_off_topic_action(idea_id, false),
            off_topic_ideas,
        );
    };

    let handle_delete = move |idea_id: i32| {
        spawn_server_action_refetch_resource(delete_idea_action(idea_id), off_topic_ideas);
    };

    view! {
        <div class="moderation-tab">
            <h2>"Off-Topic Ideas"</h2>
            <Suspense fallback=|| view! { <p>"Loading off-topic ideas…"</p> }>
                {move || off_topic_ideas.get().map(|ideas| match ideas {
                    Ok(ideas_list) if ideas_list.is_empty() => {
                        view! { <p class="empty-state">"No off-topic ideas"</p> }.into_any()
                    }
                    Ok(ideas_list) => {
                        view! {
                            <div class="off-topic-list">
                                <For
                                    each=move || ideas_list.clone()
                                    key=|iwa| iwa.idea.id
                                    children=move |iwa: IdeaWithAuthor| {
                                        let idea_id = iwa.idea.id;
                                        view! {
                                            <div class="off-topic-item callout callout-secondary">
                                                <div class="idea-content">
                                                    <h3>{iwa.idea.title.clone()}</h3>
                                                    <p>{iwa.idea.content.clone()}</p>
                                                    <span class="author">"By: " {iwa.author_name}</span>
                                                </div>
                                                <div class="moderation-actions">
                                                    <button
                                                        class="btn btn-primary"
                                                        on:click=move |_| handle_restore(idea_id)
                                                    >"Restore"</button>
                                                    <button
                                                        class="btn btn-danger"
                                                        on:click=move |_| {
                                                            if confirm_action("Permanently delete this idea? This cannot be undone.") {
                                                                handle_delete(idea_id);
                                                            }
                                                        }
                                                    >"Delete Permanently"</button>
                                                </div>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        }
                            .into_any()
                    }
                    Err(_) => view! { <p class="error">"Failed to load off-topic ideas"</p> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
