use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet};
use leptos_router::components::{Route, Router, Routes, A};
use leptos_router::path;

use crate::routes::{IdeasPage, AdminPage, IdeaDetailPage};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <link rel="shortcut icon" type="image/ico" href="/favicon.ico"/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[tracing::instrument]
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // UAB Fonts - Aktiv Grotesk and Kulturista
        <Stylesheet href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap"/>
        <Stylesheet href="/pkg/realworld-leptos.css"/>

        <Router>
            <nav class="navbar">
                <div class="container">
                    <a href="/" class="navbar-brand">
                        <span class="logo-font">"UAB IT Idea Board"</span>
                    </a>
                    <ul class="nav navbar-nav pull-xs-right">
                        <li class="nav-item">
                            <a href="/admin" class="nav-link">"Admin"</a>
                        </li>
                    </ul>
                </div>
            </nav>
            <main>
                <Routes fallback=|| view! { <div class="container"><p>"Page not found"</p></div> }>
                    <Route path=path!("/") view=|| view! { <IdeasPage/> }/>
                    <Route path=path!("/ideas/:id") view=|| view! { <IdeaDetailPage/> }/>
                    <Route path=path!("/admin") view=|| view! { <AdminPage/> }/>
                </Routes>
            </main>
            <footer class="footer">
                <div class="container">
                    <a href="/"><span class="logo-font">"UAB IT Idea Board"</span></a>
                    <span class="attribution">
                        "UAB Information Technology"
                    </span>
                </div>
            </footer>
        </Router>
    }
}
