use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::auth::{LoginSignal, Logout, SignupSignal, get_user};
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
    let auth_refresh = expect_context::<crate::auth::AuthRefresh>().0;
    let user_resource = Resource::new(
        move || auth_refresh.get(),
        move |_| async move { get_user().await },
    );
    let logout = ServerAction::<Logout>::new();

    // On logout, bump auth refresh so nav and ideas page (sidebar) both refetch user
    Effect::new(move |_| {
        if let Some(Ok(())) = logout.value().get() {
            auth_refresh.update(|v| *v += 1);
        }
    });

    view! {
        <nav class="navbar">
            <div class="container">
                <a href="/" class="navbar-brand">
                    <img src="/uab-logo.jpg" alt="UAB IT Idea Board" class="navbar-logo" width="100" height="40"/>
                </a>
                <ul class="nav navbar-nav pull-xs-right">
                    <li class="nav-item nav-item-auth">
                        <Suspense fallback=move || view! { <span class="nav-link">"…"</span> }>
                            {move || match user_resource.get() {
                                None => view! {
                                    <span class="nav-link">"…"</span>
                                }.into_any(),
                                Some(Ok(Some(_))) => view! {
                                    <ActionForm action=logout>
                                        <button type="submit" class="nav-link nav-logout-btn" aria-label="Log out">"Logout"</button>
                                    </ActionForm>
                                }.into_any(),
                                Some(Ok(None)) | Some(Err(_)) => view! {
                                    <a href="/login" class="nav-link">"Login"</a>
                                }.into_any(),
                            }}
                        </Suspense>
                    </li>
                </ul>
            </div>
        </nav>
    }
}

#[component]
fn LoginRoute() -> impl IntoView {
    let login = LoginSignal::new();
    view! { <Login login=login/> }
}

#[component]
fn SignupRoute() -> impl IntoView {
    let signup = SignupSignal::new();
    view! { <Signup signup=signup/> }
}

#[tracing::instrument]
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    provide_context(crate::auth::AuthRefresh(RwSignal::new(0)));

    view! {
        // UAB Fonts - Aktiv Grotesk and Kulturista
        <Stylesheet href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap"/>
        <Stylesheet href="/pkg/realworld-leptos.css"/>

        <Router>
            <a href="#main" class="skip-link">"Skip to main content"</a>
            <NavBar/>
            <main id="main">
                <Routes fallback=|| view! { <div class="container"><p>"Page not found"</p></div> }>
                    <Route path=path!("/") view=|| view! { <IdeasPage/> }/>
                    <Route path=path!("/ideas/:id") view=|| view! { <IdeaDetailPage/> }/>
                    <Route path=path!("/login") view=|| view! { <LoginRoute/> }/>
                    <Route path=path!("/signup") view=|| view! { <SignupRoute/> }/>
                    <Route path=path!("/reset_password") view=|| view! { <ResetPassword/> }/>
                    <Route path=path!("/profile") view=|| view! { <AccountPage/> }/>
                    <Route path=path!("/admin") view=|| view! { <AdminPage/> }/>
                </Routes>
            </main>
            <footer class="footer">
                <div class="container">
                    <a href="/"><span class="logo-font">"UAB IT Idea Board"</span></a>
                    <span class="attribution">
                        "UAB Information Technology"
                    </span>
                    <a href="/admin" class="footer-link">"Admin"</a>
                </div>
            </footer>
        </Router>
    }
}
