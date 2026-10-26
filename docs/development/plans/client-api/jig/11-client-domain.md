# Session 11: `jig-client-domain`, the client's domain

The client's half of `jig-domain-client` is woven into its page half: its
watches belong to a page generation, the request table is the root's,
and kept work is counted per party. This session makes
`jig-client-domain` (`crates/jig-client-domain`): the API's types moved
in from `jig-domain-client` with one version constant (11.1); watches by
handle, shared on the engine, with their latest documents and the
client's own watch of `me` (11.2); the request table, held by the client,
its keys carrying their askers, and kept work by key (11.3, which owns
the client wire's v8); reads by handle (11.4); and a run's text by
handle (11.5). It is a step machine that nothing above calls until jig
12 composes it with the link into `jig_client::Client`; the reference UI
and the roots keep `jig-domain-client`'s machinery until jig 13 (the
test application) and jig 21 (`ops` and the reference UI's state) move
them onto the client.

## Read first

- jig's `docs/design/client/domain/client.md`, sections 1 to 4, 6, 7, 8
  (8.1 to 8.3), 11 and 14.
- jig's `docs/design/client/domain/root.md`, sections 4, 5 and 15.
- jig's `docs/design/protocol/client.md`, sections 4.3, 9 and 10.
- jig's `docs/design/client/domain/contract.md`, sections 3.1, 7.1 to 7.3
  and 8.
- The plan's `findings/wire-and-client.md`, sections 2 and 5 (a, b, j,
  k), and `code-organization.md`, sections 1, 2, 5, 6 and 7.
- skein's `docs/design/lib.md`, section 12 (the request table), as its
  code says it: `skein-lib`'s `src/requests.rs`, lines 1 to 40.
- skein's `programming-model.md`, sections 3, 4.2, 4.4, 5.1, 5.2, 6.3
  and 9; `testing-strategy.md`, section 2.1.

## What the design asks

- `jig-client-domain` "is a step machine like every jig domain:
  `#![no_std]`, depending on skein-lib alone", holding the API's types,
  the link's state, "handles: each watch's and each read's, with its
  asker", "the request table, skein-lib's `RequestTable` ..., with each
  request's key and asker", kept work, its timers and its seed
  (client.md, 3).
- "Several handles naming one subject and one window share one watch on
  the engine and one document. It opens with the first handle and closes
  with the last" (8.1); "each opening is named afresh"; "A `Watch` on an
  open handle changes its subject or its window: the client opens the
  new one, keeps the handle's last document, marked behind, until the
  new one's first delivery"; "Dropped is not ended": a dropped watch
  "is opened again after a backoff, under a fresh name; its handle
  stays, behind"; `Ended { handle, why }` is the handle's one terminal,
  and "`Unwatch` ends a handle with no `Ended`".
- "The client watches `me` itself while a sign-in is live, under a
  handle of its own, which an application's handle on `me` shares. `me`
  names the party, which the client confirms to the request table ...
  and the retention of answers" (8.2).
- Keyed requests (client.md, 6): the key is "its asker, and a number the
  asker draws from its seed"; "A second `Send` under a key the client
  holds is joined to the first when it carries the same request ...;
  with another request, it is a key conflict, a mistake of the
  application's, and loud"; kept before sent; answered once; bounded,
  "A `Send` past them is answered at once, refused by the client's
  limit, and never queued".
- Kept work (11): each record keeps "the key the engine knows it by, its
  party, the application's key, the request ..., and the wall time of
  its first sending"; restored at `Start { kept }` and "told to the
  application as `Kept { key, request }`, which the client root routes
  by the key's asker".
- "The request table is the client's: `jig-client` holds skein-lib's
  `RequestTable`" (root.md, 4).
- A run's text (8.3): "The client keeps the text it holds of the turn
  being written, within the bound on a provisional turn's bytes, takes
  each delivery's text past its offset, and says `Text { handle }`. It
  holds that one turn alone, the newest delivered: a delivery naming a
  later turn, or a later retry of the same turn, takes the place of what
  it holds, from its own offset; one naming an earlier turn, or an
  earlier retry, changes nothing, and nothing is told"; "Opened again,
  the watch starts from the beginning of the turn being written".
- One version constant: `jig_client_protocol_wire::VERSION` is the only
  definition of the client wire's version (code-organization.md, 2).

## Choices this plan makes

- **The contract keeps its module path,** `jig_client_domain::contract`,
  so the move is `jig_domain_client::contract` to
  `jig_client_domain::contract` in each user, and `crate::contract` to
  `jig_client_domain::contract` inside `jig-domain-client`.
- **The link judges the hello.** `jig_domain_client::Event::Hello {
  versions }` becomes `Hello { served: bool }`;
  `jig_client_protocol_link::serves(versions) -> bool` compares with
  `jig_client_protocol_wire::VERSION`, and each host calls it where it
  maps the link's `Hello { family: None, .. }`.
- **The link's payload types move into the domain, names kept:**
  `Family`, `Ask`, `Readable`, `Watchable`, `Readout` and `Answered`,
  from `jig_client_protocol_link::boundary` to `jig_client_domain`
  (11.2). The link uses them from there.
- **The domain has its own vocabulary toward its link,** `Down` and `Up`
  (sketch), in its own types; jig 12.1's `jig_client::Client` maps them
  onto the link's events and requests, as each layer keeps its own
  vocabulary (programming-model.md, 4.4).
- **The window and the cursor stay inside the subject and the read,** as
  the contract has them at the base (`Subject::Tree { window }`,
  `Read::Ended { after }`): client.md, 4's separate `window` and
  `cursor` fields wait for the application's windows on the wire, jig
  14's.
- **`me` is `Subject::Frame`** until jig 22.1 gives `me` its own shape (jig
  14.1 moves its window and cursor out of the subject); the
  client's own watch of it opens at `Start` until jig 12.3 ties it to a
  live sign-in.
- **The link's table records gain the asker and the number.** The wire's
  `TableEntry` gains `asker: u32` and `number: u64`; this is the first
  record this plan changes on the client wire, so **11.3 owns the client
  wire's v8** (README, Rules): the schema files, the family's version,
  `VERSION`, the goldens and the served ranges move from 7 to 8 there,
  once. Every later change to the client wire in this plan (jig 13.1, jig
  14) edits v8 in place.
- **A `Send` before `me` has named a party** is answered at once
  `Outcome::Unconfirmed`, never kept or sent (Questions, below).
- **The root's table stays the root's until jig 13.4,** which moves the
  test application's root onto `jig_client::Client`; until then,
  `jig-domain-client`'s client half and this domain run side by side,
  the old one serving the reference UI, the new one tested on its own
  and, from jig 12, against the real engine. Nothing of this session
  edits `jig-domain-client` but 11.1's paths and its `Hello`.

## Rules that bind it

- **`jig-client-domain` links `skein-lib` alone** (client.md, 3): 11.1
  adds the boundary test `client_domain_crate_links_only_the_foundation`
  (below), which the gate runs from then on.
- **One wire version, changed once** (README, Rules; decisions.md, 9):
  `rg -n 'CLIENT_VERSION|VERSION: u32 = ' crates` prints one line per
  wire from 11.1 on; the client wire moves to v8 in 11.3 alone.
- **No type beside an existing one for the same thing**
  (code-organization.md, 6): the payload types move, names kept; no
  alias kept for an old path (code-organization.md, 2). `jig-client`'s
  re-exports of this crate's types (jig 12.1) are the client's surface,
  not such aliases.

## Increments

### 11.1 The contract in `jig-client-domain`, with one version constant
Scope: the API's types move from `jig-domain-client` into the new crate
(client.md, 3); the copy of the wire's version goes, and the link judges
the hello (protocol/client.md, 2 and 9). A move: no type changes but
`Hello`.
Needs: jig 01's renames on `main`.
Files: `crates/jig-domain-client/src/contract.rs` and `src/contract/`
moved to `crates/jig-client-domain/src/contract.rs` and `src/contract/`
(5,198 lines, `wc -l` at the base); new
`crates/jig-client-domain/{Cargo.toml, src/lib.rs}`; workspace
`Cargo.toml` and `Cargo.lock` (shared); `contract/versions.rs` loses
`CLIENT_VERSION`; `crates/jig-domain-client/src/{versions.rs,
boundary.rs, domain.rs}` (`Hello`); `crates/jig-client-protocol-link/src/
{lib.rs, component.rs}` (`serves`). The path rewrite, `sed -i
's/jig_domain_client::contract/jig_client_domain::contract/g'` over the
files `git grep -l` names, and `s/crate::contract/
jig_client_domain::contract/g` inside `crates/jig-domain-client/src`:
at the base, 578 lines in 208 files outside `jig-domain-client` (368 of
them in counted paths: the link 36, the view 189, `ops` 30, the scripted
engine 90, the test root 16, the test view 6, the transcript 1) and 247
lines in 37 files inside it (220 counted); and the `contract::` lines
inside nested `use jig_domain_client::{...}` and `use crate::{...}`
blocks (120 outside it in counted paths, 237 inside it, test files
included), whose blocks are split by hand where a moved name sits. Users of `Event::Hello { versions }` (log
`search-jig11-jig.txt`): `crates/jig-domain-client/src/tests/{hello.rs,
drafts.rs, pages.rs, pages/conversation_merge.rs, spine.rs, time.rs}`,
`crates/jig-client-view/src/tests/{cards.rs:109, historical_objects.rs:353,
history_pages.rs:89, owner_escalation.rs:303, owner_project.rs:514, 694,
1071, pages.rs:68}`, `examples/ops/client-domain/src/tests.rs:120`,
`examples/ops/client-view/src/tests.rs:102`,
`testing/jig-client-test-domain/src/tests/root.rs`,
`testing/jig-client-test-view/src/tests{.rs, /native_hold.rs}`,
`testing/jig-client-scripted-engine/src/truth.rs:249`,
`tests/client/src/{observe.rs:203, scripted.rs,
harness/navigation_tests.rs, joint/engine.rs:170, joint/front.rs,
joint/session.rs, joint/transport.rs}`,
`tests/client-protocol/src/front.rs:126` (the one map of the link's
hello), `tests/ops/src/client/{stories.rs:324, transport.rs}`; in the
repository downstream of jig, its client domain's, client view's and
terminal's test fixtures (the terminal's `tests/fixtures.rs:226`), its
terminal's `route.rs:121`, and its client world's `front.rs:81-82`, which
names `jig_domain_client::contract::CLIENT_VERSION` (11.1 removes it);
the contract path itself in 21 of its files (40 lines, at temper `ed566100`), which its own
sessions follow. `tests/boundary/src/tests/graph.rs`
(shared): `client_domain_depends_on_foundation_alone` (graph.rs:122)
lets `jig-domain-client` link `jig-client-domain`; `client_step`
(graph.rs:138) gains it; `client_view_links_only_its_domain_and_foundation`
(graph.rs:103) and the scripted engine's, test root's and test view's
dependency lists gain it.
Shape: `jig_client_domain::contract` (the module as moved, its
`pub use` lines inside it unchanged); `jig_domain_client::Event::Hello {
served: bool }`; `pub fn jig_client_protocol_link::serves(versions:
VersionRange) -> bool`, `versions.contains(jig_client_protocol_wire::VERSION)`.
The scripted engine's hello is `Hello { served: true }`, and its
`Versions` fault `served: false`.
Test shape: no new test file. The link's `tests.rs:17`, which asserts
the two constants equal, becomes
`the_link_serves_the_wire_s_version_alone`: `serves(VersionRange {
first: 7, last: 7 })` is true and `serves(VersionRange { first: 6,
last: 6 })` false (7 until 11.3 makes both literals one higher);
`crates/jig-domain-client/src/tests/hello.rs`'s tests keep their names,
`served: true` or `false` in place of the range.
Proof: joint layer over HTTP bytes: `cargo nextest run -p
jig-client-protocol-world -E
'test(an_engine_outside_the_client_s_versions_shows_out_of_date)'`,
once; it passes when the page shows "Out of date" from the link's
judgment (under 2 s). It passes; nothing else is wanted.
Don't: no other change to any moved type; no `pub use` of the contract
from `jig-domain-client`; no edit of the moved files' contents but
`contract/versions.rs`'s `CLIENT_VERSION`, which goes.
Size: at most 80 hand-written lines (the manifests, `serves`, `Hello`),
checked as code-organization.md, 7's "Moves and renames" says: the
report gives the command's number and, beside it, the added lines that
name `jig_client_domain`, which the rewrite adds (700 to 800 at the
base: 588 by the `sed`, the rest the `use` lines split out of nested
`use` blocks, which are rewrite lines too); the Size is checked against
the difference.
Removes: `jig-domain-client`'s `contract` module and `CLIENT_VERSION`.

The boundary test 11.1 adds, in `tests/boundary/src/tests/graph.rs`:
```rust
#[test]
fn client_domain_crate_links_only_the_foundation() {
    let domains: Vec<_> = manifests().into_iter().filter(|m| m.name.as_deref() == Some("jig-client-domain")).collect();
    assert_eq!(domains.len(), 1, "one client domain");
    assert_eq!(domains[0].dependencies, ["skein-lib"]);
}
```

### 11.2 Watches by handle, shared on the engine, and the client's `me`
Scope: `Watch { handle, subject }` and `Unwatch { handle }`; one engine
watch per distinct subject, shared by its handles, opened with the first
and closed with the last, each opening under a fresh token; its latest
document; each handle's state (opening, live, behind, ended);
`Changed { handle }` on each delivery; a dropped or stopping watch
opened again after a backoff drawn from the seed, its handles behind;
`Ended { handle, why }` for the engine's refusals and a subject ended;
`document(handle)`; the client's own watch of `Subject::Frame` (`me`),
opened at `Start`, which an application's handle on `Frame` shares
(client.md, 8.1 and 8.2). The link's payload types move in.
Needs: 11.1's crate.
Files: `crates/jig-client-domain/src/{lib.rs, boundary.rs, domain.rs,
watches.rs, limits.rs}`, `src/tests.rs`, `src/tests/{fixtures.rs,
watches.rs}`; `crates/jig-client-protocol-link/src/{boundary.rs,
component.rs, translate/watches.rs, translate/reads.rs, store.rs}` (the
payload types now `jig_client_domain`'s; shared with jig 10). Users of
`link::{Family, Ask, Readable, Watchable, Readout, Answered}` (log
`search-jig11-jig.txt`): `tests/client-protocol/src/{front.rs,
front/persistence.rs, native.rs, family.rs}`; downstream, its terminal's
`route.rs`, `route/family.rs`, `tests.rs:20, 122, 129`,
`tests/family.rs:55, 57, 121, 125, 150, 154, 182, 183` and
`tests/fixtures.rs:76`, and its one-process world's
`referee/client.rs:148`.
Shape: as the sketch: `Asker`, `Handle`, `State`, `Latest`,
`Document<'_>`, `Why`; `Event::{Start, Watch, Unwatch, Up}`,
`Up::{Watched, FamilyDelivered, WatchEnded}`, `Down::{Open, Close}`,
`Told::{Changed, Ended}`; `Domain::document(handle)`;
`step`, `fire`, `next_deadline`, `reclaim`, `max_out`, `worst_case`;
`Limits { handles, watches, document_bytes, backoff }` (the rest of the
sketch's fields come with 11.3 to 11.5). A `Watch` past `handles` or
`watches` is `Ended { handle, why: Why::Limit }` at once. A
`WatchEnded` of `Dropped` or `Stopping` opens again after the backoff
(first, doubling to most, jittered over half to all of the span from
the seed); `Closed` retires the watch `Unwatch` closed; the others end
each handle. A handle that goes behind is told `Changed` too, so a
reader of its state sees it (Questions). The engine watch's slot is
retired only once its terminal came (client.md, 3).
Test shape: `crates/jig-client-domain/src/tests/watches.rs` on
`src/tests/fixtures.rs` (made here): `fixtures::limits()` copies, as
literals, the client world's profile, `jig_client_world::test::limits()`
(tests/client/src/test.rs:18): `watches: 8`, `document_bytes: 65536`,
`backoff: 10 ms to 80 ms`; `handles: 8` is new, beside it. jig ships no
client profile of its own (an application's client's limits are its
own), so the world's is the profile. No dev-dependency on
`jig-client-world`: the values are copied, each with a comment naming
test.rs's line. `fixtures::Client` holds a
`Domain`, an `Env<Limits>` and a `Queue<Request>`, with `step(event)`,
`at(time)`, `downs() -> Vec<Down>` and `told() -> Vec<Told>` (each
drains), and `Client::started()` (a client given `Start` with no kept
records, its `me` opened and the downs drained); `fixtures::{ME, TASK,
OTHER}` are `Handle`s of `Asker::JIG`, numbers 1, 2 and 5;
`fixtures::task(9) -> Watchable` (`Subject::Task { task: 9 }`) and
`fixtures::delivery(task) -> Up` (a `Task` delivery on the engine watch
the domain named for that subject). Reference test:
```rust
#[test]
fn two_handles_on_one_subject_share_one_engine_watch() {
    let mut c = Client::started();
    c.step(Event::Watch { handle: TASK, subject: task(9) });
    c.step(Event::Watch { handle: OTHER, subject: task(9) });
    assert_eq!(c.downs().iter().filter(|d| matches!(d, Down::Open { subject, .. } if *subject == task(9))).count(), 1);
    c.step(delivery(9));
    assert_eq!(c.told(), vec![Told::Changed { handle: TASK }, Told::Changed { handle: OTHER }]);
    assert_eq!(c.domain.document(OTHER).map(|d| d.state), Some(State::Live));
}
```
Tests: `two_handles_on_one_subject_share_one_engine_watch`,
`the_last_unwatch_closes_the_engine_watch_and_tells_nothing`,
`a_dropped_watch_opens_again_under_a_fresh_name_after_its_backoff`,
`a_watch_moved_to_another_subject_keeps_its_document_behind`,
`a_refused_watch_ends_each_of_its_handles_once`,
`a_delivery_for_a_closed_name_is_dropped`,
`a_watch_past_the_handles_limit_ends_at_once`,
`an_application_handle_on_me_shares_the_client_s_own`. Each under 20 ms.
Proof: unit layer: the gate runs these tests; nothing else. Nothing
above calls the domain yet; jig 12.1 connects it, its story
`a_client_signed_in_by_the_printed_link_sees_me_live`
(tests/client-protocol) running this watch of `me` against the real
engine.
Don't: no per-watch heartbeat timer (the link drops a silent stream:
4.3); no edit of `jig-domain-client`'s watches; no `_` arm on `WatchEnd`.
Size: at most 280 lines.
Removes: nothing (the link's payload types move).

### 11.3 Keyed requests in the client: the table, askers, kept work by key; the wire's v8
Scope: `Send { key, request }` through skein-lib's `RequestTable`, held
by the domain (root.md, 4; client.md, 6): kept before sent, sent again
under its key, answered once (`Answered { key, outcome }`), its progress
told (`Pending { key }`) and read (`pending(key)`), bounded; joined on a
second `Send` of the same request under its key; the table confirmed to
the party, and told the retention, from `me`'s document, and up while
`me` is delivered; restored at `Start { kept }`, each told `Kept { key,
request }` (client.md, 11). The wire's `TableEntry` keeps the asker and
the number: the client wire's v8.
Needs: 11.2's `me` watch.
Files: `crates/jig-client-domain/src/{boundary.rs, domain.rs,
requests.rs, limits.rs}`, `src/tests/{fixtures.rs, requests.rs}`;
`crates/jig-client-protocol-wire/schema/client-v7.schema` and
`client-v7-part1.schema` renamed `client-v8*.schema` (`family
jig-client-wire 8`; `TableEntry` gains `asker: u32 number: u64`),
`src/generated/v7.rs` regenerated as `src/generated/v8.rs`, `src/lib.rs`
(`pub mod v8`, `VERSION: u32 = 8`), `golden/v7` regenerated as
`golden/v8` (`SKEIN_CODEGEN_WRITE=1 cargo nextest run -p
jig-client-wire-world --test drift`, which the drift test names);
`tests/client-wire/tests/{drift.rs:7-12, fuzzy_client_wire.rs:721,
test_client_wire.rs:7}` (the paths); the version sweep, `v7 as w` and
`wire::v7` to `v8` (`git grep -n -E 'client_wire::v7|v7 as w|wire::v7'`:
65 lines at the base, the link 20, the clients' component 21, the
configuration 1, the native shell 4, the test records 1, the test
service 4, the protocol world 3, the wire world 1, the clients'
component's world 10; 8 lines downstream) and the served ranges `first:
7, last: 7` (`crates/jig-protocol-engine-clients/src/tests/fixtures.rs:16`,
`tests/client-protocol/src/fixtures.rs:60`,
`tests/engine-clients/src/fixtures.rs:20`,
`tests/engine-clients/tests/stream.rs:143`); the link's
`translate/store.rs` (`table`, `restore_table`) and `boundary.rs`
(`Saved::Table { version, record: RequestRecord<Held> }`); the test
root's `store.rs` (`VERSION` 1 to 2) and `domain.rs` (its records carry
`Held` with the asker its `Owner` names: `Jig` 65536, `First` 1);
`tests/client-protocol/src/front/persistence.rs`. Downstream, its
terminal's `route.rs:61-72` (`Saved::Table`).
Shape: as the sketch: `Key`, `Held`, `Outcome::{Answered, Unknown,
Limit, Unconfirmed}`, `Pending`; `Event::{Start { kept:
List<RequestRecord<Held>> }, Send}`, `Up::Answer`, `Down::{Save, Erase,
Send}`, `Told::{Pending, Answered, Kept, Dropped}`;
`Domain::pending(key)`; `Progress::Retrying { why: Retry }`, `Retry {
Busy, NotReady, Lost, Undecodable }`: retrying says why ("busy, not
ready, lost, or a final answer that did not decode", client.md, 6), told
by `Pending { key }` once per answer, no new tell; `Up::Answer` gains
`why: Option<Retry>` (none for a final answer); `Told::Dropped { count: u32 }` says how many
kept records the client dropped at `Start`, unread or past the table's
room ("A record of a version it does not read is dropped, and the
application told", client.md, 11);
`Limits.requests: RequestLimits`, `Limits.ask_bytes: u64` (the bytes one
of jig's requests declares to the table; a family's declares its bytes'
length); `jig_client_domain::TABLE_VERSION: u16 = 2`, the table's
records' store version (1 was the test root's, whose `store::VERSION`
follows to 2). A second `Send` under a held key with another request
asserts ("a key conflict ... loud", client.md, 6). Up while `me` has a
delivery and its watch is not behind; down otherwise.
Test shape: `crates/jig-client-domain/src/tests/requests.rs`; fixtures
gain `KEY` (`Key { asker: Asker::JIG, number: 1 }`), `PARTY` (1),
`stop() -> Ask` (`Stop { project: 1, task: 9 }`), `me(party, retention)
-> Up` (a `Frame` delivery for the client's `me`), and `Client::live()`
(started, `me` delivered for `PARTY` with a retention of 1 s); the
table's limits copy, as literals, the world's profile, `table` at
tests/client/src/test.rs:70-77 (`requests: 4`, `bytes: 65536`, `out:
16`, 10 ms to 80 ms, `margin: 1 s`). Reference test:
```rust
#[test]
fn a_request_is_saved_before_it_is_sent_under_its_key() {
    let mut c = Client::live();
    c.step(Event::Send { key: KEY, request: stop() });
    let downs = c.downs();
    assert!(matches!(downs.first(), Some(Down::Save { record }) if record.request.key == KEY && record.scope == 1));
    assert!(matches!(downs.get(1), Some(Down::Send { .. })));
    assert_eq!(c.told(), vec![Told::Pending { key: KEY }]);
}
```
Tests: `a_request_is_saved_before_it_is_sent_under_its_key`,
`a_second_send_of_the_same_request_joins_the_first`,
`a_final_answer_is_told_once_with_its_outcome`,
`a_send_past_the_table_s_limit_is_answered_at_once`,
`a_send_before_me_names_a_party_is_unconfirmed`,
`another_party_s_kept_request_stays_parked`,
`a_kept_record_is_restored_and_told_kept_by_its_key`,
`a_kept_record_past_the_table_s_room_is_dropped_and_told`,
`a_retrying_request_says_why_once_per_answer`,
`a_request_past_the_retention_is_answered_unknown`. The wire's
`client_wire_schema_matches_generated_code_and_goldens` (tests/client-wire)
and the link's store tests keep their names. Each under 20 ms.
Proof: joint layer, for the record change: `cargo nextest run -p
jig-client-protocol-world -E
'test(a_reload_restores_a_pending_request_from_the_small_store_s_bytes)'`,
once; it passes when the test root's table record, now with its asker
and number, is written, read back over a reload and answered once by the
real engine (under 2 s). It passes; nothing else is wanted. The gate
runs the unit tests; jig 12.2's story
`a_stop_is_answered_once_after_its_commit_is_durable` runs the table
against the real engine.
Don't: no change of any other record (jig 14's); no translation of
another version's records (a record of another store version is
dropped, protocol/client.md, 10); no `Save`'s barrier in the domain (jig
12.2's, in the client).
Size: at most 295 lines (the generated code, the schema and the goldens
are not counted; the version sweep about 40).
Removes: the client wire's v7 (schema, generated module, goldens).

### 11.4 Reads by handle
Scope: `Read { handle, read }`, one terminal per read told as `Page {
handle, page }`; a new `Read` under a handle with one in flight abandons
the first, whose terminal is dropped; bounded reads in flight, a read
past them answered at once (client.md, 7; 4.2).
Needs: 11.3's domain.
Files: `crates/jig-client-domain/src/{boundary.rs, domain.rs, reads.rs,
limits.rs}`, `src/tests/{fixtures.rs, reads.rs}`.
Shape: `Event::Read { handle, read: Readable }`, `Up::ReadDone { read,
result: Readout }`, `Down::Read { read, what: Readable }`, `Told::Page {
handle, page: Readout }`; `Limits.reads`. A read past the limit is
refused by the client's limit, `Page { handle, page: Readout::Limit }`
(`Readout` gains `Limit`), never `Unreachable` (client.md, 7). An
`Unwatch` of a handle abandons its read in flight, whose late terminal
is dropped.
Test shape: `crates/jig-client-domain/src/tests/reads.rs`; fixtures gain
`PAGE` (a `Handle`, number 3) and `project(1) -> Readable`; `reads: 8`
copied from the world's profile. Tests:
`a_read_is_told_its_one_page_by_its_handle`,
`a_second_read_on_a_handle_drops_the_first_s_page`,
`a_read_past_the_limit_is_refused_by_the_client_s_limit`,
`unwatching_a_handle_drops_its_read_s_late_page`,
`a_family_read_s_page_is_its_bytes_under_its_owner`. Each under 20 ms.
Proof: unit layer: the gate runs these tests; nothing else. jig 12.1's
story connects it (its client reads the project its `me` names, against
the real engine).
Don't: no read kept in the small store; no retry of a read (its terminal
is the application's to act on).
Size: at most 120 lines.
Removes: nothing.

### 11.5 A run's text by handle
Scope: a watch of `Subject::Run` keeps the provisional text of one
turn, the turn being written, within `turn_bytes`, takes each delivery's
text past its offset, says `Text { handle }`; a delivery naming a later
turn, or a later retry of the same turn, takes the place of the text
held, from its own `from`; one naming an earlier turn or retry changes
nothing and tells nothing; a later turn's text never resets an earlier
one's; opened again, the watch continues from the turn and the offset
held (`Down::Continue`) (client.md, 8.3).
Needs: 11.2's watches.
Files: `crates/jig-client-domain/src/{boundary.rs, watches.rs, text.rs,
limits.rs}`, `src/tests/{fixtures.rs, text.rs}`. The offset rules for
one turn follow `jig-domain-client`'s `Conversation::stream`
(conversation.rs:298), which keeps them for the page today and goes
with jig 21.3: text past the offset taken once, a gap only with a cut,
a later retry replacing. Its list of live turns (`live`) and its fence
on the committed turn are the thread's, not the client's (client.md,
10; jig 15.4).
Shape: `Told::Text { handle }`, `Down::Continue { watch, subject }`,
`Domain::text(handle) -> Option<&RunText>`, `Limits.turn_bytes`. One
`RunText` held per engine watch of a run, its `turn` the newest
delivered; no list of turns, and no bound on their number:
`turn_bytes` is the one bound. A delivery is compared with the turn
held by `ProvisionalTurnName`'s derived order (attempt, number, retry):
the same name appends past the offset; a later one replaces from its
own `from`; an earlier one is dropped, and nothing is told.
Test shape: `crates/jig-client-domain/src/tests/text.rs`; fixtures gain
`RUN` (a `Handle`, number 4), `run(task, turn, retry, from, text) ->
Up`; `turn_bytes: 32` (the world's profile's `text`, copied). Tests:
`a_run_s_text_is_held_past_its_offset_and_told`,
`a_later_turn_s_text_never_resets_an_earlier_one` (turn 1 `ab` from 0,
turn 2 `c` from 0, then turn 1 `d` from 2, `text(RUN)` read after each:
turn 1's `ab`; turn 2's `c` from 0, neither `abc` nor under turn 1;
turn 2's `c` still; `Text { handle: RUN }` told twice in all),
`a_later_retry_replaces_the_turn_s_earlier_text` (then a delivery of
the earlier retry: `text(RUN)` unchanged, nothing told),
`a_dropped_run_watch_continues_from_the_offset_held`,
`provisional_text_from_before_a_restart_is_dropped_at_the_next_delivery`
(client.md, 8.2: "Provisional text streamed before it, never committed,
is dropped when the next delivery comes"),
`text_past_the_turn_s_bound_is_cut_and_marked`. Each under 20 ms.
Proof: unit layer: the gate runs these tests; nothing else. Connected by
jig 13.4's test root, whose reference UI watches a run through the
client, and its story
`a_person_s_words_reach_a_running_run_through_bytes`
(tests/client-protocol).
Don't: no committed turn in the text (committed turns are the task's
activity, contract.md, 7.3); no list of turns and no fence on a
committed turn in the client, which reads no activity (the thread's,
jig 15.4).
Size: at most 150 lines.
Removes: nothing.

## Interface sketches

Checked at the base with `cargo clippy -p jig-w6-sketch
--all-targets -- -D warnings` in a scratch crate, the contract named at
its base path `jig_domain_client::contract` (bodies
`unreachable!("sketch")`, unused parameters prefixed `_`), passing.
Who owns what: the domain owns handles, keys, engine watch names, the
table and every timer; the link owns encoding and the credential; the
application owns handles' and keys' numbers and their askers.

```rust
// crates/jig-client-domain/src/boundary.rs
// Moved from jig_client_protocol_link::boundary (11.2), names kept:
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Family { pub family: u16, pub version: u32, pub bytes: Box<[u8]> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask { Jig(Box<contract::Ask>), Family(Family) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Readable { Jig(contract::Read), Family(Family) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Watchable { Jig(contract::Subject), Family(Family) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Readout {
    Jig(contract::ReadResult),
    Family { family: u16, record: Option<Box<[u8]>> },
    /// Refused by the client's limit on reads in flight (11.4).
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answered { Jig(contract::Answer), Family(Family) }

/// The owner number of the root's child that asked (root.md, 15); kept, never read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Asker(pub u32);
impl Asker {
    /// jig's reference UI, one past every `u16`.
    pub const JIG: Asker = Asker(65536);
}
/// The application's name for a watch or a read; 64 bits, so a child's token fits (jig 13.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handle { pub asker: Asker, pub number: u64 }
/// The application's name for a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key { pub asker: Asker, pub number: u64 }
/// What the table keeps beside skein's record; mirrors `TableEntry`'s `asker` and `number` (v8).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Held { pub key: Key, pub request: Ask }
/// A request's one answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Done or refused, once the engine's commit is durable.
    Answered(Answered),
    /// Past the retention: whether the engine made it is unknown.
    Unknown,
    /// Refused at once by the client's limit; never queued.
    Limit,
    /// No party confirmed yet: neither kept nor sent.
    Unconfirmed,
    // jig 12.2 adds NotKept.
}
/// skein's four progresses, retrying with its reason; jig 12.4 adds `NotServed`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Progress { InFlight, Retrying { why: Retry }, Parked, Unknown }
/// Why a request is sent again (client.md, 6; protocol/client.md, 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Retry { Busy, NotReady, Lost, Undecodable }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pending { pub progress: Progress, pub party: u64, pub owner: Option<u16> }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State { Opening, Live, Behind, Ended }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Latest { Jig(contract::Delivery), Family(Family) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Document<'a> { pub state: State, pub latest: Option<&'a Latest> }
/// Why a handle ended; `Dropped` and `Stopping` open again, `Closed` is `Unwatch`'s.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why { Unknown, NotAllowed, Busy, SubjectEnded, SignedOut, Limit }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// The table's records restored, before any other event.
    Start { kept: List<RequestRecord<Held>> },
    Send { key: Key, request: Ask },
    Watch { handle: Handle, subject: Watchable },
    Unwatch { handle: Handle },
    Read { handle: Handle, read: Readable },
    /// From its link, decoded.
    Up { up: Up },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Up {
    Answer { attempt: Token, envelope: RequestEnvelope, answer: Option<Answered>, why: Option<Retry> },
    Watched { watch: Token, event: contract::WatchEvent },
    FamilyDelivered { watch: Token, record: Family },
    WatchEnded { watch: Token, end: contract::WatchEnd },
    ReadDone { read: Token, result: Readout },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request { Down { down: Down }, Told { told: Told } }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Down {
    Save { record: RequestRecord<Held> },
    Erase { key: RequestKey },
    Send { attempt: Token, key: RequestKey, request: Ask },
    /// Each opening under a token never used before.
    Open { watch: Token, subject: Watchable },
    /// A run's watch opened again from the text held (11.5).
    Continue { watch: Token, subject: contract::Subject },
    Close { watch: Token },
    Read { read: Token, what: Readable },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Told {
    Changed { handle: Handle },
    Text { handle: Handle },
    Ended { handle: Handle, why: Why },
    Page { handle: Handle, page: Readout },
    Pending { key: Key },
    Answered { key: Key, outcome: Outcome },
    Kept { key: Key, request: Ask },
    /// Kept records dropped at `Start`, unread or past the table's room (11.3).
    Dropped { count: u32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Backoff { pub first: Duration, pub most: Duration }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub requests: RequestLimits,
    pub ask_bytes: u64,
    pub document_bytes: u64,
    pub handles: u32,
    pub watches: u32,
    pub reads: u32,
    pub turn_bytes: u32,
    pub backoff: Backoff,
}
#[derive(Debug)]
pub struct Domain { /* handles, engine watches, reads, the table, timers, seed */ }
impl Domain {
    #[must_use]
    pub fn new(limits: &Limits, seed: u64) -> Domain;
    #[must_use]
    pub fn pending(&self, key: Key) -> Option<Pending>;
    #[must_use]
    pub fn document(&self, handle: Handle) -> Option<Document<'_>>;
    #[must_use]
    pub fn text(&self, handle: Handle) -> Option<&contract::RunText>;
}
pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>);
pub fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>);
#[must_use]
pub fn next_deadline(domain: &Domain) -> Option<Time>;
pub fn reclaim(domain: &mut Domain);
#[must_use]
pub fn max_out(limits: &Limits) -> u32;
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64>;
```

## Questions to raise when reached

- **A `Send` before `me` has named a party** (client.md, 6 and 11): a
  request "names its party", but the design does not say which party a
  request sent while starting is for. The plan answers it at once
  `Outcome::Unconfirmed`; if the designer rules otherwise (held until
  `me`), 11.3's `a_send_before_me_names_a_party_is_unconfirmed` changes.
- **Whether a handle going behind is told** (client.md, 8.1 and view.md,
  18.2): `Changed { handle }` says "a newer document"; the reference UI
  reads "behind" from the handle's state. The plan tells `Changed` when
  a handle's state moves too; raise it if the designer wants a distinct
  word.

## Done when

- `git -C ~/src/rust/jig ls-tree -d --name-only origin/main
  crates/jig-domain-client/src/contract crates/jig-client-domain/src/contract`
  prints only `crates/jig-client-domain/src/contract`.
- `git -C ~/src/rust/jig grep -n -E 'CLIENT_VERSION|VERSION: u32 = '
  origin/main -- crates` prints one line per wire, and
  `crates/jig-client-protocol-wire/src/lib.rs`'s says 8.
- `cargo nextest run -p jig-boundary -E
  'test(client_domain_crate_links_only_the_foundation)'` passes one
  test, in a worktree at `origin/main`.
