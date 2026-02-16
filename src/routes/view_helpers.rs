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
    user_session_state_is_logged_in(user_resource.get())
}

pub fn confirm_action(message: &str) -> bool {
    web_sys::window()
        .and_then(|window| window.confirm_with_message(message).ok())
        .unwrap_or(false)
}

fn user_session_state_is_logged_in(
    state: Option<Result<Option<UserSession>, ServerFnError>>,
) -> bool {
    matches!(state, Some(Ok(Some(_))))
}

#[cfg(test)]
mod tests {
    use super::user_session_state_is_logged_in;
    use crate::auth::UserSession;
    use leptos::prelude::ServerFnError;

    #[test]
    fn logged_in_state_is_true_only_for_present_user_session() {
        assert!(!user_session_state_is_logged_in(None));
        assert!(!user_session_state_is_logged_in(Some(Err(
            ServerFnError::new("network failure")
        ))));
        assert!(!user_session_state_is_logged_in(Some(Ok(None))));
        assert!(user_session_state_is_logged_in(Some(Ok(Some(
            UserSession {
                id: 7,
                email: "user@example.com".to_string(),
                name: "user".to_string(),
                role: 0,
            }
        )))));
    }
}
