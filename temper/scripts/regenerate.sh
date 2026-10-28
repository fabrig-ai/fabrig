#!/bin/sh
# Rewrite generated files from the tree's sources, for agents' land and
# land-set when a rebase conflicts in them (agents'
# docs/design/landing-flow.md, 3).
#
#   scripts/regenerate.sh --owns PATH...  0 when it writes every PATH,
#                                         else 1; builds nothing
#   scripts/regenerate.sh PATH...         rewrite them from the tree
#
# It owns the printed journeys under docs/design/client/journeys/ and
# reprints them. A writer that would move anything else is refused:
# the tree goes back as it was.
#
# Exit status: 0 rewritten, printing every path it changed on stdout,
# one per line; 1 (--owns) a path it doesn't write; 2 refused before
# starting: an unknown path, or the worktree not clean; 3 refused: a
# writer would move a path it doesn't own, named on stderr, and the
# tree is as it was; other: a writer or its build failed.
set -eu

cd "$(dirname "$0")/.."

# One table: writer, directory, package, test binary (lib or run), filter.
# Journeys writes skein-ui.css, journey.js and NAME/NN.html under its root.
writers() {
    cat <<'TABLE'
journeys|docs/design/client/journeys|temper-client-journeys|run|
TABLE
}

# Ownership is by place, unless it is a symlink. Keep this rule in sync
# with Python's owns() below.
owns() {
    [ ! -L "$1" ] || return 1
    if printf '%s\n' "$1" | LC_ALL=C awk '
        /^docs\/design\/client\/journeys\/(skein-ui\.css|journey\.js|[a-z][a-z0-9-]*\/[0-9][0-9]+\.html)$/ { owned = 1 }
        END { exit !(NR == 1 && owned) }
    '; then
        writer=journeys
        return 0
    fi
    return 1
}

mode='write'
if [ "${1-}" = --owns ]; then
    mode=owns
    shift
fi
if [ "$#" -eq 0 ]; then
    [ "$mode" = owns ] && exit 1
    echo 'regenerate: expected at least one generated path' >&2
    exit 2
fi
needed=' '
for path do
    if ! owns "$path"; then
        [ "$mode" = owns ] && exit 1
        printf 'regenerate: unknown generated path: %s\n' "$path" >&2
        exit 2
    fi
    if [ "$mode" = write ] && ! {
        [ -f "$path" ] && git ls-files --error-unmatch -- ":(literal)$path" >/dev/null 2>&1
    }; then
        printf 'regenerate: expected existing tracked generated file: %s\n' "$path" >&2
        exit 2
    fi
    needed="$needed$writer "
done
[ "$mode" = owns ] && exit 0
if [ -n "$(git status --porcelain --untracked-files=all)" ]; then
    echo 'regenerate: worktree must be clean (tracked and untracked files)' >&2
    exit 2
fi

# The writers need no browser build. Diagnostics and cargo's output stay on
# stderr; stdout is only the final list of generated files changed.
export TEMPER_BUNDLE=none NEXTEST_DOUBLE_SPAWN=0 SKEIN_CODEGEN_WRITE=1
writer_exit=0
writers | while IFS='|' read -r name directory package binary filter; do
    case $needed in *" $name "*) ;; *) continue ;; esac
    case ${ran-} in *" $name "*) continue ;; esac
    ran="${ran-} $name "
    if [ "$binary" = run ]; then
        cargo run --locked -p "$package" >&2 || exit "$?"
    elif [ "$binary" = lib ]; then
        cargo nextest run --locked -p "$package" --lib -E "$filter" >&2 || exit "$?"
    else
        cargo nextest run --locked -p "$package" --test "$binary" -E "$filter" >&2 || exit "$?"
    fi
done || writer_exit=$?

# NUL-delimited status handles even a writer's unexpected file names. Use
# the ownership rule to admit new and departed journeys as well as edits.
python3 -I -B - "$writer_exit" <<'PY'
import difflib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

writer_exit = int(sys.argv[1])


def git(*args):
    return subprocess.check_output(['git', *args])


entries = git('status', '--porcelain=v1', '-z', '--untracked-files=all', '--no-renames').split(b'\0')
paths = sorted({os.fsdecode(entry[3:]) for entry in entries if entry})


def owns(path):
    # Same ownership rule as the shell's owns(): place and no symlink,
    # regardless of presence or tracking; write inputs are checked separately.
    journey = re.fullmatch(r'docs/design/client/journeys/(skein-ui\.css|journey\.js|[a-z][a-z0-9-]*/[0-9]{2,}\.html)', path)
    return journey and not Path(path).is_symlink()


def values(before, after, at=''):
    if isinstance(before, dict) and isinstance(after, dict):
        for key in sorted(before.keys() | after.keys()):
            yield from values(before.get(key), after.get(key), f'{at}.{key}' if at else key)
    elif isinstance(before, list) and isinstance(after, list):
        for index in range(max(len(before), len(after))):
            yield from values(before[index] if index < len(before) else None,
                              after[index] if index < len(after) else None, f'{at}[{index}]')
    elif before != after:
        yield f'  {at}: {before!r} -> {after!r}'


others = [path for path in paths if not owns(path)]
for path in others:
    print(f'regenerate: refused change outside generated files: {path}', file=sys.stderr)
    before = subprocess.run(['git', 'show', f'HEAD:{path}'], capture_output=True)
    if before.returncode or not Path(path).is_file() or Path(path).is_symlink():
        continue
    old = before.stdout.decode('utf-8', errors='replace')
    new = Path(path).read_text(errors='replace')
    if path.endswith('.json'):
        try:
            for line in values(json.loads(old), json.loads(new)):
                print(line, file=sys.stderr)
            continue
        except ValueError:
            pass
    # For Rust literals (and other text), show the old and new source lines.
    for line in difflib.unified_diff(old.splitlines(), new.splitlines(),
                                     fromfile='old/' + path, tofile='new/' + path):
        print(line, file=sys.stderr)

if others or writer_exit:
    # Entry required a clean index and worktree, so HEAD is the complete
    # snapshot. Remove newly written files as well as restoring tracked ones.
    subprocess.run(['git', 'restore', '--source=HEAD', '--staged', '--worktree', '--', '.'], check=True)
    subprocess.run(['git', 'clean', '-fd', '--', '.'], check=True, stdout=sys.stderr)
    sys.exit(3 if others else writer_exit)
for path in paths:
    print(path)
PY
