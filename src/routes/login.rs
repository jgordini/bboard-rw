use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;
use leptos_router::hooks::use_query_map;

use crate::auth::{AuthRefresh, LoginMessages, LoginSignal};

#[component]
pub fn Login(login: LoginSignal) -> impl IntoView {
    let result_of_call = login.value();
    let query = use_query_map();
    let auth_refresh = expect_context::<AuthRefresh>().0;

    // When login succeeds, bump auth refresh so nav refetches user and shows Logout
    Effect::new(move |_| {
        if let Some(Ok(LoginMessages::Successful)) = result_of_call.get() {
            auth_refresh.update(|v| *v += 1);
        }
    });

    let error = move || {
        let cas_error = query.with(|q| q.get("cas_error"));

        if let Some(code) = cas_error {
            return match code.as_str() {
                "missing_ticket" => "CAS login was missing a ticket. Please try again.",
                "validation" => "CAS ticket validation failed. Please try again.",
                "user" => "CAS user provisioning failed. Please contact support.",
                "session" => "CAS session could not be created. Please try again.",
                "config" => "CAS login is not configured correctly. Please contact support.",
                _ => "CAS login failed. Please try again.",
            };
        }

        result_of_call.with(|msg| {
            msg.as_ref()
                .map(|inner| match inner {
                    Ok(LoginMessages::Unsuccessful) => "Incorrect user or password",
                    Ok(LoginMessages::Successful) => {
                        tracing::info!("login success!");
                        "Done"
                    }
                    Err(x) => {
                        tracing::error!("Problem during login: {x:?}");
                        "There was a problem, try again later"
                    }
                })
                .unwrap_or_default()
        })
    };

    view! {
        <Title text="Login"/>
        <div class="auth-page">
            <div class="container page">
                <div class="row">
                    <div class="col-md-6 offset-md-3 col-xs-12">
                        <h1 class="text-xs-center">"Login"</h1>

                        <a href="/auth/cas/login" class="btn btn-lg btn-secondary pull-xs-right">
                            "Login with BlazerID"
                        </a>

                        <div class="error-messages text-xs-center" aria-live="polite" aria-atomic="true">
                            {error}
                        </div>

                        <ActionForm action=login>
                            <fieldset class="form-group">
                                <label for="login-email" class="sr-only">"Email"</label>
                                <input id="login-email" name="email" class="form-control form-control-lg" type="email"
                                    placeholder="e.g. you@uab.edu…" autocomplete="email" />
                            </fieldset>
                            <fieldset class="form-group">
                                <label for="login-password" class="sr-only">"Password"</label>
                                <input id="login-password" name="password" class="form-control form-control-lg" type="password"
                                    placeholder="Password…" autocomplete="current-password" />
                            </fieldset>
                            <A href="/reset_password">Reset password</A>
                            <button class="btn btn-lg btn-primary pull-xs-right">"Sign in"</button>
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}
