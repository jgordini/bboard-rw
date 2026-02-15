use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use axum::{
    extract::Query,
    http::{HeaderValue, header::SET_COOKIE},
    response::{IntoResponse, Redirect, Response},
};

#[cfg(feature = "ssr")]
use axum_extra::extract::cookie::{Cookie, SameSite};

#[cfg(feature = "ssr")]
use regex::Regex;

/// When this signal is updated, the nav's user resource refetches (e.g. after login).
#[derive(Clone, Copy)]
pub struct AuthRefresh(pub RwSignal<u32>);

impl AuthRefresh {
    pub fn new() -> Self {
        Self(RwSignal::new(0))
    }

    pub fn signal(self) -> RwSignal<u32> {
        self.0
    }

    pub fn bump(self) {
        self.0.update(|value| *value += 1);
    }
}

pub fn provide_auth_refresh_context() {
    provide_context(AuthRefresh::new());
}

pub fn use_auth_refresh() -> RwSignal<u32> {
    expect_context::<AuthRefresh>().signal()
}

pub fn bump_auth_refresh() {
    expect_context::<AuthRefresh>().bump();
}

// Session user info
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserSession {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub role: i16,
}

impl UserSession {
    pub fn is_moderator(&self) -> bool {
        self.role >= 1
    }

    pub fn is_admin(&self) -> bool {
        self.role >= 2
    }
}

#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
pub struct CasCallbackQuery {
    pub ticket: Option<String>,
}

#[cfg(feature = "ssr")]
#[derive(Debug)]
struct CasUserInfo {
    username: String,
    email: String,
    display_name: String,
}

#[cfg(feature = "ssr")]
fn cas_login_url() -> String {
    std::env::var("CAS_LOGIN_URL")
        .unwrap_or_else(|_| "https://padlock.idm.uab.edu/cas/login".to_string())
}

#[cfg(feature = "ssr")]
fn cas_validate_url() -> String {
    std::env::var("CAS_VALIDATE_URL")
        .unwrap_or_else(|_| "https://padlock.idm.uab.edu/cas/serviceValidate".to_string())
}

#[cfg(feature = "ssr")]
fn cas_service_id() -> String {
    std::env::var("CAS_SERVICE_ID")
        .unwrap_or_else(|_| "http://localhost:3000/auth/cas/callback".to_string())
}

#[cfg(feature = "ssr")]
fn build_session_cookie_header(session: &UserSession) -> Result<HeaderValue, ServerFnError> {
    let session_json = serde_json::to_string(session)
        .map_err(|e| ServerFnError::new(format!("Session serialization error: {e}")))?;

    let cookie = Cookie::build(("user_session", session_json))
        .path("/")
        .same_site(SameSite::Lax)
        .http_only(true)
        .max_age(time::Duration::days(7));

    HeaderValue::from_str(&cookie.to_string())
        .map_err(|e| ServerFnError::new(format!("Cookie error: {e}")))
}

#[cfg(feature = "ssr")]
fn build_clear_session_cookie_header() -> Result<HeaderValue, ServerFnError> {
    let cookie = Cookie::build(("user_session", ""))
        .path("/")
        .max_age(time::Duration::seconds(0));

    HeaderValue::from_str(&cookie.to_string())
        .map_err(|e| ServerFnError::new(format!("Cookie error: {e}")))
}

#[cfg(feature = "ssr")]
fn set_session_cookie_response(session: &UserSession) -> Result<(), ServerFnError> {
    let response_options = expect_context::<leptos_axum::ResponseOptions>();
    response_options.insert_header(SET_COOKIE, build_session_cookie_header(session)?);
    Ok(())
}

#[cfg(feature = "ssr")]
fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let pattern = format!(r"(?s)<(?:\w+:)?{}>\s*([^<]+?)\s*</(?:\w+:)?{}>", tag, tag);
    Regex::new(&pattern)
        .ok()?
        .captures(xml)
        .and_then(|captures| captures.get(1))
        .map(|capture| capture.as_str().trim().to_string())
}

#[cfg(feature = "ssr")]
fn first_tag(xml: &str, tags: &[&str]) -> Option<String> {
    tags.iter().find_map(|tag| extract_tag(xml, tag))
}

#[cfg(feature = "ssr")]
fn ensure_email(username: &str, email: Option<String>) -> String {
    match email {
        Some(value) if value.contains('@') => value,
        _ if username.contains('@') => username.to_string(),
        _ => format!("{username}@uab.edu"),
    }
}

#[cfg(feature = "ssr")]
async fn validate_cas_ticket(ticket: &str, service: &str) -> Result<CasUserInfo, ServerFnError> {
    let validate_base = cas_validate_url();
    let mut validate_url = reqwest::Url::parse(&validate_base)
        .map_err(|e| ServerFnError::new(format!("Invalid CAS_VALIDATE_URL: {e}")))?;
    validate_url
        .query_pairs_mut()
        .append_pair("service", service)
        .append_pair("ticket", ticket);

    let response = reqwest::Client::new()
        .get(validate_url)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("CAS validation request failed: {e}")))?;

    let body = response
        .text()
        .await
        .map_err(|e| ServerFnError::new(format!("CAS validation response read failed: {e}")))?;

    if body.contains("authenticationFailure") {
        return Err(ServerFnError::new("CAS authentication failed"));
    }

    let username = first_tag(&body, &["user"])
        .ok_or_else(|| ServerFnError::new("CAS response missing user"))?;
    let email = ensure_email(
        &username,
        first_tag(&body, &["mail", "email", "eduPersonPrincipalName", "user"]),
    );
    let display_name =
        first_tag(&body, &["displayName", "cn", "name"]).unwrap_or_else(|| username.clone());

    Ok(CasUserInfo {
        username,
        email,
        display_name,
    })
}

#[cfg(feature = "ssr")]
async fn get_or_create_cas_user(
    cas_user: &CasUserInfo,
) -> Result<crate::models::User, ServerFnError> {
    use crate::models::User;

    if let Some(existing) = User::get_by_email(&cas_user.email)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
    {
        return Ok(existing);
    }

    // CAS-authenticated users still need a non-null password hash in local storage.
    let generated_password = format!(
        "cas:{}:{}:{}",
        cas_user.username,
        cas_user.email,
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );

    match User::create(
        cas_user.email.clone(),
        cas_user.display_name.clone(),
        generated_password,
    )
    .await
    {
        Ok(created) => Ok(created),
        Err(sqlx::Error::Database(db_error)) if db_error.is_unique_violation() => {
            User::get_by_email(&cas_user.email)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {e}")))?
                .ok_or_else(|| ServerFnError::new("CAS user lookup failed after create race"))
        }
        Err(e) => Err(ServerFnError::new(format!(
            "Failed to create CAS user: {e}"
        ))),
    }
}

#[cfg(feature = "ssr")]
fn build_cas_login_redirect() -> Result<Redirect, ServerFnError> {
    let mut login_url = reqwest::Url::parse(&cas_login_url())
        .map_err(|e| ServerFnError::new(format!("Invalid CAS_LOGIN_URL: {e}")))?;
    login_url
        .query_pairs_mut()
        .append_pair("service", &cas_service_id());
    Ok(Redirect::temporary(login_url.as_ref()))
}

#[cfg(feature = "ssr")]
pub async fn cas_login_redirect() -> Response {
    match build_cas_login_redirect() {
        Ok(redirect) => redirect.into_response(),
        Err(error) => {
            tracing::error!("CAS login redirect failed: {error}");
            Redirect::temporary("/login?cas_error=config").into_response()
        }
    }
}

#[cfg(feature = "ssr")]
pub async fn cas_callback(Query(query): Query<CasCallbackQuery>) -> Response {
    let Some(ticket) = query.ticket.filter(|value| !value.trim().is_empty()) else {
        return Redirect::temporary("/login?cas_error=missing_ticket").into_response();
    };

    let service_id = cas_service_id();
    let cas_user = match validate_cas_ticket(&ticket, &service_id).await {
        Ok(user) => user,
        Err(error) => {
            tracing::error!("CAS ticket validation failed: {error}");
            return Redirect::temporary("/login?cas_error=validation").into_response();
        }
    };

    let user = match get_or_create_cas_user(&cas_user).await {
        Ok(user) => user,
        Err(error) => {
            tracing::error!("CAS user provisioning failed: {error}");
            return Redirect::temporary("/login?cas_error=user").into_response();
        }
    };

    let session = UserSession {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role,
    };

    match build_session_cookie_header(&session) {
        Ok(cookie_header) => {
            let mut response = Redirect::temporary("/").into_response();
            response.headers_mut().insert(SET_COOKIE, cookie_header);
            response
        }
        Err(error) => {
            tracing::error!("CAS session cookie build failed: {error}");
            Redirect::temporary("/login?cas_error=session").into_response()
        }
    }
}

// Login
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LoginMessages {
    Successful,
    Unsuccessful,
}

pub type LoginSignal = ServerAction<Login>;

#[server]
pub async fn login(email: String, password: String) -> Result<LoginMessages, ServerFnError> {
    use crate::models::User;

    // Authenticate user
    let user = User::authenticate(email.clone(), password)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    match user {
        Some(user) => {
            // Create session
            let session = UserSession {
                id: user.id,
                email: user.email,
                name: user.name,
                role: user.role,
            };

            // Set session cookie
            set_session_cookie_response(&session)?;

            leptos_axum::redirect("/");
            Ok(LoginMessages::Successful)
        }
        None => Ok(LoginMessages::Unsuccessful),
    }
}

// Signup
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignupAction {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SignupResponse {
    Success,
    ValidationError(String),
    CreateUserError(String),
}

pub type SignupSignal = ServerAction<Signup>;

pub fn validate_signup(name: String, email: String, password: String) -> Result<(), String> {
    // Validate name
    if name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if name.len() < 2 {
        return Err("Name must be at least 2 characters".to_string());
    }

    // Validate email format
    if !email.contains('@') {
        return Err("Invalid email format".to_string());
    }

    // Validate password
    if password.len() < 8 {
        return Err("Password must be at least 8 characters".to_string());
    }

    // Check profanity in name
    #[cfg(feature = "ssr")]
    {
        if crate::profanity::contains_profanity(&name) {
            return Err("Name contains inappropriate language".to_string());
        }
    }

    Ok(())
}

#[server]
pub async fn signup(
    email: String,
    name: String,
    password: String,
) -> Result<SignupResponse, ServerFnError> {
    use crate::models::User;

    // Validate input
    if let Err(e) = validate_signup(name.clone(), email.clone(), password.clone()) {
        return Ok(SignupResponse::ValidationError(e));
    }

    // Check if user already exists
    let existing_user = User::get_by_email(&email)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    if existing_user.is_some() {
        return Ok(SignupResponse::CreateUserError(
            "Email already registered".to_string(),
        ));
    }

    // Create user
    let user = User::create(email.clone(), name.clone(), password)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create user: {}", e)))?;

    // Auto-login: Create session
    let session = UserSession {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role,
    };

    set_session_cookie_response(&session)?;

    Ok(SignupResponse::Success)
}

// Logout
#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    let response_options = expect_context::<leptos_axum::ResponseOptions>();
    response_options.insert_header(SET_COOKIE, build_clear_session_cookie_header()?);

    Ok(())
}

// Get current user from session
#[server]
pub async fn get_user() -> Result<Option<UserSession>, ServerFnError> {
    use axum_extra::extract::CookieJar;

    let jar: CookieJar = leptos_axum::extract().await?;

    if let Some(cookie) = jar.get("user_session") {
        let session: UserSession = serde_json::from_str(cookie.value())
            .map_err(|e| ServerFnError::new(format!("Session deserialization error: {}", e)))?;
        Ok(Some(session))
    } else {
        Ok(None)
    }
}

// Helper to require authentication
#[cfg(feature = "ssr")]
pub async fn require_auth() -> Result<UserSession, ServerFnError> {
    match get_user().await? {
        Some(user) => Ok(user),
        None => Err(ServerFnError::new("Authentication required")),
    }
}

// Helper to require moderator role
#[cfg(feature = "ssr")]
pub async fn require_moderator() -> Result<UserSession, ServerFnError> {
    let user = require_auth().await?;
    if user.is_moderator() {
        Ok(user)
    } else {
        Err(ServerFnError::new("Moderator access required"))
    }
}

// Helper to require admin role
#[cfg(feature = "ssr")]
pub async fn require_admin() -> Result<UserSession, ServerFnError> {
    let user = require_auth().await?;
    if user.is_admin() {
        Ok(user)
    } else {
        Err(ServerFnError::new("Admin access required"))
    }
}
