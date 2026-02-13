# RoboRev Continue: Admin Module Split

Date: 2026-02-13

## Goal
Split `routes/admin.rs` UI tabs into a dedicated submodule, following the same decomposition pattern used for idea detail.

## Changes
- In `src/routes/admin.rs`:
  - Added `mod components;`
  - Imported `AdminDashboard` from `components`.
  - Kept server functions, shared types, `AdminPage`, and `role_name` in the parent module.
  - Removed in-file tab/dashboard component definitions.
- Added `src/routes/admin/components.rs` containing:
  - `AdminDashboard`
  - `OverviewTab`
  - `FlagsTab`
  - `ModerationTab`
  - `UsersTab`

## Outcome
- `admin.rs` now focuses on server-side actions and route entry logic.
- Dashboard/tab rendering concerns are isolated in `admin/components.rs`.
- Build check passes after module split.
