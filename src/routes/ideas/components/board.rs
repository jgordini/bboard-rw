use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

use crate::auth::get_user;
use crate::models::IdeaWithAuthor;

use super::card::IdeaCard;
use super::submission::IdeaSubmissionDialog;
use super::super::{
    SortMode, check_user_votes, get_comment_counts, get_idea_statistics, get_ideas_with_authors,
    sort_ideas,
};

#[component]
pub fn IdeasBoard() -> impl IntoView {
    let auth_refresh = expect_context::<crate::auth::AuthRefresh>().0;
    let user_resource = Resource::new(
        move || auth_refresh.get(),
        move |_| async move { get_user().await },
    );
    let ideas_resource = Resource::new(|| (), |_| async { get_ideas_with_authors().await });
    let stats_resource = Resource::new(|| (), |_| async { get_idea_statistics().await });
    let comment_counts_resource = Resource::new(|| (), |_| async { get_comment_counts().await });
    let voted_ideas = RwSignal::new(Vec::<i32>::new());
    let sort_mode = RwSignal::new(SortMode::Popular);
    let search_query = RwSignal::new(String::new());

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
                            <h1 class="logo-font">"Spark"</h1>
                            <p>"Share your ideas to improve UAB IT services"</p>
                        </div>
                        <div class="header-actions">
                            <Suspense fallback=|| ()>
                                {move || user_resource.get().map(|user_result| {
                                    match user_result {
                                        Ok(Some(user)) => view! {
                                            <div class="user-menu">
                                                <span class="user-name">"Hello, " {user.name.clone()}</span>
                                            </div>
                                        }.into_any(),
                                        Ok(None) => view! {
                                            <div class="auth-links">
                                                <A href="/login" attr:class="auth-link">"Login"</A>
                                                <A href="/signup" attr:class="auth-link auth-link-primary">"Sign Up"</A>
                                            </div>
                                        }.into_any(),
                                        Err(_) => ().into_any()
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

                        <Suspense fallback=move || view! { <p class="loading">"Loading ideas…"</p> }>
                            {move || {
                                ideas_resource.get().map(|ideas| {
                                    match ideas {
                                        Ok(mut ideas_list) => {
                                            sort_ideas(&mut ideas_list, sort_mode.get());
                                            let query = search_query.get().to_lowercase();
                                            if !query.is_empty() {
                                                ideas_list.retain(|iwa| {
                                                    iwa.idea.title.to_lowercase().contains(&query)
                                                        || iwa.idea.content.to_lowercase().contains(&query)
                                                        || iwa.idea.tags.to_lowercase().contains(&query)
                                                        || iwa.author_name.to_lowercase().contains(&query)
                                                });
                                            }
                                            if ideas_list.is_empty() {
                                                view! {
                                                    <div class="empty-state">
                                                        <p>{move || if search_query.get().is_empty() { "No ideas yet. Be the first to submit one!" } else { "No ideas match your search." }}</p>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                let ranked: Vec<(usize, IdeaWithAuthor)> = ideas_list
                                                    .into_iter()
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
                        <article class="sidebar-card">
                            <header class="sidebar-card-header">
                                <h3 class="sidebar-card-title">"Search Ideas"</h3>
                            </header>
                            <div class="sidebar-card-body">
                                <label for="idea-search" class="sr-only">"Search Ideas"</label>
                                <input
                                    id="idea-search"
                                    type="search"
                                    name="search"
                                    class="search-input"
                                    placeholder="Search by title, content, tags…"
                                    autocomplete="off"
                                    prop:value=move || search_query.get()
                                    on:input=move |ev| search_query.set(event_target_value(&ev))
                                />
                            </div>
                        </article>

                        <IdeaSubmissionDialog
                            user_resource=user_resource
                            ideas_resource=ideas_resource
                            stats_resource=stats_resource
                        />

                        <Suspense fallback=move || view! { <p class="loading">"Loading…"</p> }>
                            {move || {
                                stats_resource.get().map(|stats| {
                                    match stats {
                                        Ok((ideas_count, votes_count)) => view! {
                                            <article class="sidebar-card">
                                                <header class="sidebar-card-header">
                                                    <h3 class="sidebar-card-title">"Community"</h3>
                                                </header>
                                                <div class="sidebar-card-body">
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
                                                </div>
                                            </article>
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
