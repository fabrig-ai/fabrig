+++
id = "delegates-document"
part = "clients"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/documents/live_delegates.rs", "jig/crates/jig-domain-core/src/documents.rs", "jig/crates/jig-domain-core/src/documents/task_tree.rs", "jig/crates/jig-domain-core/src/documents/objects.rs", "jig/crates/jig-domain-core/src/documents/project_note.rs", "jig/crates/jig-domain-core/src/documents/deployment.rs", "jig/crates/jig-domain-core/src/tests/inbox/inbox_selection.rs", "jig/crates/jig-domain-core/src/tests/documents/frame_deployment_role.rs", "jig/crates/jig-domain-core/src/tests/documents/live_delegates.rs", "jig/docs/design/client/domain/objects.status.md"]
effort = "high"
minutes = 90
+++

# A chat's document lists every live delegate, up to the tasks child's bound

## Goal

A chat (or any task) with more live delegates than `documents.lines` is still served: its
document lists each live delegate with what it would reserve, up to `tasks.delegates`, since a
close disposes of each (`jig/docs/design/client/domain/objects.md`, 11, "Close"). Today
`documents/live_delegates.rs`'s `from_row` refuses the whole document (`Missing::TooLarge`) past
`documents.lines`, which is at most 32, so a goal of the `standard` profile's 80 live delegates
(contracts K2) can never be viewed or closed.

Acceptance:

- `jig-domain-core/src/documents/live_delegates.rs`, `from_row`: the bound is
  `limits.tasks.delegates` (the list's capacity too), not `limits.documents.lines`; a row with
  more live delegates than that cannot be (the tasks child refuses them) and stays
  `Missing::TooLarge` as a guard. The module doc cites objects.md, 11.
- Wherever the core prices a task document's delegates list by `documents.lines` (the document
  bounds in `documents.rs` and its helpers: `rg -n 'delegates' jig/crates/jig-domain-core/src/documents.rs`),
  it prices it by `tasks.delegates`, so a document's worst case holds what is served; say which
  functions in the report.
- `src/tests/documents/live_delegates.rs`: the assertion near 60-62 (`documents.lines = 2` gives
  `TooLarge`) becomes: with `documents.lines` 2 and three live delegates, the document lists all
  three; a new test `a_chat_document_lists_live_delegates_past_its_lines`: with `documents.lines`
  2 and `tasks.delegates` 8, a task with 8 live delegates gives a document of 8, in order, each
  with its takeover budget.
- `jig/docs/design/client/domain/objects.status.md`: the entry "11: a chat's document is refused
  when its live delegates pass `documents.lines` (...)." removed.
- `heavy cargo nextest run -p jig-domain-core --lib -E 'test(/tests::documents::live_delegates/)'`
  passes; then `-p jig-domain-core`, jig's core world (`heavy cargo nextest run -p
  jig-core-world`; find its package with `cargo metadata` if that name is wrong) and the client
  worlds that read a task document (`-p jig-client-wire-world`, `-p jig-engine-clients-world`
  if present), and `heavy cargo check -p temper-engine-domain --tests`.

## Inputs

- `temper/docs/plan/fabrig-scale/notes/common.md`; `notes/contracts.md`, K2.
- `jig/docs/design/client/domain/objects.md`, 11 ("Close": "The chat's document lists every live
  delegate with what it would reserve, up to the tasks child's `tasks.delegates`").

## Owns and may change

- The files of the header, for what acceptance names.
- **A change of meaning:** a document that was refused is now served; a test elsewhere that
  expects `TooLarge` or a refused watch for a task of many delegates (`rg -n 'TooLarge' jig/`
  near delegates) changes its expectation to the served document, listed in the report.
- The client's receiving `delegates` capacity is the shells' (temper's `client-capacities`);
  jig's wire carries the list as a piece within `owned_record`: no wire change.
- A memory test that prices a document's worst case and moves with the larger list: its figure
  changes only if it derives from the bound you changed; say so.

## Steps

1. `from_row` and the pricing; the tests; run them alone.
2. The worlds of acceptance, one at a time.
3. The status line; `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-domain-core --all-targets -- -D warnings`; the citations check; commit and push.

## Report

As the implementer's text asks.

## Answers

- 2026-11-03. Q: the live-delegate list is priced by `documents.lines` in
  `documents/task_tree.rs`'s `overview_bound` (which takes only `DocumentLimits`, and is called by
  the private `document_bound`s of `task_tree.rs`, `objects.rs`, `project_note.rs` and
  `deployment.rs`), outside the header: extend ownership, or another task first? A: extend
  ownership; those four helpers and the two tests that pin the bounds' relation
  (`tests/inbox/inbox_selection.rs`, `tests/documents/frame_deployment_role.rs`) join the files
  (header amended), for this pricing only. Keep every public signature: `overview_bound` stops
  pricing `List<LiveDelegate>` and its chip heaps by `documents.lines`, and
  `documents::document_with_context_bound`, which has the full `Limits`, adds them priced by
  `tasks.delegates` (the inbox's allocation goes through it, `reads.rs` and `limits.rs`); the
  doc comments of both public bounds say which holds the delegates. A test whose figure derives
  from the moved price changes only that figure, listed in the report.
