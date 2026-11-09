+++
id = "see-route"
part = "reads"
kind = "feature"
needs = ["see-surface", "person-surface"]
files = ["temper/crates/temper-engine-domain/src/route/see.rs", "temper/crates/temper-engine-domain/src/route/see/**", "temper/crates/temper-engine-domain/src/route/now.rs", "temper/crates/temper-engine-domain/src/route/asks.rs", "temper/crates/temper-engine-domain/src/route/host_route.rs", "temper/crates/temper-engine-domain/src/tests/see.rs", "temper/docs/design/domain/root.status.md", "temper/docs/design/domain/agent.status.md"]
effort = "high"
minutes = 240
+++

## Goal

Make temper's root read each of `see`'s M2 addresses as the chat's person
(`temper/docs/design/domain/agent.md`, 8.4.1, *Read as the person*;
`temper/docs/design/domain/root.md`, 5.1): one or more of jig's
`PersonRead`s an address, in the order its row's *Over* names them, each
answered to the call's flight, the forge answering its application reads
with its documents, and `SeeAnswer::Person` delivered with the names its
numbers need. The renderer is `see-render`'s; this task delivers the
records of `notes/contracts.md`, section 2.

Acceptance:

- Each M2 address's `call` routes as `SeeRead`'s doc comments
  (contracts, section 2) say: the jig reads with exactly those keys, the
  project the chat's, sent as `core::NamedAction::PersonRead(read)` under
  the call's key to its flight, the next read asked when the last is
  answered (a goal's report note only where the task read shows a tracked
  task), and `HostDelivery::See { answer: SeeAnswer::Person(..) }` once
  the last is in.
- A chat whose requester is no person (`Party::Task`, `Party::Deployment`)
  answers `SeeAnswer::NotRead(SeeNotRead::NoPerson)` at once, reading
  nothing; a first read answered `None(Why::Unknown)` or `None(Why::Busy)`
  answers `NotRead(Unknown)` or `NotRead(Busy)`; a later read's `None(_)`
  stays in its place in the record. A refusal for want of the asking
  family reaches the call as the settings read's does.
- `route/now.rs`'s `Now::PersonAnswer` arm, which `person-surface` left
  `unreachable!`, hands the answer to the flight.
- `route/asks.rs`: `core::Ask::Read { reply, scope, read, party, .. }`
  whose reply is a `see` call's flight is answered with
  `domain.forge.document(read.raw(), scope, party, env.wall,
  &env.limits.forge)` to that flight (`change N`: the change's panel;
  `changes`: `changes_subject(project)` then `landings_subject(project,
  false)`, each `None` where the forge answers none); any other reply is
  answered unknown, as today.
- `SeeNames`: the project's number and name; its people, each by number
  with the name their identity keeps, or their login; the charters at the
  newest revision, each by number with the name the settings inventory
  gives it.
- `Address::NotYet` and its arms are gone; nothing answers `SeeAnswer::NotYet`.
- The two world rows the change trips are repaired:
  `temper/tests/engine/domain/tests/suite/configuring.rs`'s
  `an_asking_chat_reads_each_address` loses its rows `me`, `task 4 tree`
  and `goals ended` (the person's addresses get their own table in
  `see-stories`); `temper/tests/engine/domain/tests/suite/settings/b2.rs`'s
  `asking_declarations` expects `see me: not read: this chat has no person
  to read as.` from its delegate's run.
- `temper/docs/design/domain/agent.status.md`: the **Not yet** entry
  *Read as the person* is deleted; `temper/docs/design/domain/root.status.md`:
  the **Not yet** entry on 5.1 and 5.3 keeps only a party's read ("a
  party's read of the forge's is answered unknown: the forge makes its
  documents for watches and a run's read as its person alone").

## Inputs

- `temper/docs/plan/ui-ask-m2/notes/common.md` and
  `temper/docs/plan/ui-ask-m2/notes/contracts.md`, sections 1 and 2.
- `temper/docs/design/domain/agent.md`, 8.4.1: the rows `me` to `inbox`
  (their *Over* column), *Read as the person*, the person's *A refusal*;
  `temper/docs/design/domain/root.md`, 5.1 (*A run's read as its person*)
  and 5.3's table; jig's `jig/docs/design/domain/engine.md`, 7.3.
- The flights to copy: `route/see.rs`'s `call` (the settings flight),
  `route/see/proposal.rs` (`read`, `answered`), `route/see/forge.rs`
  (`file` and `answered`, a flight of two reads),
  `route/host_route.rs` (`FlightRender`, `settings`, the see branch at
  about line 814), `route/now.rs` (about line 153);
  `route/panels.rs`'s `document` for how the forge makes a document for a
  party; `route/see.rs`'s `Ways` arm for the requester.
- Names: `route/host_route.rs`'s project people (about line 880) and
  `jig_domain_core::documents::settings_inventory`'s setting names.

## Owns and may change

- **Owns:** `route/see.rs`'s `call` and the new `route/see/person.rs`;
  `route/see/forge.rs` where it must learn the forge's documents;
  `route/now.rs`'s person arm; `route/asks.rs`'s `Ask::Read` arm;
  `host_route.rs`'s flight variants for it; the parser test file.
- **Reader files:** the two world tests above, only as said; the arms the
  compiler names for `Address::NotYet`'s removal
  (`route/ways/view/address.rs`, `route/see/forge.rs`).
- **Options, pre-approved:** the flight may be a new `SeeReading` variant
  holding the reads gathered so far, or a `FlightRender` of its own; pick
  the smaller and say which. If a jig read's answer is too large for the
  root's flight memory bound, cut nothing: report it. If the project's
  name has no read the root can make without a sign-in, take it from the
  project read's document (the first read of `me` and `project`) and, for
  the other addresses, from the root's own record of the project; say
  which.

## Steps

1. The flight and its reads, address by address, `me` and `project`
   first; `NoPerson`; `NotRead`.
2. The forge's `Ask::Read` answer; `change N` and `changes`.
3. The names.
4. `Address::NotYet` removed; the two world tests repaired; run them.
5. The reads an address makes are pure functions of the root's, unit
   tested without a world, since jig answers every person read unknown
   until `person-reads` lands and the joined stories are `see-stories'`:
   `next(address: &Address, project: u32, gathered: &Gathered) -> Step`,
   the step after what was read so far (`Gathered` the flight's record of
   the jig answers and the forge's documents, as you shape it), `Step`
   asking a jig `Read` (an application read of the forge's among them), or
   delivering. A table in
   `temper-engine-domain/src/tests/see.rs`, a row an address: the reads it
   asks, in order, with their keys (a tracked task's report note asked, an
   untracked one's not), and what a first `None(Unknown)` or `None(Busy)`
   delivers. Break one key once (`goals` without `tracked`) to see its row
   fail, and restore.
6. The two status entries.
7. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain -p
   temper-engine-smith -p temper-engine-domain-world --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper-engine-domain --lib`;
   `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(/configuring|asking|settings::/)'`; `heavy cargo check -p
   temper-service --tests`. Commit and push.

## Report

As the implementer's text asks; for each address, the reads its flight
makes.
