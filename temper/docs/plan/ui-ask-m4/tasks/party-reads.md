+++
id = "party-reads"
part = "party"
kind = "feature"
needs = ["core-surface"]
files = ["jig/crates/jig-domain-core/src/reads/party.rs", "jig/tests/core/tests/suite/party_reads.rs", "jig/tests/client-protocol/tests/suite/party_notes.rs", "jig/tests/client-protocol/tests/suite/main.rs"]
effort = "high"
minutes = 150
+++

## Goal

Let a person read, correct and delete their own preferences through a
client, whatever their roles, and nobody else, an owner of the deployment
included (`jig/docs/design/domain/engine.md`, 10.1, *Read and kept by its
party alone*; `jig/docs/design/client/domain/contract.md`, section 6): fill
`reads/party.rs`, which `core-surface` made and wired.

Acceptance:

- `reads/party.rs`' predicate admits a party's read, edit, deletion and
  offered acts on `Scope::Party { party }` exactly when the signed-in
  party is the person numbered `party`; every other party's read is
  answered unknown and their edit or deletion refused as unknown, as a
  party who may not see a project's note is. No project role is asked of
  the admitted person: the edit and deletion name project 0, and the
  role gate (`acts/standing.rs`) is not consulted for this scope.
- A note document of a party's scope (`documents/project_note.rs`) carries
  its writer, references, `changed` and revision, and offers its person
  `Edit` and `Delete` (`documents/offers.rs`).
- `jig/tests/core/tests/suite/party_reads.rs`, its entries seeded as store
  rows before the start (as `jig/tests/core/tests/support/reads.rs:193`
  seeds a scope's line and entry), one table story,
  `a_person_alone_reads_and_keeps_their_preferences`: person P reads their
  two lines, corrects one at its revision and deletes the other, the store
  rows as expected; P's correction at an old revision refused as moved;
  person Q, a member of P's project, and the deployment's owner each read
  P's scope as unknown and are refused P's edit and deletion as unknown,
  the store unchanged; a project's notes listing (`Tasks { project }`) and
  P's project's deletion leave P's scope.
- jig's client protocol world, `tests/suite/party_notes.rs` (its `mod` line
  in `main.rs`): P's `Read::Notes { scope: Party }` over the client wire
  answered with P's lines and Q's answered unknown, P's `EditNote` and
  `DeleteNote` with project 0 answered done.
- Delete the **Not yet** entries `10.1, *Read and kept by its party alone*`
  in `jig/docs/design/domain/engine.status.md` and `6: no Party note
  scope` in `jig/docs/design/client/domain/contract.status.md`.

## Inputs

- `temper/docs/plan/ui-ask-m4/notes/common.md`; `notes/contracts.md`,
  sections 1 and 2.
- `jig/docs/design/domain/engine.md`, 10.1, *Read and kept by its party
  alone*; `jig/docs/design/client/domain/contract.md`, section 6, *Notes are
  read by scope*.
- `core-surface`'s report on the trunk and `reads/party.rs`' module doc:
  the predicate and its call sites.
- This cut's survey of the sites: `reads/vocabulary.rs:545`, `:566`;
  `reads/rows.rs:18`, `:32`, `:1063`; `acts.rs:226`; `acts/standing.rs:25`;
  `admission/roles.rs:219`; `documents/project_note.rs:458`;
  `documents/offers.rs:244`; `routing/effect/note.rs:245`.
- Patterns: `jig/tests/core/tests/suite/stories.rs:371` (a person's
  correction); `jig/tests/client-protocol/tests/suite/turn_read.rs:60` (a
  read through the client).

## Owns and may change

- **Owns:** the files in this spec's header. Siblings `party-runs`,
  `party-brief` and `withheld` run beside this task in other modules of
  `jig-domain-core`; touch none of theirs.
- **May change:** a call site's one line where `core-surface` wired
  `reads/party.rs`, if its arguments must grow, and say so;
  `documents/offers.rs`, the note acts' offer for this scope alone.
- **Options, pre-approved:** if the client protocol world cannot seed a
  note, write P's lines first through P's own corrections of seeded rows,
  or seed them as the core world does, and say which.

## Steps

1. The predicate; the offers.
2. The stories; break the predicate once (admit any person) to see Q's
   row fail, and restore.
3. `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p jig-core-world -p
   jig-client-protocol-world --all-targets -- -D warnings`; `heavy cargo
   nextest run -p jig-domain-core --lib`; `heavy cargo nextest run -p
   jig-core-world`; `heavy cargo nextest run -p jig-client-protocol-world`.
   Commit and push.

## Report

As the implementer's text asks.
