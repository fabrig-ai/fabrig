+++
id = "settings-pre-pr"
part = "settings-reference"
kind = "docs"
needs = []
files = [".temper/pre-pr", "temper/docs/design/protocol/configuration.status.md"]
effort = "medium"
minutes = 25
+++

# settings-pre-pr: the repository's checks at delivery, first edition

## Goal

fabrig's repository carries its project's checks as `.temper/pre-pr` (smith's
`domain/run.md`, 8.1: an executable at the path the conventions name, temper's being
`.temper/pre-pr`, run in each writable directory as a run delivers). Its first edition runs
what needs no build: the format check and the counted baselines (the old-form counts). The
lints and the changed packages' focused tests are left out: they build in the box's worker,
whose budget and timeouts are the user's (the plan's question `pre-pr-checks`).

Acceptance:

- `.temper/pre-pr` exists at the repository's root, mode 0755, a POSIX `sh` script:

  ```sh
  #!/bin/sh
  # The project's checks at delivery (temper/docs/design/protocol/configuration.md, 13.9;
  # smith/docs/design/domain/run.md, 8.1): the format check and the counted baselines.
  set -eu
  cd "$(dirname "$0")/.."
  cargo fmt --all --check
  for tree in skein smith jig temper; do
      (cd "$tree" && python3 -I -B scripts/old_forms.py)
  done
  ```

  (the shape; keep it this small; the comment's citations are the design's).
- Run from the repository's root on the task's base, it exits 0.
- With one Rust file misformatted (a throwaway edit, undone before the commit), it exits
  non-zero, `cargo fmt` naming the file; with the edit undone it exits 0 again.
- Run from another directory (`sh /path/to/.temper/pre-pr` from `/tmp`), it checks the
  repository it sits in.

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.9 (the bullet "What the repository
  carries"); `smith/docs/design/domain/run.md`, 8.1 (the first bullet).
- `temper/docs/design/protocol/configuration.status.md`: its **Not yet** entry "13.9: the
  project's checks: there is no `.temper/pre-pr`.", which this task removes; the entry after
  it ("13.9: `.temper/pre-pr` runs no lints and no package's focused tests.") stays.
- `temper/docs/plan/reference-m1/notes/common.md`.

## Owns and may change

- Owns `.temper/pre-pr` (new) and the one status sentence above.
- No other file. If `cargo fmt --all --check` reports files on the base already
  misformatted, do not format them: report each file under `## Left` and finish.
- If an old-forms script needs an argument or a working directory other than its tree's
  root, follow what `tools/old_forms/README.md` says and name the change in the report.

## Steps

1. Write the script; `chmod 0755`; `git add --chmod=+x .temper/pre-pr` so the mode is
   tracked.
2. Run it as the acceptance says, three times (clean, misformatted, from `/tmp`); it builds
   nothing, so it runs directly, not through `heavy`.
3. Remove the status sentence; `python3 -I -B tools/citations/citations.py --base
   origin/main`.
4. Commit ("Give the repository its first project checks at delivery") and push.

## Report

The implementer's text's report, plus each run's exit code and how long the clean run took
(`time`).
