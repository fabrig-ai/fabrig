#!/bin/sh
# The journeys' one writer: docs/design/client/journey-writer.md, section 4.
# Runs in its own checkout; keeps the opened change's task in the git directory.
set -eu
LC_ALL=C
export LC_ALL

refuse() {
    printf 'journey-pages: %s\n' "$*" >&2
    exit 2
}
fail() {
    printf 'journey-pages: %s\n' "$*" >&2
    exit 1
}

project=
budget=
allow=
prepared=
while [ "$#" -gt 0 ]; do
    case "$1" in
        --project | --budget | --prepared)
            option=$1
            [ "$#" -ge 2 ] && [ -n "$2" ] || refuse "$option needs a value"
            case "$2" in --*) refuse "$option needs a value" ;; esac
            case "$option" in
                --project) [ -z "$project" ] || refuse 'repeated --project'; project=$2 ;;
                --budget) [ -z "$budget" ] || refuse 'repeated --budget'; budget=$2 ;;
                --prepared) [ -z "$prepared" ] || refuse 'repeated --prepared'; prepared=$2 ;;
            esac
            shift 2
            ;;
        --as-allowed)
            [ -z "$allow" ] || refuse 'repeated --as-allowed'
            allow=yes
            shift
            ;;
        *) refuse "unknown argument: $1" ;;
    esac
done
[ -n "$project" ] || refuse 'needs --project PROJECT'
[ -n "$budget$allow" ] || refuse 'needs --budget AMOUNT or --as-allowed'
[ -z "$budget" ] || [ -z "$allow" ] || refuse 'use --budget or --as-allowed, not both'

root=$(git rev-parse --show-toplevel) || refuse 'run in the writer checkout'
cd "$root"
task_file=$(git rev-parse --git-path journey-pages)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/temper-journey-pages.XXXXXX")
trap 'rm -rf "$scratch"' 0
trap 'exit 1' HUP INT TERM

if [ -e "$task_file" ]; then
    task=$(cat "$task_file")
    case "$task" in '' | *[!0-9]*) fail 'cannot read the kept task number' ;; esac
    temper ctl task show "$task" --json </dev/null > "$scratch/task.json" || exit 1
    # task show is a compound read; its task document's chip carries the phase.
    phase=$(python3 -I -B - "$scratch/task.json" <<'PY'
import json
import sys

try:
    with open(sys.argv[1], encoding="utf-8") as source:
        phase = json.load(source)["task"]["chip"]["phase"]
    if phase not in (
        "waiting", "idle", "due", "preparing", "claimed", "running",
        "backing_off", "active", "closing", "held",
        "ended", "done", "failed", "cancelled",
    ):
        raise ValueError("unknown phase")
except (OSError, ValueError, KeyError, TypeError):
    sys.exit("journey-pages: cannot read the task document's phase")
print(phase)
PY
    ) || exit 1
    case "$phase" in
        ended | done | failed | cancelled) rm "$task_file" ;;
        *) printf 'waiting on change %s\n' "$task"; exit 0 ;;
    esac
fi

for state in rebase-merge rebase-apply MERGE_HEAD CHERRY_PICK_HEAD sequencer; do
    [ ! -e "$(git rev-parse --git-path "$state")" ] || refuse "$state is in progress"
done
[ -z "$(git status --porcelain --untracked-files=no)" ] || refuse 'tracked files must be clean'
git fetch origin main >&2 || exit 1
git checkout -B journey-pages origin/main >&2 || exit 1
tip=$(git rev-parse HEAD)

set -- run --locked -p temper-client-journeys -- pages
[ -z "$prepared" ] || set -- "$@" --prepared "$prepared"
printed=0
TEMPER_INTERNAL_BUNDLE=none cargo "$@" > "$scratch/print" || printed=$?
cat "$scratch/print"
case "$printed" in 0 | 1) ;; *) exit 1 ;; esac

# NUL records keep spaces, quoting and newlines in paths unambiguous. Disabling
# rename detection checks both the deleted and added paths against the same set.
git status --porcelain -z --untracked-files=all --no-renames > "$scratch/status"
python3 -I -B - "$scratch/status" "$scratch/paths" <<'PY'
import re
import sys

# The written set, tools/pre-pr/design.md, section 4, check 8.
written = re.compile(
    rb"temper/docs/design/client/(?:web|tui)/journeys/"
    rb"(?:[^/]+/(?:recording|[0-9]{2,}\.(?:html|txt))"
    rb"|skein-ui\.css|journey\.js|theme\.css|tui-journey\.js)"
)
with open(sys.argv[1], "rb") as source:
    entries = source.read().split(b"\0")
with open(sys.argv[2], "wb") as target:
    for entry in entries:
        if not entry:
            continue
        path = entry[3:]
        if not written.fullmatch(path):
            sys.exit(f"journey-pages: outside the written files: {path!r}")
        target.write(path + b"\0")
PY

if [ ! -s "$scratch/paths" ]; then
    printf "main's pages are %s's print\n" "$tip"
    exit "$printed"
fi

short=$(printf '%.12s' "$tip")
subject="Journey pages printed at $short"
# The commit and the opened change keep the same report and final paragraph.
awk '/^walked again / || /^stopped / { print }' "$scratch/print" > "$scratch/body"
[ ! -s "$scratch/body" ] || printf '\n' >> "$scratch/body"
printf 'Printed: %s\n' "$tip" >> "$scratch/body"
body=$(cat "$scratch/body")
printf '%s\n\n%s\n' "$subject" "$body" > "$scratch/message"
git --literal-pathspecs add -A --pathspec-from-file="$scratch/paths" --pathspec-file-nul
git -c commit.cleanup=verbatim commit -F "$scratch/message" >&2 || exit 1
commit=$(git rev-parse HEAD)
git push --force origin HEAD:refs/heads/journey-pages >&2 || exit 1

set -- ctl change open --project "$project" --branch journey-pages --at "$commit" --title "$subject" --body "$body"
if [ -n "$allow" ]; then
    set -- "$@" --as-allowed
else
    set -- "$@" --budget "$budget"
fi
set -- "$@" --key "journey-pages-$tip"
answer=$(temper "$@" </dev/null) || exit 1
task=$(printf '%s\n' "$answer" | awk 'NR == 1 && /^task [0-9]+$/ { number = $2 } END { if (NR != 1 || number == "") exit 1; print number }') || fail 'cannot read change open answer'
printf '%s\n' "$task" > "$task_file"
printf '%s\n' "$answer"
exit "$printed"
