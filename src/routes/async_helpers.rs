use std::future::Future;

use leptos::prelude::ServerFnError;

pub(crate) fn spawn_server_action<T, Fut, OnOk, OnErr>(
    future: Fut,
    on_ok: OnOk,
    on_err: OnErr,
)
where
    T: 'static,
    Fut: Future<Output = Result<T, ServerFnError>> + 'static,
    OnOk: FnOnce(T) + 'static,
    OnErr: FnOnce(ServerFnError) + 'static,
{
    leptos::task::spawn_local(async move {
        match future.await {
            Ok(value) => on_ok(value),
            Err(err) => on_err(err),
        }
    });
}

pub(crate) fn spawn_server_action_ok<T, Fut, OnOk>(future: Fut, on_ok: OnOk)
where
    T: 'static,
    Fut: Future<Output = Result<T, ServerFnError>> + 'static,
    OnOk: FnOnce(T) + 'static,
{
    spawn_server_action(future, on_ok, |_| {});
}

pub(crate) fn spawn_server_action_refetch<T, Fut, OnRefetch>(future: Fut, on_refetch: OnRefetch)
where
    T: 'static,
    Fut: Future<Output = Result<T, ServerFnError>> + 'static,
    OnRefetch: FnOnce() + 'static,
{
    spawn_server_action_ok(future, move |_| on_refetch());
}
