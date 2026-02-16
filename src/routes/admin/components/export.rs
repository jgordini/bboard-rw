use leptos::prelude::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum ExportStatus {
    #[default]
    Idle,
    Preparing(String),
    Success(String),
    Error(String),
}

impl ExportStatus {
    fn message(self) -> String {
        match self {
            Self::Idle => String::new(),
            Self::Preparing(message) | Self::Success(message) | Self::Error(message) => message,
        }
    }

    fn is_active(&self) -> bool {
        !matches!(self, Self::Idle)
    }

    fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }
}

fn preparing_status(dataset: &str) -> ExportStatus {
    ExportStatus::Preparing(format!("Preparing {dataset} CSV export..."))
}

fn completion_status(dataset: &str, result: Result<(), &'static str>) -> ExportStatus {
    match result {
        Ok(()) => ExportStatus::Success(format!("{dataset} CSV download started.")),
        Err(_) => ExportStatus::Error(format!(
            "Could not start {dataset} CSV download. Please try again."
        )),
    }
}

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

fn trigger_csv_download(url: &str) -> Result<(), &'static str> {
    #[cfg(feature = "hydrate")]
    {
        download_csv(url);
        Ok(())
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = url;
        Err("CSV export is only available in a browser session.")
    }
}

fn run_export(dataset: &'static str, url: &'static str, export_status: RwSignal<ExportStatus>) {
    export_status.set(preparing_status(dataset));
    let result = trigger_csv_download(url);
    export_status.set(completion_status(dataset, result));
}

#[component]
pub(super) fn ExportTab() -> impl IntoView {
    let export_status = RwSignal::new(ExportStatus::Idle);

    let handle_export_ideas = move |_| {
        run_export("Ideas", "/admin/export/ideas.csv", export_status);
    };

    let handle_export_comments = move |_| {
        run_export("Comments", "/admin/export/comments.csv", export_status);
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
                <p
                    class="admin-export-status"
                    class:active=move || export_status.get().is_active()
                    class:success=move || export_status.get().is_success()
                    class:error=move || export_status.get().is_error()
                    role="status"
                    aria-live="polite"
                    aria-atomic="true"
                >
                    {move || export_status.get().message()}
                </p>
            </section>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{completion_status, preparing_status, ExportStatus};

    #[test]
    fn export_status_transitions_idle_to_preparing_to_success() {
        let idle = ExportStatus::Idle;
        let preparing = preparing_status("Ideas");
        let success = completion_status("Ideas", Ok(()));

        assert_eq!(idle, ExportStatus::Idle);
        assert_eq!(
            preparing,
            ExportStatus::Preparing("Preparing Ideas CSV export...".to_string())
        );
        assert_eq!(
            success,
            ExportStatus::Success("Ideas CSV download started.".to_string())
        );
        assert!(!preparing.clone().is_success());
        assert!(success.is_success());
    }

    #[test]
    fn export_status_transitions_idle_to_preparing_to_error() {
        let preparing = preparing_status("Comments");
        let error = completion_status("Comments", Err("download failed"));

        assert_eq!(
            preparing,
            ExportStatus::Preparing("Preparing Comments CSV export...".to_string())
        );
        assert_eq!(
            error,
            ExportStatus::Error(
                "Could not start Comments CSV download. Please try again.".to_string()
            )
        );
        assert!(!preparing.clone().is_error());
        assert!(error.is_error());
    }
}
