use leptos::prelude::*;

use crate::auth::UserSession;

use super::{get_admin_stats, role_name};

mod export;
mod flags;
mod moderation;
mod overview;
mod users;

use export::ExportTab;
use flags::FlagsTab;
use moderation::ModerationTab;
use overview::OverviewTab;
use users::UsersTab;

#[component]
pub(super) fn AdminDashboard(user: UserSession) -> impl IntoView {
    let stats = Resource::new(|| (), |_| async { get_admin_stats().await });
    let active_tab = RwSignal::new("overview");

    let user_for_tab_button = user.clone();
    let user_for_content = user.clone();

    view! {
        <div class="admin-page admin-page-linear">
            <div class="admin-header admin-linear-header">
                <span class="hero-eyebrow">"Moderator Console"</span>
                <h1>"Admin Dashboard"</h1>
                <p>"Logged in as: " {user.name.clone()} " (" {role_name(user.role)} ")"</p>
            </div>

            <div class="admin-tabs">
                <button
                    class="btn btn-secondary admin-tab-btn"
                    class:active=move || active_tab.get() == "overview"
                    on:click=move |_| active_tab.set("overview")
                >"Overview"</button>
                <button
                    class="btn btn-secondary admin-tab-btn"
                    class:active=move || active_tab.get() == "flags"
                    on:click=move |_| active_tab.set("flags")
                >"Flagged Content"</button>
                <button
                    class="btn btn-secondary admin-tab-btn"
                    class:active=move || active_tab.get() == "moderation"
                    on:click=move |_| active_tab.set("moderation")
                >"Off-Topic Items"</button>
                {move || {
                    if user_for_tab_button.is_admin() {
                        view! {
                            <button
                                class="btn btn-secondary admin-tab-btn"
                                class:active=move || active_tab.get() == "export"
                                on:click=move |_| active_tab.set("export")
                            >"Data Export"</button>
                            <button
                                class="btn btn-secondary admin-tab-btn"
                                class:active=move || active_tab.get() == "users"
                                on:click=move |_| active_tab.set("users")
                            >"User Management"</button>
                        }
                            .into_any()
                    } else {
                        ().into_any()
                    }
                }}
            </div>

            <div class="admin-content">
                {move || match active_tab.get() {
                    "overview" => view! { <OverviewTab stats=stats /> }.into_any(),
                    "flags" => view! { <FlagsTab /> }.into_any(),
                    "moderation" => view! { <ModerationTab /> }.into_any(),
                    "export" if user_for_content.is_admin() => view! { <ExportTab /> }.into_any(),
                    "users" if user_for_content.is_admin() => view! { <UsersTab /> }.into_any(),
                    _ => view! { <p>"Unknown tab"</p> }.into_any(),
                }}
            </div>
        </div>
    }
}
