// Simple profile/account page: show current user from session or prompt to log in.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

use crate::auth::{get_user, use_auth_refresh};
use crate::routes::paths;

#[component]
pub fn AccountPage() -> impl IntoView {
    let auth_refresh = use_auth_refresh();
    let user_resource = Resource::new(
        move || auth_refresh.get(),
        move |_| async move { get_user().await },
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
                                            <A href=paths::HOME>"Back to Idea Board"</A>
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
                                            <A href=paths::LOGIN>"Log in"</A>
                                            " · "
                                            <A href=paths::SIGNUP>"Sign up"</A>
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
