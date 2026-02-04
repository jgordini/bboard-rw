use leptos::prelude::*;
use leptos_meta::Title;
use leptos_shadcn_button::Button;
use leptos_shadcn_card::{Card, CardContent};
use leptos_shadcn_textarea::Textarea;
use leptos_shadcn_label::Label;
use leptos_shadcn_badge::Badge;
use leptos_shadcn_alert::{Alert, AlertTitle, AlertDescription};
use crate::models::{Idea, IdeaForm, Vote, VoteForm};

#[server(CreateIdea, "/api", "PostJson")]
pub async fn create_idea(form: IdeaForm) -> Result<Idea, ServerFnError> {
    // Validate content length
    if form.content.trim().is_empty() {
        return Err(ServerFnError::new("Idea content cannot be empty"));
    }
    if form.content.len() > 500 {
        return Err(ServerFnError::new("Idea content cannot exceed 500 characters"));
    }

    // TODO: Add profanity filter
    // TODO: Add CAPTCHA verification

    Idea::create(form.content.trim().to_string())
        .await
        .map_err(|e| {
            tracing::error!("Failed to create idea: {:?}", e);
            ServerFnError::new("Failed to create idea")
        })
}

#[server(GetIdeas, "/api", "GetJson")]
pub async fn get_ideas() -> Result<Vec<Idea>, ServerFnError> {
    Idea::get_all()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch ideas: {:?}", e);
            ServerFnError::new("Failed to fetch ideas")
        })
}

#[server(CreateVote, "/api", "PostJson")]
pub async fn create_vote(form: VoteForm) -> Result<Vote, ServerFnError> {
    // Check if user has already voted
    let has_voted = Vote::has_voted(form.idea_id, &form.voter_fingerprint)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check vote: {:?}", e);
            ServerFnError::new("Failed to check vote")
        })?;

    if has_voted {
        return Err(ServerFnError::new("You have already voted on this idea"));
    }

    Vote::create(form.idea_id, form.voter_fingerprint)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create vote: {:?}", e);
            ServerFnError::new("Failed to create vote")
        })
}

#[server(CheckVotes, "/api", "PostJson")]
pub async fn check_votes(voter_fingerprint: String) -> Result<Vec<i32>, ServerFnError> {
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
    let voter_fingerprint = create_rw_signal("voter_placeholder".to_string());
    
    // Set fingerprint on client side
    #[cfg(not(feature = "ssr"))]
    {
        use wasm_bindgen::prelude::*;
        use web_sys::window;
        create_effect(move |_| {
            if let Some(win) = window() {
                if let Ok(Some(storage)) = win.local_storage() {
                    if let Ok(Some(existing)) = storage.get_item("voter_id") {
                        voter_fingerprint.set(existing);
                        return;
                    }
                    let id = format!("voter_{}_{}", 
                        js_sys::Date::now() as u64,
                        js_sys::Math::random()
                    );
                    let _ = storage.set_item("voter_id", &id);
                    voter_fingerprint.set(id);
                }
            }
        });
    }
    
    // Track which ideas the user has voted on
    let voted_ideas = create_rw_signal::<Vec<i32>>(vec![]);

    // Load voted ideas on mount
    let load_voted_ideas = create_action(move |_: &()| {
        let fingerprint = voter_fingerprint.get();
        async move {
            match check_votes(fingerprint).await {
                Ok(ids) => voted_ideas.set(ids),
                Err(_) => {}
            }
        }
    });

    create_effect(move |_| {
        if voter_fingerprint.get() != "voter_placeholder" {
            load_voted_ideas.dispatch(());
        }
    });

    let submit_idea = create_action(move |form: &IdeaForm| {
        let fingerprint = voter_fingerprint.get();
        async move {
            match create_idea(form.clone()).await {
                Ok(_) => {
                    ideas_resource.refetch();
                    true
                }
                Err(e) => {
                    tracing::error!("Failed to submit idea: {:?}", e);
                    false
                }
            }
        }
    });

    let submit_vote = create_action(move |idea_id: &i32| {
        let fingerprint = voter_fingerprint.get();
        let idea_id = *idea_id;
        async move {
            let form = VoteForm {
                idea_id,
                voter_fingerprint: fingerprint,
            };
            match create_vote(form).await {
                Ok(_) => {
                    ideas_resource.refetch();
                    voted_ideas.update(|v| v.push(idea_id));
                    true
                }
                Err(_) => false
            }
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
                    <IdeaSubmissionForm submit_idea />
                </div>

                <div class="ideas-list">
                    <h2>"Ideas"</h2>
                    <Suspense fallback=move || view! { <p class="loading">"Loading ideas..."</p> }>
                        <ErrorBoundary fallback=|_| view! { 
                            <Alert class="destructive">
                                <AlertTitle>"Error"</AlertTitle>
                                <AlertDescription>"Failed to load ideas. Please try again later."</AlertDescription>
                            </Alert>
                        }>
                            {move || {
                                ideas_resource.get().map(|ideas| {
                                    match ideas {
                                        Ok(ideas_list) => {
                                            if ideas_list.is_empty() {
                                                view! {
                                                    <div class="empty-state">
                                                        <p>"No ideas yet. Be the first to submit one!"</p>
                                                    </div>
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <div class="ideas-grid">
                                                        <For
                                                            each=move || ideas_list.clone()
                                                            key=|idea| idea.id
                                                            children=move |idea: Idea| {
                                                                let idea_id = idea.id;
                                                                let has_voted = voted_ideas.get().contains(&idea_id);
                                                                view! {
                                                                    <IdeaCard 
                                                                        idea 
                                                                        has_voted 
                                                                        on_vote=move || submit_vote.dispatch(idea_id)
                                                                    />
                                                                }
                                                            }
                                                        />
                                                    </div>
                                                }.into_view()
                                            }
                                        }
                                        Err(_) => view! { 
                                            <Alert class="destructive">
                                                <AlertTitle>"Error"</AlertTitle>
                                                <AlertDescription>"Failed to load ideas"</AlertDescription>
                                            </Alert>
                                        }.into_view()
                                    }
                                })
                            }}
                        </ErrorBoundary>
                    </Suspense>
                </div>
            </div>
        </div>
    }
}

#[component]
fn IdeaSubmissionForm(submit_idea: Action<IdeaForm, Result<bool, ServerFnError>>) -> impl IntoView {
    let (content, set_content) = create_signal(String::new());
    let char_count = move || content.get().len();
    let max_chars = 500;
    let remaining = move || max_chars - char_count.get();
    let is_warning = move || char_count.get() >= (max_chars as f64 * 0.9) as usize;
    let is_error = move || char_count.get() >= max_chars;

    let handle_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let content_value = content.get();
        if content_value.trim().is_empty() {
            return;
        }
        if content_value.len() > max_chars {
            return;
        }
        let form = IdeaForm {
            content: content_value,
            captcha_token: None, // TODO: Add CAPTCHA
        };
        submit_idea.dispatch(form);
        set_content.set(String::new());
    };

    view! {
        <form on:submit=handle_submit>
            <div class="form-group">
                <Label for="idea-content">"Your Idea"</Label>
                <Textarea
                    id="idea-content"
                    class="idea-textarea"
                    class:warning=is_warning
                    class:error=is_error
                    placeholder="Enter your idea here (max 500 characters)..."
                    maxlength=max_chars
                    prop:value=content
                    on:input=move |ev| {
                        let value = event_target_value(&ev);
                        set_content.set(value);
                    }
                />
            </div>
            <div class="form-footer">
                <span class="char-counter" class:warning=is_warning class:error=is_error>
                    {move || format!("{}/{}", char_count.get(), max_chars)}
                </span>
                <Button 
                    type="submit" 
                    disabled=move || content.get().trim().is_empty()
                >
                    "Post"
                </Button>
            </div>
            // TODO: Add CAPTCHA widget here
        </form>
    }
}

#[component]
fn IdeaCard(
    idea: Idea,
    has_voted: bool,
    on_vote: impl Fn() + 'static,
) -> impl IntoView {
    let relative_time = format_relative_time(&idea.created_at);

    view! {
        <Card class="idea-card">
            <CardContent class="idea-card-content">
                <div class="vote-section">
                    <Button
                        class="vote-button"
                        class:voted=has_voted
                        disabled=has_voted
                        on:click=move |_| on_vote()
                        aria-label=if has_voted { "Already voted" } else { "Vote for this idea" }
                    >
                        {if has_voted { "✓" } else { "↑" }}
                    </Button>
                    <Badge class="vote-count">
                        {idea.vote_count}
                    </Badge>
                </div>
                <div class="idea-content">
                    <p class="idea-text">{idea.content.clone()}</p>
                    <span class="idea-time">{relative_time}</span>
                </div>
            </CardContent>
        </Card>
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
