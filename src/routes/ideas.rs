use std::collections::HashMap;
use leptos::prelude::*;
use leptos::ev::SubmitEvent;
use leptos_meta::Title;
use leptos_router::components::A;
use crate::auth::{get_user, UserSession};
use crate::models::{Idea, IdeaWithAuthor, STAGES};
use leptos_shadcn_ui::{
    Button, ButtonVariant,
    Card, CardHeader, CardTitle, CardContent,
    Badge, BadgeVariant,
    Label, Input,
    Alert, AlertDescription,
    Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter, DialogClose,
};

// ============================================================================
// SERVER FUNCTIONS
// ============================================================================

#[server]
pub async fn create_idea_auth(title: String, content: String) -> Result<Idea, ServerFnError> {
    use crate::auth::require_auth;
    let user = require_auth().await?;

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

    Idea::create(user.id, title.trim().to_string(), content.trim().to_string())
        .await
        .map_err(|e| {
            tracing::error!("Failed to create idea: {:?}", e);
            ServerFnError::new("Failed to create idea")
        })
}

#[server]
pub async fn get_ideas_with_authors() -> Result<Vec<IdeaWithAuthor>, ServerFnError> {
    Idea::get_all()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch ideas: {:?}", e);
            ServerFnError::new("Failed to fetch ideas")
        })
}

#[server]
pub async fn toggle_vote(idea_id: i32) -> Result<bool, ServerFnError> {
    use crate::auth::require_auth;
    use crate::models::Vote;

    let user = require_auth().await?;
    Vote::toggle(user.id, idea_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to toggle vote: {:?}", e);
            ServerFnError::new("Failed to toggle vote")
        })
}

#[server]
pub async fn check_user_votes() -> Result<Vec<i32>, ServerFnError> {
    use crate::auth::require_auth;
    use crate::models::Vote;

    let user = require_auth().await?;
    Vote::get_voted_ideas(user.id)
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

#[server]
pub async fn flag_idea_server(idea_id: i32) -> Result<(), ServerFnError> {
    use crate::auth::require_auth;
    use crate::models::Flag;

    let user = require_auth().await?;
    Flag::create(user.id, "idea", idea_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to flag idea: {}", e)))
}

// ============================================================================
// COMPONENTS
// ============================================================================

#[derive(Clone, Copy, PartialEq)]
enum SortMode {
    Popular,
    Recent,
}

/// Main Idea Board page
#[component]
pub fn IdeasPage() -> impl IntoView {
    let user_resource = Resource::new(|| (), |_| async { get_user().await });
    let ideas_resource = Resource::new(|| (), |_| async { get_ideas_with_authors().await });
    let stats_resource = Resource::new(|| (), |_| async { get_idea_statistics().await });
    let comment_counts_resource = Resource::new(|| (), |_| async { get_comment_counts().await });
    let voted_ideas = RwSignal::new(Vec::<i32>::new());
    let sort_mode = RwSignal::new(SortMode::Popular);

    // Load user's voted ideas
    Effect::new(move |_| {
        if let Some(Ok(Some(_user))) = user_resource.get() {
            leptos::task::spawn_local(async move {
                if let Ok(ids) = check_user_votes().await {
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
                    <div class="header-content">
                        <div>
                            <h1 class="logo-font">"UAB IT Idea Board"</h1>
                            <p>"Share your ideas to improve UAB IT services"</p>
                        </div>
                        <div class="header-actions">
                            <Suspense fallback=|| ()>
                                {move || user_resource.get().map(|user_result| {
                                    match user_result {
                                        Ok(Some(user)) => view! {
                                            <div class="user-menu">
                                                <span class="user-name">"Hello, " {user.name.clone()}</span>
                                                {move || {
                                                    if user.is_moderator() {
                                                        view! { <A href="/admin" attr:class="admin-link">"Admin"</A> }.into_any()
                                                    } else {
                                                        view! {}.into_any()
                                                    }
                                                }}
                                                <A href="/profile">"Profile"</A>
                                            </div>
                                        }.into_any(),
                                        Ok(None) => view! {
                                            <div class="auth-links">
                                                <A href="/login" attr:class="auth-link">"Login"</A>
                                                <A href="/signup" attr:class="auth-link auth-link-primary">"Sign Up"</A>
                                            </div>
                                        }.into_any(),
                                        Err(_) => view! {}.into_any()
                                    }
                                })}
                            </Suspense>
                        </div>
                    </div>
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
                                                ideas_list.sort_by(|a, b| b.idea.created_at.cmp(&a.idea.created_at));
                                            }
                                            if ideas_list.is_empty() {
                                                view! {
                                                    <div class="empty-state">
                                                        <p>"No ideas yet. Be the first to submit one!"</p>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                let ranked: Vec<(usize, IdeaWithAuthor)> = ideas_list.into_iter()
                                                    .enumerate()
                                                    .map(|(i, idea)| (i + 1, idea))
                                                    .collect();
                                                view! {
                                                    <div class="digg-list">
                                                        <For
                                                            each=move || ranked.clone()
                                                            key=|(_, iwa)| iwa.idea.id
                                                            children=move |(rank, iwa): (usize, IdeaWithAuthor)| {
                                                                view! {
                                                                    <IdeaCard
                                                                        idea_with_author=iwa
                                                                        rank=rank
                                                                        user_resource=user_resource
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
                        <IdeaSubmissionDialog
                            user_resource=user_resource
                            ideas_resource=ideas_resource
                            stats_resource=stats_resource
                        />

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
    user_resource: Resource<Result<Option<UserSession>, ServerFnError>>,
    ideas_resource: Resource<Result<Vec<IdeaWithAuthor>, ServerFnError>>,
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

    let is_logged_in = move || {
        user_resource.get()
            .and_then(|r: Result<Option<UserSession>, ServerFnError>| r.ok())
            .and_then(|u| u)
            .is_some()
    };

    let can_submit = move || {
        is_logged_in()
            && !title.get().trim().is_empty()
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
            match create_idea_auth(title_value, content_value).await {
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
                {move || {
                    if is_logged_in() {
                        view! {
                            <button
                                class="submit-btn dialog-trigger-btn"
                                on:click=move |_| is_open.set(true)
                            >
                                "Post Idea"
                            </button>
                            <Dialog open=is_open on_open_change=handle_open_change>
                                <Show when=move || is_open.get() fallback=|| ()>
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
                                                <DialogClose class="btn-cancel">
                                                    "Cancel"
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
                                </Show>
                            </Dialog>
                        }.into_any()
                    } else {
                        view! {
                            <div class="login-prompt">
                                <p>"Please log in to submit ideas"</p>
                                <A href="/login" attr:class="btn-login">"Log In"</A>
                            </div>
                        }.into_any()
                    }
                }}
            </CardContent>
        </Card>
    }
}

#[component]
fn IdeaCard(
    idea_with_author: IdeaWithAuthor,
    rank: usize,
    user_resource: Resource<Result<Option<UserSession>, ServerFnError>>,
    voted_ideas: RwSignal<Vec<i32>>,
    ideas_resource: Resource<Result<Vec<IdeaWithAuthor>, ServerFnError>>,
    comment_counts_resource: Resource<Result<HashMap<i32, i64>, ServerFnError>>,
) -> impl IntoView {
    let idea_id = idea_with_author.idea.id;
    let vote_count = idea_with_author.idea.vote_count;
    let title = idea_with_author.idea.title.clone();
    let content = idea_with_author.idea.content.clone();
    let stage = idea_with_author.idea.stage.clone();
    let is_pinned = idea_with_author.idea.is_pinned();
    let created_at = idea_with_author.idea.created_at;
    let author_name = idea_with_author.author_name.clone();

    let has_voted = move || voted_ideas.get().contains(&idea_id);
    let is_logged_in = move || {
        user_resource.get()
            .and_then(|r: Result<Option<UserSession>, ServerFnError>| r.ok())
            .and_then(|u| u)
            .is_some()
    };

    let comment_count = move || {
        comment_counts_resource.get()
            .and_then(|r| r.ok())
            .and_then(|counts| counts.get(&idea_id).copied())
            .unwrap_or(0)
    };

    let handle_vote = move |_| {
        if !is_logged_in() || has_voted() {
            return;
        }
        leptos::task::spawn_local(async move {
            if toggle_vote(idea_id).await.is_ok() {
                voted_ideas.update(|v| v.push(idea_id));
                ideas_resource.refetch();
            }
        });
    };

    let relative_time = format_relative_time(&created_at);
    let stage_color = stage_badge_color(&stage);

    view! {
        <div class="digg-item" class:pinned=is_pinned>
            {move || {
                if is_pinned {
                    view! { <span class="pin-icon">"📌"</span> }.into_any()
                } else {
                    view! {}.into_any()
                }
            }}
            <div class="digg-rank">{rank}</div>
            <div class="digg-vote-box" class:voted=has_voted>
                <span class="digg-arrow">"▲"</span>
                <span class="digg-count">{vote_count}</span>
                <button
                    class="digg-btn"
                    disabled=move || !is_logged_in() || has_voted()
                    on:click=handle_vote
                    title=move || if !is_logged_in() { "Login to vote" } else if has_voted() { "Already voted" } else { "Vote for this idea" }
                >
                    {move || if has_voted() { "voted" } else { "vote" }}
                </button>
            </div>
            <a class="digg-content" href=format!("/ideas/{}", idea_id)>
                <h3 class="digg-title">{title}</h3>
                <p class="digg-text">{content}</p>
                <div class="digg-meta">
                    <Badge variant=BadgeVariant::Secondary class=format!("stage-badge stage-{}", stage_color)>
                        {stage.clone()}
                    </Badge>
                    <span class="author-name">"by " {author_name}</span>
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

// ============================================================================
// HELPERS
// ============================================================================

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

fn stage_badge_color(stage: &str) -> &str {
    match stage {
        "Ideate" => "ideate",
        "Review" => "review",
        "In Progress" => "progress",
        "Completed" => "completed",
        _ => "default",
    }
}
