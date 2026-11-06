+++
id = "history-ctl"
part = "history"
kind = "feature"
needs = []
files = [
  "temper/tests/client-browser/src/history.rs",
  "temper/tests/client-browser/src/lib.rs",
  "temper/tests/client-browser/tests/browser_all/browser_pages.rs",
  "temper/docs/plan/browser-budget/figures/history-ctl.md",
  "jig/crates/jig-protocol-engine-clients-http/src/server.rs",
  "jig/crates/jig-protocol-engine-clients-http/src/tests.rs",
  "jig/docs/design/protocol/service.md",
]
effort = "high"
minutes = 70
+++

# A reading story's history is made through ctl, and the page opens on it

## Goal

The request's first lever. `reading_history` types 16 messages through Chromium, each answered by about 65 KB, and `latest_returns` types 6 and holds a second tab for its new words. None of that typing is what they're about: Chromium renders every delivery while the harness polls a growing tree. Make the history through `temper ctl` before a page opens, and keep typing only the message sent from far back.

Acceptance:

- `Tier::ctl`, `Tier::ctl_chat`, `Tier::ctl_face`, `Tier::held_setup` and `reader_opens` exist as `notes/contracts.md` has them.
- All four stories make their history through them. The one typed message left among them is `latest_returns`' `SENT`, "Sent while far back in the conversation.": nothing else in these stories goes through `Step::Type`, `PersonStep::Type` or `Key::Enter`.
- `latest_returns`' `INCOMING` is sent through `Tier::ctl` while the page reads back, so its second tab goes, and `Stage` and `Together` go with it. The view must hold as it does today: `before-incoming` to `after-incoming` stays `held`.
- Each behaviour in `notes/common.md`'s *What stays proved* still has its step and its assertion. Each of the four stories passes by name, alone, three runs each.
- `heavy cargo nextest list --profile browser-temper -p temper-client-browser-world` still lists the seventeen stories, and no other story changed.
- `temper/docs/plan/browser-budget/figures/history-ctl.md` holds three runs of each story before your first change and three after, and the long story's CPU split after, as `notes/measuring.md` says.
- The `**Not yet:** 4.3, *A chat's history*` entry leaves `temper/docs/design/client/browser.status.md`.

The targets, which aren't limits and which the next lever shares: the long story about 12 CPU s or less, `latest_returns` about 6 or less, `earlier_words` about 5 or less. Report what you measure either way.

## Inputs

- `temper/docs/design/client/browser.md`, 4.3, *A chat's history*, and 4.1, item 5.
- `temper/docs/design/draft/chat-reading.md`, 5.
- `temper/docs/plan/browser-budget/notes/common.md`, `temper/docs/plan/browser-budget/notes/contracts.md`, `temper/docs/plan/browser-budget/notes/measuring.md`.
- `temper/docs/help/ctl.md`, for the calls and their exits.

## Owns and may change

Owns the files in the header. `lib.rs` gains `mod history;` and `pub use history::reader_opens;`, and no other edit. `temper/docs/design/client/browser.status.md` loses the one entry above.

Read but don't change: `browser_public.rs` (`sign_in`, `open_printed` and its `ctl` script, the same steps working in this tier) and `temper/crates/temper/tests/fake_backends/walk/chat_document.rs` (the calls, on a fresh box's project 1 and chat 1).

Not this task: reply sizes, `READING_MESSAGES`, `tools/tiers/run` or anything about the window (`window-story` builds on this one); `face.rs`.

Options, each decided:

- **`latest_returns`' chat** starts with scripted words through `with_replies`, not the demo's "Add CSV export to widget", which delegates a Task 2 that nothing in this story reads. Its messages and replies keep enough height that two PageUps land with `words(2)` inside the log. If it can't be shown without the delegate, keep the delegate and say why in your report.
- **The reloads:** where a story's `Reload` was there to open the page afresh on a finished history, it goes: `reader_opens` is that opening. A `Reload` that proves something else stays.
- **A deadline on a ctl call:** each call takes `WITHIN`, as the page's waits do. If a call truly needs longer, that's a finding for your report, not a new constant.
- **A CPU figure that doesn't fall:** keep the change if the stories pass, write the numbers down, and say where the CPU goes from the split. Over budget isn't yours to settle here.

## Steps

1. Before any edit: three runs of each story, for the *before* arm.
2. `src/history.rs`, as `notes/contracts.md` has it, on `github_connect`'s client directory. `ctl_chat`'s calls are `playground start`, `chat start --project 1 --model demo` and `task show 1 --until replied`, then `task message 1 WORDS` and `task show 1 --until replied` for each of `then`, each act with a `--key` of its own. Unit tests beside the crate's own in `lib.rs`'s `tests`, if a rule there can fail without a browser (say, the order of `ctl_chat`'s calls and that a key is never reused); a test that only repeats the list isn't one.
3. In `browser_pages.rs`:

   ```rust
   // reading_history becomes the ctl face's calls and the reader's opening.
   fn reading_chat(tier: &Tier, messages: u32) -> (Script, Vec<Step>);
   ```

   The reader's opening keeps `see(paragraph(&reading_words(messages)))` and the three `gone`s (loading, Older, Latest) that today follow the reload. `reading_position` then goes on as it stands, and runs `Harness::new(tier.held_setup()).run(&mut [&mut ctl, &mut reader])`.
4. `a_short_chat_opens_whole_without_a_reading_status`, then `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back`, the same way, one at a time, each run by name as it comes right.
5. Three runs of each story for the *after* arm, the long story's split, and the figures.
6. `cargo fmt`; `heavy cargo clippy -p temper-client-browser-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-browser-world --lib`; the list above. Delete `target/browser-budget/`, make sure throwaway code is gone, commit and push.

## Report

The before-and-after medians with their load, the steps that went and what proves them now, what the split says is left, and your implementation and check minutes apart.

## Answers

2026-11-06. **Who repairs `temper ctl` beside an open page: a separate repair first, or this task?**

**This task, on the pushed branch: keep `64dafc5857` and `95feff7388`, and go on from them.** Acceptance can't hold without it, since `INCOMING` through `Tier::ctl` with the page open is this task's own call and the plan's only one of the kind. The second tab doesn't come back, and `WITHIN`, the listener's 30-second `stall` and every budget stay as they are.

1. **See the cause first; it isn't known yet.** What the code says, unobserved: `temper-service`'s `limits::server` gives the listener eight connections; `jig-protocol-engine-clients-http`'s `Server::up` rejects a connection when they're all held; an idle one holds its place until the 30-second `stall`; and ctl needs its own connections while two faces' pages hold theirs. Run `latest_returns` by name, alone, with a throwaway count of connections held and rejected, in `Server::up` or by `ss` while ctl waits. It goes before the push. That makes the run a changed one, not an unchanged rerun. Write what you saw and the load into the figures.
2. **If the listener is full,** a full listener ends an idle connection, one with no call and no request begun, and admits the new one; it rejects only when none is idle. HTTP lets a server close an idle connection, and a page reconnects. Don't raise the eight: a person with two tabs and an agent can fill any number. It takes a focused test beside the crate's in `src/tests.rs`: a full listener with an idle connection admits the next, a full one with every connection in a call rejects it, and a call in progress is never ended. Add a line to `jig/docs/design/protocol/service.md`, 11.4. skein and jig stay in their order: this names no temper.
3. **If it's something else,** repair it in the lowest tier that shows it, when the repair is small: a few files, named in your report as a small edit outside the list. A cause in this task's own harness steps is yours too. If the repair is a new mechanism or reaches past that, push what you have and report `Question:` with what you observed. Don't paper over it in the story, by retries, a sleep or a longer deadline.
4. **Latest's *before* arm** is `figures/story-baseline.md`'s three passing runs, median 12.480270 CPU s and 5.329647 wall s. Don't run the two missing repetitions. Keep your failed baseline run in the table: a second tab's story that fails on the base is a finding for your report, not yours to repair, as that tab goes.
5. **The rest of acceptance stands:** three *after* runs of each story, the long story's split, and the status entry's removal. In `temper/docs/design/client/browser.status.md`, 4.3, the `browser_public` red's "no cause observed" may say what you saw, as a reading of that story: it isn't run here, so don't call it repaired.
