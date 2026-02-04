use leptos::prelude::*;
use leptos::ev::SubmitEvent;
use leptos_meta::Title;
use crate::models::Idea;
use leptos_shadcn_button::Button;
use leptos_shadcn_card::{Card, CardHeader, CardTitle, CardContent};
use leptos_shadcn_badge::{Badge, BadgeVariant};
use leptos_shadcn_label::Label;

#[server]
pub async fn create_idea(content: String) -> Result<Idea, ServerFnError> {
    if content.trim().is_empty() {
        return Err(ServerFnError::new("Idea content cannot be empty"));
    }
    if content.len() > 500 {
        return Err(ServerFnError::new("Idea content cannot exceed 500 characters"));
    }
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

#[server]
pub async fn get_idea_statistics() -> Result<(i64, i64), ServerFnError> {
    Idea::get_statistics()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch statistics: {:?}", e);
            ServerFnError::new("Failed to fetch statistics")
        })
}

#[derive(Clone, Copy, PartialEq)]
enum SortMode {
    Popular,
    Recent,
}

/// Main Idea Board page — Digg-style layout
#[component]
pub fn IdeasPage() -> impl IntoView {
    let ideas_resource = Resource::new(|| (), |_| async { get_ideas().await });
    let stats_resource = Resource::new(|| (), |_| async { get_idea_statistics().await });
    let sort_mode = RwSignal::new(SortMode::Popular);
    let voter_fingerprint = RwSignal::new("voter_placeholder".to_string());
    let voted_ideas = RwSignal::new(Vec::<i32>::new());

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
                    <p>"Share your ideas to improve UAB IT services"</p>
                </div>
            </div>

            <div class="container page">
                <div class="digg-layout">
                    <div class="main-column">
                        <div class="sort-tabs">
                            <button
                                class="sort-tab"
                                class:active=move || sort_mode.get() == SortMode::Popular
                                on:click=move |_| sort_mode.set(SortMode::Popular)
                            >
                                "Popular"
                            </button>
                            <button
                                class="sort-tab"
                                class:active=move || sort_mode.get() == SortMode::Recent
                                on:click=move |_| sort_mode.set(SortMode::Recent)
                            >
                                "Recent"
                            </button>
                        </div>

                        <Suspense fallback=move || view! { <p class="loading">"Loading ideas..."</p> }>
                            {move || {
                                ideas_resource.get().map(|ideas| {
                                    match ideas {
                                        Ok(mut ideas_list) => {
                                            if sort_mode.get() == SortMode::Recent {
                                                ideas_list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                                            }
                                            if ideas_list.is_empty() {
                                                view! {
                                                    <div class="empty-state">
                                                        <p>"No ideas yet. Be the first to submit one!"</p>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                let ranked: Vec<(usize, Idea)> = ideas_list.into_iter()
                                                    .enumerate()
                                                    .map(|(i, idea)| (i + 1, idea))
                                                    .collect();
                                                view! {
                                                    <div class="digg-list">
                                                        <For
                                                            each=move || ranked.clone()
                                                            key=|(_, idea)| idea.id
                                                            children=move |(rank, idea): (usize, Idea)| {
                                                                view! {
                                                                    <DiggCard
                                                                        idea=idea
                                                                        rank=rank
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
                                            <div class="error-state">
                                                <p>"Failed to load ideas. Please try again later."</p>
                                            </div>
                                        }.into_any()
                                    }
                                })
                            }}
                        </Suspense>
                    </div>

                    <div class="sidebar">
                        <Card class="sidebar-card">
                            <CardHeader class="sidebar-card-header">
                                <CardTitle class="sidebar-card-title">"Submit an Idea"</CardTitle>
                            </CardHeader>
                            <CardContent class="sidebar-card-body">
                                <IdeaSubmissionForm ideas_resource stats_resource />
                            </CardContent>
                        </Card>

                        <Suspense fallback=move || view! { <p class="loading">"Loading..."</p> }>
                            {move || {
                                stats_resource.get().map(|stats| {
                                    match stats {
                                        Ok((ideas_count, votes_count)) => view! {
                                            <Card class="sidebar-card">
                                                <CardHeader class="sidebar-card-header">
                                                    <CardTitle class="sidebar-card-title">"Community"</CardTitle>
                                                </CardHeader>
                                                <CardContent class="sidebar-card-body">
                                                    <div class="stats-row">
                                                        <div class="stat-box">
                                                            <span class="stat-value">{ideas_count}</span>
                                                            <span class="stat-label">"ideas"</span>
                                                        </div>
                                                        <div class="stat-box">
                                                            <span class="stat-value">{votes_count}</span>
                                                            <span class="stat-label">"votes"</span>
                                                        </div>
                                                    </div>
                                                </CardContent>
                                            </Card>
                                        }.into_any(),
                                        Err(_) => view! { <span></span> }.into_any()
                                    }
                                })
                            }}
                        </Suspense>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn IdeaSubmissionForm(
    ideas_resource: Resource<Result<Vec<Idea>, ServerFnError>>,
    stats_resource: Resource<Result<(i64, i64), ServerFnError>>,
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
                stats_resource.refetch();
            }
        });
        content.set(String::new());
    };

    view! {
        <form on:submit=handle_submit>
            <div class="form-group">
                <Label class="form-label">"Your Idea"</Label>
                <textarea
                    class="idea-textarea"
                    class:warning=is_warning
                    class:error=is_error
                    placeholder="What could UAB IT do better? (max 500 characters)"
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
                <Button
                    class="submit-btn"
                    disabled=Signal::derive(move || content.get().trim().is_empty())
                >
                    "Post Idea"
                </Button>
            </div>
        </form>
    }
}

#[component]
fn DiggCard(
    idea: Idea,
    rank: usize,
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
        <div class="digg-item">
            <div class="digg-rank">{rank}</div>
            <div class="digg-vote-box" class:voted=has_voted>
                <span class="digg-arrow">"▲"</span>
                <span class="digg-count">{vote_count}</span>
                <button
                    class="digg-btn"
                    disabled=has_voted
                    on:click=handle_vote
                >
                    {move || if has_voted() { "voted" } else { "vote" }}
                </button>
            </div>
            <div class="digg-content">
                <p class="digg-text">{content}</p>
                <Badge variant=BadgeVariant::Outline class="digg-time">
                    {format!("submitted {}", relative_time)}
                </Badge>
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
