use leptos::prelude::*;
use leptos::ev::SubmitEvent;
use leptos_meta::Title;
use crate::models::Idea;

// Admin password check (in production, use proper hashing)
#[server]
pub async fn admin_login(password: String) -> Result<bool, ServerFnError> {
    let admin_password = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin".to_string());
    Ok(password == admin_password)
}

#[server]
pub async fn get_all_ideas_admin() -> Result<Vec<Idea>, ServerFnError> {
    Idea::get_all()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch ideas: {:?}", e);
            ServerFnError::new("Failed to fetch ideas")
        })
}

#[server]
pub async fn get_statistics() -> Result<(i64, i64), ServerFnError> {
    Idea::get_statistics()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch statistics: {:?}", e);
            ServerFnError::new("Failed to fetch statistics")
        })
}

#[server]
pub async fn delete_idea(idea_id: i32) -> Result<(), ServerFnError> {
    Idea::delete(idea_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete idea: {:?}", e);
            ServerFnError::new("Failed to delete idea")
        })
}

#[server]
pub async fn delete_all_ideas() -> Result<(), ServerFnError> {
    Idea::delete_all()
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete all ideas: {:?}", e);
            ServerFnError::new("Failed to delete all ideas")
        })
}

#[server]
pub async fn delete_old_ideas(days: i32) -> Result<(), ServerFnError> {
    Idea::delete_older_than_days(days)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete old ideas: {:?}", e);
            ServerFnError::new("Failed to delete old ideas")
        })
}

/// Admin Panel page
#[component]
pub fn AdminPage() -> impl IntoView {
    let is_authenticated = RwSignal::new(false);
    let password = RwSignal::new(String::new());
    let login_error = RwSignal::new(Option::<String>::None);

    // Check localStorage for existing session
    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(move |_| {
            if let Some(win) = web_sys::window() {
                if let Ok(Some(storage)) = win.local_storage() {
                    if let Ok(Some(session)) = storage.get_item("admin_session") {
                        if session == "authenticated" {
                            is_authenticated.set(true);
                        }
                    }
                }
            }
        });
    }

    let handle_login = move |ev: SubmitEvent| {
        ev.prevent_default();
        let pwd = password.get();
        leptos::task::spawn_local(async move {
            match admin_login(pwd).await {
                Ok(true) => {
                    is_authenticated.set(true);
                    login_error.set(None);
                    // Save session to localStorage
                    #[cfg(target_arch = "wasm32")]
                    {
                        if let Some(win) = web_sys::window() {
                            if let Ok(Some(storage)) = win.local_storage() {
                                let _ = storage.set_item("admin_session", "authenticated");
                            }
                        }
                    }
                }
                Ok(false) => {
                    login_error.set(Some("Invalid password".to_string()));
                }
                Err(e) => {
                    login_error.set(Some(format!("Login failed: {}", e)));
                }
            }
        });
    };

    let handle_logout = move |_| {
        is_authenticated.set(false);
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(win) = web_sys::window() {
                if let Ok(Some(storage)) = win.local_storage() {
                    let _ = storage.remove_item("admin_session");
                }
            }
        }
    };

    view! {
        <Title text="Admin Panel - UAB IT Idea Board"/>
        <div class="admin-page">
            <div class="header-banner admin-banner">
                <div class="container">
                    <h1>"Admin Panel"</h1>
                    <p>"UAB IT Idea Board Administration"</p>
                </div>
            </div>

            <div class="container page">
                {move || {
                    if is_authenticated.get() {
                        view! {
                            <AdminDashboard on_logout=handle_logout />
                        }.into_any()
                    } else {
                        view! {
                            <AdminLogin 
                                password=password
                                login_error=login_error
                                on_submit=handle_login
                            />
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn AdminLogin(
    password: RwSignal<String>,
    login_error: RwSignal<Option<String>>,
    on_submit: impl Fn(SubmitEvent) + 'static,
) -> impl IntoView {
    view! {
        <div class="admin-login">
            <h2>"Admin Login"</h2>
            <form on:submit=on_submit>
                <div class="form-group">
                    <label for="admin-password">"Password"</label>
                    <input
                        type="password"
                        id="admin-password"
                        class="admin-input"
                        placeholder="Enter admin password..."
                        prop:value=move || password.get()
                        on:input=move |ev| password.set(event_target_value(&ev))
                    />
                </div>
                {move || login_error.get().map(|err| view! {
                    <p class="error-message">{err}</p>
                })}
                <button type="submit" class="btn-submit">"Login"</button>
            </form>
        </div>
    }
}

#[component]
fn AdminDashboard(on_logout: impl Fn(()) + 'static + Copy) -> impl IntoView {
    let stats = Resource::new(|| (), |_| async { get_statistics().await });
    let ideas = Resource::new(|| (), |_| async { get_all_ideas_admin().await });
    
    let confirm_delete_all = RwSignal::new(false);
    let confirm_delete_old = RwSignal::new(false);

    let handle_delete_idea = move |idea_id: i32| {
        leptos::task::spawn_local(async move {
            if delete_idea(idea_id).await.is_ok() {
                ideas.refetch();
            }
        });
    };

    let handle_delete_all = move |_| {
        if confirm_delete_all.get() {
            leptos::task::spawn_local(async move {
                if delete_all_ideas().await.is_ok() {
                    ideas.refetch();
                    confirm_delete_all.set(false);
                }
            });
        } else {
            confirm_delete_all.set(true);
        }
    };

    let handle_delete_old = move |_| {
        if confirm_delete_old.get() {
            leptos::task::spawn_local(async move {
                if delete_old_ideas(30).await.is_ok() {
                    ideas.refetch();
                    confirm_delete_old.set(false);
                }
            });
        } else {
            confirm_delete_old.set(true);
        }
    };

    view! {
        <div class="admin-dashboard">
            <div class="admin-header">
                <h2>"Dashboard"</h2>
                <button class="btn-logout" on:click=move |_| on_logout(())>"Logout"</button>
            </div>

            // Statistics Cards
            <div class="stats-cards">
                <Suspense fallback=|| view! { <p>"Loading stats..."</p> }>
                    {move || stats.get().map(|s| match s {
                        Ok((total_ideas, total_votes)) => view! {
                            <div class="stat-card">
                                <h3>"Total Ideas"</h3>
                                <span class="stat-number">{total_ideas}</span>
                            </div>
                            <div class="stat-card">
                                <h3>"Total Votes"</h3>
                                <span class="stat-number">{total_votes}</span>
                            </div>
                        }.into_any(),
                        Err(_) => view! { <p>"Failed to load stats"</p> }.into_any()
                    })}
                </Suspense>
            </div>

            // Bulk Actions
            <div class="bulk-actions">
                <h3>"Bulk Actions"</h3>
                <div class="action-buttons">
                    <button 
                        class="btn-danger"
                        class:confirm=move || confirm_delete_all.get()
                        on:click=handle_delete_all
                    >
                        {move || if confirm_delete_all.get() { "Click again to confirm DELETE ALL" } else { "Delete All Ideas" }}
                    </button>
                    <button 
                        class="btn-warning"
                        class:confirm=move || confirm_delete_old.get()
                        on:click=handle_delete_old
                    >
                        {move || if confirm_delete_old.get() { "Click again to confirm" } else { "Delete Ideas > 30 Days" }}
                    </button>
                    <button class="btn-secondary" on:click=move |_| ideas.refetch()>"Refresh"</button>
                </div>
            </div>

            // Ideas List
            <div class="admin-ideas-list">
                <h3>"All Submissions"</h3>
                <Suspense fallback=|| view! { <p>"Loading ideas..."</p> }>
                    {move || ideas.get().map(|i| match i {
                        Ok(ideas_list) => {
                            if ideas_list.is_empty() {
                                view! { <p class="empty-state">"No ideas submitted yet"</p> }.into_any()
                            } else {
                                view! {
                                    <div class="admin-ideas-grid">
                                        <For
                                            each=move || ideas_list.clone()
                                            key=|idea| idea.id
                                            children=move |idea: Idea| {
                                                let idea_id = idea.id;
                                                view! {
                                                    <AdminIdeaCard 
                                                        idea=idea
                                                        on_delete=move || handle_delete_idea(idea_id)
                                                    />
                                                }
                                            }
                                        />
                                    </div>
                                }.into_any()
                            }
                        }
                        Err(_) => view! { <p class="error">"Failed to load ideas"</p> }.into_any()
                    })}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn AdminIdeaCard(
    idea: Idea,
    on_delete: impl Fn() + 'static,
) -> impl IntoView {
    let formatted_date = idea.created_at.format("%Y-%m-%d %H:%M").to_string();

    view! {
        <div class="admin-idea-card">
            <div class="idea-info">
                <p class="idea-text">{idea.content.clone()}</p>
                <div class="idea-meta">
                    <span class="vote-count">{idea.vote_count}" votes"</span>
                    <span class="idea-date">{formatted_date}</span>
                </div>
            </div>
            <button class="btn-delete" on:click=move |_| on_delete()>"Delete"</button>
        </div>
    }
}
