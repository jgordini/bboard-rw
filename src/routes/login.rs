use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;
use leptos_router::hooks::use_query_map;

use crate::auth::{bump_auth_refresh, LoginMessages, LoginSignal};
use crate::routes::paths;

fn cas_error_message(code: &str) -> &'static str {
    match code {
        "missing_ticket" => "CAS login was missing a ticket. Please try again.",
        "validation" => "CAS ticket validation failed. Please try again.",
        "user" => "CAS user provisioning failed. Please contact support.",
        "session" => "CAS session could not be created. Please try again.",
        "config" => "CAS login is not configured correctly. Please contact support.",
        "link_required" => {
            "CAS login matches an existing local account. Please contact support to link accounts."
        }
        _ => "CAS login failed. Please try again.",
    }
}

fn resolve_error_text(action_error: Option<&'static str>, cas_error: Option<&str>) -> &'static str {
    if let Some(message) = action_error {
        return message;
    }

    cas_error.map(cas_error_message).unwrap_or_default()
}

#[component]
pub fn Login() -> impl IntoView {
    let login = LoginSignal::new();
    let result_of_call = login.value();
    let query = use_query_map();

    // When login succeeds, bump auth refresh so nav refetches user and shows Logout
    Effect::new(move |_| {
        if let Some(Ok(LoginMessages::Successful)) = result_of_call.get() {
            bump_auth_refresh();
        }
    });

    let error = move || {
        let action_error = result_of_call.with(|msg| {
            msg.as_ref().map(|inner| match inner {
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
        });

        let cas_error = query.with(|q| q.get("cas_error"));
        resolve_error_text(action_error, cas_error.as_deref())
    };

    view! {
        <Title text="Login"/>
        <div class="auth-page">
            <div class="container page">
                <div class="row">
                    <div class="col-md-6 offset-md-3 col-xs-12">
                        <h1 class="text-xs-center">"Login"</h1>

                        <a href=paths::CAS_LOGIN class="btn btn-lg btn-secondary pull-xs-right">
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
                            <A href=paths::RESET_PASSWORD>Reset password</A>
                            <button class="btn btn-lg btn-primary pull-xs-right">"Sign in"</button>
                        </ActionForm>
                        <p class="text-xs-center">
                            <A href=paths::SIGNUP>"Need an account?"</A>
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{cas_error_message, resolve_error_text};

    #[test]
    fn maps_link_required_cas_error() {
        assert_eq!(
            cas_error_message("link_required"),
            "CAS login matches an existing local account. Please contact support to link accounts."
        );
    }

    #[test]
    fn action_error_takes_precedence_over_cas_query_error() {
        assert_eq!(
            resolve_error_text(Some("Incorrect user or password"), Some("validation"),),
            "Incorrect user or password"
        );
    }
}
