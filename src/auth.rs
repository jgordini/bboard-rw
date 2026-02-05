use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use axum_extra::extract::cookie::{Cookie, SameSite};

/// When this signal is updated, the nav's user resource refetches (e.g. after login).
#[derive(Clone, Copy)]
pub struct AuthRefresh(pub RwSignal<u32>);

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

// Login
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginAction {
    pub email: String,
    pub password: String,
}

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
            let session_json = serde_json::to_string(&session)
                .map_err(|e| ServerFnError::new(format!("Session serialization error: {}", e)))?;

            let cookie = Cookie::build(("user_session", session_json))
                .path("/")
                .same_site(SameSite::Lax)
                .http_only(true)
                .max_age(time::Duration::days(7));

            let response_options = expect_context::<leptos_axum::ResponseOptions>();
            response_options.insert_header(
                axum::http::header::SET_COOKIE,
                axum::http::HeaderValue::from_str(&cookie.to_string())
                    .map_err(|e| ServerFnError::new(format!("Cookie error: {}", e)))?,
            );

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
pub async fn signup(email: String, name: String, password: String) -> Result<SignupResponse, ServerFnError> {
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
        return Ok(SignupResponse::CreateUserError("Email already registered".to_string()));
    }

    // Create user
    let user = User::create(email.clone(), name.clone(), password)
        .await
        .map_err(|e| {
            ServerFnError::new(format!("Failed to create user: {}", e))
        })?;

    // Auto-login: Create session
    let session = UserSession {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role,
    };

    let session_json = serde_json::to_string(&session)
        .map_err(|e| ServerFnError::new(format!("Session serialization error: {}", e)))?;

    let cookie = Cookie::build(("user_session", session_json))
        .path("/")
        .same_site(SameSite::Lax)
        .http_only(true)
        .max_age(time::Duration::days(7));

    let response_options = expect_context::<leptos_axum::ResponseOptions>();
    response_options.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&cookie.to_string())
            .map_err(|e| ServerFnError::new(format!("Cookie error: {}", e)))?,
    );

    Ok(SignupResponse::Success)
}

// Logout
#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    let cookie = Cookie::build(("user_session", ""))
        .path("/")
        .max_age(time::Duration::seconds(0));

    let response_options = expect_context::<leptos_axum::ResponseOptions>();
    response_options.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&cookie.to_string())
            .map_err(|e| ServerFnError::new(format!("Cookie error: {}", e)))?,
    );

    Ok(())
}

pub type LogoutSignal = ServerAction<Logout>;

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
