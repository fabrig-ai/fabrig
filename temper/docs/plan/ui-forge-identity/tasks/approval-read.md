+++
id = "approval-read"
part = "approval"
kind = "feature"
needs = ["link-lookup", "link-scope", "approval-wire", "approval-waits"]
files = [
  "temper/crates/temper-engine-domain/src/route/approvals.rs",
  "temper/crates/temper-engine-domain/src/route/approvals/tests.rs",
  "temper/crates/temper-engine-domain/src/route.rs",
  "temper/crates/temper-engine-domain/src/route/panels.rs",
  "temper/crates/temper-engine-domain/src/route/kinds.rs",
  "temper/crates/temper-engine-domain/src/boundary.rs",
  "temper/crates/temper-engine-domain/src/decision.rs",
  "temper/crates/temper-engine-domain/src/room.rs",
  "temper/crates/temper-engine-domain/src/tests/kinds.rs",
  "temper/crates/temper-service/src/family.rs",
  "temper/crates/temper-service/src/route.rs",
  "temper/crates/temper-service/src/tests/family.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 110
+++

# The root answers each watcher's approvals, and a proposal's `owned_approval`

## Goal

A change waiting on a review on the forge shows under Needs you to whoever's review it waits on (`temper/docs/design/domain/forge.md`, 8.12). The forge answers the waits (`approval-waits`); the root joins the people: for each watcher of the subject `approvals`, of the party's own scope, the rows of the projects they hold a role in, each saying whether their review would count. The service carries the document to the client, each row's kind and files-page address from the forge's configuration, as a panel's pull request's is. And the root's read of a proposal's items says whether one of its landing rules has an enforcing approval on an owned Forgejo (8.12, *What a proposal says of such a rule*).

Acceptance:

1. `YourReview`, `ApprovalRow`, `Approvals`, `route::approvals::{document, row}`, `Delivery::Approvals`, `room::approval_rows`, `ForgeReadAnswer::ProposalItems { items, owned_approval }` and `temper_service::family::approvals` are as `temper/docs/plan/ui-forge-identity/notes/contracts.md`, *The approval read*, *The root's part*, writes them, with doc comments in the design's words; `temper_engine_domain::APPROVALS_SUBJECT` re-exports the forge's.
2. A watch of `w::Watch::Approvals` reaches the root as `Watched::Application { scope: Own, connector: FORGE, subject: APPROVALS_SUBJECT }`; the root's document for it is `document(domain, env, party)`, delivered as `Delivery::Approvals`, encoded by `family::approvals`.
3. `row` answers as 8.12's *The root's part* says: a watcher with no role in the wait's project has none; the approval chosen is the first short one of the watcher's role, else for an owner the first; a watcher counted for it has none; `yours` the first of too few (`eligible_people` below `required`), no account (`forge_human` false), another role, counts; `goal` from `public_goal`.
4. `document`'s rows are `approval_waits`' order, at most `approval_rows(&env.limits)`, `more` the rest.
5. `retain_review_wait` marks `APPROVALS_SUBJECT` beside the change's panel when its facts are kept anew.
6. The proposal items read answers `owned_approval` true exactly when one item is a landing rule (kind 1 or 2) set, enforcing, with an approval, whose pattern covers a writable repository of a project in its scope on a forge whose configured mode is `Mode::Owned`.
7. The tests of Steps, 7 bite; the **Not yet** entry *8.12: the root answers no document...* is gone from `temper/docs/design/domain/forge.status.md`.
8. `heavy cargo nextest run -p temper-engine-domain --lib` and `-p temper-service` pass; `heavy cargo check --tests -p temper-engine-domain-world -p temper-system-world -p temper-client-world` passes; clippy clean for `temper-engine-domain` and `temper-service`.

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 7), because the design already says what this task builds.

## Inputs

- `temper/docs/plan/ui-forge-identity/notes/common.md`, and its rule that `forge_route.rs` is shared.
- `temper/docs/plan/ui-forge-identity/notes/contracts.md`: *The lookup the other readers use* (`forge_human`, `eligible_people`) and *The approval read* bind.
- `temper/docs/design/domain/forge.md`, 8.12, whole; 8.9, *The root checks approval eligibility*, for the coverage `owned_approval` judges as eligibility does.
- `temper/docs/design/client/README.md`, 4.6, *The changes waiting on a person's review, the subject `approvals`* and *A proposal's items, the read `ProposalItems`*.
- `temper/docs/design/client/forge.md`, 4.1, *Its address*, and 9.2 for the panel's pull address it follows.

## Owns and may change

**Owned:** the `files` above. In `route.rs` the `mod approvals;` line alone (`link-scope` writes it); in `kinds.rs` the `ProposalItems` arm of `read` and the coverage helper below (`link-lookup` wrote the rest).

**Readers and neighbours:**

- `temper/crates/temper-engine-domain/src/route/forge_route.rs`, `retain_review_wait`: its mark's list takes `Token::new(crate::APPROVALS_SUBJECT)` after the change's own, capacity 2. Nothing else there (shared with temper/migration's `goal-delivery` and `call-room`).
- `Delivery::Approvals`'s compiler-forced arms, beside each `Delivery::Trials`: `temper/crates/temper-engine-domain/src/boundary.rs` and `src/domain.rs`; `temper/crates/temper-service/src/clients/translation.rs`; `temper/tests/engine/domain/src/walking.rs`, `oracle.rs`, `escalation.rs`; `temper/tests/system/src/composed.rs`. In `temper/tests/client/src/component.rs` the arm is a real translation, beside `Delivery::Panel`'s: `temper_service::family::approvals(*document, &crate::fixtures::forge_settings(), &v1::CEILINGS).expect("approvals document")`.
- `ForgeReadAnswer::ProposalItems`'s readers: `temper-service`'s `forge_read` (owned) and `tests/family.rs` (~2334, owned), `temper-engine-domain`'s `tests/kinds.rs` (~767, ~777, owned).
- `temper-service/src/family.rs`, the watch translation: `w::Watch::Approvals` leaves the refusal arm `approval-wire` put it in and gets its own, as `w::Watch::Trials` has; `forge_read` sets `owned_approval` from the answer.

**Options, each with its default:**

- The coverage: if `link-lookup` left a per-repository coverage function in `kinds.rs`, call it. Otherwise lift `covers_project`'s inner test into `fn covers_repository(rule: &settings::LandingRule, repository: &forge::Repository) -> bool` in `kinds.rs`, `covers_project` calling it, and `owned_approval` (in `approvals.rs`, `pub(super)`) calling it for each writable repository whose forge's `domain.forge.configured(forge)` has `mode == Mode::Owned`. A rule of scope 0 covers every project of `domain.core.projects`, a project's its own.
- The table's fixture: `route/forge_route/tests.rs`' `forge_party(key)` (project 5, person 1 a maintainer, `two_forges`). Default: build your own in `approvals/tests.rs` as it does; if making `forge_party` reachable is two visibility edits (the test module's and the function's), do that instead and name them.
- `ApprovalWait`s for the table are literals: `row` takes one, so the table needs no forge change.
- `approval_rows`: the room of one whole delivery, `result_bytes.max(transcript_bytes)`, over the largest row (its size, two `forge.name_bytes` and `LOGGED_TITLE_BYTES`), at most 32, as `trial_rows` is written. `Delivery::deliver` admits `Approvals` by its `owned_bytes`, as `ForgeDocument` is.
- A root row whose forge the service's configuration doesn't name: `kind` Forgejo and an empty `address`. A row the wire's ceilings refuse makes the whole document `None`, delivered as `Abort`, as trials' is.

## Steps

1. `route/approvals.rs`: the types, `row`, `document` (the root's `domain.forge.approval_waits(env.wall, &env.limits.forge)`, each through `row`, cut at `approval_rows`), `owned_approval`. Every `match` on `jig_domain_people::Role` lists its variants; `public_goal` from `super::public`.
2. `route/panels.rs`: an `approvals` arm beside `trials`, when `connector == domain.config.forge_connector && scope == ApplicationScope::Own && subject == Token::new(crate::APPROVALS_SUBJECT)`, the ready document taken into `Delivery::Approvals { watcher, document: Box::new(..) }` where it was made.
3. `boundary.rs`, `decision.rs`, `room.rs`: the exports, the variant after `Trials` and its admission, `approval_rows`.
4. `kinds.rs`: `crate::ForgeReadAnswer::ProposalItems { owned_approval: super::approvals::owned_approval(domain, &items), items }`.
5. `forge_route.rs`'s one line (above).
6. `temper-service`: `family.rs`' watch arm, `approvals` (each row into `w::ApprovalRow`: `kind` and `address` from `forges` by `row.forge`, `address` the existing private `pull_address(..)` and `/files` after it, empty where that is empty or past 2048 bytes; `since` in nanoseconds; `yours` by variant), `forge_read`'s `owned_approval`; `route.rs`: `root::PartyRequest::Deliver(root::Delivery::Approvals { watcher, document })` encoded with `&engine.workers.forges`, as `Delivery::Panel` is.
7. Tests:

   ```rust
   // temper-engine-domain, route/approvals/tests.rs
   #[test]
   fn a_watcher_sees_the_waits_of_their_role_and_whether_their_review_counts() {
       // (watcher's role in the wait's project or none, its key's forge, short approvals
       //  (role, required, counted people), eligible count, expected: None or (approval role, yours))
       // maintainer, known on the change's forge, approval role 1, required 1 -> (1, Counts)
       // maintainer, known on another forge only                              -> (1, NoAccount)
       // required 2, one eligible                                             -> (1, TooFew)
       // owner, approval of role 1                                            -> (1, OtherRole)
       // owner, approvals of roles 1 and 0                                    -> (0, Counts)
       // member, approval of role 1; no role in the project                   -> None
       // maintainer already counted for it                                    -> None
       // and goal Some(g) for a change under a tracked task, None otherwise.
   }
   ```

   The document's bound and `more`: a row of the table if `document` can take a forge with waits cheaply, else a test of `approval_rows` and the cut alone. In `tests/kinds.rs`, a table beside `forge_proposal_reads_use_the_deployment_unique_key_and_preserve_edits` (which takes `owned_approval: false`):

   ```rust
   #[test]
   fn a_proposals_items_say_whether_an_approval_covers_an_owned_forgejo() {
       // (item, forge mode of the covered repository, expected)
       // kind 2, enforcing, one approval, covering, Owned        -> true
       // the same on a Connected forge; not enforcing; no approval;
       // a pattern covering no repository; a Remove edit          -> false
       // kind 1 (scope 0) covering a project's owned repository  -> true
   }
   ```

   In `temper-service`'s `tests/family.rs`: a row `(w::Watch::Approvals, root::APPROVALS_SUBJECT)` in `dismissed_and_trials_watches_name_the_partys_own_subjects`; and, beside `trials_encode_the_whole_document_in_its_order_and_with_its_bounds`,

   ```rust
   #[test]
   fn approvals_encode_each_row_with_its_kind_and_files_address() {
       // a Forgejo row: address ROOT/OWNER/NAME/pulls/N/files; a GitHub row: WEB/OWNER/NAME/pull/N/files;
       // a fake forge's and an unknown forge's: empty; each yours; more; decoded back whole.
   }
   ```

   Bites: swap too few and no account in `row`, see a row fail; drop the `Mode::Owned` check, see the Connected row fail; drop `/files`, see the address rows fail; restore.
8. The status entry (acceptance 7).
9. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain -p temper-service --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-engine-domain --lib`; `heavy cargo nextest run -p temper-service`; the `cargo check` of acceptance 8. Commit ("temper-engine-domain: answer each watcher's approvals, and a proposal's owned_approval; temper-service carries them"), push.

## Report

As the implementer's text says, with each rule broken for a bite and the failure seen, the coverage option taken, and each forced arm outside the `files` by file.
