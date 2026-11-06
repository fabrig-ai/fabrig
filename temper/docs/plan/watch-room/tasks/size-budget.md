+++
id = "size-budget"
part = "watch-rooms"
kind = "feature"
needs = ["story", "link-told"]
files = [
  "temper/crates/temper-engine-domain/src/limits.rs",
  "temper/crates/temper-engine-domain/src/lib.rs",
  "temper/crates/temper-engine-domain/src/tests/resume_fixture.rs",
  "temper/crates/temper-service/src/limits.rs",
  "temper/crates/temper-service/src/limits/reference.rs",
  "temper/crates/temper-service/src/startup.rs",
  "temper/crates/temper-service/src/tests/limits.rs",
  "temper/docs/help/reference/limits-engine.md",
  "temper/tests/engine/domain/src/walking.rs",
  "temper/tests/engine/domain/tests/suite/memory.rs",
  "temper/tests/engine/domain/tests/suite/watch_room.rs",
  "temper/tests/engine/domain/tests/suite/main.rs",
  "temper/tests/client/tests/suite/watch_room.rs",
  "temper/tests/client/src/fixtures.rs",
  "temper/tests/client/src/front.rs",
  "temper/tests/client/src/agent.rs",
]
effort = "high"
minutes = 130
+++

# The engine's watch rooms follow what it admits

## Goal

The request's item 2. The deployment admits `clients.streams` clients, each holding up to 17 watches, so the engine's room is `clients.streams` times 17, in all and to a party, not one client's bound. `views.watchers` and `views.watchers_per_party` go from temper's documents. A client past the room gets a useful answer, closing watches give their room back when they should, and you measure what a watch costs, before and after.

Acceptance:

- `temper_engine_domain` has `Limits::streams`, `views_for` and `core_limits` as `temper/docs/plan/watch-room/notes/contracts.md` has them, and `temper-service`'s `engine_candidate` takes `streams` from `jig.clients.streams`. The sizing is said beside the figures, in `limits.rs`.
- A step test in `temper-service` holds both rooms to `clients.streams` times `WATCHES_PER_STREAM`, at 2 and at 8. A `clients.streams` whose room doesn't fit is refused at the start.
- The story `watch_room::two_open_tabs_leave_a_fresh_tools_client_room_for_its_read` has no `should_panic`, runs at `streams = 8`, and passes.
- The stories in step 3 pass at `streams = 1`.
- An `engine.limits.overrides` naming `views.watchers` or `views.watchers_per_party` is refused at the start, naming `clients.streams`. No temper document or fixture names them any more, and `temper help limits-engine` lists neither and says how the rooms are sized.
- `suite/memory.rs` holds a full room to a heap ceiling, and your report gives the before-and-after figures of step 5.
- No existing heap ceiling, budget or bound rises. The worlds written out at `streams: 1` keep their room of 17.

## Inputs

- `temper/docs/design/protocol/configuration.md`, 7 (from "The engine's watch rooms follow what it admits" through "Past the room") and 13.4.
- `temper/docs/design/domain/root.md`, 10, "The watches' rooms follow the streams".
- `jig/docs/design/domain/engine.md`, 11: what a watch keeps. `jig/docs/design/client/domain/contract.md`, 7.2: a watch holds its place until the engine has ended it.
- `temper/docs/plan/watch-room/notes/contracts.md`, "temper's engine: the rooms" and "The stories".
- `temper/docs/plan/watch-room/notes/common.md`.

## Owns and may change

Owns the files in the header.

Reader and neighbour files, each edited only as said:

- **Delete the two names and nothing else,** the `views.watchers` and `views.watchers_per_party` entries, in:
  - `temper/crates/temper/tests/support/configuration.rs`
  - `temper/crates/temper/tests/support/github/engine.json`
  - `temper/crates/temper/tests/fixtures/cluster/engine.json`
  - `temper/crates/temper-playground/src/documents.rs`
  - `temper/crates/temper-playground/src/documents/channel.rs`
  - `temper/crates/temper-playground/stories/a/deployment.json`
  - `temper/tests/real-loop/src/fixtures.rs`
  - `temper/tests/client-browser/src/configuration.rs`

  Those last two and the `temper` process tier's support are slow-tier: compile them with `--no-run`, and don't run them. jig's own fixtures keep the names, since jig's root takes them.
- **Compiler-forced `streams: 1`:** any other written-out `engine::Limits`, each edited only for that. From the survey, `limits.rs`'s `engine_candidate`, `resume_fixture.rs` and `walking.rs` are all of them.
- **Unchanged, and read to confirm:**
  - `temper/tests/client/tests/suite/reloads.rs`: `VIEWS.watchers_per_party` is still the world's room, at `streams: 1`.
  - `temper/crates/temper/tests/suite/configuration.rs`, 69-71: `VIEWS` is now one stream's room, and the asserts still hold.
  - `temper-engine-domain/src/route/forge_route.rs`: `update-recovers/update-hold-words` is changing it, and `VIEWS.subjects_per_change` stands. Don't touch it.
  - `temper-engine-domain/src/tests/resume.rs`: `crate::VIEWS` passes one Text event.

Departures from your text, for this task alone:

- Delete `temper/docs/design/protocol/configuration.status.md`'s `Not yet` line that opens `7: the engine's watch rooms` and `temper/docs/design/domain/root.status.md`'s that opens `10: the root has no streams`.
- Add one sentence of measured figures to `temper/docs/design/protocol/configuration.md`, 7, at the end of "What a watch costs"'s heap point, and change nothing else there. If a measurement contradicts a sentence there, leave it and report it under `## Left`.
- `temper/docs/help/reference/limits-engine.md` is regenerated by its own test, never by hand.

**A change of meaning runs its readers' tests:** a room's figure changes for every reader of `core_limits`, so step 6 names them.

Options, each pre-approved:

- **The refusal's words.** Default: a check beside `inline_seed_policies` in `startup.rs`, reading `engine.limits.overrides` through its `member` helper and pushing `What::Against { other: b"..." }` at `engine.limits.overrides.views.watchers` and `...views.watchers_per_party`, with `other` naming `engine.limits.overrides.clients.streams` as what sizes them. If `What::Against`'s line reads wrong for it, add a variant in `startup/problems.rs`, and say so.
- **A room too large.** `views_for` saturates, so `views::valid` fails on `slots`, and `checks::engine_envelope`, `check_effective_limits`, refuses it. Keep the path it gives, even if it names `views.watchers`. Add words only if the line can't be read as a problem with `clients.streams`.
- **`with_watch_room`.** It takes `streams = 8`. Add a sibling taking `streams` for the stories at 1, or a parameter on `fixtures::watch_room_world`, as reads best, and edit `story`'s call sites if you take the parameter.
- **A tab closing.** The harness has `reload` and `go`, and no close. Default: `Joint::close_client(client)`, around `Composition::detach`. If the harness can't step a detached client, `world.go` the long chat to a page that holds fewer watches, and assert from `open_watches` that room came back. Say which.
- **`Agent::watches_full`.** Add it to `temper/tests/client/src/agent.rs`, reading `self.root.client().link().watches_full`.
- **The engine world's stories.** `panels.rs`'s `watch` shows the shape of `PartyInput::Watch`. If `Driver` can't hold a delivery back for `a_closing_watch_...`, a small helper in `suite/watch_room.rs` over `driver.immediate` is yours. If `jig-domain-views`' `closing_waits_for_both_document_and_stream_terminals` plus a story with no held delivery proves the same room, keep that, and say so.
- **The reference's words.** Default, after the page's second paragraph: `temper sizes the engine's watch rooms itself: a stream holds 17 watches, and the engine holds clients.streams times that, in all and for one party. Raise clients.streams for more.` A page past `PAGE_BYTES` may be condensed by the rule in your text.

## Steps

Commit after each step.

1. **The rooms.** `Limits::streams`, `views_for`, `core_limits`, `engine_candidate`, and `streams: 1` in the written-out worlds. In `temper-service/src/tests/limits.rs`, write `the_watch_rooms_follow_the_streams` beside `server_connections_carry_the_box_streams_and_four_calls`, and `a_room_past_what_the_views_hold_is_refused`.
2. **The story.** In `temper/tests/client`, `with_watch_room` sets `streams = 8` and the `should_panic` comes off. It passes. Report the most watches held, against the walk's 25.
3. **Past the room,** at `streams = 1`, so the room stays 17:
   - In `temper/tests/client/tests/suite/watch_room.rs`, `a_client_past_the_room_is_told_and_answered_when_a_page_closes`: the two tabs fill the room and the tools client starts. Past `fixtures::WITHIN` it isn't ready and `Agent::watches_full` is true. A tab closes, and within the tools client's `backoff.most` plus `fixtures::WITHIN` it's ready, its read answers with status 0, and `watches_full` is false. Don't use `fixtures::give` before the close, since it asserts readiness.
   - In `temper/tests/engine/domain/tests/suite/watch_room.rs`, a new file with one `mod` line:
     - `a_full_room_refuses_busy_and_a_finished_watch_gives_it_back`: the room's worth of watches admitted, the next refused `Busy`, one ended, and the next admitted.
     - `a_closing_watch_holds_its_room_until_its_delivery_is_taken`: with a delivery in flight, its `Unwatch` doesn't free the room and the next watch is `Busy`. Once the delivery is taken, the room is back.
     - `two_parties_share_the_deployments_room`: one party fills the room, and another's watch is refused `Busy`. At `streams = 2` the first party alone may hold 34.
4. **The names go.** The start's refusal and `an_override_naming_a_watch_room_is_refused`, then the eight fixtures above, then `engine_reference` and its page: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper --lib -E 'test(=tests::limits::the_pages_limits_are_what_the_tables_render)'`, then the same without the variable.
5. **What a watch costs.** In `temper/tests/engine/domain/tests/suite/memory.rs`, write `a_full_watch_room_fits_its_heap_ceiling`, using `Meter` and `Driver`, with the file's `Measured peak` comment and 25% headroom:
   - the root's heap at the start at `streams` 1 and 8: expect equal, since capacities allocate nothing until used;
   - the heap held per watch, from a full room of 17 and of 136;
   - the peak of one commit that marks them all, at 17 and at 136, on one subject shared and on distinct subjects.

   Measure the work with untracked throwaway code, and delete it before the push: the documents built in that pass, and its wall time at 17 and 136. No timing goes in a test. Report a table of all of this, and whether a full room stays within `root.journal.deliveries`, as the design says. Then add the design's one sentence.
6. `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, then:
   - `heavy cargo clippy -p temper-engine-domain -p temper-service -p temper-engine-domain-world -p temper-client-world -p temper-playground --all-targets -- -D warnings`
   - `heavy cargo nextest run -p temper-engine-domain -p temper-service -p temper-engine-domain-world -p temper-client-world -p temper-playground`, which are the readers of a room's figure
   - `heavy cargo nextest run -p temper --lib`
   - `heavy cargo nextest run -p temper --no-run` and `heavy cargo nextest run -p temper-client-browser-world -p temper-real-loop-world --no-run`, the slow-tier files you edited, compiled and not run
   - `python3 -I -B tools/citations/citations.py --base origin/main`

   Commit and push. A focused-suite budget your new stories push over is red: find the cost and cut it first.
7. Break each rule once by hand, run its test, see it fail for that rule, and restore: `views_for` back to one stream's room, the refusal removed, and a closing watch's room freed at once (a throwaway edit of `jig-domain-views/src/watch.rs`, `close`, restored before you commit).

## Report

The implementer's report, plus: the table from step 5, the most watches held in the story, which options you took, and the slow-tier files compiled and not run. Under `## Left`, anything a person's page shows past the room: temper/ui owns those words.

## Answers

2026-11-06 (temper/forge). **Two more, from temper/ui, which reached the request after your plan was cut.** `story` also writes a person's reload and a third tab as stories, on the happy path, held as its fresh tools client is. Take those to green with it: with Home and the long chat open, a reload of one and a third client root opening a page each come up whole, within the world's bound. **And your report says whether a page can still be refused Busy after this, and how a person would reach it in normal use:** how many tabs, what else. temper/ui gives that case words only if it can happen, and closing a tab being the only way out is the case it most wants to hear of.
