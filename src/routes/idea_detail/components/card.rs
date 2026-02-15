use leptos::prelude::*;

use crate::auth::UserSession;
use crate::models::Idea;
use crate::routes::async_helpers::{
    spawn_server_action, spawn_server_action_ok, spawn_server_action_refetch_resource,
    spawn_server_action_with_error,
};
use crate::routes::ideas::toggle_vote;
use crate::routes::view_helpers::{format_relative_time, stage_badge_color};

use super::super::{toggle_idea_comments, update_idea_content_mod};

#[component]
pub(super) fn IdeaDetailCard(
    idea: Idea,
    idea_resource: Resource<Result<Idea, ServerFnError>>,
    user_resource: Resource<Result<Option<UserSession>, ServerFnError>>,
    has_voted: RwSignal<bool>,
) -> impl IntoView {
    let flagged = RwSignal::new(false);
    let stage_updating = RwSignal::new(false);
    let idea_editing = RwSignal::new(false);
    let idea_edit_error = RwSignal::new(Option::<String>::None);

    let idea_id_val = idea.id;
    let idea_pinned = idea.is_pinned();
    let idea_comments_enabled = idea.comments_enabled;
    let idea_vote_count = idea.vote_count;
    let idea_title = idea.title.clone();
    let idea_content = idea.content.clone();
    let relative_time = format_relative_time(&idea.created_at);
    let stage = idea.stage.clone();
    let stage_color = stage_badge_color(&stage).to_string();
    let tags_str = idea.tags.clone();
    let edit_title = RwSignal::new(idea_title.clone());
    let edit_content = RwSignal::new(idea_content.clone());
    let edit_tags = RwSignal::new(tags_str.clone());
    let idea_title_value = StoredValue::new(idea_title.clone());
    let idea_content_value = StoredValue::new(idea_content.clone());
    let tags_str_value = StoredValue::new(tags_str.clone());

    view! {
        <article class="detail-card">
            <div class="detail-card-body">
                <div class="detail-vote-box" class:voted=move || has_voted.get()>
                    <span class="detail-vote-arrow">"▲"</span>
                    <span class="detail-vote-count">{idea_vote_count}</span>
                    <Suspense fallback=|| view! { <span class="detail-vote-label">"sparks"</span> }>
                        {move || user_resource.get().map(|ur| match ur {
                            Ok(Some(_)) => {
                                view! {
                                    <button
                                        class="detail-vote-btn btn"
                                        on:click=move |_| {
                                            let id = idea_id_val;
                                            spawn_server_action_ok(toggle_vote(id), move |now_voted| {
                                                has_voted.set(now_voted);
                                                idea_resource.refetch();
                                            });
                                        }
                                        title=move || if has_voted.get() { "Remove spark" } else { "Spark this idea" }
                                    >
                                        {move || if has_voted.get() { "sparked" } else { "spark" }}
                                    </button>
                                }
                                    .into_any()
                            }
                            _ => view! { <span class="detail-vote-label">"sparks"</span> }.into_any(),
                        })}
                    </Suspense>
                </div>
                <div class="detail-idea-content">
                    <Show
                        when=move || idea_editing.get()
                        fallback=move || {
                            view! {
                                <Show when=move || !idea_title_value.get_value().is_empty()>
                                    <h1 class="detail-idea-title">{move || idea_title_value.get_value()}</h1>
                                </Show>
                                <p class="detail-idea-text">{move || idea_content_value.get_value()}</p>
                            }
                                .into_any()
                        }
                    >
                        <form
                            on:submit=move |ev| {
                                ev.prevent_default();
                                let title_value = edit_title.get();
                                let content_value = edit_content.get();
                                let tags_value = edit_tags.get();
                                idea_edit_error.set(None);
                                let id = idea_id_val;
                                spawn_server_action_with_error(
                                    update_idea_content_mod(id, title_value, content_value, tags_value),
                                    move |_| {
                                        idea_resource.refetch();
                                        idea_editing.set(false);
                                    },
                                    idea_edit_error,
                                );
                            }
                        >
                            <Show when=move || idea_edit_error.get().is_some()>
                                <div class="dialog-alert dialog-alert-error" role="alert" aria-live="polite" aria-atomic="true">
                                    {move || idea_edit_error.get().unwrap_or_default()}
                                </div>
                            </Show>
                            <div class="form-group">
                                <label class="form-label" for="idea-edit-title">"Title"</label>
                                <input
                                    id="idea-edit-title"
                                    class="dialog-input"
                                    type="text"
                                    maxlength=100
                                    bind:value=edit_title
                                />
                            </div>
                            <div class="form-group">
                                <label class="form-label" for="idea-edit-content">"Description"</label>
                                <textarea
                                    id="idea-edit-content"
                                    class="dialog-textarea"
                                    maxlength=500
                                    bind:value=edit_content
                                />
                            </div>
                            <div class="form-group">
                                <label class="form-label" for="idea-edit-tags">"Tags (comma-separated)"</label>
                                <input
                                    id="idea-edit-tags"
                                    class="dialog-input"
                                    type="text"
                                    maxlength=200
                                    placeholder="e.g., security, performance, ui"
                                    bind:value=edit_tags
                                />
                            </div>
                            <div class="dialog-footer">
                                <button type="submit" class="submit-btn btn btn-primary">"Save"</button>
                            </div>
                        </form>
                    </Show>
                    <Show when=move || !idea_editing.get()>
                        {move || {
                            let tag_list: Vec<String> = tags_str_value
                                .get_value()
                                .split(',')
                                .map(|s| s.trim())
                                .filter(|s| !s.is_empty())
                                .map(String::from)
                                .collect();
                            if tag_list.is_empty() {
                                ().into_any()
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
                                }
                                    .into_any()
                            }
                        }}
                    </Show>
                    <div class="detail-meta-row">
                        <div class="detail-meta-info">
                            <Suspense fallback=|| ()>
                                {move || user_resource.get().map(|ur| match ur {
                                    Ok(Some(user)) if user.is_moderator() => {
                                        let current_stage = stage.clone();
                                        let current_stage_for_value = current_stage.clone();
                                        let stage_color_class = stage_color.clone();
                                        view! {
                                            <label class="sr-only" for="idea-stage">"Stage"</label>
                                            <select
                                                id="idea-stage"
                                                class=format!("stage-select stage-badge stage-{}", stage_color_class)
                                                prop:value=move || current_stage_for_value.clone()
                                                disabled=move || stage_updating.get()
                                                on:change=move |ev| {
                                                    let new_stage = event_target_value(&ev);
                                                    if new_stage == current_stage {
                                                        return;
                                                    }
                                                    stage_updating.set(true);
                                                    let id = idea_id_val;
                                                    spawn_server_action(
                                                        crate::routes::admin::update_idea_stage_action(
                                                            id,
                                                            new_stage,
                                                        ),
                                                        move |_| {
                                                            idea_resource.refetch();
                                                            stage_updating.set(false);
                                                        },
                                                        move |_| {
                                                            stage_updating.set(false);
                                                        },
                                                    );
                                                }
                                            >
                                                <option value="Ideate">"Ideate"</option>
                                                <option value="Review">"Review"</option>
                                                <option value="In Progress">"In Progress"</option>
                                                <option value="Completed">"Completed"</option>
                                            </select>
                                        }
                                            .into_any()
                                    }
                                    _ => {
                                        let stage_color_class = stage_color.clone();
                                        view! {
                                            <span class=format!("stage-badge stage-{}", stage_color_class)>{stage.clone()}</span>
                                        }
                                            .into_any()
                                    }
                                })}
                            </Suspense>
                            <span class="detail-time">
                                {format!("submitted {}", relative_time)}
                            </span>
                        </div>
                        <Suspense fallback=|| ()>
                            {move || user_resource.get().map(|ur| match ur {
                                Ok(Some(user)) => {
                                    let is_mod = user.is_moderator();
                                    view! {
                                        <div class="detail-card-actions">
                                            <Show when=move || is_mod>
                                                <button
                                                    type="button"
                                                    class="btn-edit btn btn-secondary"
                                                    on:click=move |_| {
                                                        idea_edit_error.set(None);
                                                        idea_editing.set(!idea_editing.get());
                                                    }
                                                >
                                                    {move || if idea_editing.get() { "Cancel Edit" } else { "Edit" }}
                                                </button>
                                            </Show>
                                            <button
                                                type="button"
                                                class="btn-flag btn btn-secondary"
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
                                                    class="btn-pin btn btn-secondary"
                                                    on:click=move |_| {
                                                        let id = idea_id_val;
                                                        spawn_server_action_refetch_resource(
                                                            crate::routes::admin::toggle_idea_pin_action(id),
                                                            idea_resource,
                                                        );
                                                    }
                                                >
                                                    {move || if idea_pinned { "Unpin" } else { "Pin" }}
                                                </button>
                                                <button
                                                    type="button"
                                                    class="btn-toggle-comments btn btn-secondary"
                                                    on:click=move |_| {
                                                        let id = idea_id_val;
                                                        spawn_server_action_refetch_resource(
                                                            toggle_idea_comments(id),
                                                            idea_resource,
                                                        );
                                                    }
                                                >
                                                    {move || if idea_comments_enabled { "Lock Comments" } else { "Unlock Comments" }}
                                                </button>
                                            </Show>
                                        </div>
                                    }
                                        .into_any()
                                }
                                _ => ().into_any(),
                            })}
                        </Suspense>
                    </div>
                </div>
            </div>
        </article>
    }
}
