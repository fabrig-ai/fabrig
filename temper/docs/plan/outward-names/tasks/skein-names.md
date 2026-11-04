+++
id = "skein-names"
part = "skein"
kind = "docs"
needs = []
files = [
  "skein/crates/skein-llm/README.md",
  "skein/docs/development/debt/reviews/2026-10-31-skein-areas.md",
  "skein/docs/development/debt/reviews/2026-10-31-skein-first-areas.md",
  "skein/docs/development/debt/reviews/2026-10-31-skein-first.md",
  "skein/docs/development/debt/reviews/2026-10-31-skein.md",
  "skein/tests/codec/tests/suite/diff.rs",
  "skein/tests/codec/tests/suite/patch.rs",
  "skein/tests/forge/conformance/README.md",
  "skein/tests/forge/github/README.md",
  "skein/tests/oauth/tests/suite/drift.rs",
]
effort = "medium"
minutes = 30
+++

# skein's files stop naming the lab

## Goal

fabrig names nothing of whoever runs its development (`AGENTS.md`, 1). Ten of skein's files still
show the lab's build command (`heavy cargo ...`) or its scratch checkout (`~/src/fabrig/...`).
This task says each in plain or generic words, and changes nothing else.

Acceptance:

- The ten files of `files` hold none of the lab's names: the check of `notes/common.md`
  ("Checking your files are clean"), with exactly the ten entries below deleted, passes
  (1 selected); the inventory is then restored and not committed.
- Each change is a word change: the four test files' hint strings and the READMEs' commands lose
  `heavy ` and keep the rest byte for byte; the reviews keep their findings, dates and commits.
- `heavy cargo clippy -p skein-codec-tests -p skein-oauth-world --all-targets -- -D warnings`
  passes; the tests of `## Steps`, 3 pass.
- `python3 -I -B skein/scripts/old_forms.py` raises no count; `python3 -I -B
  tools/citations/citations.py --all` prints `citations: 0 broken in HEAD`.

## Inputs

- `temper/docs/plan/outward-names/notes/common.md`: the standing rules.
- `temper/docs/plan/outward-names/notes/contracts.md`, 3: the wording.
- `temper/docs/plan/outward-names/inputs/request.md`: the request (How, Bounds).
- `tests/boundary/tests/outward.rs`: the check, its module doc and `FIXED`.
- `skein/docs/design/fake-forge.md` already says its fixtures' directory generically (this plan's
  lead changed it): you leave it as it is.

## Owns and may change

- **The ten files of `files`**, only on the lines that hold a name:
  - `skein/crates/skein-llm/README.md` lines ~103 and ~120: `heavy cargo nextest` → `cargo nextest`.
  - `skein/tests/forge/conformance/README.md` lines ~12, ~16, ~77, ~90 and
    `skein/tests/forge/github/README.md` line ~6: `heavy cargo run` → `cargo run`.
  - `skein/tests/codec/tests/suite/diff.rs` ~48, `patch.rs` ~378 and
    `skein/tests/oauth/tests/suite/drift.rs` ~7: the hint string's `heavy cargo nextest` →
    `cargo nextest`.
  - The four reviews under `skein/docs/development/debt/reviews/`, each one line: in
    `2026-10-31-skein-areas.md` ~1635 "has no caller anywhere in `~/src/fabrig/main`" → "has no
    caller anywhere in the primary checkout"; in `2026-10-31-skein-first-areas.md` ~11 "never
    touch `~/src/fabrig/*`" → "never touch another checkout"; in `2026-10-31-skein-first.md` ~3
    "The primary checkout `~/src/fabrig/main` is at" → "The primary checkout is at"; in
    `2026-10-31-skein.md` ~630 "the primary checkout `~/src/fabrig/main` was moved" → "the
    primary checkout was moved".
- **If a line moved** at your base, the name is changed where it stands. **If a file holds a
  name** this spec does not list, it is changed by `notes/contracts.md`, 3, and the report says
  where. **If the words above read badly** in their sentence, reword within that sentence only,
  keeping its claim.
- Nothing else: not `tests/boundary/outward.txt` (the last task lowers it), not any
  `docs/design/` file.

## Steps

1. Make the changes of `## Owns and may change`. `git diff --stat` shows the ten files, one to
   four lines each.
2. Check the files clean, as `notes/common.md` says, deleting these ten entries of
   `tests/boundary/outward.txt` in the working tree: `skein/crates/skein-llm/README.md`, the four
   reviews, `skein/tests/codec/tests/suite/diff.rs`, `skein/tests/codec/tests/suite/patch.rs`,
   `skein/tests/forge/conformance/README.md`, `skein/tests/forge/github/README.md`,
   `skein/tests/oauth/tests/suite/drift.rs`. Restore it.
3. Tests, the narrowest that compile and read the changed strings:
   `heavy cargo nextest run -p skein-codec-tests --test suite -E 'test(/^diff::/) | test(/^patch::/)'`
   and `heavy cargo nextest run -p skein-oauth-world --test suite -E 'test(/^drift::/)'`. Check
   each count is above zero. The READMEs are read by no test (`.config/gate/read-by-tests.txt`).
4. `cargo fmt`; clippy as in Acceptance; old forms; commit; citations; push.

## Report

As the implementer's text says, and: each changed line, before and after (one line each); the
check of step 2 and its selected count; the test counts of step 3; anything under `## Left`.
