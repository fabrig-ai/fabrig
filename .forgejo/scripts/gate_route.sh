#!/bin/sh
# A selector returns 0 for a narrow pass, 1 for the regular Gate, or 2 for a failed check.
set -eu
status=0
"$@" || status=$?
case "$status" in
    0) exit 0 ;;
    1) exec "$CI_GATE" "$CANDIDATE" "$BASE" ;;
    2) exit 1 ;;
    *) echo "Gate selection failed (exit $status)" >&2; exit 1 ;;
esac
