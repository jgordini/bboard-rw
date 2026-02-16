# Review Notes: d355277

## Summary
- Commit updates `scripts/deploy.sh` TLS update/update-prebuilt paths and adds shell invocation tests in `scripts/tests/test-deploy-invocations.sh`.

## Findings
1. Test harness can report assertion failures while still returning success.
   - Root cause: assertion calls in test functions do not short-circuit on failure while functions are invoked in `if "$fn"; then`, where `set -e` does not reliably abort nested command failures.
   - Example: with inherited `ENABLE_LETSENCRYPT=true`, non-TLS test printed an assertion failure but was still marked `[PASS]`.
   - Relevant lines: `scripts/tests/test-deploy-invocations.sh:87-109`, `scripts/tests/test-deploy-invocations.sh:112-120`.

2. Test runs are environment-sensitive because TLS flags can leak from parent env.
   - Root cause: `run_deploy` uses `env "${env_vars[@]}" ...` without clearing inherited environment and does not set `ENABLE_LETSENCRYPT=false` for non-TLS runs.
   - Relevant lines: `scripts/tests/test-deploy-invocations.sh:66-80`.

## Suggested fixes
- In each test function, make assertions strict with `assert_contains ... || return 1` and `assert_not_contains ... || return 1`.
- Execute deploy script with a sanitized environment (`env -i`) and explicitly set required vars for each scenario.
- Alternatively, explicitly set `ENABLE_LETSENCRYPT=false` in non-TLS branch to avoid leakage.
