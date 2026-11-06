+++
id = "story"
part = "watch-story"
kind = "feature"
needs = []
files = [
  "temper/tests/client/tests/suite/watch_room.rs",
  "temper/tests/client/tests/suite/main.rs",
  "temper/tests/client/src/fixtures.rs",
  "temper/tests/client/src/front.rs",
]
effort = "high"
minutes = 75
+++

# Two open tabs lock out a fresh tools client: the failing story

## Goal

The request's item 1. Two real client roots, on Home and on a long chat, run against the real engine root. Then a fresh tools client must get its readiness and a detail read's answer within the world's bound. Today it can't: the core's 17 watches are full, its `me` watch comes back Busy, and the read is never sent.

This task writes the story and proves it fails for that reason. It fixes nothing: `size-budget` does, and it removes the attribute below.

Acceptance:

- `watch_room::two_open_tabs_leave_a_fresh_tools_client_room_for_its_read` is in `temper-client-world`'s `suite`, with no HTTP, browser or kernel, and uses the pages' real watch sets.
- It carries `#[should_panic(expected = "...")]` naming the readiness assert that fires today, so it passes on the trunk and fails loudly the moment the room is sized. A failure anywhere else (the tabs not going live, the count below) is not the expected panic, so it still reads red.
- Before the tools client starts, it asserts that the two tabs hold at least `temper_engine_domain::VIEWS.watchers` watches.
- With the two rooms at 64 by hand, and the attribute off, the read answers with status 0 (step 4). This is the request's test of the cause, repeated in the world.
- Every other test in `temper-client-world` behaves as before.

## Inputs

- `temper/docs/design/protocol/configuration.md`, 7, "The engine's watch rooms follow what it admits".
- `jig/docs/design/client/domain/client.md`, 8.1 and 8.2: Busy is reopened after a backoff, and `me` refused takes the link offline.
- `temper/docs/plan/watch-room/notes/contracts.md`, "The stories".
- `temper/docs/plan/watch-room/notes/common.md`.

## Owns and may change

Owns the files in the header. Your helpers are new items, and nothing else in `fixtures.rs` or `front.rs` changes. `suite/main.rs` gains one `mod` line.

Other drafts: `long-words/composer-view` and `ctl-waits/kept-answer` add `mod` lines to `suite/main.rs`. Keep yours in its alphabetical place.

Reader files: none. The new helpers have no other readers, and you change no existing item's meaning.

Options, each pre-approved:

- **The limits.** `fixtures::domain_limits()` has `watches: 13`, `handles: 14`, `delegates: 2`, `reads: 8`. The shipped shells have 17, 19, 80 and 32 (`temper-client-shell-web/src/limits.rs`). `watch_room_limits()` takes those four, plus any other figure a page's watch set turns on. `temper-client-world` doesn't depend on the web shell, so write the numbers with that citation.
- **The room for sign-ins.** `front.rs`'s `engine_limits()` has `people.sign_ins = 3`, a goal's driver and two clients. This story has three clients. `Joint::with_watch_room` raises it for its own engine and leaves `engine_limits()` as it is. Raise the task and delegate bounds it needs the same way.
- **Filling the room.** From reading, Home holds its session watches and two of its own, about 8, and a chat holds its session watches, its task, its goal's chips, its run and one `Subject::Task` per delegate (`temper-client-domain/src/pages/chat.rs`, `delegation::Delegation`). The walk saw 17 held at the budget, 9 of them the chat's after Home closed, and 25 at most with the rooms at 64. Nobody has counted this in the world.
  - **Default:** `watch_room_chat` gives the chat enough delegates that the two tabs reach 17, and your report gives each tab's count.
  - **If settled tabs can't reach 17,** because they held room in the walk's settled phase: `world.reload` both tabs and start the tools client at that moment, which is the request's "open or reopening". Assert the count there, and say so.
  - **If neither reaches it,** ask, with the counts.
- **The count.** `Joint::open_watches` defaults to `self.transport.watches().len()`, which leaves closing ones out. A count from the core is yours if it reads closer, and say so.
- **The expected panic.** `fixtures::give` has two asserts. Default: `"agent ready within world time"`. If `"agent ready before call"` fires instead, name that one, and say so.

## Steps

1. Read `temper/tests/client/tests/suite/reloads.rs` and `fixtures::whole_turn_client` for the shape of a person client at its own limits, and `fixtures::agent`, `give` and `answered` for the tools client.
2. Write the helpers, then the story:

   ```rust
   //! Two open tabs leave a fresh client its room (temper/docs/design/protocol/configuration.md, 7).
   #[test]
   #[should_panic(expected = "agent ready within world time")]
   fn two_open_tabs_leave_a_fresh_tools_client_room_for_its_read() {
       let mut world = fixtures::watch_room_world(fixtures::SEED);
       let chat = fixtures::watch_room_chat(world.front());
       let home = fixtures::watch_room_client(&mut world, fixtures::project());
       let long = fixtures::watch_room_client(&mut world, fixtures::chat_page(chat));
       // Each tab is seen live, through a person's Step::See, as reloads.rs does.
       let held = world.front().open_watches();
       assert!(held >= /* VIEWS.watchers */, "the two tabs hold the old room: {held}");
       let agent = fixtures::agent(&mut world);
       let number = fixtures::give(&mut world, agent, &[b"task", b"show", /* chat */]);
       let printed = fixtures::answered(&mut world, agent, number);
       assert_eq!(printed.status, 0, "{printed:?}");
   }
   ```
   `fixtures::project()` is Home. `give` and `answered` already hold the tools client to `fixtures::WITHIN`, and that is the world's bound: add no other.
3. Run it without the attribute and read the failure. It must be the readiness assert, with the tools client's `me` refused Busy. Confirm that from the engine's side, not from the panic alone: while the tools client waits, the engine holds the room full and the tools client isn't ready. Put the attribute on, and run it again.
4. **The test of the cause.** By hand, set `watchers` and `watchers_per_party` to 64 in `temper-engine-domain/src/limits.rs`, take the attribute off, and run the story. It must pass, with the read's status 0. Report the most watches held and the time on the world's clock. Restore both.
5. `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy -p temper-client-world --all-targets -- -D warnings`, then `heavy cargo nextest run -p temper-client-world`. Commit and push. Check the story's CPU against its neighbours in the run's log: an expensive fixture is yours to cut.

## Report

The implementer's report, plus: each tab's watch count, whether the room filled settled or reopening, the assert that fired, the figures from step 4, and which of the options you took.

## Answers

2026-11-06 (temper/forge). **Two more, from temper/ui, which reached the request after your plan was cut.** A person's reload and a third tab are stories too, on the happy path: with Home and the long chat open, a reload of one, and a third client root opening a page, each come up whole, within the world's bound. Like the fresh tools client, they fail today and pass once `size-budget` lands, so hold them as your story holds its own until then. **And for whoever finishes the budget:** say in the report whether a page can still be refused Busy after it, and how a person would reach it in normal use. temper/ui gives that case words only if it can happen.
