+++
id = "ops-tuples"
part = "spot-review"
kind = "feature"
needs = []
files = ["jig/examples/ops/domain-infrastructure/src/application.rs", "jig/examples/ops/domain-infrastructure/src/domain.rs", "jig/examples/ops/domain/src/client_documents.rs", "jig/examples/ops/domain/src/route.rs"]
effort = "medium"
minutes = 15
+++

# jig's ops example: two same-typed tuples get names

## Goal

Finding 5 in `temper/docs/plan/debt-25/inputs/debt23-review.md`, checked against the code. `(Token, Token)` names nothing, and its call site can swap the two unseen.

Acceptance:

- `application::decide`'s `(token, owner)` is `Deciding { token, owner }`, and `client_documents::document`'s `(watcher, subject)` is `Watched { watcher, subject }`, each `pub(crate)` beside its function.
- `rg -n '\((token, owner|watcher, subject)\): \(' jig/examples/ops` reads 0.
- About +8 lines; no test changes.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/inputs/debt23-review.md`, finding 5

## Owns and may change

- **Owns:** the files in the header. A name that's already taken in its module gets a near one, and your report says so.

## Steps

1. The two structs and their call sites: `domain.rs` for the first, `route.rs` (two) for the second.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-ops-domain -p jig-ops-domain-infrastructure --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-ops-domain -p jig-ops-domain-infrastructure`, then `heavy cargo check -p jig-ops-world --tests`. Commit and push.

## Report

The common note's items. This is a move: no rule to break by hand.
