+++
id = "public-component"
part = "jig-public"
kind = "feature"
needs = ["public-limits", "public-vocabulary", "public-wire", "visibility-wire"]
files = ["jig/crates/jig-protocol-engine-clients/src/public.rs", "jig/crates/jig-protocol-engine-clients/src/public/**", "jig/crates/jig-protocol-engine-clients/src/boundary.rs", "jig/crates/jig-protocol-engine-clients/src/component.rs", "jig/crates/jig-protocol-engine-clients/src/lib.rs", "jig/crates/jig-protocol-engine-clients/src/translate/owner.rs", "jig/crates/jig-protocol-engine-clients/src/tests/public.rs", "jig/crates/jig-protocol-engine-clients/src/tests/public/**", "jig/crates/jig-protocol-engine-clients/src/tests.rs", "jig/crates/jig-protocol-engine-clients/src/tests/fixtures.rs", "jig/testing/jig-test-service/src/clients.rs", "jig/testing/jig-test-service/src/clients/**", "jig/tests/engine-clients/src/peer/public.rs", "jig/tests/engine-clients/tests/suite/public.rs"]
effort = "high"
minutes = 150
+++

# The clients' component serves public reads: identifiers, admission, kinds and records

## Goal

The clients' component takes a public read with no credential, maps its public identifiers
under a key it keeps in the secrets' store, admits it within its room, its reads' bucket and
its bytes' bucket, hands it to the engine's root as the public party's read, and turns the
answer into jig's public records or an application's envelope; `me` gives a public project's
identifier (`jig/docs/design/protocol/client.md`, section 19; `jig/docs/design/domain/people.md`,
13.5 and 13.8). The HTTP route is `public-http`'s; this task's stories hand the component the
message directly.

Acceptance:

- **The message** (`boundary.rs`, appended last in each enum; `notes/contracts.md`, section 4):
  `FromTransport::PublicRead { exchange: Token, read: PublicRead }`, no credential field;
  `pub struct PublicRead { pub owner: u16, pub version: u16, pub kind: u16, pub ids: Box<[u64]> }`
  (at most two ids); `ToTransport::Public { exchange: Token, reply: PublicReply }`;
  `pub enum PublicReply { Document { bytes: Box<[u8]>, decided: Time }, Unknown, Busy,
  NotServed { record: Box<[u8]> } }`; `Event::Public { reply: ReplyTo, answer:
  core::PublicAnswer }` (the root's `Held::PublicRead`, or its busy hand-back).
  `Request::Core(core::Event::PublicRead { reply, read })` carries the admitted read, `read`
  the core's `PublicRead { owner, version, kind, project, task }` with numbers.
- **The key:** `SecretOwner::PublicNames` appended last; at start the component asks
  `SecretsRequest::Get { owner: PublicNames }`; none kept, it draws 32 bytes from its
  `CryptoRng` (`sign_in::draw`) and keeps them (`SecretsRequest::Keep`), in the record the
  store's `PublicNames` row holds (`visibility-surface`); until it holds the key every public
  read is `Busy`, and `me` gives `None`.
- **`public::Names`** (`src/public/names.rs`), exactly as the contracts' section 4:
  `Names::new(key: &Secret) -> Self`, `project(&self, n: u32) -> u64`, `task(&self, n: u64) -> u64`,
  `number(&self, id: u64, of: Of) -> u64`, `pub enum Of { Project, Task }`; the 4-round Feistel
  permutation over two 32-bit halves, round function the first 4 bytes of
  `skein_lib::CryptoRng::new(key, stream)`, `stream = (domain << 40) | (round << 32) | half`,
  domain 1 for projects and 2 for tasks.
- **The kinds:** `Settings.public: List<PublicKind>` with `pub struct PublicKind { pub owner: u16,
  pub kind: u16, pub first: u16, pub last: u16, pub task: bool, pub bound: u32 }`; and
  `pub fn jig_kinds(bounds: &PublicIndexBounds) -> [PublicKind; 3]` (`src/public.rs`): owner 0,
  kinds 1-3 (`PUBLIC_PROJECTS`, `PUBLIC_BOARD`, `PUBLIC_CARD`), versions
  `PUBLIC_VERSION..=PUBLIC_VERSION`, *card* alone taking a task, each `bound` from
  `jig_client_protocol_wire::public_bounds`. With `limits.public.on()`, `Component::new`
  refuses (`Unusable::Limits`) a kind whose `bound` exceeds `public.answer_bytes`.
- **The order of a read**, the same work whatever its identifiers name:
  1. the ids' count against the kind (`projects` none, `board` one, `card` two, an
     application's one or two as `task` says) and the kind declared: not -> `Unknown`, nothing
     handed on; the version within `first..=last`: not -> `NotServed { record }` (the record
     `served::not_served` makes, naming the kind's versions), nothing handed on;
  2. public reading off (`rate == 0`), the key not held, the public room (`public.reads` in
     flight) full, the reads' bucket empty, or the bytes' bucket short of the kind's `bound`:
     `Busy` at once, nothing handed on;
  3. one read taken, the `bound` reserved; ids mapped (`number(_, Project)`, `number(_, Task)`)
     and handed on as `core::Event::PublicRead`, for every well-formed read alike.
  The buckets refill by `Env.now` (monotonic), at `rate` and `bytes` per second up to their
  bursts, never below empty. Every end gives the reservation back: a document gives back
  `bound` less the bytes sent; `Unknown`, `Busy` from the root, an encoding fault (answered
  `Unknown`) and an exchange the transport has gone from give back all of it.
- **Its records:** `PublicDocument` -> `jig_client_protocol_wire::public_v1` (projects carry
  `max_age` from `limits.public.max_age`), every project number through `Names::project`,
  every task through `Names::task`, `parent` and `lineage` alike; `Application` ->
  `public_v1::PublicApplication { tasks: mapped, in the connector's order, record }`; within
  `public_limits` and `answer_bytes`, else `Unknown`.
- **`me`:** `translate/owner.rs` fills `MeProject.public` with `Names::project(project)` when
  `FrameProjectRole.public` and the key is held, else `None`.
- Every other exchange without a credential is answered signed out as now (`routes::unauthenticated`),
  and none takes the public room.
- **The service** (`jig-test-service`): its secrets map takes the component's
  `PublicNames` to the store's; its `Settings` give `public: jig_kinds(..)` from the core's
  `Limits.public` and `people.projects`; the component's `core::Event::PublicRead` goes to the
  root, and the root's `Held::PublicRead` (and busy hand-back) comes back as `Event::Public`.
- Step tests in `src/tests/public.rs` (a module line in `src/tests.rs`):
  `identifiers_map_both_ways_apart_for_projects_and_tasks` (round trips over seeded numbers,
  a project's and a task's identifiers of one number differ, and an identifier mapped as the
  other kind gives another number), `identifiers_are_the_same_after_a_restart` (the kept key
  read back), `the_key_is_drawn_and_kept_once_and_reads_are_busy_until_held`,
  `a_kind_past_answer_bytes_refuses_the_start`, `a_rate_of_zero_answers_every_read_busy`,
  `past_the_room_the_rate_or_the_bytes_a_read_is_busy_and_nothing_is_handed_on`,
  `an_unknown_kind_a_wrong_id_count_and_an_unserved_version_reach_no_engine`,
  `every_well_formed_read_is_handed_on_alike` (an identifier naming nothing, a private
  project's and a public one's give the same message shape to the root),
  `reservations_are_given_back_on_every_end` (document, unknown, busy at the root, encoding
  fault, exchange gone; the bytes' bucket whole after), `bursts_bound_reads_at_once`,
  `a_document_crosses_as_public_records_with_identifiers_alone` (no number of the source
  state appears in the bytes: numbers chosen as markers past the identifiers' range),
  `an_application_document_crosses_in_jigs_envelope`,
  `me_gives_a_public_projects_identifier_and_none_for_a_private_one`.
- World stories in `jig/tests/engine-clients/tests/suite/public.rs` (a module line in
  `main.rs`), through a peer method in `src/peer/public.rs` (a module line in `src/peer.rs`)
  that hands the component `FromTransport::PublicRead` and the root's answers, client.md 19's
  component list: `public_reads_past_the_room_rate_and_bytes_are_busy_while_signed_in_reads_are_answered`,
  `two_reads_completing_at_once_and_one_whose_exchange_is_gone_leave_the_bucket_whole`,
  `an_unknown_kind_a_private_project_an_identifier_naming_nothing_and_a_task_not_public_are_unknown_with_the_same_messages_to_the_engine`,
  `identifiers_survive_a_restart`.

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md`; `notes/contracts.md`, sections 1,
  2 (the key's row), 3 and 4.
- `jig/docs/design/protocol/client.md`, section 19 (and 8, signed out); `jig/docs/design/domain/people.md`,
  13.5 and 13.8; `jig/docs/design/protocol/store.md`, section 9 (the key's row).
- As they stand: `jig-protocol-engine-clients/src/boundary.rs`, `component.rs` (the
  credential-less routes, `admitted()`, the secrets' answers, `CryptoRng`), `sign_in.rs`
  (`Pending`, `draw`), `served.rs`, `reads.rs` (how a read is handed to the core and its
  answer returned), `translate/owner.rs`; `public-wire`'s `public_v1`, `PublicIndexBounds`,
  `public_bounds`, `public_limits`; `jig-test-service/src/clients.rs`.

## Owns and may change

- Owns the files in the header.
- **Settings literals and forced arms,** each edited only for the field (`public:
  List::with_capacity(0)` in a test fixture, or `jig_kinds(..)` where a service builds its
  settings) or the arm: `jig/tests/engine-clients/src/fixtures.rs`,
  `jig/tests/client/src/joint/component.rs`, `jig/tests/client-protocol/src/{engine.rs,
  fixtures.rs}`, `jig/tests/engine-clients/src/peer.rs` and `peer/closing.rs`; temper (a
  required field and arm): `temper/crates/temper-service/src/clients.rs` (its secrets map's
  `PublicNames` arm to the store's, and `public: jig_kinds(..)` in `client_settings` from
  its core limits; the forge's kinds are `forge-public`'s), `temper/crates/temper-service/src/tests/clients.rs`,
  `temper/tests/client/src/component.rs`. Any other the compiler names: the same.
- `jig/testing/jig-test-service/src/route.rs`: the hand-off of `Held::PublicRead` to the
  component and of the component's public read to the root, only for those arms.
- Not this task's: the HTTP route (`public-http`), the core's answer (`public-reads-core`):
  until it merges the core answers unknown and the stories that need a document drive the
  component's `Event::Public` directly.
- Pre-approved: if the transport has no message that tells the component an exchange is gone,
  give the reservation back when the answer finds no exchange, and say so. If the store's
  secret record cannot hold a 32-byte value without an expiry, keep it with the record's
  greatest expiry and say so. If `Names::number` of a project identifier gives a number past
  `u32`, hand it on as is (the core's lookup misses it) — never refuse it here.

## Steps

1. The boundary's types, `Names`, the kinds and `jig_kinds`; the key at start;
   `heavy cargo check -p jig-protocol-engine-clients --tests`.
2. Admission, buckets, reservation and refunds; translation; `me`; the service's glue and the
   literals until `heavy cargo check -p jig-test-service -p temper-service -p jig-engine-clients-world --tests`
   passes.
3. The step tests, each run alone first
   (`heavy cargo nextest run -p jig-protocol-engine-clients --lib -E 'test(/public::/)'`),
   then the world's (`heavy cargo nextest run -p jig-engine-clients-world --test suite -E 'test(/public::/)'`).
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`; clippy on every package you changed; the
   focused tests of `jig-protocol-engine-clients`, `jig-test-service`,
   `jig-engine-clients-world`, `jig-client-world`, `jig-client-protocol-world` and
   `temper-service`; the citations check.
5. Commit (why: the component serves public reads with opaque identifiers within its own
   room and budgets); push your branch.

## Report

As the implementer's text says, and also: `jig_kinds`' bounds at the dev.fabrig.ai figures,
how the key is kept, how an exchange's end reaches the refund, and every literal and arm
outside the header's files.

## Answers

2026-11-03. **The user's decision on public titles.** User, 2026-11-03 15:40Z: public reading shows titles. A goal's and a task's title (the first line of its spec, as jig's client contract names a task's title) crosses into its public line, and a public project's name crosses into its public record. There is no author flag and no human publication capability for these. Everything else in people.md 13.3's "No text at all" list still never crosses: spec beyond its first line, instructions, inputs, results, words, questions and answers, transcripts, notes, reasons, labels' meanings, proposals, escalations, people's and parties' names, charters', procedures' and roles' names. Review finding 1 is settled by the user's decision.

For this task: Fill the title and the name from their sources. A spec amendment changes the title, so the public line's last-changed covers it; the recheck after asynchronous work reads the current title.

2026-11-03. **A public project's name in the wire** (answered for `public-wire`): jig's
*projects* answer is a list of `public_v1::PublicProject { project, name }`, and
`public_v1::PublicBoard` carries `name` after `project`; the translation maps the core's
`PublicProject` (its project through the permutation) and `PublicBoard.name`, and
`PublicLine.title`, as they are (`public-index` adds them to the core's types).
