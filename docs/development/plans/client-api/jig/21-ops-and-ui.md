# Session 21: `ops` and the reference UI's state on the client alone

Jig 13 put the test application on `jig_client::Client`, the reference
UI reaching it through `jig_client_view::bridge`. This session does the
same for `ops`: its families travel as bytes through their own wire
crate, `jig-ops-client-protocol-wire` (21.1), and its root holds the
client and routes by asker (21.2). Then the reference UI's state becomes
`jig_client_view::Ui`, reading the client alone (view.md, 18): its
documents, run text and link from the client by its handles, the bridge
and the moved client half gone (21.3); and its requests on `pending`,
with each child watching `me` by a handle of its own in place of the
by-value hand-offs (21.4).

The repository downstream of jig copies `ops`'s client root: in 21.2,
21.3 and 21.4, the change to `ops`'s root (`examples/ops/client-domain`,
`examples/ops/client-view`) lands as one commit of its own within the
increment, whose message says what an application's root changes and
why, so that its diff reads alone (decisions.md, 6).

## Read first

- jig's `docs/design/client/domain/view.md`, sections 2 and 18 (18.1 to
  18.10).
- jig's `docs/design/client/domain/root.md`, sections 2, 3, 5, 7, 10, 12
  and 15.
- jig's `docs/design/client/domain/client.md`, sections 4, 6, 8, 11 and
  12.
- jig's `docs/design/client/domain/pages.md`, section 8.
- jig's `docs/design/client/domain/testing.md`, sections 4, 5 and 6.
- jig's `docs/design/protocol/client.md`, section 11;
  `docs/design/examples.md`, section 8.
- The plan's `findings/wire-and-client.md`, sections 2 and 5 (b, d),
  `code-organization.md`, sections 1, 2, 6 and 7, and decisions.md, 6.
- Sessions 12 and 13's files: their sketches are this session's types.
- skein's `programming-model.md`, sections 4.5, 5.2 and 6.3;
  `testing-strategy.md`, sections 2.2, 6 and 7.

## What the design asks

- `ops`'s wire: "`jig-ops-client-protocol-wire`, its families' wire,
  which its client and its engine both read: incidents, alerts,
  environments and panels, and environments' requests and their
  outcomes" (examples.md, 8; protocol/client.md, 11); its owners "its
  two connectors' client sides, numbered as its connectors are"
  (examples.md, 8; root.md, 15).
- The reference UI: `jig_client_view::Ui: step(now, &Client, Event, &mut
  Queue<Request>), fire(now, &Client, &mut Queue<Request>),
  next_deadline()`; "It keeps no copy of a watch's document: it reads
  each from the client, by its handles"; a read's page, which the client
  hands over once, it keeps "until the page that read it is left or the
  handle's next page replaces it"; "its handles and keys carry its asker,
  jig's owner, 65536" (view.md, 18 and 18.1); "Handles, not
  generations" (18.2).
- "`me`, always ... by a handle of its own" (view.md, 18.2); "A child
  that needs who is signed in, or the projects they hold roles in,
  watches `me` by a handle of its own" (root.md, 5).
- Requests "shown as asked" from `Pending` and `client.pending(key)`;
  "Kept work ... has no card ... Refused, or unknown whether made, it
  shows as a notice ...; kept for another party, it is counted for the
  page someone else's work" (view.md, 18.4).
- `jig-client-view` "depends on `skein-ui` and `jig-client`, and on
  nothing else of jig's" (view.md, 2).

## Choices this plan makes

- **`ops`'s families' wire is made here** (21.1), not in jig 16: `ops`'s
  root cannot hand the client its families as bytes without it, and
  21.3 cannot remove the bridge while `ops`'s root uses it.
- **The bridge ends in 21.3,** with the moved client half of
  `jig-domain-client` that jig 13.2 kept in `jig-client-view`
  (`watches.rs`, `reads.rs`, `link.rs`, `versions.rs`, the spine's watch
  functions, the frame's link judgment).
- **`Named` replaces `ShowObjects`, `ShowSections` and `ObjectsLeft`**
  (view.md, 18.1: "the whole list each time").
- **Not here:** the reference UI's lists on task queries and the shape
  pages removed (jig 16); the helpers (jig 15).

## Rules that bind it

- **`ops`'s root changes in one commit of its own per increment**
  (decisions.md, 6): before each landing of 21.2, 21.3 and 21.4, `git
  log --oneline origin/main..HEAD -- examples/ops/client-domain
  examples/ops/client-view | wc -l` prints 1.
- **A root keeps no map and no `_` arm** (root.md, 10): checked by
  clippy's `wildcard_enum_match_arm` and
  `match_wildcard_for_single_variants`, denied in the workspace; and
  `git grep -n -E 'Map<|BTreeMap' -- examples/ops/client-domain/src/route.rs
  examples/ops/client-domain/src/domain.rs
  testing/jig-client-test-domain/src/route.rs
  testing/jig-client-test-domain/src/domain.rs` prints nothing at each
  increment's end.
- **jig never names the repository downstream of jig**: `git -C <the
  session's jig worktree> grep -n -i -w temper -- crates testing
  tests examples scripts` prints nothing at each increment's end
  (code-organization.md, 6).

## Increments

### 21.1 `ops`'s families as bytes: `jig-ops-client-protocol-wire`
Scope: `ops`'s families' wire (examples.md, 8; protocol/client.md, 11):
infrastructure's environment requests and their outcomes,
observability's and infrastructure's subjects and documents (incidents,
alerts, environments, panels), in a schema, with the client's side's
codec from `ops`'s client sides' values (`infrastructure::{Ask, Answer,
Subject, Document}`, `observability::{Subject, Document}`).
Needs: jig 13 on `main` (the session's start).
Files: new `examples/ops/client-protocol-wire/{Cargo.toml,
schema/ops-client-v1.schema, src/lib.rs, src/generated/v1.rs, golden/v1/,
src/codec.rs}`; workspace `Cargo.toml` and `Cargo.lock` (shared);
`tests/client-wire/tests/drift.rs` (a third drift test,
`ops_client_wire_schema_matches_generated_code_and_goldens`);
`tests/boundary/src/tests/graph.rs` (a rule as
`client_wire_links_only_its_codec_and_foundation`, graph.rs:560, for
it).
Ready check (ruling 192, jig `73734dd6f`): the codec takes and returns
the wire crate's own `v1` records, never `jig-ops-client-domain`'s types
(`infrastructure::Ask` and the rest): 21.2 makes that crate depend on
this one, and `cargo metadata` reports a cycle, which
`client_wire_links_only_its_codec_and_foundation` forbids too. The wire
rule is at `graph.rs:589`.
Shape: `jig_ops_client_protocol_wire::{v1, VERSION, codec::{ask,
answer, subject, document, Undecodable}}`: `ask(&infrastructure::Ask)
-> Box<[u8]>`, `answer(&[u8]) -> Result<infrastructure::Answer,
Undecodable>`, `subject(&Subject) -> Box<[u8]>`, `document(subject,
&[u8]) -> Result<Document, Undecodable>`, one pair per family; the
engine's side encodes documents and decodes requests with the same
records, which the world's scripted `ops` engine uses.
Test shape: `examples/ops/client-protocol-wire/src/tests.rs`: a round
trip per record kind (`an_environment_request_round_trips_through_its_bytes`,
`an_incident_document_round_trips_through_its_bytes`,
`a_record_of_another_version_is_undecodable`), and the drift test. Each
under 20 ms.
Proof: unit layer: the gate runs these tests and the drift test;
nothing else. 21.2 connects it, its root encoding with it against
`ops`'s real engine.
Don't: no record of jig's in it; no shape of work (code-organization.md,
6).
Size: at most 250 lines (the schema and generated code are not
counted).
Removes: nothing.

### 21.2 `ops`'s root on `jig_client::Client`, routing by asker
Scope: `jig-ops-client-domain` as jig 13.4 made the test root: it holds the
client, routes by asker (its owners are its connectors, and it shares
their enum, `jig_ops_domain_connectors::Connector`, as it is: root.md,
15 and examples.md, 8, "as `ops` shares `Connector`"), its families'
values as bytes by 21.1's codec,
the reference UI through jig 13.4's bridge; `jig-ops-client-view` follows.
`ops`'s root's change is one commit of its own (the session's
introduction).
Needs: 21.1's crate.
Files: `examples/ops/client-domain/src/{domain.rs, boundary.rs,
route.rs, owner.rs, limits.rs, store.rs, infrastructure.rs,
observability.rs}`; `examples/ops/client-view/src/**` (its routing);
`tests/ops/src/client/{transport.rs, app.rs, script.rs}`,
`tests/ops/src/joint/**` (their transport: `Client::unlinked` in the
scripted world, bytes in the joint world through the clients'
component); `tests/boundary/src/tests/graph.rs`
(`ops_client_domain_is_step_code_with_only_client_and_connector_vocabulary`
and `ops_client_view_is_step_code_with_only_client_and_connector_vocabulary`,
graph.rs:337 and 350, gain `jig-client` and
`jig-ops-client-protocol-wire`; `jig-client-domain` is reached through
`jig-client`'s re-exports, jig 12.1).
Ready check (ruling 192): `ops`'s root maps its values to the wire
crate's `v1` records and back in a module of its own, as jig 13.3 does
for the test family (`family/wire.rs`), which joins the Files; so do the
manifests `examples/ops/client-domain/Cargo.toml`,
`examples/ops/client-view/Cargo.toml` and `tests/ops/Cargo.toml` (each
gains `jig-client`), `lib.rs:11` (`pub mod owner`), and
`tests/ops/tests/fuzzy_joint.rs:561` (`joint::infrastructure::ask(Ask)`
for the digest). The ops rules are at `graph.rs:360` and `:373`.
Ruling 218: `tests/ops/tests/fuzzy_joint.rs:737, :746, :755` join the
Files: 21.2 removes the `#[ignore]` jig 13.4 put on
`p_acceptance_survives_the_commit_before_its_durable_decision`,
`p_acceptance_survives_its_durable_decision_commit` and
`p_acceptance_survives_the_commit_after_its_durable_decision` (the old
root had no client to judge its link `Offline` and reopen its watches),
and calls `bridge::closed` after a bridge-routed `Close`, as 13.4's
commit body says. Ruling 220: so do `:764, :772, :780, :788, :798`: 21.2
removes the same mark from
`q_provision_survives_its_acceptance_commit_and_move_end_survives_reload`,
`q_provision_survives_its_made_commit_and_move_end_survives_reload`,
`q_move_end_survives_its_durable_answer_commit_and_reload`,
`p_story_survives_seeded_commit_crashes_and_random_presses` and
`q_story_survives_seeded_commit_crashes_reload_and_random_presses`
(Offline, and a dropped watch kept behind, are the client's).
Ruling 227: so do `crates/jig-protocol-engine-clients/src/translate/owner.rs`
(`charters`, :163: the wire's goals list sized by `value.goals.len()`,
as `translate/inbox.rs:93` sizes its rows' goals) and its test in
`crates/jig-protocol-engine-clients/src/tests/documents.rs`.
Ruling 230: so do the eight `acts` translations of
`crates/jig-client-protocol-link/src/translate/` sized by
`limits.client.rows` (`pages_part3.rs:155`, `pages_part5.rs:253`,
`pages_part6.rs:41, 50, 71, 172, 255`, `pages_part7.rs:97`), each bounded
by `c::MAX_OFFERED_ACTS` as `documents_part8.rs:35` is, and a
translation test of `owner_escalation_fields`.
Shape: as jig 13.4's: `ops`'s table owner, `owner::Owner` (with `Kept {
scope }`), goes, and no owner enum takes its place; a tell routes by
`jig_client_view::Owner::of_asker` (jig 13.4), `Jig` to the reference
UI, `Application { number }` by `jig_ops_domain_connectors::connector(number)`
to `Connector::Observability` or `Connector::Infrastructure`, and none
to no child; an ask leaves under `Owner::Application { number:
jig_ops_domain_connectors::number(connector) }.asker()`. `ops`'s client
limits declare each connector whose families it sends, by
`jig_ops_domain_connectors::number`, at
`jig_ops_client_protocol_wire::VERSION` (`jig_client_domain::Limits.owners`,
jig 12.4; client.md, 8.2).
Test shape: `examples/ops/client-domain/src/tests.rs` (kept and
extended): `routes_an_infrastructure_tell_to_its_side`,
`passes_the_client_s_transport_requests_out_in_order`,
`a_tell_under_an_asker_no_connector_names_reaches_no_child`. Each under
20 ms.
Proof: joint layer, `ops`'s real engine: `cargo nextest run -p
jig-ops-world --profile fuzzy -E 'binary(fuzzy_joint) &
test(=p_accepts_the_restart_from_the_incidents_page)'`, once (0.83 s at
the base); it passes; nothing else is wanted. And, once, the eight tests
rulings 218 and 220 marked: `cargo nextest run -p jig-ops-world --profile
fuzzy -E 'binary(fuzzy_joint) & test(/_survives_/)'`; each
passes. In the default suite, the
gate runs `p_accepts_a_restart_from_the_incidents_page` (tests/ops);
nothing else.
Don't: no change to `ops`'s families' meaning or pages; no file of
`ops`'s root changed outside that one commit.
Size: at most 250 lines; ruling 225 re-estimates it with ruling 192's
four value mappings as plain functions, field by field: aim to land at
most 4,500 production lines; past 4,500, the report gives the count by
file and why, and the session goes on; at 5,000, stop and block with the
count by file. Ruling 225: an owner's read refusal crosses in `ops`'s
wire, as jig 13.3's `ReadAnswer` does: `ops-client-v1.schema`, its
generated code and goldens join the Files.
Removes: `ops`'s root's `RequestTable` and its table owner,
`owner::Owner`.

### 21.3 The reference UI's state on the client alone: `jig_client_view::Ui`
Scope: view.md, 18.1's surface: `Ui::step(now, wall, &Client, Event,
out)`, `fire`, `next_deadline`; its watches and reads by handle under
`Asker::JIG`, documents read from `client.document(handle)` when told
`Changed`, a run's text from `client.text(handle)`; its link from
`Told::Link` and `client.link()` (signed out, out of date and someone
else's work as pages not about the work, view.md, 18.6); its page's
generations become handles (18.2, "Handles, not generations"). Both
roots step it with the client they hold.
Needs: 21.2 (both roots on the client).
Files: `crates/jig-client-view/src/{ui.rs, boundary.rs, page.rs and
page/**, objects.rs, application.rs, conversation/**, frame/me.rs,
lib.rs}`; removed: `bridge.rs`, `watches.rs`, `reads.rs`, `link.rs`,
`versions.rs`, the spine's watch functions (in
`crates/jig-client-view/src/ui.rs` at jig `154e10942`: `open` :465,
`retire` :481, `close_one`, `close_page` and `close_all` :505-521,
`close_watch` :560, `watch_event` :813, `watch_end`, `terminal_watch`
and `read_final_task` :958-1109); both roots' `route.rs` (the Ui stepped
with `&Client`; `ops`'s in its own commit); `testing/jig-client-test-view`,
`examples/ops/client-view` (they read `ui.sections()`, `ui.shown()`);
the worlds' fronts.
Re-check at jig `154e10942` (after 21.2): the users of what 21.3 removes
join the Files. `Request::{SignIn, Fact}`, `Act::SignIn` and
`signin::Provider` in the view crate: `signin.rs`, `action.rs:77`,
`binding.rs:168`, `pages/not_the_work.rs:31-39`, `limits/ui.rs:19`
(`providers`), `sizes/pages.rs:292-295`, `store.rs:70, :81, :125`,
`intents.rs:114, :182`. In both roots: `boundary.rs`'s `Fact::Jig`
(`testing/jig-client-test-domain/src/boundary.rs:41`,
`examples/ops/client-domain/src/boundary.rs:50`), `domain.rs`'s
`provider()` (ops :120, the test root :97), `lib.rs`, and ops's
`tests_support.rs:24, :133`. The setups of `providers: 2` and
`provider(..)`: `tests/client/src/test.rs:29, :167, :169`,
`tests/ops/src/joint/fixture.rs:101`, `tests/ops/src/client/app.rs:39,
:156`, `tests/client-protocol/src/fixtures.rs:79`, ops client-domain
`tests.rs:26`, ops client-view `tests.rs:23`,
`testing/jig-client-test-view/src/tests/transport.rs:85`,
`testing/jig-client-test-domain/src/tests/root/fixtures.rs:19`.
`tests/client/src/observe.rs:203-241` (`jig_received`, an exhaustive
match on the Ui's `Event` that feeds the referee's receipts,
`tests/client/src/harness/referee.rs:215-221`). The view's `Domain`
becomes `Ui` in each file that names it (`git grep -n -w 'Domain' --
crates/jig-client-view/src`, and what the compiler names): those join
the Files, listed in the commit body.
Ready check (ruling 192): the watch spine's `open`, `retire` and
`close_*`, and `watch_event`, are in `ui.rs` since 13.2 (lines above);
`ui.sections()` is read by `crates/jig-client-view/src/pages.rs:108-220` and
`sections/task.rs:61` (the examples call `shown()` alone). The sketch
derives `Debug`, `PartialEq` and `Eq` on the Ui's `Event` and `Request`,
`Debug` on `Ui` and `Refused`, and `How` derives `Copy, PartialEq, Eq`;
`next_deadline` and `shown` are `#[must_use]`. `jig_client::Event` and
`jig_client::Request` derive `Debug, PartialEq, Eq` since `5c06a7e98`
(`crates/jig-client/src/client.rs:10, :29`): `client.rs` is not in the
Files; `Request` is not `Clone` (`With` is not). `step` and `fire` take
`wall`, as the sketch has them (view.md, 18.1, amended), and the Ui's
`Request::{SignIn, Fact, Continue}` go (the designer's answer of
2026-10-17, in the Shape).
Shape: as the sketch: `Ui`, `Event::{Start, Went, Left, Act, Named,
Client { told }}`, `Request::{Client { asked }, Show, Save, Erase,
ShowPart, PartsLeft}`, `How`, `Refused`. `Named` replaces
`ShowObjects`, `ShowSections` and `ObjectsLeft` (view.md, 18.1, the
whole list each time); naming the same sections again keeps their
handles and watches open (pages.md, 12.5). Until 21.4 the Ui shows a
request answered `NotKept`, `Limit`, `Unconfirmed` or `Unknown` as
`Progress::Unknown` shows today. The frame's party part says a sign-out
did not reach the engine, from `client.link().sign_out_lost`, and
offers it again (view.md, 18.6; pages.md, 2). A page whose watches and
reads would pass the Ui's handles opens what fits and says what it left
out (view.md, 18.2); on an application's page, each named section says
in its own region when it is behind, has rows left out, was refused or
is missing (pages.md, 12.5; view.md, 18.2). The sign-in page leaves a
slot for the application's own ways of signing in (pages.md, 8 and 9;
view.md, 4.1), owned by the application, `Owner::Application`, and
filled by its view root. The slot takes the place of the page's
providers: `Request::SignIn { provider, back }`, `Act::SignIn`,
`signin::Provider` and the providers' list go, and once `me` names the
party the Ui comes back to the place it kept (view.md, 18.6), with no
`back`. The test application's view puts its "Sign in with Test"
button (`tests/client/tests/shell.rs:78`) in the slot; its act is the
test root's own sign-in request, which its worlds carry as `Duty::SignIn`
does at jig `154e10942` (`Duty::SignIn`,
`tests/client/src/harness/world.rs:565`), less the `back` it goes to; `ops`'s the same, in its own
commit. `Request::Fact` and the moved `facts.rs` go (view.md, 18.1): no
root, world or shell reads a fact (each drops it:
`tests/client/src/harness/world.rs:567`,
`tests/client/src/joint/front.rs:224`,
`tests/client-protocol/src/native.rs:190`), and the five lines that name
a `Fact` in four of the moved crate's test files (`git grep -c` at jig
`5274ad08e`) check the refusal the page or the act shows instead
(view.md, 18.2 and 18.4). A handle the
client ends `Limit`, or a read it answers `Limit`, is one the page left
out (view.md, 18.2). `Continue` goes: the client continues a run's
watch (client.md, 8.3). `Told::Dropped` reaches the Ui
(`Event::Client`), which shows it from 21.4. A read's page stays with
the page that read it (view.md, 18 and 18.3, amended at jig
`de4f80564`): each holder of one stays as `main` has it
(`page/project.rs`'s `ProjectPage.owner`, `owner_spend` and read
windows, the working set's bodies read to say how an object ended), and
goes when the page is left or a page on the same handle replaces it; a
read answered with none leaves it. Its pricing stays
(`page::project::worst_case`, `windows::worst_case`). Nothing in
`crates/jig-client` or `crates/jig-client-domain` changes for completed
reads: no storage, accessor or limit.
Test shape: the moved tests keep their names on the new surface; their
fixtures (`src/tests/fixtures.rs`) hold a `jig_client::Client::unlinked`
and feed its `Up` (jig 12.5) for documents. Added:
`a_page_reads_its_documents_from_the_client_by_handle`,
`a_handle_the_page_no_longer_holds_is_dropped`,
`signed_out_shows_the_sign_in_page_and_keeps_the_place`,
`out_of_date_shows_its_page_and_sends_nothing`,
`a_sign_out_that_did_not_reach_the_engine_is_offered_again`,
`a_page_past_the_ui_s_handles_opens_what_fits_and_says_what_it_left_out`,
`naming_the_same_sections_again_keeps_their_watches_open`,
`each_named_section_says_behind_in_its_own_region`,
`the_sign_in_page_leaves_a_slot_for_the_application_s_ways`,
`a_handle_the_client_s_limit_ends_is_one_the_page_left_out`,
`a_read_s_page_stays_until_the_page_is_left_or_its_next_page_replaces_it`
(a project's settings read, then another handle's `Changed`: the
settings still show; the page left: they are gone). The client
world's referee (tests/client, `harness/referee.rs`) gains testing.md,
6.3's "A sign-out told", as jig `bbb6303aa` amended it: "After a person
asks to sign out, the sign-in page is a fault while the engine still
holds the sign-in they asked to end. Once the engine has ended it, by
that sign-out or by an end of its own ..., before the ask or after it,
the page shows that end, whatever became of the ask. The world observes
each end of a sign-in where the engine makes it". At jig `15266f311`:
`Observed::SignOutReceived { client }` becomes `Observed::SignInEnded {
client }`, pushed where `SignOutReceived` is (`scripted.rs:507`,
`joint/session/signing.rs:37`, which the joint front's `SignedOut` fault
already calls, ops' `client/transport.rs:474`) and in each scripted
front's `FrontFault::SignedOut` arm (`scripted.rs:453`, ops'
`client/transport.rs:417`); a scripted front pushes it once for each
client bound to the ended sign-in's party. The referee keeps, beside
`sign_out_asked`, the clients whose sign-in the engine ended: a
`SignInEnded` adds its client; `Seen::SignedOut` passes when its client
has no sign-out asked or is among them, and then leaves both sets;
nothing else takes a client out of the ended ones (a reload does not:
the scripted engine keeps the sign-in it ended). Referee tests:
`a_sign_out_shown_done_before_the_engine_received_it_is_caught`, as it
is, and `a_sign_in_the_engine_ended_shows_the_sign_in_page_after_a_lost_sign_out`
(`Truth(SignInEnded)`, then `SignOutAsked`, then `Seen::SignedOut`:
`Verdict::Passed`), the order of seed 101 of
`random_people_in_new_chat_starts`. No other referee rule or judge
changes, and the Ui does not: it shows the sign-in page once the link
says signed out (view.md, 18.6). Each under 20 ms.
Proof: joint layer: `cargo nextest run -p jig-client-protocol-world
-E 'test(a_person_signs_in_by_the_printed_link_and_starts_a_chat) |
test(a_slow_reader_gets_fewer_deliveries_and_others_are_answered)'`,
once (under 2 s); it passes; nothing else is wanted. In the client's
world, the gate runs
`a_dropped_watch_shows_the_page_behind_until_it_reopens` (tests/client);
nothing else.
Don't: no copy of a watch's document kept by the Ui across steps: it
reads each from `client.document(handle)` (view.md, 18); a read's page
is the page's to keep, never the client's (view.md, 18.3); no decision
the client makes (whether the link is live, when to retry).
Size: at most 320 lines (most of the change is removal; the sign-in
slot, its "Sign in with Test" button and the sections' words about 60
of them; the read's page test about 20).
Ruling 240 re-estimates it, with the `Domain` to `Ui` rename (ruling
234) in the count: aim 2,500, stop at 3,000 added lines by
code-organization.md, 7's command; no split.
Removes: the bridge; the moved `watches.rs`, `reads.rs`, `link.rs`,
`versions.rs`; the page's generations; the Ui's `Request::{SignIn,
Fact, Continue}`, `Act::SignIn`, `signin::Provider` and its list, and
the moved `facts.rs`.

### 21.4 The reference UI's requests on `pending`; siblings watch `me` themselves
Scope: the Ui's intents keyed `Key { asker: Asker::JIG, number }`, drawn
from its seed, their progress read from `client.pending(key)` and shown
as asked, answered once (`Answered`), `NotKept` offered again and
`Unknown` saying where to look (view.md, 18.4); kept work under jig's
asker shown as a notice when refused or unknown, counted for "someone
else's work" by `pending(key).party` (pages.md, 8); the by-value
hand-offs (`Frame`, `FrameIs`, `Ask { ByValue }`, findings 5d) replaced
by each child's own handle on `me` (root.md, 5), in the test
application's family sides and `ops`'s (its own commit). An act refused
by the client's limit (`Outcome::Limit`) is refused in place, "too many
requests in flight", never queued (view.md, 18.4); one answered
`Outcome::Unconfirmed` is refused in place and offered again once the
link is live. The page "someone else's work" (pages.md, 8) leaves when
its party signs in, or when each of its kept requests is answered
unknown. Kept work the client dropped at start (`Told::Dropped {
count }`, which the root gives the Ui: root.md, 8) shows as a notice
saying how many (view.md, 18.4 and 18.7).
Needs: 21.3's `Ui`.
Files: `crates/jig-client-view/src/{intents.rs, notices/kept.rs,
page/**, frame/me.rs, boundary.rs, ui.rs}`;
`testing/jig-client-test-domain/src/{first.rs, second.rs, route.rs}`;
`examples/ops/client-domain/src/{infrastructure.rs, observability.rs,
route.rs}`; the worlds' fronts.
Ready check (ruling 192): the Files gain `limits.rs` in ops (about
:38-80 since 21.2) and
in the test root (:39-49), whose factor 3 comes from the frame hand-offs
this increment removes; the tests that build `Event::Link` and
`Event::Confirmed` (at jig `154e10942`: ops client-domain `tests.rs:1135-1186`, ops
client-view `tests.rs:1619-1620`,
`testing/jig-client-test-domain/src/tests/family_terminals.rs:55-56,
:266`); and the family sides'
`Request::Frame` (ops `infrastructure.rs:97`, `observability.rs:82`;
`first.rs:63`) and `FrameShown`, which go and join the Done-when grep;
`tests/client/tests/shell.rs:30` (it builds `Event::Ask { ByValue }`)
and `tests/client/src/observe.rs:230-242` (it matches `Event::{Kept,
KeptGone, Ask, Frame}`), which no grep covers.
Re-check at jig `48a745eb9` (after 21.3, ruling 257). The lines above
moved: ops client-domain `tests.rs:1130-1181`, ops client-view
`tests.rs:1599-1600`, `family_terminals.rs:262`; `Request::Frame` at
ops `infrastructure.rs:98`, `observability.rs:83`, `first.rs:64`;
`observe.rs:187-199`; the factor 3 at ops `limits.rs:35-73`, the test
root's :33-44. The Files gain:
- in `crates/jig-client-view/src`: `ui/client.rs` (`Told::Answered`
  :27, `Told::Kept` :39-49, which builds `Event::Kept`, `Told::Dropped`
  :96); `signin.rs:44` (`Request::Confirmed { party: None }`);
  `limits/ui.rs:29, :86` and `ui.rs:82, :172` (`kept`,
  `kept_requests`: the Ui's own bounded list of kept keys, since the
  client offers `pending(key)` alone); `notices.rs:38-61`, and the
  phrases of the "too many requests in flight" refusal and the dropped
  count's notice (`phrases/jig.rs` :142, :1213, :1951, :2518,
  `english.rs:762`; none says a count or "in flight" now,
  `english.rs:958` says "Request limit reached");
- the Ui's tests the removed variants reach (74 errors in
  `cargo check -p jig-client-view` with them removed):
  `tests/domain/{spine.rs (:221, :248), lifecycle.rs,
  conversations.rs, time.rs, policy.rs, answers.rs}`,
  `tests/fixtures/client.rs:193-198`, `tests/bridge.rs:68`;
- for each family side's own `me` handle: `testing/jig-client-test-domain/src/family/client.rs`
  (`asked` :26, `told` :137, which takes only `Latest::Family`), ops
  `examples/ops/client-domain/src/family.rs` (`asked` :72, :184, `told`
  :109, :221, `Told::Link { .. } => return` :170, :282);
- the roots' `JigOut::{Answered, Unknown}`, which carry the
  `Request::{Answered, Unknown}` this increment removes: both roots'
  `boundary.rs` (test :37-38, ops :45-46) and `route.rs` (test :144-146,
  ops :191-193), ops `tests_support.rs:128-129`, the test root's
  `tests/root.rs:72, :632`, `tests/root/client.rs:72`,
  `tests/root/fixtures.rs:212`, `tests/client/src/test.rs:268`,
  `tests/ops/src/client/app.rs:198`;
- `tests/client/tests/fuzzy_memory.rs:286` (it builds `Event::Kept`);
  the users of the `limits.rs` heap bounds,
  `tests/client/tests/{client_construction.rs,
  fuzzy_random_regressions.rs, owner_budget.rs}` and
  `tests/ops/tests/fuzzy_client.rs`; the `limits.jig.kept` guards (ops
  `limits.rs:28`, the test root's :25), ops client-domain
  `tests.rs:387` and the `kept: 4` fixtures.

The factor 3 is derived again from the hand-offs that remain (the
`Named` route, `first.rs:305, :499`, and `ShowPart`/`PartsLeft`,
`application.rs:278, :280`, `page/routing.rs:117`, still nest first, jig,
first), and the commit body says the derivation. Refresh any further
copy of what this increment removes, and list it in the commit body.
Shape: `Event::Ask { ask: ByValue }`, `Event::Frame { asked }`,
`Request::FrameIs`, `Request::Answered { asked }`, `Request::Unknown {
asked }`, `Event::Kept { scope, requests }`, `Event::KeptGone`,
`Request::{Confirmed, Retention, Link}` go; the family sides' `Event::{
FrameIs, Confirmed, Link }` give way to their `me` handle and
`Told::Link`.
Test shape: the moved intent tests keep their names; added
`a_request_shows_the_progress_the_client_reports`,
`a_request_not_kept_is_offered_again`,
`a_kept_request_refused_shows_a_notice`,
`another_party_s_kept_work_shows_someone_else_s_work`,
`a_family_side_reads_the_party_from_its_own_me_handle`,
`an_act_past_the_client_s_limit_is_refused_in_place`,
`an_act_answered_unconfirmed_is_offered_again_once_live`,
`someone_else_s_work_leaves_when_its_party_signs_in`,
`someone_else_s_work_leaves_when_its_requests_are_unknown`,
`kept_work_dropped_at_start_shows_a_notice`. Each under
20 ms.
Proof: joint layer: `cargo nextest run -p jig-client-protocol-world
-E 'test(signed_out_with_requests_pending_then_signed_in_again) |
test(a_request_whose_answer_was_lost_is_made_once)'`, once (under 2 s);
and `cargo nextest run -p jig-ops-world --profile fuzzy -E
'binary(fuzzy_joint) & test(=p_accepts_the_restart_from_the_incidents_page)'`,
once; both pass; nothing else is wanted.
Don't: no intent kept across a restart (view.md, 18.4: kept work has no
card); no map in a root.
Size: at most 300 lines (the dropped notice about 10).
Ruling 258 re-estimates it, with ruling 257's Files in the count: aim
800; past 800, the report gives the count by file and why, and the
session goes on; at 950 added lines by code-organization.md, 7's
command, stop and block with the count by file; no split. Ops' two
families share one `me` module that takes the connector, in place of
`infrastructure/me.rs` and `observability/me.rs`, which differ in the
connector alone.
Removes: the by-value hand-offs; `Kept { scope, requests }`.

## Interface sketches

Checked at the base with `cargo clippy -p jig-w6-sketch
--all-targets -- -D warnings` in a scratch crate, and again at jig
`154e10942` with the derives and `#[must_use]` below, passing; `Record`
and `StoreKey` are the view's `store::{Record, StoreKey}`, which keep
their names. The Ui owns its place, page, working set, intents,
drafts and notices; it reads documents, pending requests and the link
from the client the root lends; the root owns only the client and its
children.

```rust
// crates/jig-client-view/src/ui.rs (21.3)
/// What the reference UI hears (view.md, 18.1).
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    /// Its own records, as the application kept them; the place it starts at; its seed.
    Start { records: List<Record>, place: Place, seed: u64 },
    /// A place of the reference UI's gone to.
    Went { place: Place },
    /// A place of the application's.
    Left,
    /// A person's act on one of its nodes.
    Act { action: Act },
    /// What the application's page names of jig's, the whole list each time.
    Named { objects: List<ObjectKey>, sections: List<Section> },
    /// What the client told under `Asker::JIG`, and the link, as the root routes them.
    Client { told: Told },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum How { Push, Replace }
/// What the reference UI asks (view.md, 18.1).
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    /// One of the client's events, `Watch`, `Unwatch`, `Read`, `Send` or
    /// `SignOut`, which the root gives the client in the same step.
    Client { asked: jig_client::Event },
    Show { place: Place, how: How },
    Save { records: List<Record> },
    Erase { records: List<StoreKey> },
    ShowPart { part: Part, scope: PartScope },
    PartsLeft,
}
#[derive(Debug)]
pub enum Refused { Navigation, Arrangement }
#[derive(Debug)]
pub struct Ui { /* place, page, objects, intents, drafts, notices, seed */ }
impl Ui {
    pub fn new(limits: &Limits, navigation: Navigation, home: Address, arrangement: Arrangement) -> Result<Ui, Refused>;
    pub fn step(&mut self, now: Time, wall: Wall, client: &Client, event: Event, out: &mut Queue<Request>);
    pub fn fire(&mut self, now: Time, wall: Wall, client: &Client, out: &mut Queue<Request>);
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time>;
    #[must_use]
    pub fn shown(&self) -> u64;
}
```

## Questions to raise when reached

- None beyond jig 12's and jig 13's.

## Done when

- `git -C ~/src/rust/jig ls-tree -d --name-only origin/main
  examples/ops/client-protocol-wire` prints the directory.
- `git -C ~/src/rust/jig grep -n -E 'RequestTable' origin/main --
  examples/ops/client-domain/src` prints nothing.
- `git -C ~/src/rust/jig ls-tree --name-only origin/main --
  crates/jig-client-view/src/bridge.rs crates/jig-client-view/src/watches.rs
  crates/jig-client-view/src/reads.rs crates/jig-client-view/src/link.rs
  crates/jig-client-view/src/versions.rs` prints nothing.
- `git -C ~/src/rust/jig grep -n -E -e 'FrameIs|ByValue' -e 'Kept \{ scope'
  -e 'KeptGone' -e 'JigOut::(Answered|Unknown)' -e
  'Request::(Confirmed|Retention)\b'
  -e 'fn watch_event' -e 'Event::Frame' -e 'Request::Frame' -e
  'FrameShown' origin/main --
  crates/jig-client-view/src testing/jig-client-test-domain/src
  testing/jig-client-test-view/src examples/ops/client-domain/src
  examples/ops/client-view/src tests/client tests/ops` prints nothing
  (ruling 257 added `KeptGone`, `JigOut`, `Request::(Confirmed|Retention)`
  and the two `tests` directories).
