use leptos::prelude::*;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen(inline_js = r#"
export function downloadCsv(url) {
  const link = document.createElement('a');
  link.href = url;
  link.setAttribute('download', '');
  document.body.appendChild(link);
  link.click();
  link.remove();
}
"#)]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = downloadCsv)]
    fn download_csv(url: &str);
}

fn trigger_csv_download(url: &str) {
    #[cfg(feature = "hydrate")]
    download_csv(url);

    #[cfg(not(feature = "hydrate"))]
    let _ = url;
}

#[component]
pub(super) fn ExportTab() -> impl IntoView {
    let export_status = RwSignal::new(String::new());

    let handle_export_ideas = move |_| {
        trigger_csv_download("/admin/export/ideas.csv");
        export_status.set("Ideas CSV download started.".to_string());
    };

    let handle_export_comments = move |_| {
        trigger_csv_download("/admin/export/comments.csv");
        export_status.set("Comments CSV download started.".to_string());
    };

    view! {
        <div class="export-tab">
            <h2>"Data Export"</h2>
            <section class="admin-export-panel" aria-label="Data export controls">
                <header class="admin-export-heading">
                    <h3>"Export CSV Files"</h3>
                    <p>"Download complete snapshots of ideas and comments."</p>
                </header>
                <div class="admin-export-actions">
                    <button
                        type="button"
                        class="btn btn-primary admin-export-btn"
                        on:click=handle_export_ideas
                    >
                        "Export Ideas CSV"
                    </button>
                    <button
                        type="button"
                        class="btn btn-primary admin-export-btn"
                        on:click=handle_export_comments
                    >
                        "Export Comments CSV"
                    </button>
                </div>
                <Show when=move || !export_status.get().is_empty()>
                    <p
                        class="admin-export-status"
                        class:success=true
                        aria-live="polite"
                    >
                        {move || export_status.get()}
                    </p>
                </Show>
            </section>
        </div>
    }
}
