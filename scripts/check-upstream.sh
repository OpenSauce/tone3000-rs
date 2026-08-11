#!/usr/bin/env bash
# Watch the official TONE3000 type definitions for changes.
#
# tone-3000/api's src/types.ts describes the API's *input* surface — query params, enum
# vocabularies, OAuth options — and its history tracks real API cutovers closely. This
# makes no TONE3000 API calls and needs no TONE3000 credentials.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

if ! command -v gh >/dev/null 2>&1; then
  echo "error: this script requires the GitHub CLI ('gh'), authenticated (gh auth login)." >&2
  exit 1
fi

init=0
if [[ "${1:-}" == "--init" ]]; then
  init=1
fi

PINNED_FILE="scripts/upstream-types.sha"
REPO="tone-3000/api"
FILE="src/types.ts"

current="$(gh api "repos/$REPO/contents/$FILE" --jq .sha)"

if [[ ! -f "$PINNED_FILE" ]]; then
  if [[ "$init" -eq 1 ]]; then
    echo "$current" > "$PINNED_FILE"
    echo "pinned $REPO/$FILE at $current"
    exit 0
  fi
  echo "error: $PINNED_FILE is missing." >&2
  echo "This is a hard error, not a silent self-pin: a missing pin file must never let" >&2
  echo "the guard pass having compared nothing. To create the initial pin, run:" >&2
  echo "  $0 --init" >&2
  exit 1
fi

if [[ "$init" -eq 1 ]]; then
  echo "error: $PINNED_FILE already exists; refusing to overwrite via --init." >&2
  echo "To re-pin after reviewing a real drift, write it directly:" >&2
  echo "  echo \"\$(gh api repos/$REPO/contents/$FILE --jq .sha)\" > $PINNED_FILE" >&2
  exit 1
fi

pinned="$(tr -d '[:space:]' < "$PINNED_FILE")"

if [[ "$current" == "$pinned" ]]; then
  echo "✓ $REPO/$FILE unchanged ($pinned)"
  exit 0
fi

cat <<EOF

⚠ upstream drift: $REPO/$FILE changed

  pinned:  $pinned
  current: $current

  Review:  https://github.com/$REPO/commits/main/$FILE

TONE3000 updates these definitions when the API changes, so this is an early warning,
not proof of a break. Read the diff, decide whether the SDK needs updating, run
\`make test-live\` to check the live vocabulary, then re-pin:

  echo "$current" > $PINNED_FILE

Note: types.ts is documentation and has been wrong before — it claims created_at is
absent from GET /tones/{id} and that PublicUser.id is a number, both false against the
live API. Treat it as a trigger to investigate, never a source to sync from blindly.

EOF
exit 1
