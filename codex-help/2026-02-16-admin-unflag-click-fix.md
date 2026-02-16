## Change

Hardened admin moderation action buttons so unflag/restore actions are reliably clickable.

## Files

- `src/routes/admin/components/flags.rs`
- `src/routes/admin/components/moderation.rs`

## Detail

- Added `type="button"` to action buttons in flagged and moderation lists.
- Renamed flagged-item clear action label from `"Dismiss Flags"` to `"Unflag"` for clarity.

## Verification

- `cargo check --all-targets --all-features` passes.
