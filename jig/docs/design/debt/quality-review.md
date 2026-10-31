# jig's debt after the code-quality review

2026-10-31, at `jigq/quality` `1b5c98834`: fabrig `main` `7cca2c152` with the
review's Phases 0 to 3 carried out, and `main` `4a6683c72` (temper's
playground reset) merged in. What the review left out of scope, what the
run did not finish, and what the run and two runs beside it found on the
way. Findings and recommendations, not decisions; each says what would pay
it back.

Sources:
- **The review:** "jig code-quality review, 2026-10-31", on `main`
  `c30cf6a03`. It is not in the tree (`~/src/fabrig/notes/` on hel8).
  Cited here by its own labels: section (§7), order-of-work item
  ("item 20"), theme (T3), area finding (A6 F2).
- **The run's records:** its plan, status survey, log and report, and the
  108 Codex task reports (`jq-*`). The jig findings of the playground-reset
  run (`pg-*`) and of the repair of `main` after the day's landings
  (`mr-*`) are included too. They are in `~/src/fabrig/coord/` on hel8.
- [`docs/development/tech-debt.md`](../../development/tech-debt.md): the
  earlier pass. Its sections 2, 3, 7, 10, 11 and 12 are stale after this
  run (section 8 here).

Counts are `rg` over `jig/**/*.rs` without `generated/` unless a line
says otherwise. They may differ from the review's method. "Verified" means
checked on `1b5c98834`; "unsure" means a report or survey left it open.

## 1. In one page

- **The run removed 37,351 hand-written lines of jig (−5.7%)** and
  1,177,906 generated lines (the 19 codec versions nothing compiled). The
  review estimated about 100k hand-written. Most of the difference is the
  generator (item 20, out of scope) and the hand-written contract mirrors
  (T3).
- **Fix first: design and code disagree in five places** (section 2):
  - temper's charter rail;
  - project deletion, which has no design;
  - three stale design texts;
  - new `Core` methods with no design text.
- **The biggest unfinished items:**
  - bug 2a (close grace with two templates);
  - the core's flight work (item 9);
  - ops still deciding (item 11);
  - `Lossy<T>` coverage (item 12);
  - 14 silent wire clamps (item 13);
  - the stories on three fronts (item 17);
  - kits that are still worlds (item 19).
- **Out of scope:**
  - item 20 and the process note's guards;
  - every theme and area finding that items 1 to 19 do not name (sections 4 and 5), including T4's decisions the store and engine-clients still make, and T7's grinding through cases.
- **New during the run** (section 6): hazards in deletion and restore,
  fixture constants that nothing checks, front differences the stories
  surfaced, and gaps in skein's primitives.

## 2. Design and code disagree

1. **The charter's deliver rail.** temper's design says a run that cannot
   deliver cannot write
   ([`temper/docs/design/domain/agent.md`](../../../../temper/docs/design/domain/agent.md),
   near its charter rules). temper's own code had `modify = policy.modify
   && deliver.is_some()`. After the charter policy moved into jig (item 11),
   `jig-protocol-charter`'s `assignment` grants `policy.modify`, and the
   stock chat, goal and advisor charters have `modify: true`: only the
   reviewer's is false. Verified. Pays back: the deliver rail in
   `assignment`, or the stock policies and `agent.md` changed together.
   (`jq-charter-assignment`)
2. **Project deletion has no design in jig.** The playground reset added
   these, and none of them is in jig's design:
   - `RemoveProject`, `ReleaseProject` and their routing;
   - deletion rows, whole-task erasure, people store v14 (a v13 store is refused, not migrated);
   - the `remove_project` grant word.

   Code cites `domain/people.md` 5.2, which does not cover removal, and
   `jig-conformance`'s referee cites temper's draft. Pays back: a deletion
   section in `domain/people.md` (or its own document) and `protocol/store.md`'s
   v14, with their status files. (`pg-integrate-jig-*`; playground plan
   §5)
3. **Stale design texts:**
   - [`protocol/llm.md`](../protocol/llm.md) 5.6 still describes the
     moved-record reader (JIGM/SKOT) the run deleted. Verified.
     (`jq-accounts-component`)
   - `temper/docs/design/protocol/configuration.md` (around line 539)
     still names the removed `hello_bounds` conversion. Verified.
     (`jq-link-values`)
   - `docs/development/tech-debt.md` §2 says `joint/wire/` remains for
     temper's client world; the run deleted it. Verified.
4. **New public `Core` methods have no design text:**
   - `effect_owner` (flight-b);
   - `call_room` (hub-link-temper);
   - `application_part_request` (green-2);
   - the test-only `next_finishing_deadline` and `finishing_due` (core-timers).

   Pays back: their lines in [`domain/engine.md`](../domain/engine.md), or
   the test-only ones behind the test service. Also unsure: `engine.md` 4.3
   may need words now that completion is an empty request batch
   (`jq-core-decided-2`).
5. **The root fires one due core child per pass** (core-timers):
   `World::fire_due` and temper's `settle` had to loop. Other callers that
   expect one call to fire everything were not audited. A core contract
   change with no design line.

## 3. Left from the review's order of work (items 1 to 19)

- **Bug 2a, ops' close grace with two conversation templates:** not done.
  - Why: a task carries no template identity. The client's template fill drops it, and so do the client wire, `NewTask`, `people::Member` and `tasks::New`.
  - Pays back: `template: Option<u32>` carried through the client schema, `people::Member` (people store), `tasks::New` and `TaskRecord`; the core applies that template's grace. `engine.md` (its "a task never retains which template made it") and the `templates.rs` doc change with it.
  - The Call 18 assumption "ops passes none" was wrong: a `None` grace does not take a default, it goes straight to `cancel_selected`.
  - Design. (`jq-close-grace`, `jq-ops-bugs`)
- **Bugs 3, 7, 9 and 10a, settled short of the review's fix:**
  - 3: `Core::valid` requires the two limit sources to be equal; the duplicated fields stay.
  - 7: restore still accepts odd stored source names.
  - 9: an unknown profile still falls back to Small.
  - 10a: three "Cannot show this part" lines stay in the client goldens, with the referee exempting them.
  - Unsure: bug 6 may have a twin in temper's `tokens.rs`.
- **Item 1:** codegen does not emit `VERSION` (byte-identical generated files would change). The smith schema copy is checked by bounds, not by fingerprint.
- **Item 4:** "historical" is 0 in jig and banned there. It stays in temper's engine crates and worlds (about 48 lines) and in smith's `Decoded::Historical`. The status files' 14 "unknown" clauses were deleted, not replaced.
- **Item 9, the core:**
  - `Work::Event` still wraps public entrances the core uses as its own continuations, and roots still send nine public hand-offs (`Activate` … `ClaimPrepared`; Call 8).
  - The task-number-keyed `claiming: Map<u64,u64>` and its fallbacks survive (verified).
  - Flights are found by scanning the slab, not by one lookup.
  - About 372 `unreachable!` outside tests, against about 395 at the base.
  - `quiescent` still checks state outside the slab.
  - 24 production `too_many_arguments` remain (51 → 24). They pay back with typed run, host and effect contexts like `Party`.
  - World tests still write `Core` internals.
- **Item 10:** fleet, host and accounts re-export the link values, so two import paths stay. The program-name grammar is still copied in people and the client domain's contract.
- **Item 11, roots that decide:**
  - ops' domain went from 4,011 to 3,812 non-test lines; the review aimed at about 2k and a template.
  - ops keeps `Class` in `admission.rs`: no link or core classifier for `host::Event`/`FromAgent` exists.
  - ops still decodes delegate JSON and assembles alert words.
  - ops' `route.rs` is 1,539 lines.
  - temper keeps its own link pieces: workstream override, disconnected-channel outputs, forge sections, assignment parts. Its accounts adapter still differs from jig's test-service copy (trust roots, the TLS bound, `Corrupt` against `PageRefused`, an output-room check jig has no position on).
  - `jig-test-store-files`' memory-map mode bypasses the shared store host.
- **Item 12, skein mechanisms:**
  - `Lossy<T>` covers views, fleet facts and host activity only. It lacks counted eviction, borrowed iteration, a capacity accessor and a resettable loss counter, so these stay hand-written:
    - fleet's hello history;
    - host `Refusals` and its held queue;
    - the piece counters;
    - worker-link and engine-workers counters;
    - people, tasks and accounts `facts`/`lost`.
  - 212 `usize::try_from` remain beside 641 `usize_of`, and a few `from_utf8` calls stay.
  - skein-json has no array-element cursor: jig derives one, and two private borrowed `Node` views exist (tools and ops' `json.rs`).
  - Design (skein's API).
- **Item 13, one `Limits` per concept:**
  - The client keeps 14 effective `.min(CEILINGS)` clamps, so over-ceiling configurations are capped silently, not refused (row bounds 16, model names 4096, program and effort text 32, `Create.tasks`, charter instructions, settings pieces).
  - The store's `codec::Limits` survives as a derived `codec::View` (unsure whether that meets the item).
  - Design.
- **Item 14:**
  - `Core::new` (about 200 lines) and `Config.settings: u64` are untouched.
  - `numbers.rs` and `stored.rs` stay at core's top.
  - The view's `notices` keeps state and rendering together.
- **Items 15 and 17, tests on the shipping code:**
  - The client-protocol world reuses only parts of the joint loop.
  - Six role stories are `#[ignore]`d on the Protocol front, because its fixture authenticates only party 1 (verified: jig's only six). That needs multi-identity protocol authentication.
  - `Seed` supports one project, owner party 1, `ChatProject` and `Conversation`, so the stories can't run on the ops or temper fronts.
  - The old `stories*` fixtures stay. The stories added 777 lines net, against the review's estimate of about −3k.
- **Item 19, kits:**
  - The `-world` rule ships with 57 pending package → `-world` pairs (verified), 26 of them `-world` → `-world`.
  - Seven packages still depend on `jig-core-world`; its world-bound helpers stay in `fixtures`.
  - The core world's "verbs in the kit" was never planned (A1 F10: `ask` defined 7 times, 249 `Record::Core` patterns).
  - The bundled scenarios stay bundled (Call 15, the world's CPU budget), but each reports every failing case.
  - View test source fell 5.9%, against the item's one third, with 1,627 rendered goldens added.
- **One budget moved:** the ops environment-chat story's heap ceiling, 1,400,000 → 1,900,000 bytes, by `skein/docs/foundation/testing-strategy.md` 6. Which allocation grew is unmeasured: the candidate is `Engine::commit` cloning truth and the application into pending. Pays back: the measurement, and a meter scoped to the client and view (the meter counts people and referee work too).

## 4. Out of the run's scope: item 20, the guards, and the themes

- **Item 20, the generator:** skein-codegen mirror declarations emitting the domain-side types, both translations, typed piece helpers and the limits mapping. The review's estimate: most of T3, 30 to 45k lines. Includes A6 F1 (the store codec about two thirds structural), A7 F3, A7 F14 (each limit named 4 to 5 times), A8 F2 and A12 F9 (ops' `values_1..4.rs`, 3,550 lines). A skein design job.
- **The process note's guards:** the gate's trend report for the tracked measures, and a consolidation pass before the next feature slice. Unsure: a boundary rule against uncompiled generated files was not found.
- **T3 in the short term** (A0 F5, A7 F4-F5, A8 F2, F10): a codec piece reader, `to_bytes` per record, one `take` with `let`-`else`, and files split by record family. Verified: 29 numbered `*_partN` files remain.
- **T4, decisions in the wrong layer:**
  - engine-clients answers people's `ProjectMismatch` and decodes compound asks three times (A7 F2).
  - The store codec remaps three models and copies a domain literal (A6 F2, about 300 lines).
  - `Write::CopySettingsRevision` should be a generic `Write::Copy` (A6 F5).
  - The client orders and merges the inbox for the view (A9 F6).
- **T6, unit tests that duplicate worlds:** A2 F10, A3 F14 (about 80 public-API task unit tests with copied limits), A5 F4.
- **T7, grinding through cases:**
  - classifiers belong on the owning type: `tasks::Stored` listed 16 times (A1 F15), `MessageKind` about 25 times (A3 F8), people's 1,061 or-pattern lines (A4 F5, sub-enums for `Ask`/`Outcome`), about 28 thirty-arm `ReadValue` matches (A10 F9);
  - payloads re-narrowed with `unreachable!`: notes `ReadPending` (A5 F10, verified 10), engine-clients' `Waiting` (A7 F6);
  - event payloads as named structs beyond core: tasks (A3 F6), views, host, fleet (A5 F18). jig has 70 `too_many_arguments`.
- **T8, organisation:**
  - core's `documents/` read model in a cycle with `acts` and `reads` (A2 F4; unsure whether the re-tree broke it);
  - engine-clients' 48 `#[path]` and `v19 as w` in 70 files (A7 F7, verified);
  - the `testing/` and `tests/` split.
- **T9, naming:**
  - "native" with three meanings; "admission" with four (A10 F12, A3 F11);
  - "task hub" (A0 F7);
  - `account` meaning a source (A4 F10);
  - `Hold::Effects` (A3 F11);
  - 13 public type names defined twice in core (A2 F6);
  - the three `Made` structs (A1 F18);
  - the store's `owner` that is an asker (A6 F17);
  - the view's `Field`, `Place` and `State` doubles (A9 F8);
  - crate names: `jig-client-views-world`, `jig-test-system`, `jig-test-*`'s application/harness mix (A0 F7, A5 F7);
  - emphasis words in code: "actual" (507 hits) and "whole" (945).
- **T10, the step-code subset:**
  - `impl Deref for CompoundTask` (A3 F7), the view's `trait Segment`, generics and `Deref` impls (A9 F7; 17 `Deref for` in jig);
  - the store's `macro_rules! older!` (A6 F15);
  - core's 79 `if let` and 35 `let … else` on jig enums (A1 F12).
- **T10, lint volume:**
  - `#[expect]` 795 → 504;
  - `too_many_lines` 166;
  - the `[lints]` block copied into 40 manifests;
  - `disallowed_* = "allow"` in 8, which switches off the replay bans;
  - 23 `clippy.toml` copies (A11 F12: one `testing/clippy.toml`).

## 5. Area findings no item named (A0 to A12, condensed)

Each pays back as the review's appendix says. Grouped by area.

- **Core (A1, A2):**
  - proposals inside `effects.rs` `complete` (A1 F13);
  - `completion_retry` lists 83 variants to find one (A2 F11);
  - one concept as several types: `settings::Edit`/`tasks::SettingEdit`/`reads::RevisionEdit` (A2 F5);
  - 13 of 20 `documents/` `pub` functions unused outside the crate (A2 F4);
  - `MAX_ACTS = 23` against 24 `Act` variants (A2 F16);
  - wide return types (A2 F17);
  - one fixture builder (A2 F9);
  - `tests.rs` unsplit (A2 F8).
- **Tasks (A3):**
  - near-duplicate boundary families (F2; `boundary.rs` 2,172 lines);
  - the operation vocabulary still written four ways (F3);
  - two proposal families (F4, stored format);
  - the 76-field `TaskRecord` with 78-arm `Phase::Ended` (F5);
  - shape checks written three times (F9);
  - restore built by hand 84 times (F16).
- **People, authority, accounts (A4):**
  - people carries accounts' vocabulary (F6, stored format);
  - authority's stored policy lives in people (F7);
  - `tools: u64`/`notes: u8` against authority's newtypes (F8);
  - authority's step takes no `Env` (F9);
  - accounts spells one guard three ways (F14);
  - authority's source grind (F16);
  - four test layouts, and accounts has no unit tests or referee (F17);
  - `send_inner` at 554 lines (F18);
  - authority's readiness ANDs, a design question (F19).
  - Unsure: F11 (stale docs) and F12 (dead items).
- **Small domains (A5):**
  - worlds in three shapes (F3): brief and notes lack seed, harness or heap checks;
  - brief's `Gather*` aliases and parallel lists (F8, F9);
  - views' read API piecemeal (F13);
  - fleet docs ignore program sources (F14, unsure);
  - the `JIG_WORLD_TRACE` gate copied in 6 files (F16; pays back with a skein-world `Trace`).
- **Store (A6):**
  - the kits under different lint regimes, and `jig-fake-store` earning no crate (F7);
  - `ended.rs` maps 40 `core::Now` variants to a key (F8, unsure);
  - problems dropped with `.ok()`, every overflow reported as `Path::DefaultValue` (F9);
  - 233 `pub fn` in the codec, with config naming `people_v13::*` (F10);
  - `settings.rs` reads like generator output (F11);
  - `keys.rs` child-tag literals (F15);
  - `Routes::new` ignores its size (F16);
  - `tests/phase.rs` at 1,920 lines with four disk harnesses (F18).
- **Engine protocol crates (A7):**
  - `translate/cursor.rs` is a hand-rolled format with no schema, golden or drift test (F8, 647 lines);
  - `channel_capacity` copied in engine-workers and worker-link (F10; needs a skein-channel validity API);
  - connection state as flags (F11);
  - config owns other crates' derivations (F12);
  - the engine as a half-built value (F15);
  - transcript `classify` provokes failures (F16);
  - three parallel route enums (F17);
  - hand-rolled `catch_unwind` tables (F19);
  - near-copied worker `net.rs` worlds (F20);
  - empty wire fields; transcript goldens unchecked against smith's.
- **Client stack (A8):**
  - the undocumented unlinked mode shares the linked enums, so the native shell panics (F6, verified);
  - sign-in split across layers, three identical jitter streams (F7);
  - a durability barrier across three crates (F8);
  - `up()`'s double match (F11);
  - test support in production: 49 `wire_`/`encode_` helpers in link `translate/settings*.rs` (F12, verified);
  - the native shell hard-codes `/jig` twice, and the web shell links the whole view (F13);
  - wire-world goldens re-check a generic property (F16);
  - the protocol world runs no client-world story (F17).
- **Client view (A9):**
  - 21 phrase vocabularies listed four or five times (F3; tables would save about 6k lines);
  - negative asserts that pass for any renderer (F11);
  - 106 hand literals in size companions (F15);
  - `Page::is_work()`/`is_shown()` (F16);
  - `pages::owed` missing; the overview opens an inbox watch with no read.
- **Client kits and tests (A10):**
  - `jig-client-test-kit` is a check plus an unused re-export (F6, verified);
  - the `special(page)` string dispatch and goldens compared twice (F11);
  - 18 literal protocol versions (F15, unsure).
- **Engine kits, test application (A11, A0 F14):**
  - `jig-test-domain/src/lib.rs` is 1,282 lines against ops' 40;
  - the `Application` trait has 22 methods and 10 associated types (F1);
  - two worker entrances into the test root (F5);
  - the test service joins engine and worker services (F6);
  - harness internals (F8);
  - README's "eight shared scenarios" against 13 (F9);
  - client-wire tests in the conformance world (F10);
  - `NativeClient` in 5 files (F11);
  - three dependency-check styles (F14);
  - 44 bare hex wire kinds in `tests/channel` (F16);
  - the one-process cut still placed by a count, now 195 (F17, verified).
- **Ops (A12):**
  - one concept as many types (F5);
  - `None => {}` against `unreachable!` in `client_documents.rs` (F6);
  - synchronous intercepts list 38 `core::Held` variants (F10);
  - `decimal` duplicating skein-lib (F13, unsure);
  - the ops world can't be copied: it runs the test service and re-joins core reads (F14);
  - stories in bare numbers (F15);
  - six root modules not in `root.md` 12 (F17);
  - client-view tests `#[path]` into the client domain (F20);
  - `alert_cuts` replays its baseline.
- **Tech-debt cross-checks still open:**
  - the core observer rebuilds its snapshot per commit;
  - coverage found by text search;
  - fleet's three endings collapse into one refusal.

## 6. Found during the run

- **Core:**
  - **A deleted project's live watches are not ended:** `close_views` is not wired into deletion; only a read test calls it (verified). Pays back: call it from deletion's start once the project leaves `Core::projects`, and drain its requests. (`pg-jig-closed-2`)
  - **Every restore rule is written twice,** in both LoadCore starts. One missed case showed only in fuzzy (seed 71). (`pg-jig-delete`, `pg-mm-fix-jig-conformance`)
  - **Project 0 doubles as the deployment sentinel.** The task-line converter skips its closure check to avoid a panic in the restart census. Hazard. (`pg-jig-closed-2`)
  - **People request tokens and core flight handles can share a value,** so each boundary must translate back. green-2's adoption bug came from this. Hazard.
  - **Procedure holdings and workspace hand-offs are found by scanning the slab.** `reserve_connector_subscription` gives one subscription two names. `Now::StartPreparation`'s `from` may be dead (unsure). `Work` carries a new `large_enum_variant` expect.
  - **Authority's model mapping is asymmetric with people's:** an omitted model becomes NONE one way and `Some(NONE)` the other.
  - **A deleted test leaves a behaviour question:** with one charter slot, accepting proposed settings returns `ProposalDecided{Accepted}`, not `Full`. Either a capacity check at acceptance is missing or the expectation was wrong; the `Moves` case never ran. Needs a decision. (`jq-core-world-driver`)
  - **Whole-task erasure runs one walk at a time** and reserves two extra read-slab slots. Limit by design.
  - **Notes keep a deleted project's entries; only index lines go,** and notes' restore ignores notes records. Retention gap, design.
  - **Left out of deletion:** deployment owners deleting any project, a Delete control in jig's view, and horizon retention of a forgotten project's activity.
  - **The playground run's departures** (`ReleaseProject` names no resource, cancellation without grace, pools closed with spend kept, a sole-use account keeps its use) await the user's confirmation. Design.
- **Tasks:**
  - a period can still open for a closed project: only core's gate stops it;
  - a project-scope placed hold can't be made through the placement entrance, so its erasure is tested by a unit fixture only;
  - people's and tasks' `room_refusals`, `Tries::lost` and the `facts`/`lost` pairs are unconverted queues (see item 12).
- **People and accounts:**
  - every new people key tag must be added to the restore interval tables by hand (tag 9 was missed, then fixed twice in parallel);
  - jig has no per-project model allowance, so the playground's "scripted model only" was dropped (design);
  - the moved-file envelope bound was left conservative;
  - the key world keeps its own store and issuer peers;
  - `jig-domain-accounts` has no unit tests now (its world covers it).
- **Store:**
  - each root repeats notes' `Range::Project` validation;
  - `EndedRead` carries no heap bytes, so byte accounting is split (unsure whether it matters);
  - codec functions that only forward limits remain;
  - temper's pre-settings refusal test builds a removed format (`component_v2`) by hand, and a boundary fixture names `component_v3`: fragile;
  - temper's store staging allowance went from 6 to 12 with the shared host (unsure whether intended).
- **Engine protocol crates:**
  - the link holds effects two ways (opaque, and decoded in jig-test-domain), and one `Link::fresh` sequence serves payloads and all continuations;
  - `AgentOperation` re-exports the link's `Operation` under a second name;
  - temper calls `Counters::commit_header()` directly;
  - `Name<N>` takes a closed `Grammar` enum, and `from_checked` accepts empty bytes (skein's design);
  - `skein_ui::date`'s ranges are implicit;
  - skein's `fake.rs` llm function hugs the function-line limit.
- **Client stack:**
  - the wire cannot carry non-empty `TaskContext.writer_slots`; the fixtures were removed (unsure whether that's a gap);
  - a hand-written `ReadValue` tag in the archived-turn inspector tracks generated numbering by hand;
  - a hand-added `struct_field_names` expect sits on generated v19;
  - plain-twin leftovers: domain `Part`/`PartScope`, byte helpers that only refuse, `Delivery::Run`;
  - 359 types and 3,694 goldens were renamed in place with the client protocol still at 19 (design);
  - the owner withdrawal history carries no reason text (unsure).
- **Client view:**
  - the frame's project bound counted a decoded list's spare capacity as entries, refusing valid frames. Fixed in `tree/frame/me.rs`, but other bound checks were not audited for the same mistake;
  - one new outcome means editing the view's fixed-capacity phrase arrays, the scripted engine and the joint harness.
- **Test kits and worlds:**
  - six real front differences run as named ignored tests: on the scripted front, a stopped hold's release shows no notice (×2) and an accepted proposal is missing from the chat; on the joint front, unseen Release and Cancel facts and a revision mismatch. Unsure which side is wrong on the joint ones;
  - the merged scripted front picks its request adapter by a private flag, so behaviour depends on seeding order;
  - the scripted engine still reimplements core's inbox order, phase classification and requester choice, with no cross-check;
  - breaking the dev cycles copied fixture constants (record templates, HTTP limits, store task bounds, transcript dialect) that nothing checks against their owners;
  - fixture limits must agree by hand: core-limits missed one temper fixture and 616-640 temper tests went red;
  - the wire world's drift test can't run while the view doesn't build;
  - the core world's observer and referee carry deletion exceptions;
  - the client-world harness stores typed unlinked records, so restart can't check the byte carrier;
  - conformance-key regenerated the test family's codecs in place (unsure whether approved);
  - the test family's page composition must be re-named on every sync, because `Named` can arrive before the working set;
  - the HTTP engine and temper keep their own composition loops;
  - a host-world trace assertion matches `Debug` text;
  - qa-jig's `qa_settings_proposals` hard-coded `MEMBER = 2` until the repair of `main`: hard-coded person numbers, run only by the ungated qa tier;
  - `jig-client-test-records` has `lib.test = false`, and `jig-test-profile` has no tests;
  - an ops incident-panel test was deleted with the plain chip renderer (possible coverage loss).
- **Ops:**
  - `health_words` keeps an unused `_service` parameter;
  - ops combines its own timers and encodes its charter twice;
  - its client v1 schema shrank in place without a version step (pre-release, Call 2).

## 7. Outside jig's code, seen from it

These belong to fabrig's tools (`agents/backlog.md`), not to jig's design.
They are listed so this file doesn't lose them.

- **`scripts/old_forms.py`:**
  - reads git's file list, so it fails until renames and deletions are staged;
  - fails when run from the repository's root;
  - counts every index stage during a merge, and reports false raises.
- **No binary entry in `.gitattributes`:** `git diff --check` reads binary goldens (ops `*.bin`, v11) as text.
- **The fuzzy profile selects no test** (nextest exit 4) in unit-only crates, including the view, the client test kits and temper's client crates. They get no fuzzy coverage.
- **The default suite is at nextest's 120 s global limit:** one run killed 8 tests and left 1,037 unrun (temper's real-loop and one-process). Under `testing-strategy.md` 8.1 that is a starved warning, not a red.
- **The Codex runs' check commands:**
  - a check wrapper exited 0 while cargo failed with 101;
  - codegen's write mode needs dependents to compile;
  - the journey recorder assumes a tree-local path.
- **One new act, ask or event variant ripples through 20 to 50 files,** through exhaustive tables carrying `match_same_arms`/`too_many_lines` expects.
- **`bin/loc`** counts the nested-mod `records_v1.rs` as uncompiled.
- **Flaky temper tests seen from jig work:**
  - `fake_backends::retained_peer_edits_are_checked_without_replacement`;
  - `missing_required_retained_documents_refuse_without_writes`;
  - real-loop fuzzy `case_5_4`.

## 8. Paid, or nothing to do

- **Done at the base, by the upgrades run:**
  - item 6 (two start paths; residue done by the run);
  - A4 F4 (people's default restore).
- **Gone or already there:**
  - bug 8: gone as stated;
  - 12g (`skein_json::parse`) and 12h (skein-io's owner): already there, now adopted;
  - 11b's charter translation: already in place;
  - no other dead exports were found;
  - A9 F10: already gone;
  - A2 F18 had 4 pairs, not 9, all collapsed;
  - the review itself refuted merging the three `jig-test-patches*` crates.
- **`tech-debt.md` is stale and needs re-measuring before it is cited again:**
  - §2: `joint/wire/`, deleted;
  - §1: six new `#[ignore]`s;
  - §3: transitional paths, now paid;
  - §7: `unreachable!` 404, `manual_map` paid, `too_many_arguments` 70;
  - §10: 29 `_partN` files;
  - §11: the cut at 195;
  - §12: generated code at about 273k lines.

## 9. Later

- Merge `docs/development/tech-debt.md` into this file once it is re-measured, so jig keeps one debt list.
- Re-measure every count here against the review's own method before planning on them.
- The process note's three guards, as fabrig's tools work (section 4).
