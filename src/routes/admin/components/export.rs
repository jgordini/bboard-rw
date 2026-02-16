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
export function fetchAndDownloadCsv(url, filename, onSuccess, onError) {
  fetch(url, { credentials: 'same-origin' })
    .then(response => {
      if (!response.ok) throw new Error(response.status.toString());
      return response.blob();
    })
    .then(blob => {
      const blobUrl = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = blobUrl;
      link.setAttribute('download', filename);
      document.body.appendChild(link);
      link.click();
      link.remove();
      URL.revokeObjectURL(blobUrl);
      onSuccess();
    })
    .catch(() => onError());
}
"#)]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = fetchAndDownloadCsv)]
    fn fetch_and_download_csv(
        url: &str,
        filename: &str,
        on_success: &js_sys::Function,
        on_error: &js_sys::Function,
    );
}

fn run_export(
    dataset: &'static str,
    url: &'static str,
    filename: &'static str,
    export_status: RwSignal<ExportStatus>,
    is_exporting: RwSignal<bool>,
) {
    if is_exporting.get_untracked() {
        return;
    }

    is_exporting.set(true);
    export_status.set(preparing_status(dataset));

    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::Closure;
        use wasm_bindgen::JsCast;

        let on_success: Closure<dyn FnMut()> = Closure::once(move || {
            export_status.set(completion_status(dataset, Ok(())));
            is_exporting.set(false);
        });
        let on_error: Closure<dyn FnMut()> = Closure::once(move || {
            export_status.set(completion_status(dataset, Err("request failed")));
            is_exporting.set(false);
        });
        fetch_and_download_csv(
            url,
            filename,
            on_success.as_ref().unchecked_ref(),
            on_error.as_ref().unchecked_ref(),
        );
        // Leak closures so they survive until the JS callbacks fire
        on_success.forget();
        on_error.forget();
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (url, filename);
        export_status.set(completion_status(
            dataset,
            Err("CSV export is only available in a browser session."),
        ));
        is_exporting.set(false);
    }
}

#[component]
pub(super) fn ExportTab() -> impl IntoView {
    let export_status = RwSignal::new(ExportStatus::Idle);
    let is_exporting = RwSignal::new(false);

    let handle_export_ideas = move |_| {
        run_export(
            "Ideas",
            "/admin/export/ideas.csv",
            "ideas_export.csv",
            export_status,
            is_exporting,
        );
    };

    let handle_export_comments = move |_| {
        run_export(
            "Comments",
            "/admin/export/comments.csv",
            "comments_export.csv",
            export_status,
            is_exporting,
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

    #[test]
    fn idle_is_not_active_and_has_empty_message() {
        let idle = ExportStatus::Idle;
        assert!(!idle.is_active());
        assert!(!idle.is_success());
        assert!(!idle.is_error());
        assert_eq!(idle.message(), "");
    }

    #[test]
    fn preparing_is_active_but_not_success_or_error() {
        let preparing = preparing_status("Ideas");
        assert!(preparing.is_active());
        assert!(!preparing.is_success());
        assert!(!preparing.is_error());
    }

    #[test]
    fn success_is_active_and_success_but_not_error() {
        let success = completion_status("Ideas", Ok(()));
        assert!(success.is_active());
        assert!(success.is_success());
        assert!(!success.is_error());
    }

    #[test]
    fn error_is_active_and_error_but_not_success() {
        let error = completion_status("Ideas", Err("fail"));
        assert!(error.is_active());
        assert!(!error.is_success());
        assert!(error.is_error());
    }

    #[test]
    fn message_returns_inner_text_for_all_variants() {
        assert_eq!(ExportStatus::Idle.message(), "");
        assert_eq!(
            ExportStatus::Preparing("loading...".into()).message(),
            "loading..."
        );
        assert_eq!(
            ExportStatus::Success("done".into()).message(),
            "done"
        );
        assert_eq!(
            ExportStatus::Error("failed".into()).message(),
            "failed"
        );
    }

    #[test]
    fn completion_status_error_message_is_user_facing() {
        let error = completion_status("Ideas", Err("any internal reason"));
        // Error message should be user-facing, not expose the internal reason
        assert_eq!(
            error.message(),
            "Could not start Ideas CSV download. Please try again."
        );
    }
}
