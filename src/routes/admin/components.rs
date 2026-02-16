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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResolvedTab {
    Overview,
    Flags,
    Moderation,
    Export,
    Users,
    Unknown,
}

fn show_admin_management_tabs(user: &UserSession) -> bool {
    user.is_admin()
}

fn resolve_active_tab(active_tab: &str, is_admin: bool) -> ResolvedTab {
    match active_tab {
        "overview" => ResolvedTab::Overview,
        "flags" => ResolvedTab::Flags,
        "moderation" => ResolvedTab::Moderation,
        "export" if is_admin => ResolvedTab::Export,
        "users" if is_admin => ResolvedTab::Users,
        _ => ResolvedTab::Unknown,
    }
}

#[component]
pub(super) fn AdminDashboard(user: UserSession) -> impl IntoView {
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
                    if show_admin_management_tabs(&user_for_tab_button) {
                        view! {
                            <button
                                class:active=move || active_tab.get() == "export"
                                on:click=move |_| active_tab.set("export")
                            >"Data Export"</button>
                            <button
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
                {move || match resolve_active_tab(active_tab.get(), user_for_content.is_admin()) {
                    ResolvedTab::Overview => view! { <OverviewTab stats=stats /> }.into_any(),
                    ResolvedTab::Flags => view! { <FlagsTab /> }.into_any(),
                    ResolvedTab::Moderation => view! { <ModerationTab /> }.into_any(),
                    ResolvedTab::Export => view! { <ExportTab /> }.into_any(),
                    ResolvedTab::Users => view! { <UsersTab /> }.into_any(),
                    ResolvedTab::Unknown => view! { <p>"Unknown tab"</p> }.into_any(),
                }}
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{ResolvedTab, resolve_active_tab, show_admin_management_tabs};
    use crate::auth::UserSession;

    fn user_with_role(role: i16) -> UserSession {
        UserSession {
            id: 1,
            email: "test@uab.edu".to_string(),
            name: "Test User".to_string(),
            role,
        }
    }

    #[test]
    fn admin_users_can_see_and_access_admin_tabs() {
        let admin = user_with_role(2);

        assert!(show_admin_management_tabs(&admin));
        assert_eq!(
            resolve_active_tab("export", admin.is_admin()),
            ResolvedTab::Export
        );
        assert_eq!(
            resolve_active_tab("users", admin.is_admin()),
            ResolvedTab::Users
        );
    }

    #[test]
    fn non_admin_users_cannot_access_admin_tabs() {
        let moderator = user_with_role(1);
        let regular_user = user_with_role(0);

        assert!(!show_admin_management_tabs(&moderator));
        assert!(!show_admin_management_tabs(&regular_user));
        assert_eq!(
            resolve_active_tab("export", moderator.is_admin()),
            ResolvedTab::Unknown
        );
        assert_eq!(
            resolve_active_tab("users", regular_user.is_admin()),
            ResolvedTab::Unknown
        );
    }

    #[test]
    fn shared_tabs_are_accessible_for_all_roles() {
        let non_admin = user_with_role(0);

        assert_eq!(
            resolve_active_tab("overview", non_admin.is_admin()),
            ResolvedTab::Overview
        );
        assert_eq!(
            resolve_active_tab("flags", non_admin.is_admin()),
            ResolvedTab::Flags
        );
        assert_eq!(
            resolve_active_tab("moderation", non_admin.is_admin()),
            ResolvedTab::Moderation
        );
    }
}
