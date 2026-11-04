# D038: Comments carry emphasis words, rulings and history

- **Kind:** naming
- **Where:** comments across skein's crates and tests; `skein/scripts/old-forms.txt`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (themes T6 and T8, pipeline vocabulary and "actual")
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low
- **Size:** S
- **Status:** open
- **Paid when:** no skein comment uses "actual" as emphasis (the `comment:actual` form at 0), and no comment cites a ruling, a decision or a runtime by number, a task letter, or the commit a file was extracted from; it says what is true and cites the design.
- **Measure:** `grep '^comment:actual' skein/scripts/old-forms.txt`: `comment:actual 79` at base (the ratchet, lowered by each task)

## What

- 79 comments use "actual", counted by skein's old-forms ratchet.
- Comments cite pipeline history: "ruling 106"
  (`skein-ui-text/src/render.rs`, in an expect's reason), "ruling 111"
  (`skein-forge/src/forgejo/document.rs`), "decision 25"
  (`skein-forge/src/tests.rs`), "runtime 20" and "runtime 24"
  (`skein-forge/src/forgejo/credential.rs`,
  `skein-forge/src/github/credential.rs`), "task C"
  (`skein/tests/readiness/tests/suite/readiness.rs`), and where a crate
  came from (`skein-fake-checkout/src/lib.rs`,
  `skein/testing/skein-world/src/domain.rs`).
- The conformance suite's and the shell's "macOS run N" notes cite the
  observed kernel behaviour they encode; they are evidence, not
  history, and are not part of this item.

## Why it matters

`AGENTS.md`, section 2: comments say what is true now, in the design's
words, with no history and no emphasis words. A ruling number means
nothing to a reader without the pipeline's records.

## What pays it back

A sweep: drop "actual", replace each ruling or decision with the design
section that holds it, delete the extraction notes. Lower the ratchet
with `old_forms.py --write`.
