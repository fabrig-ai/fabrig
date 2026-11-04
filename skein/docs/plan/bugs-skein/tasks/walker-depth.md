+++
id = "walker-depth"
part = "forge-dialects"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge/src/github/decode/walk.rs", "skein/crates/skein-forge/src/github/tests.rs", "skein/crates/skein-forge/src/github/tests/delivery.rs"]
effort = "medium"
minutes = 30
+++

# The GitHub walker's stack takes the configured depth

## Goal

`skein-forge`'s GitHub walker sizes its stack by `github::limits::DEPTH` (10) and refuses an
unknown field's nesting past it, whatever `limits.json.depth` says: `decode/walk.rs`, `root` and
the unknown-field arm. `check` only says the depth is at least 10; a deeper configured depth is
honoured by the tokenizer and refused by the walker. Forgejo's walker already uses
`limits.json.depth` (`forgejo/decode/walk.rs`). (Bug 10.) A change of meaning: its readers' tests
run.

Acceptance:

- `root` makes `Stack::with_capacity(limits.json.depth)`; the unknown-field check compares against
  `self.limits.json.depth`. `limits::DEPTH` stays what `check` and the writers use.
- A document at the configured depth decodes; one level past it is `TooDeep`, at a depth above 10.
- The two tests below fail on the old code.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/forge.md`, 4.1 (a dialect's limits); `skein/docs/design/forge-github.md`,
  section on `github::Limits` ("a depth below 10"), read only: no design edit here, and that file
  is read by tests.

## Owns and may change

- Owns: `src/github/decode/walk.rs` (`root`, the unknown-field check, the `DEPTH` import if unused).
- Tests that pin the old behaviour, to change: `src/github/tests.rs` (`relaxed_depth`, near 421)
  and `src/github/tests/delivery.rs` (`relaxed`, near 470). Each now asserts the document at the
  relaxed depth decodes (`Ok(Document::Rows)`; `Ok(read(b"issues", ISSUE))`) and adds one level past
  it as `TooDeep`.
- If `rg 'json\.depth|DEPTH' skein/crates/skein-forge` shows another test pinning the old refusal,
  the same change there. `github/server/log.rs` and `github/request.rs` use `DEPTH` for what they
  write: leave them.

## Steps

1. Change the two tests; run them on the old code and quote the failures.
2. The two lines in `walk.rs`.
3. `heavy cargo nextest run -p skein-forge --lib -E 'test(/github/)'`, then the focused tests of
   `skein-forge`, `skein-forge-connection`, `skein-forge-setup`, `skein-forge-protocol-world`,
   `skein-forge-setup-world`, `skein-forge-machine-world`, `github-conformance`. `cargo fmt`,
   `python3 -I -B skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-forge --all-targets --
   -D warnings`.
4. Commit, push.

## Report

As the implementer's text says, with what the two tests showed on the old code.
