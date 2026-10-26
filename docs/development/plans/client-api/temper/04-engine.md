# Session 04: the engine and the forge's family on templates and the collapsed wire

jig's engine takes its shapes of work from templates in the configuration
(jig 09), its client wire collapses onto the plumbing operations, `me` and
the task line (jig 14, 22), and the shapes of work leave its engine and its
wire (jig 16). temper follows each: its configurations name jig's two
templates in place of `chats` (`configuration.md`, 2); the forge's family
carries jig's refusal facts as the collapsed wire encodes them, and the
forge's form learns from jig's typed answer that a version is not served
(`client.md`, 4.4, 4.6, 8); the engine root's arms and records for chats,
goals and lists go when jig's do, and the tests that started chats with
`StartChat` make them from the chat template. A last increment checks that
a task line of the forge's deepest path is served under the shipped
profile, whatever `documents.lines`, and that temper's client takes a
path's segments apart from its rows (`configuration.md`, 7).

## Read first

- temper's `docs/design/configuration.md`: 2 (the bullet "Templates are
  jig's two defaults"), 7 (the bullets "The engine's documents fit
  temper's client" and "The forge's paths fit a document").
- temper's `docs/design/client.md`: 4.4 (the bullet "Out of date"), 4.6
  (the outcome "refused by the core"), 8 (the bullets "jig's clients'
  component" and "The forge's request, decoded before the root").
- temper's `docs/design/STATUS.md`, 3, "temper on jig's client API" (the
  sentence on `temper-service/src/family/refusals.rs`).
- jig's `docs/design/protocol/configuration.md`, 3.1; jig's
  `docs/design/client/domain/contract.md`, 3.6, 4 and 8.
- This plan's `jig/09-templates.md`, `jig/14-wire.md` and
  `jig/16-shed.md`: their Shapes, and how each rewrote jig's own tests
  and fixtures (`jig-test-service`, the core's and the channel world's
  fixtures); `decisions.md`, 1c and 6; `findings/temper.md`, 4 and 6.
- skein's `programming-model.md`: 3, 4.4, 6.3 (by number only); skein's
  `testing-strategy.md`: 2.2, 2.9, 7 (by number only).

## What the design asks

- **Templates** (`configuration.md`, 2): "Templates are jig's two
  defaults, chat (template 1, label 1) and goal (template 2, label 2), as
  jig's `protocol/configuration.md`, 3.1 gives them, the chat's executor
  the charter chats run under; temper defines no template of its own."
- **A refusal by the core** (`client.md`, 4.6): "jig's refusal facts,
  carried whole as `jig-client-protocol-wire` encodes them, and shown in
  jig's words for its refusals"; and STATUS: `refusals.rs` "copies jig's
  private translation of refusals ...; it goes when temper takes jig's
  typed answers."
- **Out of date** (`client.md`, 4.4): "when the engine does not serve
  this client's version of the forge's records (the stream's hello, or
  jig's typed answer that a version is not served, naming the ranges
  served ...), the page says 'This temper is out of date for adopting
  repositories: run the temper the engine runs.', the button is
  disabled, and a pending adoption stays in the small store."
- **The forge's paths fit a document** (`configuration.md`, 7): "jig's
  documents carry a resource's path and a pattern whole, with as many
  segments as the engine admits in a name ..., and `documents.lines`
  bounds a document's short lists alone, at most the client's rows";
  "temper's client takes a path's segments, 9, apart from its rows".

## Choices this plan makes

- **temper defines no template.** The configurations temper writes by
  hand name jig's two defaults, in jig 09's format, the chat's charter 1
  and its grace 5,000 ms (today's `close_grace_ms`); those made from
  jig's `engine_document()` take jig's fixture as it is.
- **The engine follows jig's arms, and adds none.** Each arm and record
  of temper's engine root for a chat, a goal or a list goes in the
  increment that follows the jig landing that removes it, replaced by
  nothing (code-organization.md, 6: "No shape of work in the engine or
  the wire").
- **`refusals.rs`:** if jig 14's Shape makes the translation of
  `jig_domain_people::Refusal` into the wire's refusal record callable by
  an application (a `pub fn` of `jig-protocol-engine-clients` or of the
  wire crate), 04.2 calls it and `crates/temper-service/src/family/refusals.rs`
  goes. Otherwise, the default, 04.2 edits it as jig 14's diff edits
  `translate/refusals.rs`, and it stays a copy (STATUS's row keeps it).
- **The chats tests made by `StartChat`** are made from the chat
  template as jig 16 makes jig's own (the helper its Test shape names);
  where it names none, by one fixture, `walking::chat(project, words)`,
  in `tests/engine/domain/src/walking.rs`, written once and used by the
  others.
- **temper's store version.** temper's `CoreRefused` record keeps jig's
  people refusal's bytes (`crates/temper-protocol-store/schema/store-v1.schema:175-176`,
  bound 23). The first temper increment whose repin includes jig
  02.3's landing (people v3) bumps `STORE_VERSION` to 2 and writes
  `store-v2.schema` (README, "Format versions"); 04.1 bumps it if no
  increment did before it (`git -C <worktree> show
  origin/main:crates/temper-protocol-store/src/lib.rs | grep
  'STORE_VERSION: u32 = 1'` prints a line); a later increment of this
  session edits `store-v2.schema` in place.
- **A path is checked, not assumed:** 04.4 serves a task line of the
  forge's deepest path in the engine's domain world under jig's small
  profile and under the process tier's override, and holds the engine's
  bounds on a name's segments to the client's.

## Rules that bind it

- **Each increment whose Needs name a jig capability starts with a
  repin commit of its own** onto the landing of the newest jig step its
  Needs name (README, Rules, "Pins": 04.1 jig 09.4's, 04.2 jig 22's
  last, 04.3 jig 16's last), found by `git -C ~/src/rust/jig fetch origin && git -C ~/src/rust/jig log
  origin/main --all-match --grep='^Plan: client-api$' --grep='^Step:
  jig-NN.k$' --format=%H -1`
  with the step's number, never jig's `origin/main`; when `git -C
  ~/src/rust/jig merge-base --is-ancestor <that landing> <temper's
  present jig pin>` succeeds, no repin commit is made. After a repin,
  `grep -o 'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*'
  Cargo.lock | sort -u` prints three lines: `jig.git#<the landing>`,
  smith's line equal to that of `git -C ~/src/rust/jig show <the
  landing>:Cargo.lock`, and skein's a commit that holds it or, while it
  does not hold temper's skein floor, the floor (`git -C
  ~/src/rust/skein merge-base --is-ancestor <that commit> <skein's
  line>` succeeds), which the repin's `cargo update -p skein-lib
  --precise` takes only when temper's skein does not already hold it
  (README, "Pins", "The terminal's room", "The print's room" and
  "temper's skein ahead"); a second revision of a
  source still there after a second `cargo update` is a question.
- **jig 08.4's census:** the first increment of this session whose
  repin includes jig 08.4's landing repairs temper's census walk, as the
  README's "jig 08.4's census" says.
- **A repin's repairs outside its Shape** follow jig's own diffs of
  `jig-test-service`, `jig-client-test-domain` and `ops`, counted with
  `git diff --numstat origin/main...HEAD -- <the changed paths outside
  the increment's Files> ':!Cargo.lock'`, added lines, at most 200
  (README, "Pins"); past it, a question.
- **No shape of work in the engine or the wire** (code-organization.md,
  6): no arm, record, key or fixture is added for a chat, a goal or a
  list; `git -C <worktree> grep -n -w -e StartChat -e SetGoal -e
  Prioritise -e SeenChat -e ForgetChat -e ReadChat -e Disposal -- crates
  tests testing` prints nothing at 04.3's end.
- **Limits from the shipped profile:** each world and fixture takes its
  limits from `jig_protocol_config::profile(Profile::Small)` and
  `temper_service::limits::profile(Profile::Small)`; 04.4's process-tier
  case names its `documents.lines` value, 2, beside the profile's, with
  its source (`crates/temper/tests/support/configuration.rs:32`).

## Corrections after jig 16.1's block of 2026-10-17T23:27:43Z (ruling 294)

- **04.3, Files** gain `tests/one-process/src/referee/durable.rs`, a copy
  of jig's channel store observer that reads `skein-kv` commit frames
  from every file under `store/` (`:76`, `:94`, `:149`; the check at
  `:256`). It takes jig 16.1's repair (`jig/16-shed.md`, ruling 294): it
  reads frames only from commit logs (`store/log-<20 digits>`), a repair
  snapshot gives it none, and the commits the snapshot holds stay
  durable. Follow jig 16.1's diff of `tests/channel/src/world/store.rs`;
  refresh any further copy, and list it in the commit body.

## Corrections after temper 04.3's block of 2026-10-18T05:21:08Z (ruling 313)

- **04.3, Files** gain `crates/temper-engine-domain/src/limits.rs:100-105`:
  `VIEWS`'s `watchers` and `watchers_per_party` become 16, the client's
  own watch budget (`crates/temper-client-shell-terminal/src/limits.rs:29`),
  and its doc comment cites jig's small profile's 4 (configuration.md, 7).
  The root's `max_out` (`domain.rs:362-363`) and what is sized from it
  grow with them; the process tiers' overrides to 16 stay.

## Corrections after the designer's answer of 2026-10-18T05:52:56Z

The answer to temper-04.3's question of 2026-10-18T05:45:00Z (jig
544a57091, cd2cac35f; temper's landing of this section).

- **Ruling 313's `VIEWS` stays,** 16 a party and 16 in all: it is the
  design's now (`configuration.md`, 7, "The engine's watch room fits
  temper's client").
- **A stream's room is still 4.** jig's clients component refuses a
  watch busy once a stream holds `Limits::watches_per_stream`
  (`jig-protocol-engine-clients/src/watches.rs:168` on jig's main), 4 in
  both of jig's profiles (`profile/small.rs`, `CLIENTS`; `standard.rs`
  takes `..small::CLIENTS`), and no `clients.` override exists
  (`jig-protocol-config/src/overrides.rs:17-34`). temper takes it
  unchanged (`crates/temper-service/src/clients.rs:112`), so `temper
  serve` and the process tier refuse a client's fifth watch on its
  stream; configuration.md, 7 asks 16. temper's client world joins the
  client to the engine root without the component, and meets the views'
  room alone. No increment of this file sets it:
  `temper-04.s1` does, `temper/04-s1-repin-rooms.md` (ruling 319).
- **A watch or `me` refused busy** is opened again after a backoff in
  jig's design (`client/domain/client.md`, 8.1, 8.2); jig's client still
  ends it (`jig-client-domain/src/watches.rs:153`), and lets a watch's
  closing overtake its opening. The repair is jig's code, then temper's
  repin: jig-17.s2, `jig/17-s2-busy-watches.md`, then `temper-04.s1`
  (ruling 319). No temper code works around a busy `me`, and this
  file's increments do not change jig's client.

## Corrections after temper 04.4's block of 2026-10-18T05:51:17Z (ruling 317, interim)

- **04.4, Test shape:** `a_change_on_the_forges_deepest_branch_is_served_in_its_task_document`
  reads the producer's `task` document as a party of `change_world`'s
  project that holds no Amend on it (a role beside `policy.roles[0]`, as
  `forge_routes.rs:243-247` adds the maintainer and the member), within
  the file's 40 lines. The path is what the step proves; the owner's
  read, refused when its Amend authority passes `documents.lines`
  (jig `documents/offers.rs:59-62`), waits on the designer's answer to
  the question of 2026-10-18T05:58:00Z, and the test asserts nothing of it.

## Corrections after the designer's answer of 2026-10-18T06:20:06Z

The answer to temper-04.4's question of 2026-10-18T05:58:00Z (jig
35ab18732; temper's landing of this section).

- **An authority crosses whole** (jig `client/domain/contract.md`, 5.3):
  the engine never cuts an authority's lists at `documents.lines`, nor
  refuses a task document or its Amend offer for one. Ruling 317's
  reader stays. No temper code works around the refusal: the repair is
  a jig step, then a temper step, which the supervisor adds and names
  (the default: a jig supplement step after the newest landed jig step,
  and `temper-04.s1` after it). The jig step needs nothing unlanded.
- **The jig step** is jig-17.s1, `jig/17-s1-authority-whole.md`, and
  **the temper step** `temper-04.s1`, `temper/04-s1-repin-rooms.md`
  (ruling 319): their Shapes are those files'.

## Increments

### 04.1 `templates` in temper's configurations, in place of `chats`

Scope: jig's two templates in the configurations temper writes, in
place of the `chats` member, and jig's new core limit on templates
taken by temper's engine root (`configuration.md`, 2 and 8; jig 09.1).
Needs: jig 09.4 (jig 09's last increment: `templates` and the
policies' overrides) and temper 03.4 (its repin past jig 09.4), not
temper 03.5 or 03.6 (ruling 231).
Files (temper-03.3 carried `Limits.templates`, `core_limits` and the
service's `templates: jig.core.templates`, ruling 164, and `ENGINE`'s `templates`
at `fixtures.rs:46`, :68 and :441, ruling 169: 04.1 does what remains.
Ruling 188: temper 03.4's repin onto jig 13.2 is the first past jig
09.4, so it carries the repairs below that come "at the repin"; 04.1
makes no repin commit when temper's jig pin holds jig 09.4's landing
`3f2ae98ad` (README, Pins), and adds what 03.4 left of them): `Cargo.lock` (the repin); `crates/temper-service/src/tests/fixtures.rs:46`
(`ENGINE`'s `"chats": {"charter": 1, "close_grace_ms": 5000}` becomes
`"templates"`, jig's two defaults written out in jig 09.1's format, as
jig 09's fixtures write them: "jig's defaults are written, not
implied", their reports' `words` 32, within `ENGINE`'s
`"tasks.result_bytes":32`; its policy at :65 gains `"templates": []`,
and the `Project` literal at :436 `templates`);
`crates/temper-engine-domain/src/limits.rs:29-54` (`Limits` gains
`templates: u32`) and `:109-143`
(`core_limits`, the one core `Limits` literal of temper's production
code, gains `templates`, taken from the engine's limits as `documents`
is at :113, through `temper_service::limits::engine`, never a literal;
without it jig 09.1's core `worst_case` is `None`, `limits.rs:216`);
`crates/temper-service/src/limits.rs:264-278` (`templates:
jig.core.templates`);
the configurations made from jig's `engine_document()`
(`crates/temper/tests/support/configuration.rs:15`,
`tests/one-process/src/fixtures.rs:78`) take jig 09's fixture
unchanged; the process tier's `crates/temper/tests/support/forgejo.rs`
sets `tasks.delegates` 4 (:148) over jig's channel document, whose
`people.operations` 2 after jig 09.4 is below 4 + 1 (`What::Operations`,
refusing `process.rs:37`, `:74` and `live.rs:9`): its overrides
(:135-168) gain `("people.operations", 5)`. If no temper increment bumped `STORE_VERSION` before
(Choices), `crates/temper-protocol-store/` too: `src/lib.rs:15`
(`STORE_VERSION` 2), `schema/store-v1.schema` moved to
`store-v2.schema` with `CoreRefused`'s bound from jig's people v3
ceilings, its generated code and goldens (by `tests/codecs/tests/drift.rs`'s
write mode, README "Generated code"), the generated module `v1` moved to
`v2` as jig's people family moved to `people_v3.rs`, and their users:
`src/lib.rs:8-10`, `src/tests/sizes.rs:48` (`STORE_VERSION` 2),
`tests/codecs/tests/drift.rs:56-58` (the schema, `generated/v2.rs`,
`golden/v2`), the `v1::` users in `src/records.rs`,
`src/records/application.rs`, `src/tests/fixtures.rs`,
`src/tests/records.rs` and `tests/codecs/tests/fuzzy_store.rs`. The
core's `Settings` literal at `tests/engine/domain/src/walking.rs:256-263`
keeps `chat: 1` (:260) and `close_grace` (:257): jig 09 keeps both,
filled from template 1, until jig 16 removes them (04.3); it gains
`templates` (:256), and the world's `Config` (:245)
`template_overrides`, and its limits literal (:119) `templates:
jig_protocol_config::profile(Profile::Small).core.templates`, from
which the world's other test limits derive. Users found by `git grep -n -e '"chats"' -e
close_grace -e 'chat: 1' -e 'jig_domain_core::Limits {' -- '*.rs'` in
temper: the three above at the base. jig 09's new refusals, at the
repin (its 200 lines): `crates/temper-service/src/startup/problems.rs`'s
two matches over `config::What` (their `What::Grace` arms are in, :97
and :144) gain `What::Operations { .. }`, printing its numbers
(operations, delegates, goals) as `Reserve` does;
`crates/temper-forge-setup/src/problem.rs`'s match (:34-49) gains it
too; and `crates/temper-service/src/tests/startup.rs:117-140`'s
`jig_refusals_print_words` a row for `Grace` and one for `Operations`. Shared:
`crates/temper-engine-domain/src/store.rs` (README, "Shared files"),
if the repin's repairs reach it.
Shape: one field of temper's, `temper_engine_domain::Limits.templates`,
taken as `documents` is (in since 03.3); `temper-service`'s startup
hands the whole `engine` member to `config::engine`
(`crates/temper-service/src/startup.rs:60`), which reads `templates`:
temper adds no code for it (ruling 188).
Test shape: `crates/temper-service/src/tests/startup.rs`, on
`fixtures::engine()` (`crates/temper-service/src/tests/fixtures.rs:79`).
The chats test renames `ENGINE`'s first `"templates": [` alone
(`replacen(…, 1)`): the project policy's `"templates": []` (:68) matches
too (ruling 188).
The reference test, its field names jig 09's:
```rust
#[test]
fn the_example_engine_keeps_jigs_chat_and_goal_templates() {
    let jig = fixtures::engine().jig;
    let labels: Box<[u32]> = jig.templates.iter().map(|template| template.label).collect();
    assert_eq!(*labels, [1, 2], "jig's chat and goal templates, in order");
    assert_eq!(jig.templates[0].number, 1);
}
```
Tests, each under 20 ms:
`the_example_engine_keeps_jigs_chat_and_goal_templates`,
`a_chats_member_is_refused_as_unknown_naming_its_path` (the start
refused with `engine.chats: unknown member`, as jig 09's reader refuses
it). The engine world's tests keep their names and expected values.
Proof: the one-process world, once: `cargo nextest run -p
temper-one-process-world -E
'test(a_person_signs_in_by_the_printed_link_and_a_chat_is_answered)'`:
`temper serve`'s system, started from configuration bytes that name
jig's templates, answers the person's chat (about 1 s).
Don't: add a template of temper's; read `templates` in temper's
`application` reader; set a template's field by hand where jig's fixture
gives it.
Size: at most 40 lines (the repin's repairs counted apart, at most 200).
Removes: nothing (03.3 removed `chats` from `ENGINE`, ruling 169).

### 04.2 The forge's family on the collapsed wire

Scope: the forge's outcome "refused by the core" encoded with the
collapsed wire's refusal record, through jig's typed translation where
jig 14 gives one (`client.md`, 4.6; STATUS); the served versions per
family as jig 14 names them; the forge's form out of date on jig's typed
answer that a version is not served (`client.md`, 4.4); the one-process
referee reading the collapsed wire.
Needs: jig 22's last increment (jig 14's records and version refusal
and jig 22.1's `me` among it).
Files: `crates/temper-service/src/family.rs` (192 lines: `answer`, its
jig wire limits at :105), `crates/temper-service/src/family/refusals.rs`
(230 lines: goes, or follows jig 14's diff of
`crates/jig-protocol-engine-clients/src/translate/refusals.rs`, by the
choice above), `crates/temper-service/src/clients.rs` (the served list
at :78-91), `crates/temper-client-domain/src/forge.rs` (`Served`,
`Event::Served`); tests `crates/temper-service/src/tests/{family.rs,
clients.rs, clients/peer.rs}`, `tests/codecs/tests/client_wire.rs`,
`tests/one-process/src/referee/client.rs` (198 lines) and
`referee/client/cache.rs` (157), `tests/one-process/src/world.rs:353-354`
(the `Frame` delivery and read, jig 22.1's `me`),
`tests/client/src/translate/forge.rs`. Users of jig's client wire found
by `git grep -n -w jig_client_protocol_wire -- '*.rs'` in temper (at the
base, under its old name `jig_protocol_client_wire`: 11 files, 13 lines,
`logs/w9/users-jig-client-crates.txt`). This repin is the first to take
jig 14, and its changes reach, at temper `ad71c83c6`:
`crates/temper-service/src/family.rs:12-61` and
`crates/temper-service/src/tests/family.rs:36,103,118` (they match and
build `ToApplication`, which jig 14.1 gives a `scope`, and a `window` for
`Read` and `Watch`), `crates/temper-engine-domain/src/route/applications.rs:25`
(derives the application's scope itself),
`crates/temper-client-shell-terminal/src/route/family.rs:70,177` (builds
`jig_client::Family` and uses the wire's `Answer::FinalAnswer`, `KeyedAnswer`
after jig 14.1) and `crates/temper-service/src/family/refusals.rs` (its
arms for jig 14.4's and 14.5's refusals; its `proposable` arm, :25,
stays) (ruling 222). Ruling 261 adds, at temper `9bd979292`:
`crates/temper-client-domain/src/forge/family.rs:115` (`RefusedParts`
gains `at`, jig 14.3) and :123, `crates/temper-client-shell-terminal/src/client.rs:472`
and `src/tests/fixtures.rs:666` (`Answer::FinalAnswer`, `KeyedAnswer`
since jig 14.1).
Shape:
- `temper_service::family::answer(outcome:
  Option<forge::ApplicationOutcome>, limits: &w::Limits, jig:
  &jig_client_protocol_wire::<jig's current version>::Limits) ->
  Option<Box<[u8]>>`, unchanged in form; its `CoreRefused` bytes are the
  collapsed wire's `Refusal` record, the facts jig 14 adds among them
  (no one to escalate to, not well formed); a
  compound request's failing position is the answer's, not the
  refusal's, and an adoption is no compound request.
- The forge side's version: `forge::Event::Served { versions }` also
  from `jig-client`'s typed answer that the forge's version is not
  served, under the adoption's key; `Served::OutOfDate` disables Adopt,
  and the pending adoption stays kept (`client.md`, 4.4).
Test shape: `crates/temper-service/src/tests/family.rs` (its fixtures at
the top of the file) and `crates/temper-client-domain/src/tests/form.rs`
(on `crates/temper-client-domain/src/tests/fixtures.rs`). The reference
test:
```rust
#[test]
fn a_core_refusal_is_carried_as_jigs_refusal_record() {
    let outcome = forge::ApplicationOutcome::CoreRefused(jig_domain_people::Refusal::NotOffered);
    let bytes = family::answer(Some(outcome), &fixtures::wire_limits(), &fixtures::jig_wire_limits()).unwrap();
    let answer = w::AdoptAnswer::decode(&fixtures::wire_limits(), &mut Reader::new(&bytes)).unwrap();
    let w::Outcome::CoreRefused(refused) = answer.outcome() else { panic!("a core refusal") };
    assert_eq!(refused.refusal(), fixtures::NOT_OFFERED_RECORD, "jig's record for NotOffered, as a literal");
}
```
Tests, each under 20 ms:
`a_core_refusal_is_carried_as_jigs_refusal_record`,
`a_refusal_naming_no_one_to_escalate_to_is_carried_with_its_facts`,
`an_authority_refusal_keeps_whom_a_proposal_could_go_to`,
`a_version_not_served_answer_marks_the_form_out_of_date`,
`an_out_of_date_form_keeps_its_pending_adoption`. The base's family and
form tests keep their names and expected values.
Proof: the one-process world, once: `cargo nextest run -p
temper-one-process-world -E
'test(an_adopted_repository_gets_its_change_pushed_through_git)'`: a
person's adoption crosses the collapsed wire as the forge's bytes, the
outcome comes back as "Adopted acme/app on forge 1 as owned.", and the
referee decodes both sides (about 3 s).
Don't: decode a refusal of jig's in temper beyond what the wire crate's
decoder gives; keep a `v7` path; add a refusal of temper's own.
Size: at most 120 lines.
Removes: `refusals.rs` where jig 14 gives the typed translation.

### 04.3 The engine's shapes of work gone, with jig's

Scope: the engine root's arms, reads and restart keys for chats, goals
and lists, and the store's `Disposal` refusal, gone with jig 16's; the
tests' chats made from the chat template (`STATUS.md`, "temper on jig's
client API"; code-organization.md, 6).
Needs: jig 16's last increment, and jig-16.s1 (ruling 305): its pin is
jig-16.s1's landing.
Files (production, found by `git grep -n -w -e StartChat -e SetGoal -e
Prioritise -e SeenChat -e ForgetChat -e ReadChat -e SeenChatLoaded -e
Disposal -e DisposalIssue -- '*.rs'` in temper: 79 lines in 30 files at
the base, `logs/w9/users-shape-asks.txt`):
- `crates/temper-engine-domain/src/store.rs:528-584` (the sizes of
  `SetGoal` :541, `Prioritise` :545, `StartChat` :550, `Close`'s
  disposals :556-557, `Stored::ReadChat` :584);
- `crates/temper-engine-domain/src/archive.rs:200-220` (`EndedRead::SeenChat`,
  `EndedRead::Chat`), `boundary.rs:578-584` (those variants),
  `route/now.rs:141, 236` (`core::Now::SeenChat`), `restart.rs:420-432`
  (the conversation and goal keys' arms);
- `crates/temper-service/src/clients/translation.rs:66-67`
  (`ForgetChat`, `SeenChatLoaded`);
- the core's `Settings.chat` and `Settings.close_grace`, which jig 16
  removes: `tests/engine/domain/src/walking.rs:257, 260` (test code);
- `crates/temper-protocol-store/src/records/application/refusal.rs:5-17,
  292-296, 328-330` (`Disposal`), and the store schema's `CoreRefused`
  bound (store-v2, edited in place);
- `crates/temper-service/src/family/refusals.rs:7-14`, if 04.2 kept it.
Tests (not counted): `crates/temper-protocol-store/src/tests/records.rs`,
`crates/temper-service/src/tests/{clients.rs, fixtures.rs}`,
`tests/client/src/{fixtures.rs, translate.rs}`,
`tests/conformance/src/{lib.rs, scenarios.rs, stories.rs}`,
`tests/engine/domain/src/{direct.rs, escalation.rs, escalation_referee.rs,
roles.rs, roles_referee.rs, walking.rs, walking_referee.rs}`,
`tests/engine/domain/tests/{document_limits.rs, forge_routes.rs,
fuzzy_endings.rs, root_routes.rs}`, `tests/system/src/{composed.rs,
forge/stories.rs, goals.rs, world.rs}`. Shared:
`crates/temper-engine-domain/src/store.rs` (README, "Shared files").
The scripts on jig's reference pages, where jig 16.1 to 16.3 change
what the person finds there (the conversations section and the goals
board on task queries, the chat and goal acts through `shapes`):
`tests/one-process/src/stories.rs:15-46`, `src/world.rs:209-222`,
`crates/temper/tests/support/person.rs:16-27, 231-262`,
`crates/temper/tests/process.rs:34-63`, `tests/client/tests/stories.rs`,
their expected names followed as jig 16 follows its own worlds'.
`root_routes.rs` and `forge_routes.rs` are over 800 lines: at most 40
added lines each (code-organization.md, 7).
Shape: no new type. Each removed arm is replaced by nothing, but for a
conversation's read position, which stays under jig-16.s1's names
(ruling 305): `Key`/`Stored::ReadConversation`, `core::Now::SeenConversation`
and `Event::SeenConversationLoaded`, and temper's own `EndedRead::SeenChat`
becomes `EndedRead::SeenConversation`. Their users at temper `90a3f7048`
(`git grep -n -w -E 'ReadChat|SeenChat|SeenChatLoaded' -- '*.rs'`, 11
lines in 9 files): `crates/temper-engine-domain/src/{archive.rs:232,
boundary.rs:586, route/now.rs:141, 241, store.rs:622}`,
`crates/temper-service/src/clients/translation.rs:70`,
`tests/client/src/translate.rs:78`,
`tests/conformance/src/scenarios/conversations.rs:156`,
`tests/engine/domain/src/{roles.rs:221, roles_referee.rs:102, 146}`.
Each test that sent `people::Ask::StartChat { project, words }` sends the chat
template filled into jig's `Create`, by jig 16's helper or by
`walking::chat(project, words)` (Choices).
Test shape: the tests named above keep their names and expected values;
none is added beyond `walking::chat`'s own, in
`tests/engine/domain/tests/root_routes.rs`:
```rust
#[test]
fn a_chat_made_from_the_chat_template_is_a_conversation_task() {
    let mut driver = Driver::configured(Store::new(), walking::config(91), &walking::limits());
    driver.hello(Token::new(7));
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver.send(walking::ask(1, walking::chat(1, b"hello")));
    driver.settle();
    let task = driver.task(1).expect("the chat's task");
    assert_eq!((task.label, task.conversation), (1, true), "the chat template's label and flag");
}
```
(under 20 ms; `walking::ask` and `driver.task` as the file's other tests
use them, or their nearest names there).
Proof: the one-process world, once, as 04.1's command: the person's
chat, made from the chat template by the client, is answered by the run
through bytes (about 1 s).
Don't: rename a removed variant to something of temper's; keep a test
of a removed mechanism (its behaviour moves to jig's tests, or the test
goes, README, Rules, "a removed mechanism's tests").
Size: at most 60 lines (mostly removals; `walking::chat` is test code).
Removes: the arms and variants named in Files; `Disposal` in temper's
store codec.

### 04.4 The forge's deepest path in a task line, and in the client

Scope: a task naming the forge's deepest branch path is listed, not
refused as too large, in its project's task query (the task line) and
its `task` document under jig's small profile and under the process
tier's override, whatever `documents.lines` (jig's
`client/domain/contract.md`, 5.3: "A document carries each whole, with
as many segments as the engine admits in a name"); and temper's client
takes a path's segments apart from its rows (`client.md`, 5.4;
`configuration.md`, 7; the design answer to the question of
2026-10-16T14:55:38Z).
Needs: 04.3, and jig-13.s1's landing (jig's step for a path's segments,
`jig/13-s1-path-segments.md`; README, "Pins": this increment's repin
takes it, and "A path's segments").
Files: `tests/engine/domain/tests/forge_routes.rs` (over 800 lines: at
most 40 added), on its `change_world` (`:633`);
`crates/temper-client-shell-terminal/src/limits.rs` (`segments: 9`
beside `rows: 2`, `:51`: at least the forge's tier's names' bounds, 9,
`crates/temper/tests/support/forgejo.rs:152-156`, and the example's 8,
`configuration.md`, 7); `crates/temper/tests/configuration.rs:17-21`;
`tests/one-process/tests/one_process.rs:43` (ruling 163's `#[ignore]`
removed: the story runs in the gate again); `tests/one-process/tests/fuzzy_one_process.rs:39`
(ruling 216's `#[ignore]` on `a_merge_in_progress_is_pushed_with_both_parents`
removed: it runs in the gate's fuzzy profile again).
Shape: no type. temper writes no `documents.lines` for a path: the
process tier's `documents.lines`, 2 (`support/configuration.rs:32`,
`support/forgejo.rs:137`), stays, at most the client's rows. In
`tests/configuration.rs`, `:18`'s assertion says "every document's short
list" in place of "every document path", and one more holds the most of
the engine's `authority.segments`, `tasks.hold_segments` and
`tasks.authority_segments` at most `native.segments`, "the native client
can receive every document path".
Test shape: in `tests/engine/domain/tests/forge_routes.rs`, on
`change_world(silent_ci, passes, approval)` (`:633`), whose producer
works on a change branch of temper's namespace (the forge's deepest
path: five segments, then the branch's), and one new helper in the file,
`task_line(world, project, task) -> Option<Line>`: the party's read of
the project's task query (jig 22.3's `Read` for task queries), the row of
`task`, or none when the read is refused. The documents' limits come
from `jig_protocol_config::profile(Profile::Small).core.documents`, as
`tests/engine/domain/tests/document_limits.rs:10-14` takes them, and the
process tier's `lines: 2` beside them, named with its source. The
reference test, a table test of one behaviour, its rows in a `const`:
```rust
const LINES: &[(&str, u32)] = &[("jig's small profile", 4), ("the process tier's override", 2)];
#[test]
fn a_change_on_the_forges_deepest_branch_is_listed_in_a_task_query() {
    for (source, lines) in LINES {
        let (mut world, _chat, producer, _branch) = change_world_with_lines(*lines);
        let row = task_line(&mut world, 1, producer.task);
        assert_eq!(row.map(|line| line.task), Some(producer.task), "{source}: listed, not refused as too large");
    }
}
```
(`change_world_with_lines` is `change_world(true, 1000, None)` with the
world's `documents.lines` set, new beside it.) Tests, each row under
0.5 s (world): the one above, and
`a_change_on_the_forges_deepest_branch_is_served_in_its_task_document`, the
producer's `task` document read the same way.
Proof: the world layer only: no page of temper's reads a task line
before temper 06.4's Chats page, whose joint-world story then runs over
the same configuration. And, once, the story ruling 163 deferred: `cargo
nextest run -p temper-one-process-world -E
'test(an_adopted_repository_gets_its_change_pushed_through_git)'`.
And, once, the merge story ruling 216 deferred: `cargo nextest run
-p temper-one-process-world --profile fuzzy -E
'test(a_merge_in_progress_is_pushed_with_both_parents)'`. At jig
`40d1bec49` with `segments` 8, a scratch run of it passed step 11 and
failed at step 18 (`world.rs:442`: `person step 18: Missing { find: Text
"@plan Fix the adopted repository" }`, the task's document never read),
cause not found (ruling 216): if it still fails, diagnose it at most 30
minutes, then block with the first failing line and its file:line.
Don't: set `documents.lines` for a path; change jig's profile; cut a
path in temper.
Size: at most 10 lines of production code.
Removes: nothing in code.

## Interface sketches

None: this session adds no type. The shapes it follows are jig 09's,
14's and 16's.

## Questions to raise when reached

- If jig 14 gives a typed translation of refusals that drops a fact
  temper's `CoreRefused` shows today, raise it with the fact named.

## Done when

- 04.1 to 04.4 have landed on temper's `origin/main`.
- `git -C <worktree> grep -n -w -e StartChat -e SetGoal -e Prioritise -e
  SeenChat -e ForgetChat -e ReadChat -e Disposal -- crates tests testing`
  prints nothing.
- `git -C <worktree> grep -n -e '"chats"' -e close_grace -- crates tests`
  prints nothing.
