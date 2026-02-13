use std::future::Future;

use leptos::prelude::{Resource, RwSignal, ServerFnError, Set};
use serde::{de::DeserializeOwned, Serialize};

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

pub(crate) fn spawn_server_action_refetch_resource<T, Fut, V>(
    future: Fut,
    resource: Resource<V>,
)
where
    T: 'static,
    Fut: Future<Output = Result<T, ServerFnError>> + 'static,
    V: Send + Sync + Serialize + DeserializeOwned + 'static,
{
    spawn_server_action_refetch(future, move || resource.refetch());
}

pub(crate) fn spawn_server_action_with_error<T, Fut, OnOk>(
    future: Fut,
    on_ok: OnOk,
    error_signal: RwSignal<Option<String>>,
)
where
    T: 'static,
    Fut: Future<Output = Result<T, ServerFnError>> + 'static,
    OnOk: FnOnce(T) + 'static,
{
    spawn_server_action(
        future,
        on_ok,
        move |error| error_signal.set(Some(error.to_string())),
    );
}
