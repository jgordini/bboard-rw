# RoboRev Fix: Admin Export Design Refresh

Date: 2026-02-13

## Goal
Improve visual design and UX clarity of the admin "Data Export" section.

## Changes
- `src/routes/admin/components/overview.rs`
  - Restructured export block with semantic heading wrapper.
  - Added dedicated export button class (`admin-export-btn`).
  - Added status message states (`success` / `error`) with `aria-live` for feedback.
  - Added signal-based error state to style status outcomes clearly.

- `style/main.scss`
  - Added dedicated styles for:
    - `.admin-export-panel`
    - `.admin-export-heading`
    - `.admin-export-actions`
    - `.admin-export-btn`
    - `.admin-export-status` (+ success/error variants)
  - Added mobile behavior: export buttons collapse to one column at narrow widths.

## Validation
- `SQLX_OFFLINE=true cargo check` passed.
