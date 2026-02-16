use std::env;

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{hooks::use_query, params::Params};
#[cfg(feature = "ssr")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
struct EmailCredentials {
    email: String,
    passwd: String,
    smtp_server: String,
}

#[cfg(feature = "ssr")]
static EMAIL_CREDS: std::sync::OnceLock<EmailCredentials> = std::sync::OnceLock::new();

#[cfg(feature = "ssr")]
#[derive(Deserialize, Serialize)]
struct ResetTokenClaims {
    sub: String,
    exp: usize,
}

#[cfg(feature = "ssr")]
fn get_email_creds() -> Result<&'static EmailCredentials, ServerFnError> {
    if let Some(creds) = EMAIL_CREDS.get() {
        return Ok(creds);
    }

    let creds = EmailCredentials {
        email: env::var("MAILER_EMAIL")
            .map_err(|e| ServerFnError::new(format!("MAILER_EMAIL is not configured: {e}")))?,
        passwd: env::var("MAILER_PASSWD")
            .map_err(|e| ServerFnError::new(format!("MAILER_PASSWD is not configured: {e}")))?,
        smtp_server: env::var("MAILER_SMTP_SERVER").map_err(|e| {
            ServerFnError::new(format!("MAILER_SMTP_SERVER is not configured: {e}"))
        })?,
    };

    let _ = EMAIL_CREDS.set(creds);
    EMAIL_CREDS
        .get()
        .ok_or_else(|| ServerFnError::new("Failed to initialize email credentials"))
}

#[cfg(feature = "ssr")]
fn reset_token_secret() -> Result<String, ServerFnError> {
    env::var("RESET_TOKEN_SECRET")
        .or_else(|_| env::var("JWT_SECRET"))
        .map_err(|e| {
            ServerFnError::new(format!(
                "Neither RESET_TOKEN_SECRET nor JWT_SECRET is configured: {e}"
            ))
        })
}

#[cfg(feature = "ssr")]
fn encode_reset_token(email: &str) -> Result<String, ServerFnError> {
    use jsonwebtoken::{encode, EncodingKey, Header};

    let secret = reset_token_secret()?;
    let claims = ResetTokenClaims {
        sub: email.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| ServerFnError::new(format!("Failed to create reset token: {e}")))
}

#[cfg(feature = "ssr")]
fn decode_reset_token(token: &str) -> Result<ResetTokenClaims, ServerFnError> {
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

    let secret = reset_token_secret()?;
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    decode::<ResetTokenClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| ServerFnError::new(format!("Invalid reset token: {e}")))
}

#[tracing::instrument]
#[server(ResetPasswordAction1, "/api")]
pub async fn reset_password_1(email: String) -> Result<String, ServerFnError> {
    let exists = match crate::models::User::get_by_email(&email).await {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(error) => {
            tracing::error!(?error, "failed to query user for password reset");
            false
        }
    };
    if exists {
        let creds = match get_email_creds() {
            Ok(creds) => creds,
            Err(error) => {
                tracing::error!(?error, "password reset email credentials not configured");
                return Ok(String::from("Check your email"));
            }
        };
        let host = leptos_axum::extract::<axum_extra::extract::Host>()
            .await
            .map(|h| h.0)
            .unwrap_or_else(|_| "localhost:3000".to_string());
        let schema = if cfg!(debug_assertions) {
            "http"
        } else {
            "https"
        };
        let token = match encode_reset_token(&email) {
            Ok(token) => token,
            Err(error) => {
                tracing::error!(?error, "failed to create reset token");
                return Ok(String::from("Check your email"));
            }
        };
        let uri = format!("{schema}://{host}/reset_password?token={token}");
        let message = mail_send::mail_builder::MessageBuilder::new()
            .from(("Realworld Leptos", creds.email.as_str()))
            .to(vec![("You", email.as_str())])
            .subject("Your password reset from realworld leptos")
            .text_body(format!(
                "You can reset your password accessing the following link: {uri}"
            ));

        let mut client = match mail_send::SmtpClientBuilder::new(creds.smtp_server.as_str(), 587)
            .implicit_tls(false)
            .credentials((creds.email.as_str(), creds.passwd.as_str()))
            .connect()
            .await
        {
            Ok(client) => client,
            Err(error) => {
                tracing::error!(
                    ?error,
                    "failed to connect to smtp server for password reset"
                );
                return Ok(String::from("Check your email"));
            }
        };
        if let Err(error) = client.send(message).await {
            tracing::error!(?error, "failed to send password reset email");
        }
    }
    Ok(String::from("Check your email"))
}

fn validate_reset(password: &str, confirm: &str) -> bool {
    password == confirm
}

fn action_status_message<F>(read_result: F, log_context: &'static str) -> String
where
    F: FnOnce() -> Option<Result<String, ServerFnError>>,
{
    match read_result() {
        Some(Ok(message)) => message,
        Some(Err(error)) => {
            tracing::error!("{log_context}: {error:?}");
            String::from("There was a problem, try again later")
        }
        None => String::new(),
    }
}

#[tracing::instrument]
#[server(ResetPasswordAction2, "/api")]
pub async fn reset_password_2(
    token: String,
    password: String,
    confirm: String,
) -> Result<String, ServerFnError> {
    let message = String::from("Something went wrong, try again later");
    if !validate_reset(&password, &confirm) {
        return Ok(message);
    }
    if password.len() < 8 {
        return Ok(String::from("Password must be at least 8 characters"));
    }

    let Ok(claims) = decode_reset_token(token.as_str()) else {
        tracing::info!("Invalid token provided");
        return Ok(message);
    };
    let email = claims.sub;
    let Ok(Some(_)) = crate::models::User::get_by_email(&email).await else {
        tracing::info!("User does not exist");
        return Ok(message);
    };

    if let Err(error) = crate::models::User::set_password_by_email(&email, password).await {
        tracing::error!(email, ?error, "error while resetting the password");
        return Ok(message);
    }

    Ok(String::from(
        "Password successfully reset, please, proceed to login",
    ))
}

#[derive(Params, PartialEq)]
struct TokenQuery {
    token: Option<String>,
}

#[component]
pub fn ResetPassword() -> impl IntoView {
    let q = use_query::<TokenQuery>();
    view! {
        <Title text="Reset Password"/>
        <div class="auth-page">
            <div class="container page">
                <div class="row">
                    {q.with(|x| {
                        if let Ok(token_query) = x {
                            if let Some(token) = token_query.token.as_ref() {
                                return view! {<ConfirmPassword token={token.clone()}/>}.into_any()
                            }
                        }
                        view! {<AskForEmail/> }.into_any()
                    })}
                </div>
            </div>
        </div>
    }
}

#[component]
fn AskForEmail() -> impl IntoView {
    let reset = ServerAction::<ResetPasswordAction1>::new();
    let result_of_call = reset.value();

    let error = move || {
        action_status_message(
            || result_of_call.get(),
            "Problem while handling reset-password email action",
        )
    };
    view! {
        <div class="col-md-6 offset-md-3 col-xs-12">
            <h1 class="text-xs-center">"Reset password"</h1>

            <p class="text-xs-center">
                {error}
            </p>

            <ActionForm action=reset>
                <fieldset class="form-group">
                    <input name="email" class="form-control form-control-lg" type="email"
                        placeholder="Your Email" />
                </fieldset>
                <button class="btn btn-lg btn-primary pull-xs-right">"Reset Password"</button>
            </ActionForm>
        </div>
    }
}

#[component]
fn ConfirmPassword(token: String) -> impl IntoView {
    let reset = ServerAction::<ResetPasswordAction2>::new();
    let result_of_call = reset.value();

    let error = move || {
        action_status_message(
            || result_of_call.get(),
            "Problem while handling reset-password confirm action",
        )
    };
    view! {
        <div class="col-md-6 offset-md-3 col-xs-12">
            <h1 class="text-xs-center">"Reset password"</h1>

            <p class="text-xs-center">
                {error}
            </p>

            <ActionForm action=reset on:submit=move |ev| {
                let Ok(data) = ResetPasswordAction2::from_event(&ev) else {
                    return ev.prevent_default();
                };
                if !validate_reset(&data.password, &data.confirm) {
                    result_of_call.set(Some(Ok(String::from("Password is not the same"))));
                    ev.prevent_default();
                }
            }>
                <fieldset class="form-group">
                    <input name="password" class="form-control form-control-lg" type="password"
                        placeholder="Your new password" />

                    <input name="confirm" class="form-control form-control-lg" type="password"
                        placeholder="Confirm your password" />

                    <input name="token" type="hidden" value={token} />
                </fieldset>
                <button class="btn btn-lg btn-primary pull-xs-right">"Reset Password"</button>
            </ActionForm>
        </div>
    }
}
