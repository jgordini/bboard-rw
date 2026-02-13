use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos_router::components::A;

use crate::auth::UserSession;
use crate::models::IdeaWithAuthor;
use crate::routes::async_helpers::spawn_server_action;

use super::super::create_idea_auth;

#[component]
pub(super) fn IdeaSubmissionDialog(
    user_resource: Resource<Result<Option<UserSession>, ServerFnError>>,
    ideas_resource: Resource<Result<Vec<IdeaWithAuthor>, ServerFnError>>,
    stats_resource: Resource<Result<(i64, i64), ServerFnError>>,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let title = RwSignal::new(String::new());
    let content = RwSignal::new(String::new());
    let tags = RwSignal::new(String::new());
    let error_message = RwSignal::new(Option::<String>::None);
    let is_submitting = RwSignal::new(false);

    let max_title_chars: usize = 100;
    let max_content_chars: usize = 500;
    let title_count = move || title.get().len();
    let content_count = move || content.get().len();

    let title_warning = move || title_count() >= (max_title_chars as f64 * 0.9) as usize;
    let title_error = move || title_count() >= max_title_chars;
    let content_warning = move || content_count() >= (max_content_chars as f64 * 0.9) as usize;
    let content_error = move || content_count() >= max_content_chars;

    let is_logged_in = move || {
        user_resource
            .get()
            .and_then(|r: Result<Option<UserSession>, ServerFnError>| r.ok())
            .and_then(|u| u)
            .is_some()
    };

    let can_submit = move || {
        !title.get().trim().is_empty()
            && !content.get().trim().is_empty()
            && title.get().len() <= max_title_chars
            && content.get().len() <= max_content_chars
            && !is_submitting.get()
    };

    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        if !can_submit() {
            return;
        }
        is_submitting.set(true);
        error_message.set(None);

        let title_value = title.get();
        let content_value = content.get();
        let tags_value = tags.get();
        spawn_server_action(
            create_idea_auth(title_value, content_value, tags_value),
            move |_| {
                ideas_resource.refetch();
                stats_resource.refetch();
                title.set(String::new());
                content.set(String::new());
                tags.set(String::new());
                is_open.set(false);
                is_submitting.set(false);
            },
            move |e| {
                error_message.set(Some(e.to_string()));
                is_submitting.set(false);
            },
        );
    };

    view! {
        <article class="sidebar-card">
            <header class="sidebar-card-header">
                <h3 class="sidebar-card-title">"Got an Idea?"</h3>
            </header>
            <div class="sidebar-card-body">
                <p class="sidebar-intro">"Share your suggestions to improve UAB IT services."</p>
                <Suspense fallback=move || view! { <p class="loading">"…"</p> }>
                    <Show when=is_logged_in fallback=move || view! {
                        <A href="/login" attr:class="submit-btn dialog-trigger-btn">"Log in"</A>
                    }>
                        <button
                            class="submit-btn dialog-trigger-btn"
                            on:click=move |_| is_open.set(true)
                        >
                            "Post Idea"
                        </button>
                        <div role="dialog" class="dialog-overlay" aria-modal="true" aria-labelledby="dialog-title" style:display=move || if is_open.get() { "flex" } else { "none" }>
                                <div class="idea-dialog-content">
                                    <header class="dialog-header">
                                        <h2 id="dialog-title" class="dialog-title">"Submit Your Idea"</h2>
                                    </header>
                                    <form on:submit=handle_submit>
                                        <Show when=move || error_message.get().is_some()>
                                            <div class="dialog-alert dialog-alert-error" role="alert" aria-live="polite" aria-atomic="true">
                                                {move || error_message.get().unwrap_or_default()}
                                            </div>
                                        </Show>
                                        <div class="form-group">
                                            <label class="form-label" for="idea-title">"Title"</label>
                                            <input
                                                id="idea-title"
                                                type="text"
                                                class="dialog-input"
                                                placeholder="Brief title for your idea…"
                                                maxlength=max_title_chars
                                                prop:value=move || title.get()
                                                on:input=move |ev| {
                                                    title.set(event_target_value(&ev));
                                                }
                                            />
                                            <span class="char-counter" class:warning=title_warning class:error=title_error>
                                                {move || format!("{}/{}", title_count(), max_title_chars)}
                                            </span>
                                        </div>
                                        <div class="form-group">
                                            <label class="form-label" for="idea-description">"Description"</label>
                                            <textarea
                                                id="idea-description"
                                                class="dialog-textarea"
                                                placeholder="Describe your idea in more detail…"
                                                maxlength=max_content_chars
                                                prop:value=move || content.get()
                                                on:input=move |ev| {
                                                    content.set(event_target_value(&ev));
                                                }
                                            />
                                            <span class="char-counter" class:warning=content_warning class:error=content_error>
                                                {move || format!("{}/{}", content_count(), max_content_chars)}
                                            </span>
                                        </div>
                                        <div class="form-group">
                                            <label class="form-label" for="idea-tags">"Tags"</label>
                                            <input
                                                id="idea-tags"
                                                type="text"
                                                class="dialog-input"
                                                placeholder="e.g. accessibility, software, hardware"
                                                bind:value=tags
                                            />
                                        </div>
                                        <div class="dialog-footer">
                                            <button
                                                type="button"
                                                class="btn-cancel"
                                                on:click=move |_| {
                                                    is_open.set(false);
                                                    error_message.set(None);
                                                    tags.set(String::new());
                                                }
                                            >
                                                "Cancel"
                                            </button>
                                            <button
                                                type="submit"
                                                class="submit-btn"
                                                disabled=move || !can_submit()
                                            >
                                                {move || if is_submitting.get() { "Submitting…" } else { "Submit Idea" }}
                                            </button>
                                        </div>
                                    </form>
                                </div>
                        </div>
                    </Show>
                </Suspense>
            </div>
        </article>
    }
}
