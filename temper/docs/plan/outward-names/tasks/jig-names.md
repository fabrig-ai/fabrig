+++
id = "jig-names"
part = "jig"
kind = "feature"
needs = []
files = [
  "jig/docs/development/debt/reviews/2026-10-31-jig-quality-run.md",
  "jig/scripts/live-sign-in.sh",
  "jig/tests/client-browser/src/lib.rs",
  "jig/tests/client-wire/tests/suite/placed_holds.rs",
  "jig/tests/codec-families.rs",
  "jig/tests/protocol-accounts/tests/suite/drift.rs",
  "jig/tests/worker-wire/tests/suite/drift.rs",
]
effort = "medium"
minutes = 35
+++

# jig's files stop naming the lab

## Goal

fabrig names nothing of whoever runs its development (`AGENTS.md`, 1). jig's live sign-in script
looks for the lab's build tool on `PATH`; five test files show its command in hint strings; a
review names its machine and scratch checkout. The script runs plain cargo, or the wrapper its
caller names in `JIG_WRAPPER`; the rest says it in plain or generic words.

Acceptance:

- `jig/scripts/live-sign-in.sh` follows `notes/contracts.md`, 1 with `JIG_WRAPPER`: no
  `command -v heavy`, no `$heavy`; its one build command runs as
  `${JIG_WRAPPER:+"$JIG_WRAPPER"} cargo nextest run -p jig-test --profile live --no-run`;
  nothing else in it changes. `bash -n` passes, and `shellcheck` too where installed.
- The five test files' hint strings lose `heavy ` and keep the rest byte for byte; the review
  keeps its findings.
- The check of `notes/common.md` ("Checking your files are clean"), with the seven entries of
  `## Steps`, 3 deleted, passes (1 selected); the inventory is then restored and not committed.
- `heavy cargo clippy -p jig-client-browser-world -p jig-client-wire-world -p
  jig-store-codecs-world -p jig-protocol-accounts-world -p jig-worker-wire-world --all-targets
  -- -D warnings` passes; the tests of `## Steps`, 4 pass.
- `python3 -I -B jig/scripts/old_forms.py` raises no count; `python3 -I -B
  tools/citations/citations.py --all` prints `citations: 0 broken in HEAD`.

## Inputs

- `temper/docs/plan/outward-names/notes/common.md`: the standing rules.
- `temper/docs/plan/outward-names/notes/contracts.md`, 1 (the wrapper) and 3 (the wording).
- `temper/docs/plan/outward-names/inputs/request.md`: the request.
- `jig/docs/design/protocol/service.md`, 8: the person-driven live layer the script runs (read
  only; it names no build tool).
- `tests/boundary/tests/outward.rs`: the check.

## Owns and may change

- **`jig/scripts/live-sign-in.sh`** ~l.7-12: the comment "The machine's build queue where it has
  one, plain cargo where it hasn't." and `heavy=$(command -v heavy || true)` give way to a comment
  saying `JIG_WRAPPER`, when set and not empty, names one command that runs the build; unset,
  cargo runs plainly. The `$heavy cargo nextest ...` line takes the form above; its
  `# shellcheck disable=SC2086` goes. Nothing else of the script changes.
- **The hint strings:** `jig/tests/client-browser/src/lib.rs` ~l.18 ("build the browser server
  with heavy cargo build -p jig-test") → "with cargo build -p jig-test";
  `jig/tests/client-wire/tests/suite/placed_holds.rs` ~l.80, `jig/tests/codec-families.rs`
  ~l.152, `jig/tests/protocol-accounts/tests/suite/drift.rs` ~l.13 and
  `jig/tests/worker-wire/tests/suite/drift.rs` ~l.13: `heavy cargo nextest` → `cargo nextest`.
  The rest of each string stays as it is, even where its `--test` name is not the binary's.
- **`jig/docs/development/debt/reviews/2026-10-31-jig-quality-run.md`** ~l.13 "It is not in the
  tree (`~/src/fabrig/notes/` on hel8)." → "It is not in the tree (the review machine's notes)."
  and ~l.19 "They are in `~/src/fabrig/coord/` on hel8." → "They are on the review machine, with
  its coordination records." Findings stay.
- **If a line moved** at your base, the name is changed where it stands; a name this spec does
  not list is changed by `notes/contracts.md`, 3, and the report says where.
- Nothing else: not `tests/boundary/outward.txt`, not any `docs/design/` file.

## Steps

1. The script, the hint strings and the review, as `## Owns and may change`.
2. `bash -n jig/scripts/live-sign-in.sh`; `shellcheck jig/scripts/live-sign-in.sh` where
   installed. The script is person-run (a live sign-in): do not run it.
3. Check the files clean, as `notes/common.md` says, deleting these seven entries of
   `tests/boundary/outward.txt` in the working tree: the seven paths of `files`. Restore it.
4. Tests, the narrowest that compile and read the changed strings:
   `heavy cargo nextest run -p jig-client-wire-world --test suite -E 'test(/^drift::/) | test(/^placed_holds::/)'`
   (`codec-families.rs` is included by `drift`),
   `heavy cargo nextest run -p jig-protocol-accounts-world --test suite -E 'test(/^drift::/)'`,
   `heavy cargo nextest run -p jig-worker-wire-world --test suite -E 'test(/^drift::/)'`, and
   `heavy cargo check -p jig-client-browser-world -p jig-store-codecs-world --tests` (the browser
   world's tests need a browser; the store codecs' drift includes `codec-families.rs` too). Check
   each count is above zero.
5. `cargo fmt`; clippy as in Acceptance; old forms; commit; citations; push.

## Report

As the implementer's text says, and: each changed line, before and after; `bash -n` and
`shellcheck`'s outcome (or that it is not installed); the check of step 3 and its selected
count; the test counts of step 4; anything under `## Left`.
