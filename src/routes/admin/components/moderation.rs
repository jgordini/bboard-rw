use leptos::prelude::*;
use web_sys::window;

use crate::models::IdeaWithAuthor;

use super::super::{delete_idea_action, get_off_topic_ideas, mark_idea_off_topic_action};

#[component]
pub(super) fn ModerationTab() -> impl IntoView {
    let off_topic_ideas = Resource::new(|| (), |_| async { get_off_topic_ideas().await });

    let handle_restore = move |idea_id: i32| {
        leptos::task::spawn_local(async move {
            if mark_idea_off_topic_action(idea_id, false).await.is_ok() {
                off_topic_ideas.refetch();
            }
        });
    };

    let handle_delete = move |idea_id: i32| {
        leptos::task::spawn_local(async move {
            if delete_idea_action(idea_id).await.is_ok() {
                off_topic_ideas.refetch();
            }
        });
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
                                            <div class="off-topic-item">
                                                <div class="idea-content">
                                                    <h3>{iwa.idea.title.clone()}</h3>
                                                    <p>{iwa.idea.content.clone()}</p>
                                                    <span class="author">"By: " {iwa.author_name}</span>
                                                </div>
                                                <div class="moderation-actions">
                                                    <button
                                                        class="btn-primary"
                                                        on:click=move |_| handle_restore(idea_id)
                                                    >"Restore"</button>
                                                    <button
                                                        class="btn-danger"
                                                        on:click=move |_| {
                                                            if let Some(w) = window() {
                                                                if w.confirm_with_message("Permanently delete this idea? This cannot be undone.").unwrap_or(false) {
                                                                    handle_delete(idea_id);
                                                                }
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
