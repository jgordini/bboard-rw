use std::collections::HashMap;
use leptos::prelude::*;
use leptos::ev::SubmitEvent;
use leptos_meta::Title;
use crate::models::Idea;
use leptos_shadcn_button::{Button, ButtonVariant};
use leptos_shadcn_card::{Card, CardHeader, CardTitle, CardContent};
use leptos_shadcn_badge::{Badge, BadgeVariant};
use leptos_shadcn_label::Label;
use leptos_shadcn_input::Input;
use leptos_shadcn_alert::{Alert, AlertDescription};
use leptos_shadcn_dialog::{Dialog, DialogTrigger, DialogContent, DialogHeader, DialogTitle, DialogFooter, DialogClose};

#[server]
pub async fn create_idea(title: String, content: String) -> Result<Idea, ServerFnError> {
    if title.trim().is_empty() {
        return Err(ServerFnError::new("Idea title cannot be empty"));
    }
    if title.len() > 100 {
        return Err(ServerFnError::new("Idea title cannot exceed 100 characters"));
    }
    if content.trim().is_empty() {
        return Err(ServerFnError::new("Idea description cannot be empty"));
    }
    if content.len() > 500 {
        return Err(ServerFnError::new("Idea description cannot exceed 500 characters"));
    }
    if crate::profanity::contains_profanity(&title) || crate::profanity::contains_profanity(&content) {
        return Err(ServerFnError::new("Your submission contains inappropriate language. Please revise and try again."));
    }
    Idea::create(title.trim().to_string(), content.trim().to_string())
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

#[server]
pub async fn get_comment_counts() -> Result<HashMap<i32, i64>, ServerFnError> {
    use crate::models::Comment;
    Comment::count_all_grouped()
        .await
        .map(|counts| counts.into_iter().collect())
        .map_err(|e| {
            tracing::error!("Failed to fetch comment counts: {:?}", e);
            ServerFnError::new("Failed to fetch comment counts")
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
    let comment_counts_resource = Resource::new(|| (), |_| async { get_comment_counts().await });
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
                                                                        comment_counts_resource=comment_counts_resource
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
                        <IdeaSubmissionDialog ideas_resource stats_resource />

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
fn IdeaSubmissionDialog(
    ideas_resource: Resource<Result<Vec<Idea>, ServerFnError>>,
    stats_resource: Resource<Result<(i64, i64), ServerFnError>>,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let title = RwSignal::new(String::new());
    let content = RwSignal::new(String::new());
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
        leptos::task::spawn_local(async move {
            match create_idea(title_value, content_value).await {
                Ok(_) => {
                    ideas_resource.refetch();
                    stats_resource.refetch();
                    title.set(String::new());
                    content.set(String::new());
                    is_open.set(false);
                }
                Err(e) => {
                    error_message.set(Some(e.to_string()));
                }
            }
            is_submitting.set(false);
        });
    };

    let handle_open_change = Callback::new(move |open: bool| {
        is_open.set(open);
        if !open {
            error_message.set(None);
        }
    });

    view! {
        <Card class="sidebar-card">
            <CardHeader class="sidebar-card-header">
                <CardTitle class="sidebar-card-title">"Got an Idea?"</CardTitle>
            </CardHeader>
            <CardContent class="sidebar-card-body">
                <p class="sidebar-intro">"Share your suggestions to improve UAB IT services."</p>
                <Dialog open=is_open on_open_change=handle_open_change>
                    <DialogTrigger>
                        <Button class="submit-btn dialog-trigger-btn">"Post Idea"</Button>
                    </DialogTrigger>
                    <DialogContent class="idea-dialog-content">
                        <DialogHeader>
                            <DialogTitle class="dialog-title">"Submit Your Idea"</DialogTitle>
                        </DialogHeader>
                        <form on:submit=handle_submit>
                            <Show when=move || error_message.get().is_some()>
                                <Alert class="dialog-alert dialog-alert-error">
                                    <AlertDescription>
                                        {move || error_message.get().unwrap_or_default()}
                                    </AlertDescription>
                                </Alert>
                            </Show>
                            <div class="form-group">
                                <Label class="form-label">"Title"</Label>
                                <Input
                                    class="dialog-input"
                                    placeholder="Brief title for your idea"
                                    value=title
                                    on_change=Callback::new(move |val: String| {
                                        if val.len() <= max_title_chars {
                                            title.set(val);
                                        }
                                    })
                                />
                                <span class="char-counter" class:warning=title_warning class:error=title_error>
                                    {move || format!("{}/{}", title_count(), max_title_chars)}
                                </span>
                            </div>
                            <div class="form-group">
                                <Label class="form-label">"Description"</Label>
                                // Using native textarea since shadcn Textarea has a rendering bug
                                <textarea
                                    class="dialog-textarea"
                                    placeholder="Describe your idea in more detail..."
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
                            <DialogFooter class="dialog-footer">
                                <DialogClose>
                                    <Button variant=ButtonVariant::Outline class="btn-cancel">"Cancel"</Button>
                                </DialogClose>
                                <Button
                                    class="submit-btn"
                                    disabled=Signal::derive(move || !can_submit())
                                >
                                    {move || if is_submitting.get() { "Submitting..." } else { "Submit Idea" }}
                                </Button>
                            </DialogFooter>
                        </form>
                    </DialogContent>
                </Dialog>
            </CardContent>
        </Card>
    }
}

#[component]
fn DiggCard(
    idea: Idea,
    rank: usize,
    voter_fingerprint: RwSignal<String>,
    voted_ideas: RwSignal<Vec<i32>>,
    ideas_resource: Resource<Result<Vec<Idea>, ServerFnError>>,
    comment_counts_resource: Resource<Result<HashMap<i32, i64>, ServerFnError>>,
) -> impl IntoView {
    let idea_id = idea.id;
    let vote_count = idea.vote_count;
    let title = idea.title.clone();
    let content = idea.content.clone();
    let created_at = idea.created_at;

    let has_voted = move || voted_ideas.get().contains(&idea_id);

    let comment_count = move || {
        comment_counts_resource.get()
            .and_then(|r| r.ok())
            .and_then(|counts| counts.get(&idea_id).copied())
            .unwrap_or(0)
    };

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
            <a class="digg-content" href=format!("/ideas/{}", idea_id)>
                <h3 class="digg-title">{title}</h3>
                <p class="digg-text">{content}</p>
                <div class="digg-meta">
                    <Badge variant=BadgeVariant::Outline class="digg-time">
                        {format!("submitted {}", relative_time)}
                    </Badge>
                    <Badge class="digg-comments-badge">
                        {move || {
                            let count = comment_count();
                            if count == 1 {
                                "1 comment".to_string()
                            } else {
                                format!("{} comments", count)
                            }
                        }}
                    </Badge>
                </div>
            </a>
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
