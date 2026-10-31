+++
id = "stage-person-browser"
part = "stage-sites"
kind = "migration"
needs = ["echo-stage", "kit-shell", "counts"]
files = ["skein/testing/skein-fake-person-browser/**", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 35
+++
# The fake person's browser on the shared stage

## Goal

Draft, 3.3: skein-fake-person-browser's harness writes io's stage by
hand in three places; they call `skein_io::stage` instead, and its test
peer host moves onto the provided deadline, by `notes/stage.md`.

Acceptance:

- Moved: `src/harness.rs:879`, `src/harness/capacity.rs:109`,
  `src/harness/peers/tests.rs:89` (`impl Host for Peer`, from 84),
  unless a default leaves one (named).
- `io_resume` falls by the sites moved (3 expected).
- `Peer`'s `next_deadline` (`peers/tests.rs:141`) follows `stage.md`
  item 2.
- `heavy cargo nextest run -p skein-fake-person-browser` passes; its
  readers in skein pass too: `heavy cargo nextest run -p
  skein-ui-web-tests -p skein-browser-world` (they drive the fake
  person; `skein-lab-browser` also reads it). Counts named; clippy clean.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/stage.md`.
- `skein/docs/design/browser.md` for the words.
- `skein_io::stage` on your base.

## Owns and may change

- `skein/testing/skein-fake-person-browser/**`, `skein/scripts/old-forms.txt`.
- A change of meaning here (a stage now stopping at the kernel room)
  reaches its readers; if one of their tests fails, fix it only in this
  crate, or stop with a `Question:`.

## Steps

1. Each stage, item 1; the deadline, item 2.
2. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`.
