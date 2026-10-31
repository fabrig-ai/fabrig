+++
id = "stage-lab-browser"
part = "stage-sites"
kind = "migration"
needs = ["echo-stage", "kit-shell", "counts"]
files = ["skein/testing/skein-lab-browser/**", "skein/tests/browser/**", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 40
+++
# The lab browser and the browser world on the shared stage

## Goal

Draft, 3.3: skein-lab-browser's runtime and its test support, and the
browser world's real-browser support, write io's stage by hand; they
call `skein_io::stage` instead, by `notes/stage.md`.

Acceptance:

- Moved: `skein/testing/skein-lab-browser/src/runtime.rs:417`,
  `skein/testing/skein-lab-browser/tests/support/mod.rs:173`,
  `skein/tests/browser/tests/support/real.rs:316` and `:408`, unless a
  default leaves one (named).
- `io_resume` falls by the sites moved (4 expected).
- `heavy cargo nextest run -p skein-lab-browser -p skein-browser-world`
  passes (tests that need a Chromium are skipped as they are today, and
  named); clippy clean.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/stage.md`.
- `skein/docs/design/lab-browser.md` for the words.
- `skein_io::stage` on your base.

## Owns and may change

- The directories in the header, `skein/scripts/old-forms.txt`.
- The pop loops in `real.rs` and `tests/browser/src/` use what they pop:
  leave them.

## Steps

1. Each stage, item 1; any host deadline or referee in these files,
   items 2 and 4.
2. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`.
