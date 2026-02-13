use leptos::prelude::{Get, Resource, ServerFnError};

use crate::auth::UserSession;

pub fn format_relative_time(dt: &chrono::DateTime<chrono::Utc>) -> String {
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

pub fn stage_badge_color(stage: &str) -> &'static str {
    match stage {
        "Ideate" => "ideate",
        "Review" => "review",
        "In Progress" => "progress",
        "Completed" => "completed",
        _ => "default",
    }
}

pub fn is_user_logged_in(
    user_resource: &Resource<Result<Option<UserSession>, ServerFnError>>,
) -> bool {
    matches!(user_resource.get(), Some(Ok(Some(_))))
}

pub fn confirm_action(message: &str) -> bool {
    web_sys::window()
        .and_then(|window| window.confirm_with_message(message).ok())
        .unwrap_or(false)
}
