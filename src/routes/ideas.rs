use leptos::prelude::*;
use leptos::ev::SubmitEvent;
use leptos_meta::Title;
use crate::models::{Idea, IdeaForm, VoteForm};

#[server]
pub async fn create_idea(content: String) -> Result<Idea, ServerFnError> {
    // Validate content length
    if content.trim().is_empty() {
        return Err(ServerFnError::new("Idea content cannot be empty"));
    }
    if content.len() > 500 {
        return Err(ServerFnError::new("Idea content cannot exceed 500 characters"));
    }
    
    // Check for profanity
    if crate::profanity::contains_profanity(&content) {
        return Err(ServerFnError::new("Your submission contains inappropriate language. Please revise and try again."));
    }

    Idea::create(content.trim().to_string())
        .await
        .map_err(|e| {
            tracing::error!("Failed to create idea: {:?}", e);
            ServerFnError::new("Failed to create idea")
        })
}

#[server]
pub async fn get_ideas() -> Result<Vec<Idea>, ServerFnError> {
    Idea::get_all()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch ideas: {:?}", e);
            ServerFnError::new("Failed to fetch ideas")
        })
}

#[server]
pub async fn create_vote(idea_id: i32, voter_fingerprint: String) -> Result<bool, ServerFnError> {
    use crate::models::Vote;
    
    // Check if user has already voted
    let has_voted = Vote::has_voted(idea_id, &voter_fingerprint)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check vote: {:?}", e);
            ServerFnError::new("Failed to check vote")
        })?;

    if has_voted {
        return Err(ServerFnError::new("You have already voted on this idea"));
    }

    Vote::create(idea_id, voter_fingerprint)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create vote: {:?}", e);
            ServerFnError::new("Failed to create vote")
        })?;
    
    Ok(true)
}

#[server]
pub async fn check_votes(voter_fingerprint: String) -> Result<Vec<i32>, ServerFnError> {
    use crate::models::Vote;
    
    Vote::get_voted_ideas(&voter_fingerprint)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check votes: {:?}", e);
            ServerFnError::new("Failed to check votes")
        })
}

/// Main Idea Board page
#[component]
pub fn IdeasPage() -> impl IntoView {
    let ideas_resource = Resource::new(|| (), |_| async { get_ideas().await });

    // Generate or retrieve voter fingerprint (will be set on client side)
    let voter_fingerprint = RwSignal::new("voter_placeholder".to_string());
    
    // Track which ideas the user has voted on
    let voted_ideas = RwSignal::new(Vec::<i32>::new());

    // Set fingerprint on client side
    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(move |_| {
            if let Some(win) = web_sys::window() {
                if let Ok(Some(storage)) = win.local_storage() {
                    if let Ok(Some(existing)) = storage.get_item("voter_id") {
                        voter_fingerprint.set(existing);
                        return;
                    }
                    let id = format!("voter_{}_{}", 
                        js_sys::Date::now() as u64,
                        (js_sys::Math::random() * 1000000.0) as u64
                    );
                    let _ = storage.set_item("voter_id", &id);
                    voter_fingerprint.set(id);
                }
            }
        });
    }

    // Load voted ideas when fingerprint is set
    Effect::new(move |_| {
        let fp = voter_fingerprint.get();
        if fp != "voter_placeholder" {
            leptos::task::spawn_local(async move {
                if let Ok(ids) = check_votes(fp).await {
                    voted_ideas.set(ids);
                }
            });
        }
    });

    view! {
        <Title text="UAB IT Idea Board"/>
        <div class="ideas-page">
            <div class="header-banner">
                <div class="container">
                    <h1 class="logo-font">"UAB IT Idea Board"</h1>
                    <p>"Share improvements and suggestions for UAB IT"</p>
                </div>
            </div>

            <div class="container page">
                <div class="idea-submission-form">
                    <h2>"Submit an Idea"</h2>
                    <IdeaSubmissionForm ideas_resource />
                </div>

                <div class="ideas-list">
                    <h2>"Ideas"</h2>
                    <Suspense fallback=move || view! { <p class="loading">"Loading ideas..."</p> }>
                        {move || {
                            ideas_resource.get().map(|ideas| {
                                match ideas {
                                    Ok(ideas_list) => {
                                        if ideas_list.is_empty() {
                                            view! {
                                                <div class="empty-state">
                                                    <p>"No ideas yet. Be the first to submit one!"</p>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="ideas-grid">
                                                    <For
                                                        each=move || ideas_list.clone()
                                                        key=|idea| idea.id
                                                        children=move |idea: Idea| {
                                                            let idea_id = idea.id;
                                                            view! {
                                                                <IdeaCard 
                                                                    idea=idea
                                                                    voter_fingerprint=voter_fingerprint
                                                                    voted_ideas=voted_ideas
                                                                    ideas_resource=ideas_resource
                                                                />
                                                            }
                                                        }
                                                    />
                                                </div>
                                            }.into_any()
                                        }
                                    }
                                    Err(_) => view! { 
                                        <div class="error">
                                            <p>"Failed to load ideas. Please try again later."</p>
                                        </div>
                                    }.into_any()
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}

#[component]
fn IdeaSubmissionForm(
    ideas_resource: Resource<Result<Vec<Idea>, ServerFnError>>
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
        leptos::task::spawn_local(async move {
            if create_idea(content_clone).await.is_ok() {
                ideas_resource.refetch();
            }
        });
        content.set(String::new());
    };

    view! {
        <form on:submit=handle_submit>
            <div class="form-group">
                <label for="idea-content">"Your Idea"</label>
                <textarea
                    id="idea-content"
                    class="idea-textarea"
                    class:warning=is_warning
                    class:error=is_error
                    placeholder="Enter your idea here (max 500 characters)..."
                    maxlength=max_chars
                    prop:value=move || content.get()
                    on:input=move |ev| {
                        content.set(event_target_value(&ev));
                    }
                />
            </div>
            <div class="form-footer">
                <span class="char-counter" class:warning=is_warning class:error=is_error>
                    {move || format!("{}/{}", char_count(), max_chars)}
                </span>
                <button 
                    type="submit" 
                    class="btn-submit"
                    disabled=move || content.get().trim().is_empty()
                >
                    "Post"
                </button>
            </div>
        </form>
    }
}

#[component]
fn IdeaCard(
    idea: Idea,
    voter_fingerprint: RwSignal<String>,
    voted_ideas: RwSignal<Vec<i32>>,
    ideas_resource: Resource<Result<Vec<Idea>, ServerFnError>>,
) -> impl IntoView {
    let idea_id = idea.id;
    let vote_count = idea.vote_count;
    let content = idea.content.clone();
    let created_at = idea.created_at;
    
    let has_voted = move || voted_ideas.get().contains(&idea_id);
    
    let handle_vote = move |_| {
        if has_voted() {
            return;
        }
        let fp = voter_fingerprint.get();
        leptos::task::spawn_local(async move {
            if create_vote(idea_id, fp).await.is_ok() {
                voted_ideas.update(|v| v.push(idea_id));
                ideas_resource.refetch();
            }
        });
    };

    let relative_time = format_relative_time(&created_at);

    view! {
        <div class="idea-card">
            <div class="vote-section">
                <button
                    class="vote-button"
                    class:voted=has_voted
                    disabled=has_voted
                    on:click=handle_vote
                >
                    {move || if has_voted() { "✓" } else { "↑" }}
                </button>
                <span class="vote-count">{vote_count}</span>
            </div>
            <div class="idea-content">
                <p class="idea-text">{content}</p>
                <span class="idea-time">{relative_time}</span>
            </div>
        </div>
    }
}

fn format_relative_time(dt: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now - *dt;
    
    if duration.num_seconds() < 60 {
        format!("{} seconds ago", duration.num_seconds())
    } else if duration.num_minutes() < 60 {
        format!("{} minutes ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_days() < 30 {
        format!("{} days ago", duration.num_days())
    } else {
        dt.format("%Y-%m-%d").to_string()
    }
}
