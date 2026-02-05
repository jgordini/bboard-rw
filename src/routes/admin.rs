use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;
use web_sys::window;
use crate::auth::{get_user, UserSession};
use crate::models::{IdeaWithAuthor, User};
#[cfg(feature = "ssr")]
use crate::models::{Idea, Flag};

// ============================================================================
// SERVER FUNCTIONS
// ============================================================================

#[server]
pub async fn get_admin_stats() -> Result<AdminStats, ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    let (total_ideas, total_votes) = Idea::get_statistics()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch statistics: {}", e)))?;

    let total_users = User::get_all()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to count users: {}", e)))?
        .len() as i64;

    let flagged_items = Flag::get_flagged_items()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get flagged items: {}", e)))?
        .len() as i64;

    Ok(AdminStats {
        total_ideas,
        total_votes,
        total_users,
        flagged_items,
    })
}

#[server]
pub async fn get_flagged_content() -> Result<Vec<FlaggedItemDetail>, ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    let flagged_items = Flag::get_flagged_items()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get flagged items: {}", e)))?;

    let mut details = Vec::new();
    for item in flagged_items {
        let content = if item.target_type == "idea" {
            Idea::get_with_author(item.target_id)
                .await
                .map_err(|e| ServerFnError::new(format!("Failed to get idea: {}", e)))?
                .map(|iwa| format!("{}: {}", iwa.idea.title, iwa.idea.content))
        } else {
            // For comments, just get the content
            Some(format!("Comment ID: {}", item.target_id))
        };

        if let Some(content_text) = content {
            details.push(FlaggedItemDetail {
                target_type: item.target_type,
                target_id: item.target_id,
                flag_count: item.flag_count,
                content_preview: content_text.chars().take(200).collect(),
            });
        }
    }

    Ok(details)
}

#[server]
pub async fn clear_flags_action(target_type: String, target_id: i32) -> Result<(), ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    Flag::clear_flags(&target_type, target_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to clear flags: {}", e)))?;

    Ok(())
}

#[server]
pub async fn mark_idea_off_topic_action(idea_id: i32, is_off_topic: bool) -> Result<(), ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    Idea::mark_off_topic(idea_id, is_off_topic)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to mark as off-topic: {}", e)))?;

    // Clear flags when marking off-topic
    if is_off_topic {
        Flag::clear_flags("idea", idea_id)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to clear flags: {}", e)))?;
    }

    Ok(())
}

#[server]
pub async fn delete_idea_action(idea_id: i32) -> Result<(), ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    Idea::delete(idea_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to delete idea: {}", e)))?;

    Ok(())
}

#[server]
pub async fn update_idea_stage_action(idea_id: i32, stage: String) -> Result<(), ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    Idea::update_stage(idea_id, stage)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update stage: {}", e)))?;

    Ok(())
}

#[server]
pub async fn toggle_idea_pin_action(idea_id: i32) -> Result<bool, ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    Idea::toggle_pin(idea_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to toggle pin: {}", e)))
}

#[server]
pub async fn get_off_topic_ideas() -> Result<Vec<IdeaWithAuthor>, ServerFnError> {
    use crate::auth::require_moderator;
    require_moderator().await?;

    Idea::get_off_topic()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get off-topic ideas: {}", e)))
}

#[server]
pub async fn get_all_users_admin() -> Result<Vec<User>, ServerFnError> {
    use crate::auth::require_admin;
    require_admin().await?;

    User::get_all()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get users: {}", e)))
}

#[server]
pub async fn update_user_role_action(user_id: i32, role: i16) -> Result<(), ServerFnError> {
    use crate::auth::require_admin;
    require_admin().await?;

    User::update_role(user_id, role)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update role: {}", e)))
}

// ============================================================================
// TYPES
// ============================================================================

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AdminStats {
    pub total_ideas: i64,
    pub total_votes: i64,
    pub total_users: i64,
    pub flagged_items: i64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FlaggedItemDetail {
    pub target_type: String,
    pub target_id: i32,
    pub flag_count: i64,
    pub content_preview: String,
}

// ============================================================================
// COMPONENTS
// ============================================================================

#[component]
pub fn AdminPage() -> impl IntoView {
    let user_resource = Resource::new(|| (), |_| async { get_user().await });

    view! {
        <Title text="Admin Dashboard - UAB IT Idea Board"/>
        <Suspense fallback=|| view! { <p>"Loading…"</p> }>
            {move || user_resource.get().map(|user_result: Result<Option<UserSession>, ServerFnError>| {
                match user_result {
                    Ok(Some(user)) if user.is_moderator() => {
                        view! { <AdminDashboard user=user /> }.into_any()
                    }
                    Ok(Some(_)) => {
                        view! {
                            <div class="container page">
                                <p class="error">"Access denied. Moderator privileges required."</p>
                                <A href="/">"Back to Home"</A>
                            </div>
                        }.into_any()
                    }
                    Ok(None) => {
                        view! {
                            <div class="container page">
                                <p class="error">"Please log in to access the admin dashboard."</p>
                                <A href="/login">"Go to Login"</A>
                            </div>
                        }.into_any()
                    }
                    Err(_) => {
                        view! {
                            <div class="container page">
                                <p class="error">"Failed to load user session."</p>
                            </div>
                        }.into_any()
                    }
                }
            })}
        </Suspense>
    }
}

#[component]
fn AdminDashboard(user: UserSession) -> impl IntoView {
    let stats = Resource::new(|| (), |_| async { get_admin_stats().await });
    let active_tab = RwSignal::new("overview");

    let user_for_tab_button = user.clone();
    let user_for_content = user.clone();

    view! {
        <div class="admin-page">
            <div class="admin-header">
                <h1>"Admin Dashboard"</h1>
                <p>"Logged in as: " {user.name.clone()} " (" {role_name(user.role)} ")"</p>
            </div>

            <div class="admin-tabs">
                <button
                    class:active=move || active_tab.get() == "overview"
                    on:click=move |_| active_tab.set("overview")
                >"Overview"</button>
                <button
                    class:active=move || active_tab.get() == "flags"
                    on:click=move |_| active_tab.set("flags")
                >"Flagged Content"</button>
                <button
                    class:active=move || active_tab.get() == "moderation"
                    on:click=move |_| active_tab.set("moderation")
                >"Off-Topic Items"</button>
                {move || {
                    if user_for_tab_button.is_admin() {
                        view! {
                            <button
                                class:active=move || active_tab.get() == "users"
                                on:click=move |_| active_tab.set("users")
                            >"User Management"</button>
                        }.into_any()
                    } else {
                        view! {}.into_any()
                    }
                }}
            </div>

            <div class="admin-content">
                {move || match active_tab.get() {
                    "overview" => view! { <OverviewTab stats=stats /> }.into_any(),
                    "flags" => view! { <FlagsTab /> }.into_any(),
                    "moderation" => view! { <ModerationTab /> }.into_any(),
                    "users" if user_for_content.is_admin() => view! { <UsersTab /> }.into_any(),
                    _ => view! { <p>"Unknown tab"</p> }.into_any(),
                }}
            </div>
        </div>
    }
}

#[component]
fn OverviewTab(stats: Resource<Result<AdminStats, ServerFnError>>) -> impl IntoView {
    view! {
        <div class="overview-tab">
            <h2>"Statistics"</h2>
            <Suspense fallback=|| view! { <p>"Loading stats…"</p> }>
                {move || stats.get().map(|s| match s {
                    Ok(stats) => view! {
                        <div class="stats-grid">
                            <div class="stat-card">
                                <h3>"Total Ideas"</h3>
                                <span class="stat-number">{stats.total_ideas}</span>
                            </div>
                            <div class="stat-card">
                                <h3>"Total Votes"</h3>
                                <span class="stat-number">{stats.total_votes}</span>
                            </div>
                            <div class="stat-card">
                                <h3>"Total Users"</h3>
                                <span class="stat-number">{stats.total_users}</span>
                            </div>
                            <div class="stat-card">
                                <h3>"Flagged Items"</h3>
                                <span class="stat-number">{stats.flagged_items}</span>
                            </div>
                        </div>
                    }.into_any(),
                    Err(_) => view! { <p class="error">"Failed to load statistics"</p> }.into_any()
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn FlagsTab() -> impl IntoView {
    let flagged_items = Resource::new(|| (), |_| async { get_flagged_content().await });

    let handle_clear_flags = move |target_type: String, target_id: i32| {
        leptos::task::spawn_local(async move {
            if clear_flags_action(target_type, target_id).await.is_ok() {
                flagged_items.refetch();
            }
        });
    };

    let handle_mark_off_topic = move |idea_id: i32| {
        leptos::task::spawn_local(async move {
            if mark_idea_off_topic_action(idea_id, true).await.is_ok() {
                flagged_items.refetch();
            }
        });
    };

    let handle_delete = move |target_type: String, target_id: i32| {
        leptos::task::spawn_local(async move {
            if target_type == "idea" {
                if delete_idea_action(target_id).await.is_ok() {
                    flagged_items.refetch();
                }
            }
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
                                                            }.into_any()
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
                        }.into_any()
                    }
                    Err(_) => view! { <p class="error">"Failed to load flagged content"</p> }.into_any()
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn ModerationTab() -> impl IntoView {
    let off_topic_ideas = Resource::new(|| (), |_| async { get_off_topic_ideas().await });

    let handle_restore = move |idea_id: i32| {
        leptos::task::spawn_local(async move {
            if mark_idea_off_topic_action(idea_id, false).await.is_ok() {
                off_topic_ideas.refetch();
            }
        });
    };

    let handle_delete = move |idea_id: i32| {
        leptos::task::spawn_local(async move {
            if delete_idea_action(idea_id).await.is_ok() {
                off_topic_ideas.refetch();
            }
        });
    };

    view! {
        <div class="moderation-tab">
            <h2>"Off-Topic Ideas"</h2>
            <Suspense fallback=|| view! { <p>"Loading off-topic ideas…"</p> }>
                {move || off_topic_ideas.get().map(|ideas| match ideas {
                    Ok(ideas_list) if ideas_list.is_empty() => {
                        view! { <p class="empty-state">"No off-topic ideas"</p> }.into_any()
                    }
                    Ok(ideas_list) => {
                        view! {
                            <div class="off-topic-list">
                                <For
                                    each=move || ideas_list.clone()
                                    key=|iwa| iwa.idea.id
                                    children=move |iwa: IdeaWithAuthor| {
                                        let idea_id = iwa.idea.id;
                                        view! {
                                            <div class="off-topic-item">
                                                <div class="idea-content">
                                                    <h3>{iwa.idea.title.clone()}</h3>
                                                    <p>{iwa.idea.content.clone()}</p>
                                                    <span class="author">"By: " {iwa.author_name}</span>
                                                </div>
                                                <div class="moderation-actions">
                                                    <button
                                                        class="btn-primary"
                                                        on:click=move |_| handle_restore(idea_id)
                                                    >"Restore"</button>
                                                    <button
                                                        class="btn-danger"
                                                        on:click=move |_| {
                                                            if let Some(w) = window() {
                                                                if w.confirm_with_message("Permanently delete this idea? This cannot be undone.").unwrap_or(false) {
                                                                    handle_delete(idea_id);
                                                                }
                                                            }
                                                        }
                                                    >"Delete Permanently"</button>
                                                </div>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        }.into_any()
                    }
                    Err(_) => view! { <p class="error">"Failed to load off-topic ideas"</p> }.into_any()
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn UsersTab() -> impl IntoView {
    let users = Resource::new(|| (), |_| async { get_all_users_admin().await });

    let handle_role_change = move |user_id: i32, new_role: i16| {
        leptos::task::spawn_local(async move {
            if update_user_role_action(user_id, new_role).await.is_ok() {
                users.refetch();
            }
        });
    };

    view! {
        <div class="users-tab">
            <h2>"User Management"</h2>
            <Suspense fallback=|| view! { <p>"Loading users…"</p> }>
                {move || users.get().map(|users_result| match users_result {
                    Ok(users_list) => {
                        view! {
                            <table class="users-table">
                                <thead>
                                    <tr>
                                        <th>"ID"</th>
                                        <th>"Name"</th>
                                        <th>"Email"</th>
                                        <th>"Role"</th>
                                        <th>"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <For
                                        each=move || users_list.clone()
                                        key=|user| user.id
                                        children=move |user: User| {
                                            let user_id = user.id;
                                            let current_role = user.role;
                                            view! {
                                                <tr>
                                                    <td>{user.id}</td>
                                                    <td>{user.name}</td>
                                                    <td>{user.email}</td>
                                                    <td>{role_name(user.role)}</td>
                                                    <td>
                                                        <select
                                                            on:change=move |ev| {
                                                                let new_role = event_target_value(&ev).parse::<i16>().unwrap_or(0);
                                                                handle_role_change(user_id, new_role);
                                                            }
                                                            prop:value=move || current_role.to_string()
                                                        >
                                                            <option value="0">"User"</option>
                                                            <option value="1">"Moderator"</option>
                                                            <option value="2">"Admin"</option>
                                                        </select>
                                                    </td>
                                                </tr>
                                            }
                                        }
                                    />
                                </tbody>
                            </table>
                        }.into_any()
                    }
                    Err(_) => view! { <p class="error">"Failed to load users"</p> }.into_any()
                })}
            </Suspense>
        </div>
    }
}

// ============================================================================
// HELPERS
// ============================================================================

fn role_name(role: i16) -> &'static str {
    match role {
        2 => "Admin",
        1 => "Moderator",
        _ => "User",
    }
}
