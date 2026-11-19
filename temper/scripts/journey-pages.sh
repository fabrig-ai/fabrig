#!/bin/sh
# The journeys' one writer: docs/design/client/journey-writer.md, section 4.
# Runs in its own checkout; keeps its lock and last task or commit in the git directory.
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

started=$(date +%s)
project=
budget=
allow=
prepared=
branch=
push=
within=
while [ "$#" -gt 0 ]; do
    case "$1" in
        --project | --budget | --prepared | --branch | --push | --within)
            option=$1
            [ "$#" -ge 2 ] && [ -n "$2" ] || refuse "$option needs a value"
            case "$2" in --*) refuse "$option needs a value" ;; esac
            case "$option" in
                --project) [ -z "$project" ] || refuse 'repeated --project'; project=$2 ;;
                --budget) [ -z "$budget" ] || refuse 'repeated --budget'; budget=$2 ;;
                --prepared) [ -z "$prepared" ] || refuse 'repeated --prepared'; prepared=$2 ;;
                --branch) [ -z "$branch" ] || refuse 'repeated --branch'; branch=$2 ;;
                --push) [ -z "$push" ] || refuse 'repeated --push'; push=$2 ;;
                --within) [ -z "$within" ] || refuse 'repeated --within'; within=$2 ;;
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
if [ -n "$project" ]; then
    [ -n "$budget$allow" ] || refuse 'needs --budget AMOUNT or --as-allowed'
    [ -z "$budget" ] || [ -z "$allow" ] || refuse 'use --budget or --as-allowed, not both'
else
    [ -z "$budget$allow" ] || refuse '--budget and --as-allowed need --project'
fi
if [ -n "$within" ]; then
    case "$within" in
        *s | *m | *h) digits=${within%?} ;;
        *) refuse '--within needs digits then s, m or h' ;;
    esac
    case "$digits" in '' | *[!0-9]*) refuse '--within needs digits then s, m or h' ;; esac
fi
force=
[ -z "$project" ] || [ -n "$branch" ] || force=yes
branch=${branch:-journey-pages}
git check-ref-format --branch "$branch" >/dev/null 2>&1 || refuse "invalid branch: $branch"
if [ -n "$push" ]; then
    git remote get-url "$push" >/dev/null 2>&1 || refuse "not a configured remote: $push"
elif [ -n "$project" ]; then
    push=origin
fi

root=$(git rev-parse --show-toplevel) || refuse 'run in the writer checkout'
cd "$root"
task_file=$(git rev-parse --git-path journey-pages) || exit 1
lock=$(git rev-parse --git-path journey-pages.lock) || exit 1
if ! mkdir "$lock" 2>/dev/null; then
    holder=$(cat "$lock/pid" 2>/dev/null) || refuse "lock $lock has no process number"
    case "$holder" in '' | 0 | *[!0-9]*) refuse "lock $lock has invalid process number: $holder" ;; esac
    if kill -0 "$holder" 2>/dev/null; then
        refuse "lock $lock held by process $holder"
    fi
    rm -rf "$lock" || refuse "cannot take over lock $lock from process $holder"
    mkdir "$lock" 2>/dev/null || refuse "lock $lock taken while replacing process $holder"
fi
scratch=
trap 'rm -rf "$lock"; [ -z "$scratch" ] || rm -rf "$scratch"' 0
trap 'exit 1' HUP INT TERM
printf '%s\n' "$$" > "$lock/pid"
scratch=$(mktemp -d "${TMPDIR:-/tmp}/temper-journey-pages.XXXXXX") || exit 1

if [ -e "$task_file" ]; then
    offer=$(python3 -I -B - "$task_file" <<'PY'
import re
import sys

with open(sys.argv[1], encoding="utf-8") as source:
    offer = source.read()
if not re.fullmatch(r"(?:task [0-9]+|commit (?:[0-9a-f]{40}|[0-9a-f]{64}))\n", offer):
    sys.exit(f"journey-pages: cannot read kept offer in {sys.argv[1]}")
print(offer, end="")
PY
    ) || exit 1
    case "$offer" in
        'task '*)
            task=${offer#task }
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
            ;;
        'commit '*)
            kept=${offer#commit }
            git cat-file -e "$kept^{commit}" 2>/dev/null || fail "kept commit $kept is missing ($task_file)"
            kept_tip=$(git rev-parse "$kept^") || fail "cannot read parent of kept commit $kept"
            git fetch origin main >&2 || exit 1
            held=
            if git merge-base --is-ancestor "$kept" origin/main; then
                held=yes
            else
                git log --format=%B origin/main > "$scratch/messages" || exit 1
                if grep -Fx "Printed: $kept_tip" "$scratch/messages" >/dev/null; then
                    held=yes
                fi
            fi
            if [ -z "$held" ]; then
                printf 'waiting on %s\n' "$kept"
                exit 0
            fi
            rm "$task_file"
            ;;
    esac
fi

for state in rebase-merge rebase-apply MERGE_HEAD CHERRY_PICK_HEAD sequencer; do
    [ ! -e "$(git rev-parse --git-path "$state")" ] || refuse "$state is in progress"
done
tracked=$(git status --porcelain --untracked-files=no) || exit 1
[ -z "$tracked" ] || refuse 'tracked files must be clean'
git fetch origin main >&2 || exit 1
git checkout -B "$branch" origin/main >&2 || exit 1
tip=$(git rev-parse HEAD) || exit 1

moved=
for walk in temper/docs/design/client/web/journeys/*/walk temper/docs/design/client/tui/journeys/*/walk; do
    [ -f "$walk" ] || continue
    first=
    IFS= read -r first < "$walk" || :
    [ "$first" != 'walk 2' ] || moved=yes
done
: > "$scratch/print"
walk_printed=0
if [ -n "$moved" ]; then
    {
        status=0
        TEMPER_INTERNAL_BUNDLE=none cargo run --locked -p temper-client-walk -- pages || status=$?
        printf '%s\n' "$status" > "$scratch/exit"
    } | tee -a "$scratch/print"
    walk_printed=$(cat "$scratch/exit")
    case "$walk_printed" in 0 | 1) ;; *) exit 1 ;; esac
fi

set -- run --locked -p temper-client-journeys -- pages
[ -z "$prepared" ] || set -- "$@" --prepared "$prepared"
if [ -n "$within" ]; then
    rest=$(python3 -I -B - "$within" "$started" "$(date +%s)" <<'PY'
import sys

duration, start, now = sys.argv[1:]
seconds = int(duration[:-1]) * {"s": 1, "m": 60, "h": 3600}[duration[-1]]
print(f"{max(0, seconds - (int(now) - int(start)))}s")
PY
    ) || exit 1
    set -- "$@" --within "$rest"
fi
{
    printed=0
    TEMPER_INTERNAL_BUNDLE=none cargo "$@" || printed=$?
    printf '%s\n' "$printed" > "$scratch/exit"
} | tee -a "$scratch/print"
printed=$(cat "$scratch/exit")
case "$printed" in 0 | 1) ;; *) exit 1 ;; esac
[ "$walk_printed" != 1 ] || printed=1

# NUL records keep spaces, quoting and newlines in paths unambiguous. Disabling
# rename detection checks both the deleted and added paths against the same set.
git status --porcelain -z --untracked-files=all --no-renames > "$scratch/status" || exit 1
python3 -I -B - "$scratch/status" "$scratch/paths" <<'PY'
import re
import sys

# The written set, tools/pre-pr/design.md, section 4, check 8.
written = re.compile(
    rb"temper/docs/design/client/(?:"
    rb"web/journeys/(?:[^/]+/(?:recording|[0-9]{2,}\.html)|skein-ui\.css|journey\.js)"
    rb"|tui/journeys/(?:[^/]+/(?:recording|[0-9]{2,}\.(?:html|txt))|theme\.css|tui-journey\.js))"
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
awk '/^stopped / { sub(/; scratch kept at .*/, "") } /^walked again / || /^stopped / { print }' "$scratch/print" > "$scratch/body"
[ ! -s "$scratch/body" ] || printf '\n' >> "$scratch/body"
printf 'Printed: %s\n' "$tip" >> "$scratch/body"
body=$(cat "$scratch/body")
printf '%s\n\n%s\n' "$subject" "$body" > "$scratch/message"
git --literal-pathspecs add -A --pathspec-from-file="$scratch/paths" --pathspec-file-nul >&2 || exit 1
git -c commit.cleanup=verbatim commit -F "$scratch/message" >&2 || exit 1
commit=$(git rev-parse HEAD) || exit 1
if [ -n "$push" ]; then
    if [ -n "$force" ]; then
        git push --force "$push" "HEAD:refs/heads/$branch" >&2 || exit 1
    else
        remote=$(git ls-remote "$push" "refs/heads/$branch") || exit 1
        remote_commit=$(printf '%s\n' "$remote" | cut -f1)
        [ -z "$remote_commit" ] || [ "$remote_commit" = "$commit" ] || fail "$push holds $branch at $remote_commit"
        git push "$push" "HEAD:refs/heads/$branch" >&2 || exit 1
    fi
fi

if [ -n "$project" ]; then
    set -- ctl change open --project "$project" --branch "$branch" --at "$commit" --title "$subject" --body "$body"
    if [ -n "$allow" ]; then
        set -- "$@" --as-allowed
    else
        set -- "$@" --budget "$budget"
    fi
    set -- "$@" --key "journey-pages-$tip"
    answer=$(temper "$@" </dev/null) || exit 1
    task=$(printf '%s\n' "$answer" | awk 'NR == 1 && /^task [0-9]+$/ { number = $2 } END { if (NR != 1 || number == "") exit 1; print number }') || fail 'cannot read change open answer'
    printf 'task %s\n' "$task" > "$task_file"
else
    printf 'commit %s\n' "$commit" > "$task_file"
fi
printf 'wrote %s %s\n' "$branch" "$commit"
exit "$printed"
