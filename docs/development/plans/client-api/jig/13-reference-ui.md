# Session 13: the test application on the public client

The reference UI's state is the page half of `jig-domain-client`, which
does its own watching, reading, link judging and sign-in, beside a
request table its root holds; the test application's client root routes
a fixed set of children's vocabularies and hands values between
siblings. This session puts the test application on `jig_client::Client`
(jig 12): the client's small store keeps only the client's records
(13.1); `jig-domain-client` moves whole into `jig-client-view` and is
gone, which empties the naming check's `PENDING` (13.2); the test
application's family travels as bytes through its wire crate (13.3); and
the test application's root holds the client and routes by asker (13.4),
the reference UI reaching it through a bridge. Session 21 then puts
`ops` on the client and makes the reference UI's state read the client
alone, ending the bridge. The page half's 9,900 lines move (13.2,
counted 0 by `git diff -M` but for their paths); what changes is where
they read from.

## Read first

- jig's `docs/design/client/domain/root.md`, sections 1 to 9, 12, 13 and
  15.
- jig's `docs/design/client/domain/view.md`, sections 2, 18.1, 18.2 and
  18.8.
- jig's `docs/design/client/domain/client.md`, sections 4, 6, 8, 11 and
  12.
- jig's `docs/design/client/domain/testing.md`, sections 2, 4, 5 and 6.
- jig's `docs/design/protocol/client.md`, sections 10 and 11.
- jig's `docs/design/README.md`, section 15.
- The plan's `findings/wire-and-client.md`, sections 2, 4 and 5 (a, b,
  k), `findings/renames.md`, section 4, and `code-organization.md`,
  sections 1, 2, 4, 6 and 7.
- Sessions 11 and 12's files: their sketches are this session's types.
- skein's `programming-model.md`, sections 4.5, 5.2 and 6.3;
  `testing-strategy.md`, sections 2.2, 6 and 7.

## What the design asks

- The root: "every ask of a child into the client; every request the
  client gives the transport out, in its order; everything the client
  tells the application routed by its asker" (root.md, 1 and 3); "No
  root routes a child of jig's, or a request table" (1); "The request
  table is the client's" (4); "The root matches the asker to its child,
  exhaustively, and keeps no map from handles or keys to children" (5).
- "The client's records are the client's ... Each child's own records,
  a presentation's (the reference UI's drafts and its place) and a
  family's, are the child's, keyed by its own keys and wrapped by the
  root" (root.md, 8); "The client's `Save` carries only the client's
  records" (protocol/client.md, 10).
- "An application's requests, reads and subjects ride the client as
  bytes under their owner's number and version ...; its wire crate
  encodes and decodes them" (root.md, 1 and 7).
- The reference UI's records go "through the application" with "keys of
  the reference UI's own" (view.md, 18.8).
- `jig-client-view` "depends on `skein-ui` and `jig-client`, and on
  nothing else of jig's" (view.md, 2).
- The naming rule's check holds with no crate pending
  (code-organization.md, 4).

## Choices this plan makes

- **The page half moves with its client half, then sheds it.** 13.2
  moves `jig-domain-client` whole (both halves) into `jig-client-view`
  under view.md, 2's names, so the move is a move. The moved client half
  (`watches.rs`, `reads.rs`, `link.rs`, `versions.rs`, the spine's watch
  functions and the frame's link judgment) stays, transitional, until
  jig 21.3 removes it; 13.4 already stops its reopening and silence
  timers.
- **Module names after the move** (view.md, 2): the spine's `domain.rs`
  is `ui.rs` (replacing the view's orphan `src/ui.rs`, a copy of
  skein-ui's root no module includes); `pages` is `page`; the moved
  `notices.rs` is `notices/kept.rs`, `frame.rs` is `frame/me.rs` and
  `limits.rs` is `limits/ui.rs`, under the view's modules of those
  names; each other module keeps its name at the crate root.
- **The view names the API through `jig-client`.** `jig-client`
  re-exports `jig-client-domain`'s types, which are the client's surface
  (client.md, 3; jig 12.1), so `jig-client-view` depends on `jig-client`
  alone, as view.md, 2 says; 13.2 rewrites its `jig_client_domain::`
  paths to `jig_client::`.
- **A bridge, from 13.4 to jig 21.3.** 13.4 moves the test root onto the
  client while the reference UI still speaks its present vocabulary:
  `jig_client_view::bridge` maps that vocabulary onto the client's
  events and tells, so that `ops`'s root (jig 21.2) uses the same bridge.
  Jig 21.3 removes it (code-organization.md, 6 lists it).
- **The test family's codec moves into its wire crate**
  (`jig-client-test-protocol-wire`), from the engine's side
  (`testing/jig-test-service/src/family.rs`) and the world's
  (`tests/client/src/joint/wire/family.rs`), so that the root encodes
  what its family asks (root.md, 7).
- **The client's world runs on `Client::unlinked`** (jig 12.5): the
  scripted engine keeps the contract's types; the world decodes the
  families' bytes with their wire crates.
- **Not here:** `ops` and the reference UI's state on the client alone
  (jig 21); the reference UI's lists on task queries and the shape pages
  removed (jig 16); the helpers (jig 15); the collapsed records (jig 14).

## Rules that bind it

- **The naming check waits for nothing from 13.2 on:**
  `git -C ~/src/rust/jig grep -n 'PENDING' origin/main --
  tests/boundary/src` prints a line with `[]`, and `cargo nextest
  run -p jig-boundary -E 'test(client_crates_carry_the_client_prefix)'`
  passes one test (README, Done when 4).
- **A root keeps no map and no `_` arm** (root.md, 10): the root's match
  on an asker is exhaustive over its owners, checked by clippy's
  `wildcard_enum_match_arm` and `match_wildcard_for_single_variants`,
  denied in the workspace; and `git grep -n -E '\bMap\b|BTreeMap|HashMap'
  -- testing/jig-client-test-domain/src/route.rs
  testing/jig-client-test-domain/src/domain.rs` prints nothing at 13.4's
  end.
- **jig never names the repository downstream of jig**: `git -C <the
  session's jig worktree> grep -n -i -w temper -- crates testing
  tests examples scripts` prints nothing at each increment's end
  (code-organization.md, 6).

## Increments

### 13.1 The client's small store keeps only the client's records
Scope: the reference UI's records (its place and its drafts) leave the
link: `jig-client-view` encodes them with its own codec, and the
application keeps them under the reference UI's keys beside the
client's (view.md, 18.8; root.md, 8); a family's records are its own,
encoded by its owner, never by the link (protocol/client.md, 10). The
client wire's `JigRecord`, `Place`, `Draft`, `Address*`,
`ProjectSection`, `TaskSection` and `Keeper` go (edits of v8 in place:
jig 11.3 owns v8).
Needs: jig 12 on `main` (the session's start).
Files: new `crates/jig-client-view/schema/records-v1.schema` (`family
jig-client-view-records 1`: `Place`, `Draft`, the address and its
sections, as the client wire has them today, `client-v8-part1.schema`'s
lines for them), `src/generated/records_v1.rs`, `golden/records-v1/`,
`src/records.rs`, and its `Cargo.toml` (`skein-codec`);
`tests/client-wire/tests/drift.rs` (a second test,
`view_records_schema_matches_generated_code_and_goldens`, on the first's
pattern at drift.rs:6); `crates/jig-client-protocol-wire/schema/client-v8-part1.schema`,
`src/generated/v8.rs`, `golden/v8` (regenerated);
`crates/jig-client-protocol-link/src/{boundary.rs, store.rs,
translate/store.rs}` (`Saved::{Jig, Family}`, `Erased::{Jig, Family}`,
`Keeper`, `StoreVersion` go; `Event::Start.versions:
List<StoreVersion>` (`boundary.rs:43`) goes with them, and
`Request::Dropped` stays, losing its `keeper` (`boundary.rs:99-100`;
ruling 191): `crates/jig-client/src/client.rs:400` tells `Told::Dropped
{ count: 1 }` from it, and `a_kept_record_of_another_version_is_dropped_and_told`
(`crates/jig-client/src/tests/store.rs:162-175`) keeps passing; a
storage key is the table's 16-byte key), and its tests `src/tests/fixtures/link_store.rs` and
`src/tests/store.rs`;
users (log `search-jig13-records.txt`): `tests/client-protocol/src/
{front.rs, front/persistence.rs, native.rs, family.rs}`, the native
shell's `connections.rs` and `shell.rs`,
`tests/client-wire/tests/fuzzy_client_wire.rs:25, :27-35` (ten `check!`
on the removed types: the fuzzy test fails to compile otherwise) and
`crates/jig-client-protocol-wire/src/tests.rs:229, :244-255`;
`crates/jig-client/src/client.rs:118-120, :395-412` (`StoreVersion`,
`Keeper::Table`, `Saved::{Table, Jig, Family}`, `Erased::Table`) and its
`src/tests/{store.rs, fixtures.rs:211}`; the link's `src/component.rs:66`
(`Event::Start { versions, kept }`); `tests/client-protocol/src/client.rs:35-36`
(`w::Keeper::Table`, `kept.keeper()`) (ruling 191);
downstream, its terminal's `route.rs` (21 lines naming `Saved`,
`Erased`, `Keeper` or `StoreVersion`) and `src/tests/family.rs:153-163` (3
more), which its own sessions follow.
`tests/boundary/src/tests/graph.rs`:
`client_view_links_only_its_domain_and_foundation` gains `skein-codec`.
Shape: `jig_client_view::records::{Record, RecordKey, RECORDS_VERSION,
encode, decode}` as the sketch, `Record` mirroring the page's
`store::Record` (`Place { version, party, address }`, `Draft { version,
party, task, words, touched }`) field by field; the link's `Saved` is
the table's record alone, `Saved { version: u16, record:
RequestRecord<Held> }`, and the client wire's `Kept` is `versioned record
Kept { store_version: u16 record: bytes 65536 }`. The application gives
the link only the client's records at `Start`, and the reference UI its
own (root.md, 8: "each child's to the child"; ruling 191). Each `pub fn`
of `records.rs` that returns a value takes `#[must_use]` (clippy's
`must_use_candidate`).
Test shape: unit tests in `crates/jig-client-view/src/tests/records.rs`;
new fixtures `src/tests/fixtures.rs`: `place()` and `draft()` records
with the literals of the link's `src/tests/fixtures/link_store.rs:122-130`,
`:232-275` today. Tests:
`a_place_record_round_trips_through_its_bytes`,
`a_draft_record_round_trips_through_its_bytes`,
`a_record_of_another_version_is_dropped`,
`a_malformed_record_is_dropped`, and the drift test above. Each under
20 ms.
Proof: joint layer: `cargo nextest run -p jig-client-protocol-world
-E 'test(a_reload_restores_a_pending_request_from_the_small_store_s_bytes)'`,
once, passing with the table's records alone in the link's store (under
2 s); it passes; nothing else is wanted. In the client's world, the gate
runs `a_draft_is_kept_across_a_reload` (tests/client), the draft kept
by the world under the reference UI's key; nothing else.
Don't: no family's record encoded by the link; no translation of the
old records (pre-release: dropped, protocol/client.md, 10).
Size: at most 150 lines (the schema, the generated code and the goldens
are not counted).
Removes: the link's jig and family keepers and their codecs; six
records (`AddressInbox`, `AddressConversations`, `AddressTask`,
`AddressProject`, `Place`, `Draft`) and five enums (`Keeper`, `Address`,
`ProjectSection`, `TaskSection`, `JigRecord`) of the client wire.

### 13.2 `jig-domain-client` moved into `jig-client-view`; the naming check waits for nothing
Scope: a move with paths rewritten: each module of `jig-domain-client`
into `crates/jig-client-view/src/` under the names above; the crate
removed; `PENDING` emptied.
Needs: 13.1 (the link names no `jig_domain_client` item:
`git grep -n jig_domain_client -- crates/jig-client-protocol-link`
prints nothing first).
Files: `crates/jig-domain-client/src/**` moved (18,803 production lines
and 13,242 test lines, findings 2); `crates/jig-client-view/{Cargo.toml,
src/lib.rs}`; the orphan `crates/jig-client-view/src/ui.rs` removed;
workspace `Cargo.toml` and `Cargo.lock` (shared);
`tests/boundary/src/tests/graph.rs` (`PENDING` to `[]`; `client_step`
loses `jig-domain-client`; `client_view_links_only_its_domain_and_foundation`
becomes `["jig-client", "skein-codec", "skein-lib", "skein-ui"]`; the
test root's, the test view's, `ops`'s and the scripted engine's
dependency lists; `:124-136`, `:180` and `:307` name the crate:
`client_roles_cover_step_crates_and_exclude_the_world` (:180) names the
merged crate; `client_domain_depends_on_foundation_alone` (:124-136)
asserts on the merged crate rather than going vacuous) and
`tests/boundary/src/tests.rs:142`. The path rewrites:
`jig_client_domain::` to `jig_client::` inside `jig-client-view` and the
moved files (277 lines in the view, 234 outside tests, and 325 in
`jig-domain-client`, 261 outside tests, at `23e3fbbdb`), so that the
view names the API through `jig-client`'s re-exports (jig 12.1) and
depends on `jig-client` alone (view.md, 2);
and `jig_domain_client::` to `jig_client_view::`, and inside
`jig-client-view` to `crate::`:
1,055 lines in 207 `.rs` files in jig at the base, 449 of them in
counted paths (the view 523, `ops` 146, the test root 106, the test
view 37, `tests/client` 186, `tests/ops` 16, `tests/client-protocol`
13, the scripted engine 1, the clients' component's tests 2), and 14
manifests with `Cargo.lock`; inside the moved files, 235 counted
`crate::` lines name a renamed module. Downstream, 191 lines in 38
files, which its own sessions follow.
Re-check (ruling 191, jig `73734dd6f`): the world's dependency list is
at `graph.rs:309`; `client_engine_breaches` (`graph.rs:471`) names the
crate too; `jig_domain_client::` is 1,050 lines in 204 files, 188 in
`tests/client`, 152 in 27 files of temper; the comment-only copies
`tests/client/README.md:52`, `client-v8.schema:2` and
`client-v8-part1.schema:63` are renamed too.
Shape: no type changes. `jig_client_view`'s root gains what
`jig_domain_client`'s exported, by these lines: `pub use
boundary::{Event, Request};`, `pub use ui::{Domain, fire, max_out,
next_deadline, reclaim, step};` and `pub use limits::ui::{Limits,
worst_case};`. None of these names is exported at the view's root at
the base (its root exports `JigStyle`, `JigBindings`, `JigMeaning`,
`decode`, `jig_bindings_worst_case`, `after_page`, `jig`, `Owner`,
`conversation`, `owner_project` and the `*_nodes` functions, `pub use`
lines at lib.rs:31, 41, 44-53 and 97 at the base); the view's
own `limits` module keeps its functions (`of`, `of_words`, `compose`,
`envelope`, `jig_worst_case`), the moved one beside them as
`limits::ui`. Where both crates have a test module of one name, the
moved crate's modules go under `src/tests/domain/`, keeping their
names, and its `src/tests.rs` becomes `src/tests/domain.rs`; list each
moved module in the commit body. The 15 shared names: `chip_facts`,
`frame_facts`, `historical_objects`, `historical_pages`,
`owner_deployment`, `owner_inbox`, `owner_person_task`, `owner_project`,
`owner_proposal`, `owner_tree`, `pages`, `question_sender`,
`task_context`, `task_ending`, `task_resources`. The Needs check
("prints nothing first") needs 12.1's landing as well as 13.1's: the
link names `jig_domain_client` in 25 lines in 6 files at `23e3fbbdb`.
Test shape: the moved tests move with their modules, names kept;
`client_crates_carry_the_client_prefix` (jig 01.4) runs with `PENDING`
empty.
Proof: joint layer: `cargo nextest run -p jig-client-protocol-world
-E 'test(a_person_signs_in_by_the_printed_link_and_starts_a_chat)'`,
once (under 2 s); it passes; nothing else is wanted. The gate's suites
run the rest.
Don't: no change of behaviour; no alias of the old crate or its paths.
Size: at most 60 hand-written lines (the manifests and the root's
exports). The path rewrites add about 1,100 counted lines, a rename's
count, given apart in the landing's report (code-organization.md, 7).
Removes: `crates/jig-domain-client`; the naming check's `PENDING` entry.

### 13.3 The test family as bytes, by its wire crate
Scope: the test application's family codec moves into
`jig-client-test-protocol-wire` (root.md, 7: "its records, in the
application's wire crate ..., which encodes what the owner asks and
decodes what the client hands back"): from
`testing/jig-test-service/src/family.rs` (235 lines, on
`jig-test-connector`'s vocabulary) and from the world's
`tests/client/src/joint/wire/family.rs` (264 lines, the test root's
values to that vocabulary), which moves into the test root as
`testing/jig-client-test-domain/src/family/wire.rs`. The engine's side
uses the same codec.
Needs: 13.2.
Files: `testing/jig-client-test-protocol-wire/{Cargo.toml (+
jig-test-connector), src/lib.rs, src/codec.rs (moved), src/codec/**}`;
`testing/jig-test-service/src/{family.rs, lib.rs, clients.rs:220,
clients/translation.rs:93-99, tests/clients.rs, tests/family.rs,
tests/fixtures/family.rs}` (uses it);
`testing/jig-client-test-domain/src/{family.rs, family/wire.rs (moved)}`;
`tests/client/src/joint/wire/{family.rs (moved), mod.rs}`,
`tests/client-protocol/src/{family.rs, engine.rs:89, :157, :164, :168,
front/persistence.rs:63}` (uses them);
`tests/boundary/src/tests/graph.rs`
(`test_client_domain_links_only_its_child_owner_vocabulary_and_foundation`
gains `jig-client-test-protocol-wire`).
Re-check (ruling 191, jig `73734dd6f`): `engine.rs:89, :157, :164, :168`
are now `:93, :164, :171, :175`, and `translation.rs:93-99` is `:95,
:98, :101`; the codec is `family.rs` (235 lines) and
`family/{documents, facts, parts}.rs` (1,398), all under `src/codec/**`.
Shape: `jig_client_test_protocol_wire::codec::{request_record, request,
outcome, outcome_value, subject, subject_value, read, read_value,
document, document_value, read_answer, read_answer_value, limits,
Undecodable, DecodedRequest}`, as `jig_test_service::family` has them;
`limits`, `document_value`, `read_answer` and `read_answer_value` move
with `request` (it needs `limits`); `to_root` (`family.rs:127`) uses
`jig_domain_core` and the engine's clients, and stays in
`jig-test-service`. `jig_client_test_domain::family::wire::{request,
subject, answer, document, Invalid}`, as
`jig_client_world::joint::wire::family` has them.
Test shape: the moved tests move; `tests/client-protocol/tests/protocol.rs`'s
`a_family_place_record_round_trips_through_its_owner_s_small_store_bytes`
keeps its name.
Proof: joint layer: `cargo nextest run -p jig-client-protocol-world
-E 'test(a_person_s_part_request_is_answered_from_the_connector_s_outcome)'`,
once, passing with the family's request encoded by the moved codec
(under 2 s); it passes; nothing else is wanted.
Don't: no change to the family's records or their version.
Size: at most 80 lines (the moves count only their changed lines).
Removes: the world's and the service's copies of the codec.

### 13.4 The test application's root on `jig_client::Client`, routing by asker
Scope: `jig-client-test-domain` holds `jig_client::Client` in place of
its `RequestTable` (root.md, 4), and its children ask through it: the
reference UI through `jig_client_view::bridge` (transitional, removed by
jig 21.3), the test family's sides by asker 1 and 2 with their values as
bytes (13.3); the root passes the client's transport requests out in
their order and routes each tell by its asker, exhaustively, and
`Link` to each child (root.md, 3 and 5). The reference UI's own watch
spine stops reopening and timing silence (the client's, jig 11.2), and
its link follows `Told::Link`. The client's world runs this root on
`Client::unlinked`; the protocol world and the native host on
`Client::new`. The native host's shell, `jig-client-shell-native`,
carries the client's transport requests and events,
`jig_client::{Request, Event}`, in place of the link's
(protocol/client.md, 12: the shell is the application's "loop around
`jig-client`"; the designer's answer of 2026-10-17).
Needs: 13.3, and jig-13.s2 before the family sides' values cross the client (ruling 205).
Files: `testing/jig-client-test-domain/src/{domain.rs, boundary.rs,
route.rs, owner.rs, limits.rs, store.rs, first.rs, second.rs}` (its 26
`jig_client_domain` uses in 13 files, 13.3's `family/wire.rs:179`
among them, go to `jig_client::`, as 13.2's do);
`testing/jig-client-test-domain/Cargo.toml` (`jig-client` for
`jig-client-domain`), `Cargo.lock` and
`tests/boundary/src/tests/graph.rs:244`, together (ruling 207);
`crates/jig-client-view/src/{bridge.rs (new), ui.rs (the spine's
silence and reopen timers go), lib.rs}`;
`testing/jig-client-test-view/{src/tests.rs:72, :115,
src/tests/native_hold.rs}` (its `Request` routing and the tests' uses
of `Request::Send`, `JigOut::*`, `Event::Answer` and `Limits.table`);
`tests/client/src/{harness/world.rs, scripted.rs, joint/front.rs,
joint/transport.rs, observe.rs, test.rs:5, :71, :220,
joint/requests.rs:43-53, joint/session.rs:364, :407, :419, :451,
joint/packets.rs:358}`,
`tests/client-protocol/src/{front.rs, front/persistence.rs, native.rs,
native/world.rs, stories.rs:112-130}`, `tests/one-process` (its client),
`testing/jig-test`'s
`tests/support/mod.rs`; `tests/boundary/src/tests/graph.rs` (the test
root links `jig-client`, `jig-client-view`,
`jig-client-test-protocol-wire`, `jig-test-connector` and `skein-lib`);
`crates/jig-client-shell-native/{Cargo.toml (`jig-client` in place of
`jig-client-protocol-link`), src/shell.rs, src/connections.rs,
src/files.rs, src/tests.rs}` (90 lines name `link::` in the four
sources at jig `5274ad08e`); `crates/jig-client/src/client.rs` and its
`src/tests/` (a sign-out that does not reach the engine, below).
Re-check (ruling 191, jig `73734dd6f`): users the Files miss:
`testing/jig-client-test-view/src/tests.rs:14, :338, :526, :537, :578,
:639, :766, :779, :870`, `tests/native_effect.rs:47`,
`tests/world_limits.rs:16-17` (`domain.table`);
`tests/client-protocol/src/fixtures.rs:136` (`profile.table`),
`tests/protocol.rs:204` (`store::Record::Table`);
`tests/client/src/joint/people_stories.rs:385`,
`tests/client/src/joint/stories/referee_stories.rs:40` (`Event::Answer`),
`tests/client/tests/{faults.rs:85, :153, shell.rs:48}` (`Record::Table`),
`tests/client/tests/unlinked.rs:24` (`world.table`, jig 12.5's);
`tests/client-protocol/src/client.rs` (:4 the link crate, :148
`Request`);
`tests/ops/src/joint/front.rs:129-144`, which matches the root's
`Event` exhaustively and takes its new arm. The reference test's
`root.first()` and `root.jig()` exist
(`testing/jig-client-test-domain/src/domain.rs:30-36`); it imports
`crate::route` and `jig_client::Told`, calls `crate::max_out(..)`, and
names `Link::Live` (ruling 207). The sketches take `#[must_use]` on each `pub
fn` that returns a value, and `Owner::asker` converts by `to_le_bytes`
as `Owner::opaque` does, never `as u32` (`as_conversions` is denied).
The bridge's tells and values without a reference UI counterpart, and
the native host: the designer's answer of 2026-10-17, in the Shape.
Shape: the root's `Event` gains `Client { event: jig_client::Event }`
for the transport's and the store's events, and loses `Answer`; its
`Request` gains `Client { request: jig_client::Request }` for the
client's transport requests (out, in order) and loses `Send` and
`Jig(JigOut::{Read, Open, Continue, Close})`; the root's table owner,
`owner::Owner`, goes: its owners are the test application's connectors,
`owner::Connector` as it is (root.md, 15: an application whose owners
are all its connectors' sides "shares the enum of its connectors"),
beside jig's reference UI. `jig_client_view::Owner` (view.md, 3) gains
the client's asker by root.md, 15's total and partial functions,
`asker(self) -> Asker` (`Jig` 65536, `Application { number }` the same
number) and `of_asker(Asker) -> Option<Owner>`; the root routes a tell
by `Owner::of_asker`, then `Connector::of(number)`, exhaustively
(sketch). Within a step, the root gives the reference UI's `Save` and
`Erase` of its records out before the client's exchanges of that step
("so they are written before the step's exchanges leave", view.md,
18.8). `jig_client_view::bridge`: `asked(request: &Request) ->
Option<jig_client::Event>` (the reference UI's `Send`, `Read`, `Open`,
`Close`, `SignOut` as the client's events under `Asker::JIG`, a
handle's number its token's, a key's below) and `told(domain, env,
client: &Client, told: Told, out)` (a tell under `Asker::JIG` as the
reference UI's present events: `Changed` as `Watch {
Delivered(client.document(..)) }`, `Pending` as `Progress` from
`client.pending(..)` (`Progress::NotServed` as `Parked`: only a
family's request is held for its version, `jig-client-domain`'s
`requests.rs:21, :161`), `Answered` as `Answered` (`NotKept`, `Limit`,
`Unconfirmed` and `Unknown` as `Progress::Unknown` until jig 21.4),
`Kept` counted per party as `Kept { scope, requests }`; `Text`,
`Page`, `Ended`, `Link` and `Dropped` as below). The bridge's other
rules (the designer's answer of 2026-10-17; view.md, 18.1, 18.2, 18.4
and 18.6; client.md, 4, 8.3 and 11; line numbers at jig `5274ad08e`):
- *A key's number* is its intent's token xored with a salt the root
  draws once from its seed at `new`, and an answer's token is its
  number xored with it again: a token's slot and generation start again
  at a reload, and a key kept from before it comes back under its
  number, so a token alone would repeat it (view.md, 18.1: "draws its
  keys' numbers from its seed, fresh for each request it means"). `asked`
  and `told` take the salt; each start of a world's client gives the
  root a seed of its own (client.md, 11).
- *A run's text.* For a run's handle the client tells `Changed`, then
  `Text`, in one step (`crates/jig-client-domain/src/watches.rs:110,
  :112`): `Changed` delivers the run's document, with the turn's text
  the client holds, which the conversation's `stream` takes; `Text`
  maps to nothing. `asked` gives `None` for `Continue`, and the root
  nothing out: the client continues a run's watch from the text it
  holds when it opens it again (`text.rs:93-104`), and the engine sends
  the rest of a turn unasked (contract.md, 7.3).
- *The link.* `Starting` and `Offline` as the UI's link; `Live` as
  `Hello { served: true }`, which the UI waits for before it takes `me`
  (`jig-domain-client`'s `domain.rs:939`), then its link `Live` (the
  client tells `Link` before the `me` handle's `Changed`,
  `watches.rs:102`); `SignedOut` as `SignedOut` (the sign-in page);
  `OutOfDate` as `Hello { served: false }` (the out-of-date page,
  `jig-domain-client`'s `versions.rs:12-21`). At a `Link` whose `client.link().sign_out_lost`
  is set, the UI gets `SignOutLost` too.
- *A sign-out that does not reach the engine.* `jig-client` tells
  `Link` when it sets `sign_out_lost` (client.md, 4.2, amended; the
  arm at `crates/jig-client/src/client.rs:276-277`). `Client::unlinked`
  answers `SignOut` at once (`client.rs:232`); it leaves the sign-out to
  its world instead, which answers it signed out or not reached, as the
  link does from the wire, so that
  `a_lost_sign_out_keeps_the_person_signed_in_and_says_so`
  (`tests/client/tests/shell.rs:81`) runs on it. Ruling 197: under
  `unlinked`, `Event::SignOut` sets `signing_out` and pushes the unit
  `Request::SignOut` to the world, which answers `Event::Up { up:
  Up::SignedOut }` or the unit `Event::SignOutLost`, both
  `jig_client`'s, unlinked only, as `Request::Down` and `Event::Up` are;
  `Down` and `Up` do not change.
- *Ends.* `Ended { why }` as `Ended { end }`: `Unknown`, `NotAllowed`,
  `Busy`, `SubjectEnded` and `SignedOut` by their names; `Limit` as
  `Closed`, so the watch retires as one past the UI's own limit does
  (view.md, 18.2); `NotServed` as `Unknown`: only a family's watch ends
  so (`jig-client-domain`'s `domain.rs:140, :175`).
- *Pages.* `Page { page }` as `Read { result }`: `Jig(result)` as it
  is; `Limit` as `None(Busy)`; `NotServed`, which comes under jig's
  asker only while the link is out of date (`reads.rs:18`), and
  `Family { .. }`, which never does, as `None(Refused)`.
- *Dropped.* The root gives `Dropped { count }` to the reference UI
  (root.md, 3 and 8); the bridge maps it to nothing until jig 21.4
  shows its notice.
- *Kept work answered.* The bridge gives no `KeptGone`: no table owner
  names a kept request's party once it is answered (root.md, 5). Until
  jig 21.4, someone else's work leaves when its party signs in
  (`refresh_kept_page`, `jig-domain-client`'s `domain.rs:804-819`), not
  when its requests are answered unknown.

The family sides' `Out::{Read, Open, Close}` and
`Send` reach the client as `Read`, `Watch`, `Unwatch` and `Send` under
their asker, encoded by 13.3's codec as jig-13.s2 extends it (ruling 205), and its tells come back decoded.
The root's client limits declare the family owners its sides send, each
at `jig_client_test_protocol_wire::VERSION` (`jig_client_domain::Limits.owners`,
jig 12.4; client.md, 8.2); a family message of an owner not declared
`assert!`s in the client.
Test shape: `testing/jig-client-test-domain/src/tests/root.rs` (the
root's, kept and extended): `routes_a_tell_to_the_child_its_asker_names`,
`passes_the_client_s_transport_requests_out_in_order`,
`a_kept_request_of_an_asker_no_owner_names_reaches_no_child`,
`the_link_reaches_each_child`,
`the_reference_ui_s_records_leave_before_the_step_s_exchanges`;
`crates/jig-client-view/src/tests/owner.rs`:
`jig_s_owner_is_the_asker_one_past_every_u16`,
`an_asker_past_jig_s_is_no_owner`; `crates/jig-client-view/src/tests/bridge.rs`:
`a_page_s_watch_is_the_client_s_under_jig_s_asker`,
`a_changed_handle_is_the_page_s_delivery`,
`a_run_s_text_reaches_the_page_in_its_changed_document`,
`out_of_date_shows_the_out_of_date_page`,
`a_lost_sign_out_reaches_the_ui`,
`a_watch_the_client_s_limit_ends_retires`,
`a_key_is_never_a_kept_key_after_a_reload`; `crates/jig-client/src/tests/`:
`a_sign_out_that_did_not_reach_the_engine_tells_the_link`,
`an_unlinked_sign_out_its_world_loses_keeps_the_party_signed_in`; the
native shell's tests keep their names on the client's vocabulary.
Reference test, on the
root's fixtures (`src/tests/root.rs:20-80`'s `limits()` and `env()`,
which gain the client's limits from the world's profile):
```rust
#[test]
fn the_link_reaches_each_child() {
    let mut root = new(&limits(), 7);
    let mut out = Queue::with_capacity(max_out(&limits()));
    route::told(&mut root, &env(), Told::Link { link: jig_client::Link::Live }, &mut out);
    assert_eq!(root.first().link(), jig_client_view::link::Link::Live);
    assert_eq!(root.jig().link(), jig_client_view::link::Link::Live);
}
```
Each under 20 ms. The worlds' stories keep their names.
Proof: joint layer, the root's client against the real engine:
`cargo nextest run -p jig-client-protocol-world -E
'test(a_reload_restores_a_pending_request_from_the_small_store_s_bytes)
| test(a_person_s_words_reach_a_running_run_through_bytes)'`, once,
both passing with the root holding the client (under 2 s); it passes;
nothing else is wanted. In the client's world, on `Client::unlinked`, the
gate runs `signing_in_and_starting_a_chat` (tests/client); nothing
else.
Don't: no map from handles or keys to children in the root; no `_` arm
on an asker or an owner; no change to `ops` (jig 21.2).
The view's timers' limits and states (ruling 207; jig `view.md`,
1097-1100: "Backoffs and heartbeats are the client's (client.md,
section 8)"): in `crates/jig-client-view/src/`, `Limits.backoff` and
`Limits.heartbeat` go (`limits/ui.rs`), with their readers (`ui.rs:489,
:927, :981, :983, :1239`), the silence timer (`Timer::Silent`), `retry`
(`ui.rs:986`), `Following::Reopening` (`watches.rs`,
`page/routing.rs:64`, `versions.rs`) and what then has no reader; the
view's `Watch.behind` is set from the client's handle state alone: the
bridge gives a `Changed` whose `client.document(..)` is `State::Behind`
as a new `WatchEvent::Behind`, which sets `behind` and keeps the
document, `State::Live` as `Delivered(..)`, which clears it
(`ui.rs:884`), `Opening` as nothing; `WatchEnd::{Dropped, Busy,
Stopping, Closed}` each retire the watch, as `Closed` does today outside
recovery. `tests/client/src/joint/session.rs:118`,
`tests/client-protocol/src/fixtures.rs:143, :153` and
`tests/client/tests/unlinked.rs:35` take the heartbeat and backoff from
the client's `Limits` (`jig-client-domain`'s `limits.rs:19, :29`).
Refresh each further site the compiler names, and list it in the
commit body. Downstream, the temper step that first pins a jig holding
13.4 drops the two fields at `temper-client-domain/src/tests/fixtures.rs:40,
:44`, `temper-client-shell-terminal/src/limits.rs:55, :59` and
`temper-client-view/src/tests/fixtures.rs:37, :41`, and lists them in
its commit body.
Size (ruling 215): aim for at most 850 production lines, ruling 207's
items 2 to 4 included; past 850, report the count by file and why; at
950, stop and block. It was: at most 380 lines (the root about 120, the bridge about 170, the
family sides about 40, the native shell about 25, `jig-client`'s
sign-out about 25, ruling 197). The parts count together.
Removes: the test root's `RequestTable` and its table owner `Owner`
(with `Kept { scope }`), `KeptAnswered`; the reference UI's silence and
reopen timers, with the view's `Limits.backoff` and `heartbeat`, `retry`
and `Following::Reopening` (ruling 207); the native shell's dependency on
`jig-client-protocol-link`.

## Interface sketches

Checked at the base with `cargo clippy -p jig-w6-sketch
--all-targets -- -D warnings` in a scratch crate, the page's types named
at their base paths (`jig_domain_client::{address, contract}`), passing.
The reference UI owns its records' codec; the application keeps the
bytes under the reference UI's keys; the root owns only the client and
its children.

```rust
// crates/jig-client-view/src/records.rs (13.1)
/// One of its records (view.md, 18.8); mirrors records-v1.schema's `Record` field by field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    Place { version: u16, party: u64, address: Address },
    Draft { version: u16, party: u64, task: u64, words: contract::Text, touched: u64 },
}
/// A record's key under the reference UI's own keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordKey { Place, Draft { task: u64 } }
/// Its records' version: a record of another is dropped.
pub const RECORDS_VERSION: u16 = 1;
/// A record as the key's bytes and the record's bytes the application keeps.
pub fn encode(record: &Record) -> (Box<[u8]>, Box<[u8]>);
/// A kept record read back: none for another version or a malformed one.
pub fn decode(key: &[u8], bytes: &[u8]) -> Option<Record>;
```

```rust
// crates/jig-client-view/src/owner.rs (13.4): jig's typed owner onto the client's asker (root.md, 15).
impl Owner {
    /// The asker the client carries: 65536 for `Jig`, the same number for `Application { number }`.
    pub const fn asker(self) -> Asker;
    /// The owner of an asker, or none past 65536: a fact, never a guess.
    pub const fn of_asker(asker: Asker) -> Option<Owner>;
}

// testing/jig-client-test-domain/src/route.rs (13.4): a tell to the child its asker names,
// the test application's owners its connectors' enum, `owner::Connector`, as it is.
fn told_to(asker: Asker) -> Child {
    match Owner::of_asker(asker) {
        Some(Owner::Jig) => Child::Ui,
        Some(Owner::Application { number }) => match Connector::of(number) {
            Some(Connector::First) => Child::First,
            Some(Connector::Second) => Child::Second,
            None => Child::None,
        },
        None => Child::None,
    }
}
```
(`Child` stands for the arm each case takes; the root routes in place
and keeps no such value.)

## Questions to raise when reached

- **Kept work of an asker no owner of the build names** (root.md, 5: "its
  kept requests are sent and answered as any, and what the client tells
  of them reaches no child"): the plan drops those tells in the root;
  raise it if a notice is wanted.

## Done when

- `git -C ~/src/rust/jig ls-tree -d --name-only origin/main
  crates/jig-domain-client` prints nothing.
- `git -C ~/src/rust/jig grep -n 'PENDING' origin/main --
  tests/boundary/src` prints a line with `[]`.
- `git -C ~/src/rust/jig grep -n -E 'RequestTable' origin/main --
  testing/jig-client-test-domain/src` prints nothing.
- `git -C ~/src/rust/jig grep -n -E 'jig-client-domain' origin/main --
  crates/jig-client-view/Cargo.toml` prints nothing.
