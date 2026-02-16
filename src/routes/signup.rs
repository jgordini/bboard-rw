use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

use crate::auth::{AuthRefresh, validate_signup, SignupAction, SignupResponse, SignupSignal};

#[component]
pub fn Signup(signup: SignupSignal) -> impl IntoView {
    let result_of_call = signup.value();
    let auth_refresh = expect_context::<AuthRefresh>().0;

    // When signup succeeds, bump auth refresh so nav updates without reload
    Effect::new(move |_| {
        if let Some(Ok(SignupResponse::Success)) = result_of_call.get() {
            auth_refresh.update(|v| *v += 1);
        }
    });

    let error_cb = move || {
        result_of_call
            .get()
            .map(|msg| match msg {
                Ok(SignupResponse::ValidationError(x)) => format!("Problem while validating: {x}"),
                Ok(SignupResponse::CreateUserError(x)) => {
                    format!("Problem while creating user: {x}")
                }
                Ok(SignupResponse::Success) => {
                    tracing::info!("Signup success! redirecting");
                    "Done".into()
                }
                Err(x) => {
                    tracing::error!("Problem during signup: {x:?}");
                    "There was a problem, try again later".into()
                }
            })
            .unwrap_or_default()
    };

    view! {
        <Title text="Signup"/>
        <div class="auth-page">
            <div class="container page">
                <div class="row">
                    <div class="col-md-6 offset-md-3 col-xs-12">
                        <h1 class="text-xs-center">"Sign up"</h1>
                        <p class="text-xs-center">
                            <A href="/login">"Have an account?"</A>
                        </p>

                        <p class="error-messages text-xs-center" aria-live="polite" aria-atomic="true">
                            {error_cb}
                        </p>

                        <ActionForm action=signup on:submit=move |ev| {
                            let Ok(data) = SignupAction::from_event(&ev) else {
                                return ev.prevent_default();
                            };
                            if let Err(x) = validate_signup(data.name.clone(), data.email.clone(), data.password.clone()) {
                                result_of_call.set(Some(Ok(SignupResponse::ValidationError(x))));
                                ev.prevent_default();
                            }
                        }>
                            <fieldset class="form-group">
                                <label for="signup-name" class="sr-only">"Your Name"</label>
                                <input id="signup-name" name="name" class="form-control form-control-lg" type="text" placeholder="Your name…" required=true autocomplete="name"/>
                            </fieldset>
                            <fieldset class="form-group">
                                <label for="signup-email" class="sr-only">"Email"</label>
                                <input id="signup-email" name="email" class="form-control form-control-lg" type="email" placeholder="e.g. you@uab.edu…" required=true autocomplete="email"/>
                            </fieldset>
                            <fieldset class="form-group">
                                <label for="signup-password" class="sr-only">"Password"</label>
                                <input id="signup-password" name="password" class="form-control form-control-lg" type="password" placeholder="At least 8 characters…" required=true autocomplete="new-password"/>
                            </fieldset>
                            <button class="btn btn-lg btn-primary pull-xs-right">"Sign up"</button>
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}
