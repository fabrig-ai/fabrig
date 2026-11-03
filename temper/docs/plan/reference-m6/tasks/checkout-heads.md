+++
id = "checkout-heads"
part = "checkout"
kind = "feature"
needs = []
files = ["temper/crates/temper-worker-domain-checkout/src/**", "temper/crates/temper-worker-domain/src/**", "temper/crates/temper-protocol-worker-wire/**", "temper/crates/temper-service/src/translate.rs", "temper/crates/temper-service/src/tests/translate.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/link.rs", "temper/tests/worker/checkout/**", "temper/tests/worker/domain/**", "temper/tests/fake-checkout/**", "temper/crates/temper-fake-git/src/**", "temper/docs/design/domain/worker.status.md", "temper/docs/design/protocol/channel.status.md"]
effort = "high"
minutes = 120
+++

# checkout-heads: a base with heads merged in, and its conflict

## Goal

The worker's checkout gains a start that is a base branch's tip with commits merged into it
one by one, each merge committed at once, and a preparation failure for a head that does not
merge cleanly, carried up the worker wire to the engine as a fact of the attempt (contract
C3, and C4's `Preparation::Conflicted`). Nothing in the engine asks for the start yet: task
`checkout-heads-engine` maps the forge's start down to it.

Acceptance:

- C3's `Start::Merged` and `Failure::Conflicted` in the checkout, the worker domain's wire
  types, and the worker wire's schema, generated code and goldens, as C3 has them; the
  service's translation decodes `Start::Merged` for the worker (where it decodes `Merge`
  today) and `Preparation::Conflicted` up into the engine's `Parts.preparation`, as
  `temper_engine_domain::Preparation::Conflicted { tag, head }` (C4), which `link.rs`
  reports as `Fact::PreparationFailed` as it does `Missing` and `Refused`.
- Prepared from `Merged { branch, heads: [h1, h2] }` on fakes: the checkout's head is a
  commit whose first parent is a commit with parents (the base's tip, h1) and whose second
  parent is h2, each authored by the repository's identity; the tree holds each side's
  files; the checkout reports no conflicts. A head the base already contains makes no
  commit. The run's first delivery pushes the new branch, its history holding the base's
  tip and every head; a save of an unchanged tree is nothing to save.
- A head whose merge conflicts fails the prepare as `Failure::Conflicted { repository, head
  }` naming that head (the second of two, when only it conflicts); nothing is pushed or
  saved, the hold stays for the client to release as any failed prepare's, and the next
  prepare of the workstream starts clean.
- `heads` empty or past 64 is refused at the entrance as an invalid start.
- The worker's answer for that attempt carries `Preparation::Conflicted` with the
  repository's tag and the head, and jig's class for it is permanent with that resource, as
  for `missing` (`channel.md`, section 5).
- The stories below pass; every test of the packages below and of the worlds
  `temper/tests/worker/checkout` and `temper/tests/worker/domain` passes.

## Inputs

- `temper/docs/design/domain/worker.md`, 4.1 (where to start: a base with heads merged in)
  and section 5 (the bullet "A base with heads merged in"; "A merge in progress" for the
  merge and its commit; "Push commits what the run checked").
- `temper/docs/design/protocol/channel.md`, section 4 (`start`'s row) and section 5 (`enum
  Preparation`, `record Conflicted`, the `preparation` bullet).
- `temper/docs/design/domain/worker.status.md`: its **Not yet** entry "4.1, 5: a base with
  heads merged in, ... (`temper-worker-domain-checkout/src/boundary.rs`, `Start`).", which
  this task removes; `temper/docs/design/protocol/channel.status.md`: its **Not yet** entry
  "4, 5: the `Merged` start and the `conflicted` preparation: ...", which this task removes.
- `temper/docs/plan/reference-m6/notes/common.md`; `notes/contracts.md`, C3 and C4's
  `Preparation::Conflicted`.

## Owns and may change

Owns the files in the header. In `temper-engine-domain` it owns `Preparation`'s new arm in
`boundary.rs` and its fact in `link.rs` alone; task `checkout-heads-engine` adds
`ForgeStart::Merged` after you.

- The prepare (`temper-worker-domain-checkout/src/hold.rs`): reuse the merge in progress's
  steps (`FetchBase`'s fetch by commit, `CheckOutMerge`, `Op::Merge`, and `Op::Commit {
  merging }` as `push_from` uses it), looping over the heads; the module's step enum and
  `fits` (the entrance) gain what the new start needs. Keep `Merge`'s behaviour as it is.
- Reader and neighbour files: every exhaustive match on the checkout's, the worker domain's,
  the wire's or the engine's `Start` and `Preparation` (`temper-worker-domain/src/{translate,
  assignment,workspace}.rs`, `temper-service/src/translate.rs`, the world's
  `tests/worker/domain/src/runtime.rs`, `tests/worker/checkout/src/world.rs`), the new arm
  only; their fixtures' literals, unchanged. If the fake checkout or fake git
  (`temper-fake-checkout`, `temper-fake-git`) cannot commit a merge during a prepare, add
  what it lacks there, in the fake's own idiom, and say so.
- Options: if the engine's `Preparation` is matched exhaustively outside `link.rs` (an
  oracle, the domain world's facts), add the arm there with the same meaning as `Refused`'s
  and say so. If the wire's goldens are generated by a script or a test flag, use it and
  name it.

## Steps

1. Types: C3 in the checkout, the worker domain and the wire schema (regenerate the
   generated code and goldens as `temper-protocol-worker-wire`'s tests require); C4's
   `Preparation::Conflicted` in the engine, its translation in the service, its fact.
2. The prepare and the entrance, then the checkout's unit tests in
   `temper-worker-domain-checkout/src/tests.rs`, beside `merge_spec` and
   `merge_preparation_transfers_conflict_paths_and_facts_keep_only_counts`:
   `merged_heads_commit_each_merge_in_order`, `merged_head_already_contained_makes_no_merge`,
   `merged_head_conflict_fails_naming_it`, `merged_heads_empty_or_too_many_are_invalid`.
3. The world `temper/tests/worker/checkout` (`src/merges.rs`, `tests/suite/merges.rs` and
   `git.rs`): `a_base_with_two_heads_merged_in_pushes_one_history_holding_all` and
   `a_conflicting_head_fails_the_prepare_and_the_next_starts_clean`. The worker domain's
   world (`temper/tests/worker/domain`) or the service's `tests/translate.rs`:
   `a_conflicted_preparation_reaches_the_engine_with_its_tag_and_head`.
4. Remove the two status texts above. `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-worker-domain-checkout -p
   temper-worker-domain -p temper-protocol-worker-wire -p temper-service -p
   temper-engine-domain --all-targets -- -D warnings`; the focused tests of those packages
   and the two worlds. Commit, push.

## Report

As the implementer's text says; name the merge commits' message and how the fake records
them.
