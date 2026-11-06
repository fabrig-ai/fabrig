+++
id = "plan-copies"
part = "plan-copies"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge/src/forgejo/plans/commits.rs", "skein/crates/skein-oauth/src/machine/fetch.rs"]
effort = "medium"
minutes = 20
+++

# Two leftovers: `commits.rs`'s copies, and sign-in's `Option` pair

## Goal

Two more of the spot review of debt-24's finding 4, checked against the trunk by the cut. Behaviour stays unchanged, and lines fall.

Acceptance:

- `skein-forge/src/forgejo/plans/commits.rs` reads `self.row.at()` and `self.row.id(..)` in `begin`, and `super::failure::refused(super::failure::Asked::Leaf, status, None, fields.retry_after, repository, tables)` in place of its classify-and-drop. About -7.
- `skein-oauth/src/machine/fetch.rs`'s sign-in start holds no `(visit, url)` pair and no `url.expect(..)`, if the shape below holds. Otherwise it stays, and the report says why.
- `skein-forge`'s and `skein-oauth`'s lib tests pass unchanged; so do the tests of `skein-oauth-accounts`.

## Inputs

- `temper/docs/plan/debt-26/inputs/debt24-review.md`, first review, finding 4.
- `skein/docs/design/oauth.md`, 3, on a sign-in's fetch and visit.
- `temper/docs/plan/debt-26/notes/common.md`.

## Owns and may change

Owns the two files. `forgejo-kinds` holds the rest of `plans/`, and may touch `commits.rs`'s `done` if it has a match on a decoded kind: stay on `begin` and the head arm.

The pre-approved option for `fetch.rs`: hold the URL in `Grant::SignIn`'s `visit` on both paths, and on the no-fetch path take it out the way the move after a fetch does (`visit.take()`, about line 164). If that shape needs a new `unreachable!`, or doesn't lose lines, leave the pair and say why in the report: a finding that doesn't pay is skipped, not forced.

## Steps

`commits.rs`, `begin`:

```rust
Step::send(Request::Commits { at: self.row.at(), before: self.row.id(self.before), after: self.row.id(self.after) })
```

Check that `Repository::id` builds the same `ObjectId`, and that `refused` does what the head arm's five lines did (`failure.rs`). `heavy cargo nextest run -p skein-forge --lib`, `heavy cargo nextest run -p skein-oauth --lib`, `heavy cargo nextest run -p skein-oauth-accounts-world` for its readers. fmt, `python3 -I -B scripts/old_forms.py` from `skein/`, `heavy cargo clippy -p skein-forge -p skein-oauth --all-targets -- -D warnings`. Commit, push.

## Report

As your text says: lines before and after in each file, and what you did with `fetch.rs`.
