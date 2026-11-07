+++
id = "originals"
part = "tui-journeys"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-shell-web/src/journey/originals.rs", "temper/crates/temper-client-shell-web/src/journey/originals/tests.rs", "temper/crates/temper-client-shell-web/src/journey/redact.rs", "temper/crates/temper-client-shell-web/src/journey/redact/tests.rs", "temper/crates/temper-client-shell-web/src/journey/scan.rs", "temper/crates/temper-client-shell-web/src/journey/scan/tests.rs", "temper/crates/temper-client-shell-web/src/journey.rs"]
effort = "high"
minutes = 75
+++

## Goal

The browser's redaction and last scan, shared with the TUI's journeys with the same code (`temper/docs/design/client/journeys.md`, 3.6 and 3.7; `temper/docs/design/client/tui/journeys.md`, section 5), gain what the TUI's lines need, additions only: three kinds of original (`Approval`, `Credential`, `Bearer`), `Originals::new` from candidates, `redact_field` for one field, and the scan's `original` and `shape` apart. What the browser's journeys redact and refuse stays exactly as it is.

Acceptance:

- Contracts, section 4, holds as written in `temper-client-shell-web`'s `journey` module, re-exported from `journey.rs` beside the existing names.
- `Originals::of` builds its candidates as today and calls `new`; `redact` calls `redact_field` for each field and stream record as it calls `replace` today; `scan::bytes` is `original(..)` then `shape(..)`, as today.
- Red first, in the module's own tables: each new kind, given to `Originals::new` with fake bytes built in the test, is sought in every form 3.6 names (its bytes, percent-encoded in either case, `dom::print`'s escapes, Rust's Debug escapes) and by its sha-256, is replaced by `redact_field` in a framed field (`x`s of the form's length) and a text field (the placeholder), and is found by `scan::original`; `count` counts each kind; the same bytes given under two kinds are sought once and counted under each.
- The browser's existing tests pass unchanged: `heavy cargo nextest run -p temper-client-shell-web` and `heavy cargo nextest run -p temper-client-journeys`, with every committed journey reprinting byte for byte.
- Clippy clean for `temper-client-shell-web` and `temper-client-journeys`.

## Inputs

- `temper/docs/design/client/journeys.md`, 3.6 (the originals, the placeholder, the forms, a form's size) and 3.7 (the scan).
- `temper/docs/design/client/tui/journeys.md`, section 5.
- `temper/docs/plan/tui-recorded/notes/contracts.md`, section 4 (binding).
- `temper/docs/plan/tui-recorded/notes/common.md`.
- The code: `src/journey/originals.rs` (`Original::new`, `Originals::of`, its dedup and `counted`), `src/journey/redact.rs` (`replace`, `seek`), `src/journey/scan.rs` (`bytes`, `shapes::scan`), and their tests.

## Owns and may change

- Owned: the files above. Nothing else of `temper-client-shell-web` changes.
- Readers: `tools/journeys/src/lib.rs` reads `count` for four kinds: it stays as it is (`tui-print` adds the TUI's). No exhaustive match on `Kind` exists outside `originals.rs` (`rg -n 'Kind::' temper`); one you find is a compiler-forced arm, the nearest behaviour.
- Options:
  - `redact_field` as `replace` renamed and made public (default), or a public wrapper over the private `replace`: either.
  - The new kinds' order in `Kind`: after `Given` (default).

## Steps

1. Read the inputs. Add the kinds, `new`, `redact_field`, `original` and `shape`, moving no logic but what `new` takes out of `of`.
2. The red rows first, then the code. Break a rule once by hand (`new` skipping the Debug form for a new kind; `shape` returning before `original` in `bytes`), see the row fail, restore; name them.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-shell-web -p temper-client-journeys --all-targets -- -D warnings`; the focused tests above.
4. Commit and push.

## Report

The implementer's report, with: the rows and their red runs; confirmation that no browser test or committed journey changed; anything of 3.6's forms the new kinds could not take.
