use leptos::prelude::*;
use web_sys::window;

use crate::routes::async_helpers::spawn_server_action_refetch;

use super::super::{
    FlaggedItemDetail, clear_flags_action, delete_idea_action, get_flagged_content,
    mark_idea_off_topic_action,
};

#[component]
pub(super) fn FlagsTab() -> impl IntoView {
    let flagged_items = Resource::new(|| (), |_| async { get_flagged_content().await });

    let handle_clear_flags = move |target_type: String, target_id: i32| {
        spawn_server_action_refetch(clear_flags_action(target_type, target_id), move || {
            flagged_items.refetch();
        });
    };

    let handle_mark_off_topic = move |idea_id: i32| {
        spawn_server_action_refetch(mark_idea_off_topic_action(idea_id, true), move || {
            flagged_items.refetch();
        });
    };

    let handle_delete = move |target_type: String, target_id: i32| {
        if target_type != "idea" {
            return;
        }
        spawn_server_action_refetch(delete_idea_action(target_id), move || {
            flagged_items.refetch();
        });
    };

    view! {
        <div class="flags-tab">
            <h2>"Flagged Content"</h2>
            <Suspense fallback=|| view! { <p>"Loading flagged content…"</p> }>
                {move || flagged_items.get().map(|items| match items {
                    Ok(flagged) if flagged.is_empty() => {
                        view! { <p class="empty-state">"No flagged content"</p> }.into_any()
                    }
                    Ok(flagged) => {
                        view! {
                            <div class="flagged-items-list">
                                <For
                                    each=move || flagged.clone()
                                    key=|item| (item.target_type.clone(), item.target_id)
                                    children=move |item: FlaggedItemDetail| {
                                        let target_type = item.target_type.clone();
                                        let target_type_for_check = target_type.clone();
                                        let target_id = item.target_id;
                                        let item_type_display = item.target_type.clone();
                                        let content_preview = item.content_preview.clone();
                                        let flag_count = item.flag_count;
                                        view! {
                                            <div class="flagged-item">
                                                <div class="flagged-info">
                                                    <span class="flag-badge">{flag_count}" flags"</span>
                                                    <span class="content-type">{item_type_display}</span>
                                                    <p class="content-preview">{content_preview}</p>
                                                </div>
                                                <div class="flagged-actions">
                                                    <button
                                                        class="btn-secondary"
                                                        on:click=move |_| handle_clear_flags(target_type.clone(), target_id)
                                                    >"Dismiss Flags"</button>
                                                    {move || {
                                                        let target_type_for_delete = target_type_for_check.clone();
                                                        if target_type_for_check == "idea" {
                                                            view! {
                                                                <>
                                                                    <button
                                                                        class="btn-warning"
                                                                        on:click=move |_| handle_mark_off_topic(target_id)
                                                                    >"Mark Off-Topic"</button>
                                                                    <button
                                                                        class="btn-danger"
                                                                        on:click=move |_| {
                                                                            if let Some(w) = window() {
                                                                                if w.confirm_with_message("Delete this idea? This cannot be undone.").unwrap_or(false) {
                                                                                    handle_delete(target_type_for_delete.clone(), target_id);
                                                                                }
                                                                            }
                                                                        }
                                                                    >"Delete"</button>
                                                                </>
                                                            }
                                                                .into_any()
                                                        } else {
                                                            view! {}.into_any()
                                                        }
                                                    }}
                                                </div>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        }
                            .into_any()
                    }
                    Err(_) => view! { <p class="error">"Failed to load flagged content"</p> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
