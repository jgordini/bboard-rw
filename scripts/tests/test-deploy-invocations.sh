#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
DEPLOY_SCRIPT="${REPO_ROOT}/scripts/deploy.sh"

TEST_ROOT="$(mktemp -d)"
TEST_BIN="${TEST_ROOT}/bin"
TEST_DEPLOY_DIR="${TEST_ROOT}/deploy"
TEST_LEGACY_DIR="${TEST_ROOT}/legacy"

cleanup() {
    rm -rf "${TEST_ROOT}"
}
trap cleanup EXIT

mkdir -p "${TEST_BIN}" "${TEST_DEPLOY_DIR}" "${TEST_LEGACY_DIR}"
cat > "${TEST_DEPLOY_DIR}/docker-compose.yml" <<'EOF'
services:
  web:
    image: realworld-leptos:0.1.0
  db:
    image: postgres:17-alpine
  caddy:
    image: caddy:2
    profiles: ["tls"]
EOF

cat > "${TEST_BIN}/docker" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "${LOG_FILE}"
if [ "${1:-}" = "compose" ] && [ "${2:-}" = "version" ]; then
    echo "Docker Compose version v2.0.0"
fi
EOF
chmod +x "${TEST_BIN}/docker"

assert_contains() {
    local file="$1"
    local expected="$2"
    if ! rg -F --quiet -- "$expected" "$file"; then
        echo "Expected log to contain: ${expected}" >&2
        echo "Log output:" >&2
        cat "$file" >&2
        return 1
    fi
}

assert_not_contains() {
    local file="$1"
    local unexpected="$2"
    if rg -F --quiet -- "$unexpected" "$file"; then
        echo "Expected log to not contain: ${unexpected}" >&2
        echo "Log output:" >&2
        cat "$file" >&2
        return 1
    fi
}

run_deploy() {
    local mode="$1"
    local tls_enabled="$2"
    local log_file="${TEST_ROOT}/${mode}-${tls_enabled}.log"
    local -a env_vars=(
        "PATH=${TEST_BIN}:${PATH}"
        "LOG_FILE=${log_file}"
        "DEPLOYMENT_DIR=${TEST_DEPLOY_DIR}"
        "LEGACY_DIR=${TEST_LEGACY_DIR}"
        "VERIFY_SLEEP_SECONDS=0"
    )
    if [ "${tls_enabled}" = "true" ]; then
        env_vars+=(
            "ENABLE_LETSENCRYPT=true"
            "CLOUDFLARE_DNS_API_TOKEN=test-token"
        )
    else
        env_vars+=("ENABLE_LETSENCRYPT=false")
    fi

    env "${env_vars[@]}" "${DEPLOY_SCRIPT}" "${mode}" >/dev/null
    echo "${log_file}"
}

test_update_tls() {
    local log_file
    log_file="$(run_deploy update true)"
    assert_contains "${log_file}" "compose --profile tls up -d --no-deps --force-recreate --build web" || return 1
    assert_not_contains "${log_file}" "compose --profile tls up -d --remove-orphans --no-deps --force-recreate --build web" || return 1
}

test_update_non_tls() {
    local log_file
    log_file="$(run_deploy update false)"
    assert_contains "${log_file}" "compose up -d --no-deps --force-recreate --build web" || return 1
    assert_not_contains "${log_file}" "compose --profile tls up -d --no-deps --force-recreate --build web" || return 1
}

test_update_prebuilt_tls() {
    local log_file
    log_file="$(run_deploy update-prebuilt true)"
    assert_contains "${log_file}" "compose --profile tls up -d --no-deps --force-recreate --no-build web" || return 1
    assert_not_contains "${log_file}" "compose --profile tls up -d --remove-orphans --no-deps --force-recreate --no-build web" || return 1
}

test_replace_prebuilt_tls() {
    local log_file
    log_file="$(run_deploy replace-prebuilt true)"
    assert_contains "${log_file}" "compose --profile tls up -d --remove-orphans --no-build db web" || return 1
    assert_contains "${log_file}" "compose --profile tls up -d --remove-orphans --no-deps --build caddy" || return 1
}

run_test() {
    local name="$1"
    local fn="$2"
    if "$fn"; then
        echo "[PASS] ${name}"
    else
        echo "[FAIL] ${name}" >&2
        return 1
    fi
}

run_test "update uses profiled compose in TLS mode without orphan removal" test_update_tls
run_test "update keeps non-TLS compose invocation when TLS disabled" test_update_non_tls
run_test "update-prebuilt uses profiled compose in TLS mode without orphan removal" test_update_prebuilt_tls
run_test "replace-prebuilt TLS flow keeps web/db prebuilt and builds caddy explicitly" test_replace_prebuilt_tls

echo "All deploy invocation checks passed."
