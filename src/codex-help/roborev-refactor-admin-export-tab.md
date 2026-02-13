# RoboRev Refactor: Move Data Export Into Admin Tabs

Date: 2026-02-13

## Goal
Move data export controls out of Overview and into a dedicated Admin tab.

## Changes
- `src/routes/admin/components/export.rs`
  - New `ExportTab` component.
  - Contains ideas/comments CSV export actions and status UI.
  - Reuses hydrate-only browser download bridge.

- `src/routes/admin/components.rs`
  - Added `mod export;` and `use export::ExportTab;`.
  - Added `Data Export` tab button for admins.
  - Added `"export"` tab content route.
  - Reverted `OverviewTab` call to stats-only.

- `src/routes/admin/components/overview.rs`
  - Removed export section; now statistics-only again.

- `style/main.scss`
  - Included `.export-tab` in shared admin tab content styling selectors.

## Validation
- `SQLX_OFFLINE=true cargo check` passed.
