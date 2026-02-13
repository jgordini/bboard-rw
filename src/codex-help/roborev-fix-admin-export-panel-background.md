# RoboRev Fix: Remove Export Panel Gradient

Date: 2026-02-13

## Change
- Updated `style/main.scss` for `.admin-page .admin-export-panel`.
- Replaced gradient background with a solid background:
  - from `linear-gradient(180deg, var(--evergreen-5) 0%, var(--white) 100%)`
  - to `var(--white)`

## Outcome
- Export panel now uses a flat background (no gradient).
