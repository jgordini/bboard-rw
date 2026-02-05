use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

use crate::auth::{LoginMessages, LoginSignal};

#[component]
pub fn Login(login: LoginSignal) -> impl IntoView {
    let result_of_call = login.value();

    let error = move || {
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
