+++
id = "forgejo-kinds"
part = "forgejo-kinds"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge/src/forgejo/plans/**", "skein/docs/design/forge.status.md"]
effort = "high"
minutes = 70
+++

# Forgejo's plans: one rule for a decoded kind, and `Some(_) | None` gone

## Goal

`Some(_) | None` in the Forgejo plans is the `_` lint's escape under another spelling. fabrig/debt decided, under the user's standing word and for the user's later review: one written rule, `_ =>` under a named `#[expect]`, and the wrappers that made the `Option` go.

Acceptance:

- `rg -n 'Some\(_\) \| None' skein/crates/skein-forge/src/forgejo/plans` prints `reading.rs`'s arm alone: that one follows guards on two optional times, picks no kind, and stays.
- The five wrappers are gone: `reads.rs`'s `let value = Some(value)`, `reads/done.rs`'s `let document = Some(document)` and `match Some(document)` in `check_repository`, `turn.rs`'s `self.done(Some(document), bounds)` and `start.rs`'s `document_answer(read, Some(document))`. Their parameters take the value itself: `document_answer`, `turn.rs`'s `done`, `reads/done.rs`'s `keep_repository`, `repository`, `pull` and `listed`.
- Every new `_ =>` arm ends a match over a decoded `d::Document` or `d::Row`, or over the `Option` of one where `None` is real (`writes.rs`, `actions.rs`, `contents.rs`), and its function carries the one named expect below. No `_` arm on a plan's `State`, on `Heard`, or on any other enum; the expect sits on a function, never on a module or an `impl`, so that a `State` arm still can't go silent.
- `skein/docs/design/forge.status.md`, 4.2 holds the rule, in the words below, in place of the `50 arms` sentence.
- Production lines under `plans` fall, about 100 by the review's reading: report the count, with inline `#[cfg(test)]` split out. Size cap: no growth.
- The `skein-forge` lib tests and `skein-forge-machine-world`'s tests pass unchanged, and clippy is clean.

## Inputs

- `skein/docs/foundation/programming-model.md`, 10.3, *a `_` arm on an own enum*.
- `skein/docs/design/forge.status.md`, the **Departs** entry, 4.2.
- `temper/docs/plan/debt-26/inputs/debt22-review.md`, finding 2.
- `temper/docs/plan/debt-26/notes/common.md`.

## Owns and may change

Owns `skein/crates/skein-forge/src/forgejo/plans/**`, its inline tests included, and the one sentence of `forge.status.md` (a stated departure from the implementer's text), with that file's date line. `forge.status.md` is not read by tests.

Options, each pre-approved:

- **The row lists.** A tail arm that lists `d::Row` or `d::Document` kinds only to reject or ignore them (as in `writes.rs`'s row match) becomes `_ =>` too, under the same expect, where it falls out cleanly. Where one doesn't (a kind that's handled in several states, a dispatch whose arms nest a `State` match) leave it listed, and say which in the report.
- **`match_wildcard_for_single_variants`** denies a `_` over one variant left: name that variant instead.
- **`clippy::manual_let_else`:** where it fires on a kind's `let x = match ... { .. , _ => return Err(..) }`, keep the `match`, and put the lint in the same expect: `let … else` on an own enum is out by programming-model.md, 10.2. The expects at `contents.rs`'s `done` and `actions.rs`'s `done` say "matched exhaustively" or "explicit matches", which stops being true: they take the rule's reason, and `actions.rs`'s keeps its other lints with what's still true of them.
- If `writes.rs`'s `check_repository` or `check_pull_ids` read better with `Option::is_some` or a helper, keep them as they are: this isn't their refactor.

Don't touch `plans/commits.rs`'s copy of `at()` (a later task's) or the plans' `State` lists.

## Steps

The expect, one line of reason, the same on every function that needs it:

```rust
#[expect(clippy::wildcard_enum_match_arm, reason = "the exchange's decoder picks the kind: forge.status.md, 4.2")]
```

Sketches:

```rust
// reads.rs, `row`
State::Searching { .. } => match value {
    d::Row::User(_) => {}
    _ => return Err(api::Error::InvalidAnswer),
},
// reads/done.rs
fn repository(document: d::Document) -> Result<d::Repository, api::Error> {
    match document {
        d::Document::Repository(repository) => Ok(repository),
        _ => Err(api::Error::InvalidAnswer),
    }
}
```

`reads/done.rs`'s `check_repository` takes `&d::Document` already: its `match Some(document)` becomes `match document`.

The status sentence, replacing the one that begins `4.2: skein/docs/foundation/programming-model.md, 10.3 has no _ arm on an own enum: 50 arms` (keep the entries on either side, and put `2026-11-06` on the date line if it isn't):

> 4.2: `skein/docs/foundation/programming-model.md`, 10.3 has no `_` arm on an own enum: the exchange's decoder picks the kind, so under `skein-forge/src/forgejo/plans` a match over a decoded `d::Document` or `d::Row` ends in `_`, under a named `#[expect]`, on the function that holds it. A kind a state didn't ask for is an invalid answer whichever it is. A new kind goes silent there and nowhere else: a plan's `State` keeps every arm. fabrig/debt decided this under the user's standing word, 2026-11-06, for the user's later review.

Check the claim as you write it: read how the exchange picks its decoder under `skein-forge/src/forgejo/` (`decode` and the plans' `Exchange`). If a site's kind isn't the decoder's pick, say so in the report and leave that site listed.

Tests: nothing new; this changes no behaviour. `heavy cargo nextest run -p skein-forge --lib`, then `heavy cargo nextest run -p skein-forge-machine-world`. Then fmt, `python3 -I -B scripts/old_forms.py` from `skein/`, `heavy cargo clippy -p skein-forge --all-targets -- -D warnings`, and the citations tool. Commit, push.

## Report

As your text says: the count before and after, which row lists went and which stayed, and any site that the decoder's pick didn't cover.
