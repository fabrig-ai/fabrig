#!/bin/sh
# Keep the person's look-requests on origin/main, dropping requests whose
# Look-request names main carries, then push with the checkout's old lease.
# Implements docs/design/client/journeys.md, section 7.
#
# Takes no arguments; runs in the current checkout, never fetches requests.
# Exit status: 0 rebased and pushed; 1 git failed (a conflict leaves the
# rebase for the person); 2 refused arguments, checkout or request commits;
# 3 the remote requests branch moved beyond the saved lease. Diagnostics
# go to stderr; stdout reports each request in order and the pushed head.
set -eu
# Git's porcelain push reasons are used to distinguish a stale lease.
LC_ALL=C
export LC_ALL

refuse() {
    printf 'look-refresh: %s\n' "$*" >&2
    exit 2
}

[ "$#" -eq 0 ] || refuse 'takes no arguments'
branch=$(git symbolic-ref --quiet --short HEAD) || refuse 'check out look-requests first'
[ "$branch" = look-requests ] || refuse 'check out look-requests first'
for state in rebase-merge rebase-apply MERGE_HEAD CHERRY_PICK_HEAD sequencer; do
    [ ! -e "$(git rev-parse --git-path "$state")" ] || refuse "$state is in progress"
done
[ -z "$(git status --porcelain --untracked-files=no)" ] || refuse 'tracked files must be clean'

# Capture this before fetching anything. An absent tracking ref means that
# the remote branch must also be absent, not that any remote head is safe.
lease=$(git rev-parse --verify refs/remotes/origin/look-requests 2>/dev/null) || lease=
git fetch origin main >&2 || exit 1

scratch=$(mktemp -d "${TMPDIR:-/tmp}/temper-look-refresh.XXXXXX")
trap 'rm -rf "$scratch"' 0
trap 'exit 1' HUP INT TERM

git rev-list --reverse origin/main..look-requests > "$scratch/commits"
: > "$scratch/requests"
while IFS= read -r commit; do
    parents=$(git show -s --format=%P "$commit")
    case $parents in *' '*) refuse "$commit is a merge commit" ;; esac
    # Retain keys until counting: an empty last value must not disappear
    # when command substitution strips trailing newlines.
    trailers=$(git show -s --format='%(trailers:key=Look-request)' "$commit")
    name=$(printf '%s\n' "$trailers" | awk '
        { sub(/^[^:]*:[ \t]*/, "") }
        /^[a-z0-9][a-z0-9-]*$/ { valid++; name = $0 }
        END { if (NR != 1 || valid != 1) exit 1; print name }
    ') || refuse "$commit must carry exactly one Look-request trailer with NAME [a-z0-9][a-z0-9-]*"
    if awk -v name="$name" '$2 == name { found = 1 } END { exit !found }' "$scratch/requests"; then
        refuse "$commit repeats Look-request: $name"
    fi
    printf '%s %s\n' "$commit" "$name" >> "$scratch/requests"
done < "$scratch/commits"

# Keep a commit next to each done name, including multiple trailers on a
# landing. The prefix distinguishes hashes from even hash-shaped names.
git log --format='commit %H%n%(trailers:key=Look-request,valueonly)' origin/main > "$scratch/main"
LC_ALL=C awk '
    /^commit / { commit = $2; next }
    /^[a-z0-9][a-z0-9-]*$/ { print $0, commit }
' "$scratch/main" > "$scratch/done"
: > "$scratch/todo"
: > "$scratch/report"
while read -r commit name; do
    done_by=$(awk -v name="$name" '$1 == name { print $2; exit }' "$scratch/done")
    if [ -n "$done_by" ]; then
        printf 'drop %s\n' "$commit" >> "$scratch/todo"
        short=$(git rev-parse --short "$done_by")
        printf 'dropped %s, done on main by %s\n' "$name" "$short" >> "$scratch/report"
    else
        printf 'pick %s\n' "$commit" >> "$scratch/todo"
        printf 'kept %s\n' "$name" >> "$scratch/report"
    fi
done < "$scratch/requests"
[ -s "$scratch/todo" ] || printf 'noop\n' > "$scratch/todo"

# Supply the complete, validated todo. Keep empty/patch-equivalent requests
# too: only a landing's named trailer completes one. Never consult an editor.
base=$(git merge-base origin/main look-requests)
LOOK_REFRESH_TODO=$scratch/todo
export LOOK_REFRESH_TODO
if ! GIT_SEQUENCE_EDITOR='cp "$LOOK_REFRESH_TODO"' git \
    -c rebase.autoStash=false -c rebase.updateRefs=false -c rebase.abbreviateCommands=false \
    rebase --interactive --no-autosquash --no-autostash --no-update-refs \
    --reapply-cherry-picks --keep-empty --empty=keep --onto origin/main "$base" >&2; then
    stopped=$(git rev-parse --verify REBASE_HEAD 2>/dev/null) || stopped=
    name=$(awk -v commit="$stopped" '$1 == commit { print $2 }' "$scratch/requests")
    printf 'look-refresh: stopped at request %s; settle it, git rebase --continue, then run this script again; nothing pushed\n' "${name:-unknown}" >&2
    exit 1
fi

cat "$scratch/report"
if git push --porcelain "--force-with-lease=look-requests:$lease" origin look-requests > "$scratch/push" 2>&1; then
    cat "$scratch/push" >&2
else
    cat "$scratch/push" >&2
    if awk '/\[rejected\].*\(stale info\)/ { stale = 1 } END { exit !stale }' "$scratch/push"; then
        printf 'look-refresh: remote look-requests moved elsewhere; fetch it and settle it by hand before running again; branch left rebased\n' >&2
        exit 3
    fi
    exit 1
fi
printf 'pushed look-requests at %s\n' "$(git rev-parse --short HEAD)"
