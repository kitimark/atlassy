#!/usr/bin/env bash

# Source this script so exported variables persist in your shell:
#   source qa/scripts/setup-confluence-env.sh

_atlassy_is_sourced=0
(return 0 2>/dev/null) && _atlassy_is_sourced=1
if [ "${_atlassy_is_sourced}" -ne 1 ]; then
  printf 'This script must be sourced so exports persist.\n' >&2
  printf 'Usage: source qa/scripts/setup-confluence-env.sh\n' >&2
  exit 1
fi
unset _atlassy_is_sourced

atlassy_prompt() {
  _var_name="$1"
  _label="$2"
  _default="$3"

  if [ -n "${_default}" ]; then
    printf '%s [%s]: ' "${_label}" "${_default}"
  else
    printf '%s: ' "${_label}"
  fi

  IFS= read -r _input
  if [ -z "${_input}" ]; then
    _input="${_default}"
  fi

  eval "${_var_name}=\$_input"
}

atlassy_prompt_secret() {
  _var_name="$1"
  _label="$2"
  _default="$3"

  if [ -n "${_default}" ]; then
    printf '%s [hidden, Enter keeps current]: ' "${_label}"
  else
    printf '%s: ' "${_label}"
  fi

  IFS= read -r -s _input
  printf '\n'

  if [ -z "${_input}" ]; then
    _input="${_default}"
  fi

  eval "${_var_name}=\$_input"
}

atlassy_prompt \
  ATLASSY_CONFLUENCE_BASE_URL \
  'ATLASSY_CONFLUENCE_BASE_URL (example: https://yourdomain.atlassian.net)' \
  "${ATLASSY_CONFLUENCE_BASE_URL:-}"

ATLASSY_CONFLUENCE_BASE_URL="${ATLASSY_CONFLUENCE_BASE_URL%/}"
case "${ATLASSY_CONFLUENCE_BASE_URL}" in
  */wiki)
    ATLASSY_CONFLUENCE_BASE_URL="${ATLASSY_CONFLUENCE_BASE_URL%/wiki}"
    printf 'Adjusted base URL by removing trailing /wiki.\n'
    ;;
esac

atlassy_prompt \
  ATLASSY_CONFLUENCE_EMAIL \
  'ATLASSY_CONFLUENCE_EMAIL' \
  "${ATLASSY_CONFLUENCE_EMAIL:-}"

atlassy_prompt_secret \
  ATLASSY_CONFLUENCE_API_TOKEN \
  'ATLASSY_CONFLUENCE_API_TOKEN' \
  "${ATLASSY_CONFLUENCE_API_TOKEN:-}"

atlassy_prompt PAGE_ID 'PAGE_ID (optional)' "${PAGE_ID:-}"
atlassy_prompt ARTIFACTS_DIR 'ARTIFACTS_DIR' "${ARTIFACTS_DIR:-.}"

if [ -z "${ATLASSY_CONFLUENCE_BASE_URL}" ]; then
  printf 'ATLASSY_CONFLUENCE_BASE_URL is required.\n' >&2
  return 1
fi

if [ -z "${ATLASSY_CONFLUENCE_EMAIL}" ]; then
  printf 'ATLASSY_CONFLUENCE_EMAIL is required.\n' >&2
  return 1
fi

if [ -z "${ATLASSY_CONFLUENCE_API_TOKEN}" ]; then
  printf 'ATLASSY_CONFLUENCE_API_TOKEN is required.\n' >&2
  return 1
fi

if [ -z "${ARTIFACTS_DIR}" ]; then
  ARTIFACTS_DIR='.'
fi

export ATLASSY_CONFLUENCE_BASE_URL
export ATLASSY_CONFLUENCE_EMAIL
export ATLASSY_CONFLUENCE_API_TOKEN
export ARTIFACTS_DIR

if [ -n "${PAGE_ID}" ]; then
  export PAGE_ID
fi

printf '\nConfigured environment variables:\n'
printf '  ATLASSY_CONFLUENCE_BASE_URL=%s\n' "${ATLASSY_CONFLUENCE_BASE_URL}"
printf '  ATLASSY_CONFLUENCE_EMAIL=%s\n' "${ATLASSY_CONFLUENCE_EMAIL}"
printf '  ATLASSY_CONFLUENCE_API_TOKEN=<hidden, length %s>\n' "${#ATLASSY_CONFLUENCE_API_TOKEN}"
printf '  ARTIFACTS_DIR=%s\n' "${ARTIFACTS_DIR}"
if [ -n "${PAGE_ID}" ]; then
  printf '  PAGE_ID=%s\n' "${PAGE_ID}"
fi

printf '\nReady. Example preflight command:\n'
printf '  cargo run -p atlassy-cli -- run --request-id live-preflight-001 --page-id "${PAGE_ID:-REPLACE_PAGE_ID}" --edit-intent "sandbox preflight" --mode no-op --runtime-backend live --force-verify-fail --artifacts-dir "${ARTIFACTS_DIR}"\n'

unset -f atlassy_prompt
unset -f atlassy_prompt_secret
