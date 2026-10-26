# Session 03: jig's read of one turn

A committed turn can be read by task, attempt and turn number, with the access of its task's activity. The engine keeps
smith's bytes opaque; the client's link decodes them. Missing or invisible is unknown, past the transcript's horizon is gone,
and an oversized turn carries its size, both alone and on a page. Five increments, each landing alone. Needs at start:
**jig on skein's and smith's main** only. This session runs beside jig 02; it builds **jig's read of one turn, landed**.
The downstream repository's tools plan repins onto the last landing containing all three capabilities (jig's named send,
this turn read and jig 05's deadline spans), including this session's last step, `jig-03.5`. No pin moves here.

## Read first

- **Design:** `docs/design/client/domain/contract.md` 5.1, 6, 7.1 and 8; `client/domain/client.md` 7, 10 and 14;
  `protocol/store.md` 7.3, 8.3, 8.5 and 11.1; `protocol/client.md` 3, 6 and 9; `protocol/README.md` 7.
- **Plan:** `decisions.md`, designer's answer 4 and planner's decision 3; `later.md`, transcript retention;
  `findings/jig-section-13.md` 2 and 3; `jig/01-repin.md` (smith's v5 turns); `jig/02-named-send.md` (shared files).
  The dated findings precede the design answer: their absent contract row is now present.
- **Code:** `crates/jig-domain-core/src/reads.rs` (`start_jig`, `loaded`), `reads/vocabulary.rs`
  (`Read::Turn`, `Loaded`, `Why`), `reads/rows.rs` (`scope_allowed`, `from_store`, `object`);
  `crates/jig-protocol-store/src/reads/point.rs` (`turn`, `finish`) and `src/transcripts/read.rs`
  (`party`, `length`, `header`, `arrival`); the two ends' `translate/reads.rs`; the link's `component.rs`
  (`read_answer`, `read_value`) and `translate/pages.rs` (`turn_fields`).
- **Worlds:** `tests/core/src/stories/reads.rs`, `tests/core/tests/reads.rs`; `tests/client-protocol/src/client/thread.rs`
  (`Story::reading_chat`) and `tests/client-protocol/tests/client.rs` (the committed activity story).
- **Rules:** `AGENTS.md`; `docs/development/workflow.md` 2; skein's `docs/foundation/programming-model.md` 3, 6.3 and 10.3;
  `testing-strategy.md` 2.1, 2.2, 6, 8 and 10; `development.md` 5.3, 5.5 and 5.6.

## What the design asks

- **contract.md 6 and 7.1:** `turn{n, attempt, turn}` is a read, never a subject, with Activity's access. It has one terminal
  and changes no state. Busy is one answer, not a request to retry inside the client. Existing sign-out and version terminals
  apply. Unknown includes an absent turn and a task the party may not see.
- **contract.md 5.1 and 6; store.md 7.3 and 8.3:** a page gives an oversized turn's byte count; a single read gives its
  bytes or `TooLarge { bytes }` at that read's bound. A failed frame or a turn that does not decode is malformed.
- **client.md 7, 10 and 14:** the client also answers a single turn past `document_bytes` with its size. Page turns keep
  their existing decoded/undecodable shape, adding the size to `TurnProblem::TooLarge`.
- **contract.md 8; protocol/README.md 7:** one current wire version, both ends from one pin; another version is refused,
  nothing translated. Planner's decision 3 makes this version 9.
- **store.md 8.5 and 11.1:** preserve the gone terminal, but build no retention policy or horizon producer here;
  transcript retention belongs to `later.md`. A missing frame is not evidence of a horizon.

## Choices this plan makes

- **The existing archive is the value.** Add `ReadValue::Turn(Box<ArchivedTurn>)`, reusing task, name, cumulative spend,
  read position, time and decoded turn from activity. The wire reuses `ArchivedTurn`; no parallel turn document.
  A single read's undecodable archive becomes `ReadNone::Malformed` or `TooLarge { bytes }`; a page keeps `TurnRead`'s
  version/malformed/too-large distinction. A smith version the decoder cannot read is malformed for the single read.
- **A size is the smith payload's length,** the same number `TurnTooLarge.bytes` already carries on pages. Admission
  still counts the complete store row and joined owner, and the complete client document; neither overhead is reported as
  the turn's size. The store must also admit its small failure evidence within the read's budget.
- **Failure evidence keeps its owner.** A typed store failure carries the requested identity and the durable task owner.
  The core authenticates that owner before publishing a size or corruption result, including after the task leaves live
  state. An unadmitted evidence row is Busy; an absent owner or mismatched identity is Unknown. No empty transcript is
  invented to stand for an oversized or unread frame.
- **The bounds stay where they are.** `start_jig` uses `Limits.load_bytes` for single reads and activity pages. A page
  counts its activity line as well as the archive, so their costs differ, but raising the admitted transcript size alone
  cannot make a turn larger than `load_bytes` readable whole. The downstream large-turn fixture must account for this;
  this session adds no separate public limit without design. The legacy wire `activity_turn_turn` cap must not silently
  cap the new single read: its archive uses `turn_carried_bytes` and the client's document limit.
- **Wire ownership:** 03.1 moves the version alone. Later increments extend that v9 schema and regenerate its code and
  goldens, without another version move. This changes neither the request table's version nor the store's core family.
- **Shared files with jig 02:** 03.1's import/version updates touch `crates/jig-client/src/tests/{sign_in,store}.rs`,
  `tests/client-protocol/src/client.rs` and `tests/client-protocol/tests/client.rs`; exhaustive read matches also
  meet `testing/jig-client-test-domain/src/family/client.rs`. Rebase these edits at each landing; there is no capability
  dependency on named send. `jig-client-domain/src/contract/{reads,turn}.rs` and `jig-client/src/facts.rs` are this session's;
  jig 02 owns `boundary.rs`'s Send/Outcome and `jig-client/src/client.rs`'s Send forwarding. New turn stories have their own
  integration-test file, adding no lines to the nearly full `tests/client-protocol/tests/client.rs` beyond its version update.

## Rules that bind it

- The README's naming rule and code-organization's one-owner rule: jig names no downstream application; the core links no
  smith crate. Only the link decodes smith's bytes, through `jig-protocol-transcript`. Edit no design document or STATUS row.
- Change the existing enums and every exhaustive consumer in the same increment; add no catch-all, compatibility alias,
  retry loop, watched turn, or second fixture for the same world. The new modules below keep additions out of long files.
- All cargo commands run unoptimized. Unit tests stay under 20 ms, focused world/joint stories under 0.5 s each;
  the focused and fuzzy budgets remain `.config/nextest.toml`'s `global-timeout`s, 15 s and 60 s. Run each named proof once, besides the gate. A proof naming a new
  test becomes executable when its increment builds it; the supervisor checks the sketches at readiness.

## Increments

### 03.1 The client wire at version 9
Scope: the version move alone, protocol/README.md 7 and contract.md 8; step `jig-03.1`.
Needs: nothing beyond jig on skein's and smith's main.
Files: `crates/jig-client-protocol-wire/schema/client-v8{,-part1}.schema` renamed to `client-v9{,-part1}.schema`;
`src/generated/v8.rs` replaced by `v9.rs`, `src/lib.rs`, `golden/v8/` replaced by `golden/v9/`;
`tests/client-wire/tests/{drift,fuzzy_client_wire}.rs`. Mechanical imports and explicit jig-version literals in the users
found by `rg -l 'jig_client_protocol_wire|client-v8|golden/v8' crates testing tests examples --glob '*.rs' --glob '!**/generated/**'`;
notably both protocol ends, their HTTP mappings, the native shell, jig-client's tests, test-service and the client worlds.
The jig-version literals are those `rg -n 'first: 8|last: 8|version: 8|Owner::Jig, 8|minimum: 8|maximum: 8|version\(\), 8|contains\(8\)|\[1, 0, 0, 0, 0, 8,' crates testing tests examples --glob '*.rs' --glob '!**/generated/**'`
finds, in 32 files at jig `1899792f`:
- the scripted engines' hello answers, `self.engine.versions().contains(8)`, in `tests/client/src/scripted.rs` and
  `tests/ops/src/client/transport.rs`: left at 8, every client story reads "Client out of date";
- `assert_eq!(….version(), 8)` in `crates/jig-protocol-engine-clients/src/tests/fixtures.rs` (`final_record`),
  `crates/jig-client-protocol-link/src/tests/attempts.rs` and `tests/engine-clients/tests/requests.rs`;
- `SERVICE_ANSWER`'s version byte, `[1, 0, 0, 0, 0, 8, …`, in `crates/jig-protocol-engine-clients/src/tests/fixtures/answers.rs`;
  both numbers of `Owner::Jig, 8, 8` in `src/tests/fixtures/envelope.rs`; `Versions { minimum: 8, maximum: 8 }` in
  `tests/core/src/world.rs`;
- the rest: `crates/jig-protocol-engine-clients/src/tests/{asks,reads}.rs`, `src/tests/fixtures/{reads,services,watches}.rs`;
  `crates/jig-client-protocol-link/src/{tests,tests/stream,tests/fixtures,tests/fixtures/link_stream}.rs`;
  `crates/jig-client-protocol-wire/src/tests/fixtures.rs`; `crates/jig-client/src/tests/unlinked.rs`;
  `testing/jig-client-scripted-engine/src/truth.rs`; `tests/client/src/{stories,stories/client_construction,joint/transport}.rs`;
  `tests/ops/src/{client/stories,joint/bytes}.rs`; `tests/client-protocol/src/fixtures.rs`;
  `tests/engine-clients/src/{peer,fixtures}.rs`, `tests/engine-clients/tests/{closing,requests,stream,reads}.rs`.
A family's number or version is no jig version, and stays: `Served { family: Some(9), first: 2 }`, the link's `first: 23`
and `24`. The words go too: the three `a_v8_*` tests of `tests/engine-clients/tests/requests.rs` become `a_v9_*`; `v8` leaves
the `unreachable!` and `expect` texts of `tests/client/src/joint/wire/{refusals,historical,historical/tests}.rs`;
`docs/development/tech-debt.md` names `client-v9*.schema` and `generated/v9.rs`.
Shape: `pub mod v9`, its existing root re-export, and `VERSION: u32 = 9`; every jig served range is `9..=9`. `schema/client-v9.schema`'s first line is
`family jig-client-wire 9`, which the regeneration carries into `v9.rs` and the goldens. Families keep
their own versions. Update drift's two input paths, generated-file arm and golden directory, and the fuzzy golden reader.
Bootstrap the generated filename with the existing generated source so the test runner builds, then regenerate with
`SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-client-wire-world --test drift -E 'test(=client_wire_schema_matches_generated_code_and_goldens)'`.
Run the same drift test without the environment variable. No v8 module or translated reader remains. Add no turn fields yet.
Test shape: the existing drift and wire round trips; in `tests/client-protocol/tests/client.rs`, retain
`an_engine_outside_jig_s_version_leaves_the_client_out_of_date_sending_nothing`, with its incompatible version explicitly 8:
`Story::serve_jig(8)` in place of its `u32::MAX`, expecting `VersionRange { first: 8, last: 8 }`; and
`a_request_pending_across_a_restart_that_stops_serving_its_family_stays_unsent` serves 9 to 9 where it served 8. `cargo fmt`
runs after, as it rewraps a `vec![Readout::NotServed { .. }]`.
Use `fixtures::domain_limits`, `engine_limits` and the existing `Story::serve_jig`; family-version expectations stay intact.
Proof: joint, `cargo nextest run -p jig-client-protocol-world --test client -E 'test(=an_engine_outside_jig_s_version_leaves_the_client_out_of_date_sending_nothing)'`.
A v9 client refuses an engine serving v8 and sends nothing; under 0.5 s, above the wire's round trips.
Don't: change message fields, the store or table versions, smith's transcript version, generator behaviour, or `Cargo.lock`.
Size: at most 180 hand-written production lines; generated code, goldens and mechanical test imports aside.
Removes: the v8 schema paths, generated module, golden directory and imports; no retained old-version reader.

### 03.2 The store and core preserve the turn's failure
Scope: single-turn TooLarge and Malformed through the real store and core, with typed terminals through the wire and link;
contract.md 6, store.md 7.3 and 8.3; step `jig-03.2`.
Needs: the client wire at version 9, from 03.1; nothing beyond the session's start.
Files: `crates/jig-domain-core/src/reads.rs`, `reads/vocabulary.rs`, new `reads/turn.rs`, `src/tests/read_activity.rs`;
`crates/jig-protocol-store/src/{reads/point.rs,transcripts/read.rs}`, new `transcripts/read/party.rs`,
`src/tests/reads/turns.rs` and the existing `src/tests/fixtures/reads.rs`;
`crates/jig-client-protocol-wire/schema/client-v9.schema`, `src/tests.rs`, generated v9 and goldens;
`crates/jig-protocol-engine-clients/src/translate/reads.rs`, `src/tests/reads.rs`;
`crates/jig-client-domain/src/contract/reads.rs`; `crates/jig-client-protocol-link/src/translate/reads.rs`, `src/tests/reads.rs`;
the exhaustive Why/ReadNone adapters in `testing/jig-client-test-domain/src/family/client.rs`,
`crates/jig-client-view/src/{application,page/routing}.rs`, `tests/client/src/joint/read_packets.rs`,
`tests/ops/src/client/transport.rs`, and the client `ReadNone` matches in `examples/ops/client-domain/src/family.rs` (two),
`tests/client-protocol/src/family.rs` and `tests/client/src/scripted.rs`; `tests/core/src/stories/reads.rs`, `tests/core/tests/reads.rs`.
Shape: `Loaded::TurnProblem` and `TurnFailure` as sketched below; `Why::{TooLarge { bytes }, Malformed}`;
matching wire `ReadNone` variants (`too_large: TurnTooLarge`, `malformed`) and client `ReadNone` variants.
The reference UI and family-read adapters never ask for a single turn; the new cases join their existing unavailable/unknown
arms, without adding UI words. Extend the existing wire-none round-trip table and regenerate v9 as in 03.1.
`reads/turn.rs` validates the scope, identity, evidence size and owner role, after `loaded`'s existing sign-in check;
its answer still leaves through `Held::Read`. Ordinary `Loaded::Gone` keeps its existing route.
In the store, distinguish exhausted search/no transcript place from a failed frame. The former stays an empty result;
the latter gives Malformed for a single turn. Read the fixed metadata and payload length before allocating the payload;
reuse the existing header/length validation and bounded frame walk. An oversized selected frame gives its payload length,
not `Failure::Cut`; metadata and owner still count against `party.bytes`. Normal frame checksums remain checked. Keep the
new party-specific handling in `read/party.rs`, called by `read.rs`, whose 752-line base must stay below 800. Preserve the
existing reply/page routes. Recompute core/store worst cases for the evidence carrier; no unpriced owner is held.
Test shape: replace `an_unread_turn_is_an_empty_page` with `an_unread_turn_is_malformed`; replace the one-byte-budget
expectation with Busy (even evidence cannot fit). Reuse `party_store`, `turn_range`, `waiting_party`, `read_disk`,
`corrupt_disk` from `src/tests/reads.rs`; add `an_oversized_turn_keeps_its_payload_size`. In core's `read_activity.rs`, reuse
`watch_core`, `read_start`, `loaded_answer`: `an_invisible_turns_failure_is_unknown`, `a_turn_failure_rechecks_its_sign_in`,
`a_mismatched_turn_failure_is_unknown`. Existing wire-none round-trip tables gain both variants. Units under 20 ms.
The joint proof adds `a_turn_past_the_read_bound_reports_its_size` to `tests/core/tests/reads.rs`, using
`stories::reads::world`, `reads::{ask,latest}` and `effects::World::turn`; one fixture variant in `stories/reads.rs`
derives limits from its existing builder, admitting a payload larger than the read budget while its owner evidence fits.
Name both bounds beside that deviation; retain the world's durable-read fence and memory checks. Under 0.5 s.
Proof: joint, `cargo nextest run -p jig-core-world --test reads -E 'test(=a_turn_past_the_read_bound_reports_its_size)'`.
The real core and store, on simulated files, answer the payload's size after durability. The public client's turn constructor
arrives in 03.3; this proof uses the core's already-existing `Read::Turn`.
Don't: expose failure facts before access checks; turn an absent frame into Gone; build transcript retention;
decode smith in the store/core; discard the owner's bytes from the bound; change another read's cut semantics.
Size: at most 300 production lines, including exhaustive adapters; tests and regenerated code aside.
Removes: the single-turn paths that flatten oversize or unread into an empty page and hence Unknown.

### 03.3 One turn through the public client
Scope: contract `Read::Turn`, its archive value, routing in both directions, client document admission and its one terminal;
contract.md 6 and client.md 7; step `jig-03.3`.
Needs: the store and core preserve the turn's failure, from 03.2; nothing beyond the session's start.
Files: `crates/jig-client-domain/src/contract/{reads,documents}.rs`, `src/tests/reads.rs`;
`crates/jig-client-protocol-wire/schema/client-v9{,-part1}.schema`, `src/tests{,/fixtures/read_values}.rs`, generated v9 and goldens;
`crates/jig-protocol-engine-clients/src/translate/{reads,activity}.rs`, `src/tests/{reads,pages}.rs`;
`crates/jig-client-protocol-link/src/{component.rs,sign_in.rs,translate/reads.rs,translate/pages.rs}`, new `translate/turn.rs`,
`src/translate.rs`, `src/tests/reads.rs`, `src/tests/fixtures/{link_reads,turns}.rs`;
`crates/jig-client/src/facts.rs`, `src/tests/unlinked.rs`;
exhaustive consumers in `crates/jig-client-view/src/{application,objects,page/inbox,page/routing,page/shown}.rs`,
`testing/jig-client-scripted-engine/src/{reads,watches,requests,truth}.rs`, `testing/jig-client-test-kit/src/check.rs`,
`tests/client/src/joint/{read_packets,wire/reads,wire/historical}.rs`, `ReadValue`'s matches in `tests/client/src/{observe,scripted,
harness/referee,harness/referee/offers}.rs`, `tests/ops/src/client/transport.rs`;
`tests/client-protocol/src/client{,/thread}.rs`, `src/fixtures.rs`, new `tests/client-protocol/tests/turn_read.rs`;
`testing/jig-client-scripted-engine/src/payload.rs` (`impl Payload for Read`, `impl Payload for ReadValue`),
`crates/jig-client-view/src/tests/domain/objects.rs` (its `ReadValue` match), and `tests/client/src/joint/read_ledger_pages.rs`,
whose `match what` lists every `client::Read`: `client::Read::Turn { .. }` joins its `panic!("one of the two original
histories")` arm.
No forge, LLM or provider hotspot.
Shape: client `Read::Turn { task: u64, attempt: u64, turn: u32 }`, wire `ReadTurn` with those same fields;
client `ReadValue::Turn(Box<ArchivedTurn>)`, wire `ReadValue::Turn(ArchivedTurn)`. No subject gains a variant.
Regenerate v9 as in 03.1. The engine maps the read to core's existing read, and `Past::Turn` to the archive encoding, shared with activity;
keep `pages.rs` rejecting a turn as a page. The link checks the requested identity and the returned archive's identity,
retaining only the turn identity in its read attempt, not a second document. Extend its worst-case accounting accordingly.
Decode a single turn through `jig-protocol-transcript` with its archive bound, not the legacy activity-turn bound.
`wire.owned_record` already equals `document_bytes`, so inspect an oversized turn before `ReadAnswer::decode` rejects its
envelope. `translate/turn.rs` uses a borrowed `Reader` over the received bytes: validate the schema tags, owner/version,
record lengths, archive identity, payload length and exact end, without copying or decoding the oversized payload.
Cross-check this inspector against generated encodings in the boundary tests; no generator change or unchecked tag sniff.
A valid oversized archive yields `ReadNone::TooLarge { bytes }` with its payload length. In-bound archives use generated
decoding and the complete-document check; do not increase transport limits. A transport failure remains Unreachable.
Decoder failure gives Malformed; version refusal stays NotServed. All end as one `Told::Page` and send nothing again.
Use the existing read state machine and `jig-client`'s re-exports. Facts returns no offers for a turn. Reference pages never
request one; their exhaustive arms keep that invariant. The scripted engine reads its committed activity's same archive;
the typed joint adapter carries the same identity, metadata and failure facts as the byte path. In `client/thread.rs`,
factor `reading_chat` through one helper taking a body and checked client/core limits; subsequent stories reuse it.
Test shape: existing link `tests/reads.rs`, `fixtures::Harness::{read,read_answered,read_terminals}` and the v5 archive
fixture from jig 01: `a_turn_read_keeps_all_three_numbers`, `a_turn_read_keeps_its_archive_metadata`,
`a_turn_at_the_document_bound_decodes`, `a_turn_past_the_document_bound_keeps_its_size`,
`an_undecodable_single_turn_is_malformed`, `a_turn_answer_for_another_identity_is_rejected`. Units under 20 ms.
The inspector's boundary cases encode with the generated v9 types; also reject truncated/trailing records and false
declared lengths. Keep the HTTP body ceiling above the tested document sizes, naming it separately from `document_bytes`.
Extend domain's existing one-terminal/late-read tests with the new read; `jig-client/src/tests/unlinked.rs` verifies the
same terminal through `Client::unlinked`. In the new integration file, reuse `Story`, `fixtures::SEED`, `Story::read`,
`pages` and `run_until`; the reference (its constants name an absent task and the test's own handle):

```rust
use jig_client::{Asker, Handle, Readable, Readout, contract as c};
use jig_client_protocol_world::{client::Story, fixtures};
use skein_lib::Duration;
const TURN: Handle = Handle { asker: Asker::JIG, number: 31 };
const ABSENT: u64 = 9;
const WAIT: Duration = Duration::from_secs(1);
#[test]
fn an_absent_turn_is_unknown_through_the_client() {
    let mut story = Story::live(fixtures::SEED);
    story.read(TURN, Readable::Jig(c::Read::Turn { task: ABSENT, attempt: 1, turn: 1 }));
    assert!(story.run_until(|s| !s.pages(TURN).is_empty(), WAIT));
    assert_eq!(story.pages(TURN), vec![Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown))]);
}
```

Beside it, `a_committed_turn_is_read_whole_through_the_client`: `Story::reading_chat`, `running_task`, the existing message
and activity-wait setup, then read the identity the activity supplied. Compare its archive to the page's, including decoded
parts, spend, read position and time. Share that setup once in this file; no additional seed sweep. Under 0.5 s each.
Proof: joint, `cargo nextest run -p jig-client-protocol-world --test turn_read -E 'test(=a_committed_turn_is_read_whole_through_the_client)'`.
The client reads the real engine/store's committed turn through HTTP bytes; under 0.5 s.
Don't: add a page, a subject, a client retry, a second turn representation, or a dependency on jig 02; use `Readout::Limit`
for a turn's document-size refusal (that variant is still the client's full read table and has no size).
Size: at most 300 production lines; generated code and tests aside.
Removes: rejection of `Past::Turn` on the single-turn response path; the archive comment saying it is never fetched separately.

### 03.4 Page turns keep their byte counts
Scope: the client's `TurnProblem::TooLarge { bytes }`, from both the wire's page marker and its own decoder's admission;
contract.md 5.1 and 6; step `jig-03.4`.
Needs: one turn through the public client, from 03.3; nothing beyond the session's start.
Files: `crates/jig-client-domain/src/contract/turn.rs`;
`crates/jig-client-protocol-link/src/translate/{pages,pages_part0,pages_part1,turn}.rs`, `src/tests/{pages,fixtures/turns}.rs`;
`crates/jig-client-view/src/turns/turn.rs`, `src/tests/turn_read.rs`;
`tests/client/src/joint/wire/{activity,turns,pages}.rs` (`pages.rs`'s two `client::TurnProblem::TooLarge`),
`tests/client-protocol/tests/turn_read.rs`,
`tests/client-protocol/src/client/thread.rs`, `tests/client-protocol/src/fixtures.rs`.
Shape: change the variant in place. `archived_turn_fields` copies `w::TurnTooLarge::bytes()`; `turn_fields` captures the
input payload length before decoding and keeps it on the `jig_protocol_transcript::Undecodable::TooLarge` arm of its match on `jig_protocol_transcript::decode` (the link's
`translate::Undecodable` is another enum). Share that decoder with the single read,
with the applicable byte bound passed explicitly; a single read converts its problem to 03.2's `ReadNone`.
The reference UI keeps its existing too-large phrase, matching `{ .. }`; no new UI is designed here. Update every constructor
and exhaustive match, including the typed joint world's decoder. Page markers never trigger an automatic second read.
Test shape: extend `tests/pages.rs`'s `a_turn_past_its_admission_bound_is_carried_as_too_large` and
`fixtures::owner_turn_pages`: assert the source length, not the bound or zero. Keep separate boundary tests for a carried
payload at the limit and one byte past it. Use jig 01's v5 golden and existing `turn_limits`; text/part cuts remain decoded,
not TooLarge. In `turn_read.rs`, `an_activity_size_marker_keeps_its_bytes_through_the_client` uses the same committed-turn
fixture with a named page-budget deviation that fits the marker and owner but excludes the complete turn. Units under 20 ms,
joint story under 0.5 s. Existing page goldens keep their words.
Proof: joint, `cargo nextest run -p jig-client-protocol-world --test turn_read -E 'test(=an_activity_size_marker_keeps_its_bytes_through_the_client)'`.
The real store's page marker reaches the application with its payload size, under 0.5 s.
Don't: replace the marker with a cut, discard bytes on the typed transport, or change the single-read/document limits to
make this test pass. Do not import a downstream application's rendering or tools.
Size: at most 70 production lines.
Removes: the unit-like TooLarge variant and the wire match that discards its payload.

### 03.5 The turn-read stories meet at the real store
Scope: the finished read and its page fallback, through the public client and real engine/store; step `jig-03.5`.
Needs: page turns keep their byte counts, from 03.4; nothing beyond the session's start.
Files: `tests/client-protocol/tests/turn_read.rs`, `tests/client-protocol/src/client/thread.rs`,
`tests/client-protocol/src/client.rs` (the child/helper entry only), `tests/client-protocol/src/fixtures.rs`;
`tests/core/tests/reads.rs`, `tests/core/src/stories/reads.rs`. No seam hotspot.
Shape: extend 03.3's `Story::reading_chat` fixture helper, keeping `reading_chat` as its existing convenience call;
no alternative world or fake store. Use the existing real store's files,
durability fences and restart. Each altered limit starts at the existing fixture/profile builder, names the difference
and why, and passes its startup/worst-case checks. Choose a turn whose complete single row fits and whose activity row
does not fit the page; measure both row costs, with the payload admitted by the transcript, wire and client bounds.
This proves the supported overlap, not a promise that a turn past `load_bytes` becomes readable by raising transcript limits.
Test shape: extend the fixture of 03.3, no new test file. Add `a_page_sized_turn_is_read_whole_once`,
`a_turn_read_after_restart_keeps_its_archive`, `a_single_turn_past_the_engine_bound_keeps_its_size`,
`a_single_turn_past_the_client_bound_keeps_its_size`, `an_invisible_turn_is_unknown_after_its_task_ends`.
Each observes one `Told::Page`, with no second exchange after its terminal, under 0.5 s. The first compares decoded text and
calls with the source fixture and the page's size with its payload length. Reuse the read-world's role and restart setup
for access after ending. Gone and Busy remain small core/link terminal tests: use the existing `Loaded::Gone` and occupied
load-slot paths, without inventing a retained horizon. Malformed frame and malformed smith bytes are already 03.2/03.3's tests.
Proof: joint, `cargo nextest run -p jig-client-protocol-world --test turn_read -E 'test(=a_page_sized_turn_is_read_whole_once)'`.
The application's explicit follow-up read retrieves the same committed turn its page could only size; under 0.5 s.
Don't: build retention, introduce a second bound to force a fixture, alter the fake LLM/providers, or duplicate fixture
captures and polls from jig 02. Reconcile any shared helper at rebase; this step waits for no named-send capability.
Size: at most 80 lines in the existing world helpers; story tests aside.
Removes: duplicated setup introduced by this session, if any; no production behaviour.

## Interface sketches

Only additions/changed variants are shown; the existing variants stay. The store owns failure evidence, the core owns
visibility and durability, and the link owns smith decoding. No failure is a new durable store record.

```rust
// crates/jig-domain-core/src/reads/vocabulary.rs (03.2)
/// A selected transcript frame could not be supplied; the core decides whether its party may hear why.
pub enum TurnFailure {
    /// Past the read's budget; the store gives the original smith payload length.
    TooLarge { bytes: u64 },
    /// A frame could not be read whole; no invented length.
    Malformed,
}
/// The store's existing load terminal, with this additional variant.
pub enum Loaded {
    /// A single turn's evidence, checked against its pending scope and durable owner before a terminal leaves.
    TurnProblem { task: u64, attempt: u64, turn: u32, owner: Box<jig_domain_tasks::TaskRecord>, why: TurnFailure },
}
/// The core's existing no-value answer; both additions cross as the identically shaped client ReadNone variants.
pub enum Why {
    /// The visible turn's payload length, terminal after durability.
    TooLarge { bytes: u64 },
    /// The visible turn's frame was unreadable, terminal after durability.
    Malformed,
}
```

```rust
// crates/jig-client-domain/src/contract/reads.rs (03.2, 03.3); re-exported by jig-client
/// The application's existing read vocabulary, with one additional read and no corresponding Subject.
pub enum Read {
    /// One immutable committed turn; exactly one Page terminal, no key and no cursor.
    Turn { task: u64, attempt: u64, turn: u32 },
}
/// The existing value vocabulary; ArchivedTurn mirrors the wire archive field for field except for decoded turn bytes.
pub enum ReadValue {
    /// A successfully decoded archive, from the engine through the link.
    Turn(Box<ArchivedTurn>),
}
/// The existing no-value vocabulary, with two terminal additions.
pub enum ReadNone {
    /// Past the engine read budget or the client's document budget; original payload bytes.
    TooLarge { bytes: u64 },
    /// Frame unreadable or smith bytes not decodable, including an unsupported smith version.
    Malformed,
}
// crates/jig-client-domain/src/contract/documents.rs: existing type, reused (03.3)
/// Metadata from the committed frame, with its payload decoded by the link.
pub struct ArchivedTurn {
    pub task: u64,
    pub name: TurnName, // existing { attempt: u64, number: u32 }
    pub cumulative: u64,
    pub read: Option<u64>,
    pub at: skein_lib::Wall,
    pub turn: TurnRead,
}
// crates/jig-client-domain/src/contract/turn.rs (03.4)
/// Why a page's carried turn could not decode; the other variants are unchanged.
pub enum TurnProblem {
    /// The wire's size or the decoder's input length; never the bound that refused it.
    TooLarge { bytes: u64 },
}
```

```rust
// crates/jig-client-protocol-link/src/translate/turn.rs (03.3)
/// Checks a complete received turn envelope without owning its payload; Some is its oversize payload length.
pub(crate) fn oversized(
    body: &[u8], task: u64, attempt: u64, turn: u32, limits: &crate::Limits,
) -> Result<Option<u64>, super::Undecodable>;
```

## Questions to raise when reached

- A changed carrier or fixture fails the shipped limits/worst-case check: report the failing inequality and measured
  ownership before changing a budget. If 03.5's measured row costs have no overlap, report both costs and limits; do not
  invent a public limit or weaken the assertion. The downstream fixture's known `load_bytes` conflict is already a report
  to the planner, not an unanswered design choice hidden in this increment.
- A new joint story exceeds 0.5 s: profile its cause and make it cheaper; do not take jig 04's profiling work as a dependency
  or change optimization. No question here concerns transcript retention, which remains later work.

## Done when

Run in jig, reading `origin/main` after fetching. The commands test properties of the final files, not step history.

- `git show origin/main:crates/jig-client-protocol-wire/src/lib.rs | grep -qx 'pub const VERSION: u32 = 9;'`
- `git show origin/main:tests/client-wire/tests/drift.rs | grep -q -F 'schema/client-v9.schema'`
- `test -z "$(git ls-tree -r --name-only origin/main -- crates/jig-client-protocol-wire | grep -E 'client-v8|generated/v8|golden/v8/')"`
- `test -z "$(git grep -l -E 'jig_client_protocol_wire::v8|v8 as w' origin/main -- crates testing tests examples)"`
- `git show origin/main:crates/jig-client-domain/src/contract/reads.rs | grep -q -F 'Turn { task: u64, attempt: u64, turn: u32 }'`
- `git show origin/main:crates/jig-client-domain/src/contract/reads.rs | grep -q -F 'Turn(Box<'`
- `git show origin/main:crates/jig-client-domain/src/contract/reads.rs | grep -q -F 'TooLarge { bytes: u64 }'`
- `git show origin/main:crates/jig-client-domain/src/contract/watches.rs | sed -n '/^pub enum Subject/,/^}/p' | grep -q -F 'Run {' && ! git show origin/main:crates/jig-client-domain/src/contract/watches.rs | sed -n '/^pub enum Subject/,/^}/p' | grep -q -E '^    Turn[ ({,]'`
- `git show origin/main:crates/jig-client-domain/src/contract/turn.rs | grep -q -F 'TooLarge { bytes: u64 }'`
- `git grep -q -F 'TurnProblem' origin/main -- crates/jig-domain-core/src/reads crates/jig-protocol-store/src`
- `git show origin/main:crates/jig-protocol-engine-clients/src/translate/reads.rs | grep -q -F 'w::Read::Turn'`
- `git grep -q -F 'ReadNone::TooLarge' origin/main -- crates/jig-client-protocol-link/src`
- `git grep -q -F 'ReadNone::Malformed' origin/main -- crates/jig-client-protocol-link/src`
- `test -n "$(git grep -l --all-match -e 'Read::Turn' -e 'ReadValue::Turn' origin/main -- tests/client-protocol/tests)"`
- `git show origin/main:tests/client-protocol/tests/turn_read.rs | grep -q -F 'fn a_turn_read_after_restart_keeps_its_archive'`
