#!/bin/sh
# Rewrite generated files from the tree's sources, for agents' land and
# land-set when a rebase conflicts in them (agents'
# docs/design/landing-flow.md, 3).
#
#   scripts/regenerate.sh --owns PATH...  0 when it writes every PATH,
#                                         else 1; builds nothing
#   scripts/regenerate.sh PATH...         rewrite them from the tree
#
# It owns the exact memory figures, crates/temper/tests/figures/memory/,
# and reruns their writers with SKEIN_CODEGEN_WRITE=1. A writer that
# would move anything else, a declaration (memory_bytes in
# docs/examples/owned.json or temper init's template), a literal in a
# source, is refused: the tree goes back as it was.
#
# Exit status: 0 rewritten, printing every path it changed on stdout,
# one per line; 1 (--owns) a path it doesn't write; 2 refused before
# starting: an unknown path, or the worktree not clean; 3 refused: a
# writer would move a path it doesn't own, named on stderr, and the
# tree is as it was; other: a writer or its build failed.
set -eu

cd "$(dirname "$0")/.."

# One table: writer, figure directory, package, test binary (or lib), filter.
# Owned precedes init (protocol/setup.md, 8.1); turn_bounds is deliberately
# absent: changing a limit or declaration needs a person's decision.
writers() {
    cat <<'TABLE'
owned|owned/engine|temper|startup|test(=owned_memory_figures_match_startup_requirements_at_bounded_strings)
owned|owned/worker|temper|startup|test(=owned_memory_figures_match_startup_requirements_at_bounded_strings)
init|init/engine|temper|startup|test(=owned_memory_figures_for_init_match_its_own_document)
init|init/worker|temper|startup|test(=owned_memory_figures_for_init_match_its_own_document)
worker|worker-alone/worker|temper|startup|test(=owned_memory_figures_for_a_worker_alone_include_its_process_and_no_engine)
view|client-view|temper-client-view|lib|test(account_bounds_have_a_literal_price) | test(english_phrase_declarations_match_the_vocabulary)
web|client-web|temper-client-shell-web|lib|test(the_web_preset_has_a_literal_price)
TABLE
}

prefix=crates/temper/tests/figures/memory
# Ownership is by place: a .txt directly in a registered directory, unless
# it is a symlink. Keep this rule in sync with Python's figure() below.
owns() {
    case $1 in
        "$prefix"/*.txt) ;;
        *) return 1 ;;
    esac
    directory=${1%/*}
    directory=${directory#"$prefix"/}
    writer=$(writers | awk -F '|' -v directory="$directory" '$2 == directory { print $1 }')
    [ -n "$writer" ] && [ ! -L "$1" ]
}

mode='write'
if [ "${1-}" = --owns ]; then
    mode=owns
    shift
fi
if [ "$#" -eq 0 ]; then
    [ "$mode" = owns ] && exit 1
    echo 'regenerate: expected at least one figure path' >&2
    exit 2
fi
needed=' '
for path do
    if ! owns "$path"; then
        [ "$mode" = owns ] && exit 1
        printf 'regenerate: unknown figure: %s\n' "$path" >&2
        exit 2
    fi
    if [ "$mode" = write ] && ! {
        [ -f "$path" ] && git ls-files --error-unmatch -- ":(literal)$path" >/dev/null 2>&1
    }; then
        printf 'regenerate: expected existing tracked figure: %s\n' "$path" >&2
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
# stderr; stdout is only the final list of figures changed.
export TEMPER_BUNDLE=none NEXTEST_DOUBLE_SPAWN=0 SKEIN_CODEGEN_WRITE=1
writer_exit=0
writers | while IFS='|' read -r name directory package binary filter; do
    case $needed in *" $name "*) ;; *) continue ;; esac
    case ${ran-} in *" $name "*) continue ;; esac
    ran="${ran-} $name "
    if [ "$binary" = lib ]; then
        cargo nextest run --locked -p "$package" --lib -E "$filter" >&2 || exit "$?"
    else
        cargo nextest run --locked -p "$package" --test "$binary" -E "$filter" >&2 || exit "$?"
    fi
done || writer_exit=$?

# NUL-delimited status handles even a writer's unexpected file names. Use
# the same table to admit new terms and departed terms as well as edits.
directories=$(writers | cut -d '|' -f 2)
python3 -I -B - "$writer_exit" "$prefix" "$directories" <<'PY'
import difflib
import json
import os
from pathlib import Path
import subprocess
import sys

writer_exit = int(sys.argv[1])
directories = {sys.argv[2] + '/' + directory for directory in sys.argv[3].splitlines()}


def git(*args):
    return subprocess.check_output(['git', *args])


entries = git('status', '--porcelain=v1', '-z', '--untracked-files=all', '--no-renames').split(b'\0')
paths = sorted({os.fsdecode(entry[3:]) for entry in entries if entry})


def figure(path):
    # Same ownership rule as the shell's owns(): place and no symlink,
    # regardless of presence or tracking; write inputs are checked separately.
    parent, _, name = path.rpartition('/')
    return parent in directories and name.endswith('.txt') and not Path(path).is_symlink()


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


others = [path for path in paths if not figure(path)]
for path in others:
    print(f'regenerate: refused change outside figures: {path}', file=sys.stderr)
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
