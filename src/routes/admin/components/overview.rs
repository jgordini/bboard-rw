use leptos::prelude::*;

use super::super::AdminStats;

#[component]
pub(super) fn OverviewTab(stats: Resource<Result<AdminStats, ServerFnError>>) -> impl IntoView {
    view! {
        <div class="overview-tab">
            <h2>"Statistics"</h2>
            <Suspense fallback=|| view! { <p>"Loading stats…"</p> }>
                {move || stats.get().map(|s| match s {
                    Ok(stats) => view! {
                        <div class="stats-grid grid">
                            <div class="stat-card callout callout-primary">
                                <h3>"Total Ideas"</h3>
                                <span class="stat-number">{stats.total_ideas}</span>
                            </div>
                            <div class="stat-card callout callout-primary">
                                <h3>"Total Sparks"</h3>
                                <span class="stat-number">{stats.total_votes}</span>
                            </div>
                            <div class="stat-card callout callout-secondary">
                                <h3>"Total Users"</h3>
                                <span class="stat-number">{stats.total_users}</span>
                            </div>
                            <div class="stat-card callout callout-secondary">
                                <h3>"Flagged Items"</h3>
                                <span class="stat-number">{stats.flagged_items}</span>
                            </div>
                        </div>
                    }
                        .into_any(),
                    Err(_) => view! { <p class="error">"Failed to load statistics"</p> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
