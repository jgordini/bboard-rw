#![cfg_attr(not(feature = "ssr"), allow(dead_code))]

use leptos::prelude::ServerFnError;

#[allow(dead_code)]
pub(crate) fn server_fn_error_with_log<E: std::fmt::Debug>(
    context: &str,
    error: E,
    user_message: &str,
) -> ServerFnError {
    tracing::error!("{context}: {error:?}");
    ServerFnError::new(user_message)
}

#[allow(dead_code)]
pub(crate) fn server_fn_server_error_with_log<E: std::fmt::Debug>(
    context: &str,
    error: E,
    user_message: &str,
) -> ServerFnError {
    tracing::error!("{context}: {error:?}");
    ServerFnError::ServerError(user_message.to_string())
}

#[cfg(test)]
mod tests {
    use super::{server_fn_error_with_log, server_fn_server_error_with_log};
    use leptos::prelude::ServerFnError;

    fn extract_server_error_message(error: ServerFnError) -> String {
        match error {
            ServerFnError::ServerError(message) => message,
            _ => panic!("expected ServerFnError::ServerError"),
        }
    }

    #[test]
    fn server_fn_error_with_log_returns_server_error_with_message() {
        let error = server_fn_error_with_log(
            "problem while fetching home articles",
            "db timeout",
            "Problem while fetching home articles",
        );
        assert_eq!(
            extract_server_error_message(error),
            "Problem while fetching home articles",
        );
    }

    #[test]
    fn server_fn_server_error_with_log_returns_server_error_with_message() {
        let error = server_fn_server_error_with_log(
            "problem while fetching tags",
            "db timeout",
            "Problem while fetching tags",
        );
        assert_eq!(
            extract_server_error_message(error),
            "Problem while fetching tags",
        );
    }

    #[test]
    fn route_style_map_err_preserves_client_facing_message() {
        let mapped = Result::<(), &str>::Err("db down")
            .map_err(|e| {
                server_fn_error_with_log("Failed to fetch ideas", e, "Failed to fetch ideas")
            })
            .expect_err("mapping should preserve the error");

        assert_eq!(
            extract_server_error_message(mapped),
            "Failed to fetch ideas"
        );
    }
}
