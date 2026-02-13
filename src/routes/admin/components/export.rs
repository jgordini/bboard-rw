use leptos::prelude::*;

use crate::routes::async_helpers::spawn_server_action;

use super::super::{export_comments_csv, export_ideas_csv};

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = r#"
export function downloadCsv(filename, content) {
  const blob = new Blob([content], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.setAttribute('download', filename);
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
}
"#
)]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = downloadCsv)]
    fn download_csv(filename: &str, content: &str);
}

fn trigger_csv_download(filename: &str, content: &str) {
    #[cfg(feature = "hydrate")]
    download_csv(filename, content);

    #[cfg(not(feature = "hydrate"))]
    let _ = (filename, content);
}

#[component]
pub(super) fn ExportTab() -> impl IntoView {
    let export_status = RwSignal::new(String::new());
    let is_exporting = RwSignal::new(false);
    let export_failed = RwSignal::new(false);

    let handle_export_ideas = move |_| {
        if is_exporting.get() {
            return;
        }

        is_exporting.set(true);
        export_failed.set(false);
        export_status.set("Preparing ideas CSV...".to_string());

        spawn_server_action(
            export_ideas_csv(),
            move |csv| {
                trigger_csv_download("ideas_export.csv", &csv);
                export_status.set("Ideas CSV downloaded.".to_string());
                export_failed.set(false);
                is_exporting.set(false);
            },
            move |error| {
                export_status.set(format!("Failed to export ideas CSV: {}", error));
                export_failed.set(true);
                is_exporting.set(false);
            },
        );
    };

    let handle_export_comments = move |_| {
        if is_exporting.get() {
            return;
        }

        is_exporting.set(true);
        export_failed.set(false);
        export_status.set("Preparing comments CSV...".to_string());

        spawn_server_action(
            export_comments_csv(),
            move |csv| {
                trigger_csv_download("comments_export.csv", &csv);
                export_status.set("Comments CSV downloaded.".to_string());
                export_failed.set(false);
                is_exporting.set(false);
            },
            move |error| {
                export_status.set(format!("Failed to export comments CSV: {}", error));
                export_failed.set(true);
                is_exporting.set(false);
            },
        );
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
                        disabled=move || is_exporting.get()
                        on:click=handle_export_ideas
                    >
                        "Export Ideas CSV"
                    </button>
                    <button
                        type="button"
                        class="btn btn-primary admin-export-btn"
                        disabled=move || is_exporting.get()
                        on:click=handle_export_comments
                    >
                        "Export Comments CSV"
                    </button>
                </div>
                <Show when=move || !export_status.get().is_empty()>
                    <p
                        class="admin-export-status"
                        class:error=move || export_failed.get()
                        class:success=move || !export_failed.get()
                        aria-live="polite"
                    >
                        {move || export_status.get()}
                    </p>
                </Show>
            </section>
        </div>
    }
}
