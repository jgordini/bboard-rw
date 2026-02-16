# Review Notes: 6945a7d

Date: 2026-02-16

## Summary
- Hardens CAS validation by parsing XML and enforcing success semantics.
- Introduces CAS subject-based identity linking (`users.cas_subject`).
- Prevents automatic CAS-to-local account linking by matching email.
- Adds focused unit tests for CAS XML parsing and login error precedence.

## Findings
1. High regression risk: legacy CAS users created before `cas_subject` now hit `link_required` and cannot sign in automatically.
   - refs: `src/auth.rs:242`, `src/auth.rs:250`, `migrations/20260216000000_add_cas_subject_to_users.up.sql:3`
   - fix direction: provide one-time migration/linking workflow to populate `cas_subject` for existing CAS users.

2. Medium regression risk: subject extraction can change identity key across logins (`eduPersonPrincipalName` -> `uid` -> `user`) if attributes vary, causing lockouts.
   - ref: `src/auth.rs:161`
   - fix direction: require a single canonical subject attribute and normalize it (e.g., lowercase) before lookup/insert.

3. Testing gap: no unit/integration test covers `get_or_create_cas_user` for legacy-account conflict and subject-attribute drift.
   - refs: `src/auth.rs:228`, `src/auth.rs:553`
   - fix direction: add tests for existing-email/no-subject path and varying CAS subject attribute sets.
