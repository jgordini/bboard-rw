use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet};
use leptos_router::components::{Route, Router, Routes, A};
use leptos_router::path;

use crate::auth::{bump_auth_refresh, get_user, use_auth_refresh, Logout};
use crate::routes::paths;
use crate::routes::{
    AccountPage, AdminPage, IdeaDetailPage, IdeasPage, Login, ResetPassword, Signup,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="theme-color" content="#1a5632"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <link rel="shortcut icon" type="image/ico" href="/favicon.ico?v=20260214"/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
fn NavBar() -> impl IntoView {
    let auth_refresh = use_auth_refresh();
    let user_resource = Resource::new(
        move || auth_refresh.get(),
        move |_| async move { get_user().await },
    );
    let logout = use_logout_action();

    view! {
        <nav class="navbar">
            <div class="container">
                <A href=paths::HOME attr:class="navbar-brand">
                    <img src="/uab-logo.jpg" alt="UAB IT Idea Board" class="navbar-logo"/>
                </A>
                <ul class="nav navbar-nav pull-xs-right">
                    <li class="nav-item nav-item-auth">
                        <Suspense fallback=move || view! { <span class="nav-link">"…"</span> }>
                            {move || render_auth_nav_item(auth_nav_state(user_resource.get()), logout)}
                        </Suspense>
                    </li>
                </ul>
            </div>
        </nav>
    }
}

#[derive(Clone, Copy)]
enum AuthNavState {
    Loading,
    Authenticated,
    Anonymous,
}

fn auth_nav_state<T, E>(user_result: Option<Result<Option<T>, E>>) -> AuthNavState {
    match user_result {
        None => AuthNavState::Loading,
        Some(Ok(Some(_))) => AuthNavState::Authenticated,
        Some(Ok(None)) | Some(Err(_)) => AuthNavState::Anonymous,
    }
}

fn use_logout_action() -> ServerAction<Logout> {
    let logout = ServerAction::<Logout>::new();
    Effect::new(move |_| {
        if let Some(Ok(())) = logout.value().get() {
            bump_auth_refresh();
        }
    });
    logout
}

fn render_auth_nav_item(auth_state: AuthNavState, logout: ServerAction<Logout>) -> AnyView {
    match auth_state {
        AuthNavState::Loading => view! { <span class="nav-link">"…"</span> }.into_any(),
        AuthNavState::Authenticated => view! {
            <ActionForm action=logout>
                <button type="submit" class="nav-link nav-logout-btn" aria-label="Log out">"Logout"</button>
            </ActionForm>
        }
        .into_any(),
        AuthNavState::Anonymous => view! {
            <A href=paths::LOGIN attr:class="nav-link">"Login"</A>
        }
        .into_any(),
    }
}

#[component]
fn GlobalStyles() -> impl IntoView {
    view! {
        <Stylesheet href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap"/>
        <Stylesheet href="/pkg/uab-spark.css"/>
    }
}

#[component]
fn AppRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| view! { <div class="container"><p>"Page not found"</p></div> }>
            <Route path=path!("/") view=IdeasPage/>
            <Route path=path!("/ideas/:id") view=IdeaDetailPage/>
            <Route path=path!("/login") view=Login/>
            <Route path=path!("/signup") view=Signup/>
            <Route path=path!("/reset_password") view=ResetPassword/>
            <Route path=path!("/profile") view=AccountPage/>
            <Route path=path!("/admin") view=AdminPage/>
        </Routes>
    }
}

#[component]
fn AppFooter() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="container">
                <A href=paths::HOME>
                    <span class="logo-font">"UAB IT Idea Board"</span>
                </A>
                <span class="attribution">
                    "UAB Information Technology"
                </span>
                <A href=paths::ADMIN attr:class="footer-link">"Admin"</A>
            </div>
        </footer>
    }
}

#[tracing::instrument]
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    crate::auth::provide_auth_refresh_context();

    view! {
        <GlobalStyles/>
        <Router>
            <a href="#main" class="skip-link">"Skip to main content"</a>
            <NavBar/>
            <main id="main">
                <AppRoutes/>
            </main>
            <AppFooter/>
        </Router>
    }
}
