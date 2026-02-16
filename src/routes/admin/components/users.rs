use leptos::prelude::*;
use web_sys::window;

use crate::models::User;
use crate::routes::async_helpers::spawn_server_action;
use crate::routes::view_helpers::confirm_action;

use super::super::{delete_user_action, role_name, update_user_role_action, get_all_users_admin};

fn show_admin_error(error: ServerFnError) {
    if let Some(w) = window() {
        let _ = w.alert_with_message(&error.to_string());
    }
}

#[component]
pub(super) fn UsersTab() -> impl IntoView {
    let users = Resource::new(|| (), |_| async { get_all_users_admin().await });

    let handle_role_change = move |user_id: i32, new_role: i16| {
        spawn_server_action(
            update_user_role_action(user_id, new_role),
            move |_| users.refetch(),
            show_admin_error,
        );
    };

    let handle_delete = move |user_id: i32| {
        spawn_server_action(
            delete_user_action(user_id),
            move |_| users.refetch(),
            show_admin_error,
        );
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
                                            let is_admin = user.role >= 2;
                                            view! {
                                                <tr>
                                                    <td>{user.id}</td>
                                                    <td>{user.name}</td>
                                                    <td>{user.email}</td>
                                                    <td>
                                                        {move || {
                                                            if is_admin {
                                                                view! { <span>{role_name(current_role)}</span> }.into_any()
                                                            } else {
                                                                view! {
                                                                    <label for=format!("user-role-{}", user_id) class="sr-only">"Role"</label>
                                                                    <select
                                                                        id=format!("user-role-{}", user_id)
                                                                        on:change=move |ev| {
                                                                            let val = event_target_value(&ev);
                                                                            if let Ok(role) = val.parse::<i16>() {
                                                                                handle_role_change(user_id, role);
                                                                            }
                                                                        }
                                                                        prop:value=move || current_role.to_string()
                                                                    >
                                                                        <option value="0">"User"</option>
                                                                        <option value="1">"Moderator"</option>
                                                                    </select>
                                                                }.into_any()
                                                            }
                                                        }}
                                                    </td>
                                                    <td>
                                                        {move || {
                                                            if is_admin {
                                                                view! { <span aria-hidden="true">"—"</span> }.into_any()
                                                            } else {
                                                                view! {
                                                                    <button
                                                                        type="button"
                                                                        class="btn-danger"
                                                                        on:click=move |_| {
                                                                            if confirm_action("Permanently delete this user? This cannot be undone.") {
                                                                                handle_delete(user_id);
                                                                            }
                                                                        }
                                                                    >"Delete"</button>
                                                                }.into_any()
                                                            }
                                                        }}
                                                    </td>
                                                </tr>
                                            }
                                        }
                                    />
                                </tbody>
                            </table>
                        }.into_any()
                    }
                    Err(_) => view! { <p class="error">"Failed to load users"</p> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
