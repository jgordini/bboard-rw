# RoboRev Fix: Admin CSV Export for Ideas and Comments

Date: 2026-02-13

## Goal
Allow admins to export ideas and comments as CSV files.

## Changes
- `src/routes/admin.rs`
  - Added admin-only server functions:
    - `export_ideas_csv()`
    - `export_comments_csv()`
  - Added SSR helpers for CSV generation:
    - `csv_escape()`
    - `build_ideas_csv()`
    - `build_comments_csv()`
  - Export queries include joined author metadata and timestamps.

- `src/routes/admin/components.rs`
  - Updated `OverviewTab` call to pass `is_admin` flag.

- `src/routes/admin/components/overview.rs`
  - Added admin-only "Data Export" panel in Overview tab.
  - Added buttons:
    - "Export Ideas CSV"
    - "Export Comments CSV"
  - Added one-click browser file download via hydrate-only inline JS bridge (`downloadCsv`).
  - Added user-visible export status text for success/failure.

## Authorization
- Export server functions require `require_admin()`.
- Non-admin users do not see export controls in the admin UI.

## Validation
- `SQLX_OFFLINE=true cargo check` passed.
