#!/usr/bin/env bash
# Download a public GitHub release tool, falling back to the asset API
# when the browser download route repeatedly returns a gateway error.
set -euo pipefail

REPO="${1:?repository required}"
TAG="${2:?tag required}"
ASSET="${3:?asset name required}"
DEST="${4:?destination required}"

# Keep partial downloads non-executable and outside the reusable cache.
PART="$(mktemp "${DEST}.part.XXXXXX")"
META="$(mktemp "${DEST}.metadata.XXXXXX")"
trap 'rm -f "${PART}" "${META}"' EXIT
CURL=(curl --fail --show-error --location --retry 3 --retry-delay 2
    --retry-max-time 180 --connect-timeout 20 --max-time 180)

if ! "${CURL[@]}" --output "${PART}" \
    "https://github.com/${REPO}/releases/download/${TAG}/${ASSET}"; then
    echo "Browser download failed; trying the GitHub release asset API." >&2
    "${CURL[@]}" --output "${META}" \
        "https://api.github.com/repos/${REPO}/releases/tags/${TAG}"
    ASSET_URL="$(jq -er --arg name "${ASSET}" \
        '.assets[] | select(.name == $name) | .url' "${META}")"
    "${CURL[@]}" --header 'Accept: application/octet-stream' \
        --output "${PART}" "${ASSET_URL}"
fi

# A successful HTTP response must still contain an ELF AppImage, not
# an empty response or release metadata returned by a proxy/API.
MAGIC="$(od -An -tx1 -N4 "${PART}" | tr -d ' \n')"
if [[ "${MAGIC}" != 7f454c46 ]]; then
    echo "Downloaded tool is not an ELF AppImage: ${ASSET}" >&2
    exit 1
fi
chmod 755 "${PART}"
mv -f "${PART}" "${DEST}"
