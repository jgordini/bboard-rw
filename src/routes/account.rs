// Simple profile/account page: show current user from session or prompt to log in.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

use crate::auth::{get_user};

#[component]
pub fn AccountPage() -> impl IntoView {
    let user_resource = Resource::new(
        || (),
        |_| async move { get_user().await },
    );

    view! {
        <Title text="Profile — UAB IT Idea Board"/>
        <div class="container page">
            <h1 class="text-xs-center">"Profile"</h1>
            <Suspense fallback=move || view! { <p class="text-xs-center">"Loading…"</p> }>
                {move || user_resource.get().map(|result| {
                    match result {
                        Ok(Some(user)) => view! {
                            <div class="auth-page">
                                <div class="row">
                                    <div class="col-md-6 offset-md-3 col-xs-12">
                                        <p class="text-xs-center">"Logged in as " <strong>{user.name.clone()}</strong> " (" {user.email} ")"</p>
                                        <p class="text-xs-center">
                                            <A href="/">"Back to Idea Board"</A>
                                        </p>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),
                        Ok(None) | Err(_) => view! {
                            <div class="auth-page">
                                <div class="row">
                                    <div class="col-md-6 offset-md-3 col-xs-12">
                                        <p class="text-xs-center">"You are not logged in."</p>
                                        <p class="text-xs-center">
                                            <A href="/login">"Log in"</A>
                                            " · "
                                            <A href="/signup">"Sign up"</A>
                                        </p>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}
