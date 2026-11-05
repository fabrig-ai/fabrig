+++
id = "leaves-back"
part = "leaves-back"
kind = "feature"
needs = ["now-link"]
files = ["jig/crates/jig-domain-hub-link/src/leaves.rs", "jig/crates/jig-domain-hub-link/src/lib.rs", "jig/crates/jig-domain-hub-link/src/tests.rs", "jig/examples/ops/domain/src/route.rs", "jig/testing/jig-test-domain/src/routes.rs"]
effort = "medium"
minutes = 20
+++

# jig: `no_workspace` and `complete_brief` go back to the roots

## Goal

Finding 6, as far as it holds. `no_workspace` has one caller, and `complete_brief` saves each of its two callers one line
at the price of a function and a test. Both go back to the roots.

`settled_answer` stays as it is, returning its `Result`: the finding's `Option` is wrong, since the test root takes the
answer back from the `Err` (`call.answer = answer`), and `temper-root-shared` makes temper its third caller.

Acceptance:

- `rg -n 'no_workspace|complete_brief' jig temper --glob '*.rs'` finds nothing.
- `jig-domain-hub-link/src/leaves.rs` holds `job_assignment`, `settled_answer` and `brief_plan`, and no `impl Link`.
- Each root's output is unchanged: `heavy cargo nextest run -p jig-ops-world -p jig-core-world` passes.

## Inputs

- `temper/docs/plan/debt-16/notes/common.md`
- `jig/docs/design/domain/root.md`, 5 and 14: this cut has already written them without these two.

## Owns and may change

- **Owns:** `jig-domain-hub-link/src/leaves.rs`, `src/lib.rs` (the export), `src/tests.rs`.
- **May change, these arms and the import:** `jig/examples/ops/domain/src/route.rs`,
  `jig/testing/jig-test-domain/src/routes.rs`.
- **The two approved test deletions, and no other:** `a_completed_brief_keeps_its_order_and_answers_ready` and
  `no_workspace_prepares_empty_holds_and_some_empty_writes`, in `jig-domain-hub-link/src/tests.rs`, with what only they
  used.

## Steps

1. `route.rs` (ops): the `Now::WorkspaceRequest` arm spells `core::Event::WorkspacePrepared { .. }` with the values
   `no_workspace` has, and the import drops it.
2. `ops`'s and the test root's `Now::CompleteBrief` arms call `domain.hub_link.keep_brief(brief.raw(), order)` and push
   `core::Event::BriefAssembled { task: brief.raw(), ready: true }`, in that order.
3. `leaves.rs`: both go, with the `impl Link` and any import they alone used. `lib.rs`'s export drops `no_workspace`.
4. Break by hand: take `keep_brief` out of `ops`'s arm, and see `jig-ops-world` fail, which shows the world covers what
   the deleted test did. Restore, and name the test that failed. If no test fails, stop and ask rather than delete the
   hub-link test.
5. `heavy cargo nextest run -p jig-domain-hub-link -p jig-ops-domain -p jig-test-domain -p jig-ops-world -p
   jig-core-world`.
6. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-hub-link -p jig-ops-domain -p
   jig-test-domain --all-targets -- -D warnings`; the citations check. Commit and push.

## Report

The common note's items, with the break above.
