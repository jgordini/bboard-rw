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
