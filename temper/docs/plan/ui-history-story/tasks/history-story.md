+++
id = "history-story"
part = "story"
kind = "feature"
needs = ["pressed-step"]
files = [
  "temper/tests/client-browser/tests/browser_all/browser_pages.rs",
]
effort = "high"
minutes = 70
+++

# The history story reads the Chats filter by its name and pressed state

## Goal

`browser_pages::history_keeps_the_chat_focus_and_filter_across_reload_and_back` failed twice at `77544586ee` (red-63) at step 74, `keep selector main .temper-filters > button[aria-pressed='true']:nth-child(3) ... no element`, right after a reload of `/chats?state=closed`. Find out by running it whether Chats really loses its filter, then fix what the runs show. Reading the code says the client is right: the page takes its filter from the address when it opens (`temper-client-domain`'s `pages.rs`, pinned by `arriving_at_chats_sets_its_address_filter`), and the view puts a Drafts button before Closed only once the project's ways are read and offer drafts (`temper-client-view`'s `pages/chats.rs`, `offers_drafts`; the stock ways do). So Closed is the third button before the ways answer and the fourth after, and the story reads it once, by place.

This task departs from the implementer's text and from the lab's rule against slow-tier runs in one way: it is red-63's repair, so it runs this one story **by name, alone**, in the browser profile, as a red's repair may (`AGENTS.md`, 5; notes/common.md). Never the tier, and no other browser story.

Acceptance:

1. **Before any edit,** five runs of the story alone (Step 1) are reported: each one's outcome, and for each failure its step, its message, and what the failure's accessibility snapshot shows of the filter row (which buttons, which one is pressed).
2. The story reads the filter by its name and pressed state, waiting as its sibling steps do, with skein's `Step::Pressed` (from `pressed-step`): after the reload, `Closed` pressed; after Back, `All` pressed. No selector in the story names a position among siblings (`:nth-child`, `:first-child`, `:last-child`, `:nth-of-type`).
3. Every other check the story makes stays as it is, the `see` of each button before its pressed check among them. No deadline widened: the new steps wait `WITHIN`.
4. After the edit, three runs of the story alone pass.
5. `heavy cargo check -p temper-client-browser-world --tests`, clippy for that package, and its focused `--lib` tests pass.
6. If the runs showed a real loss of the filter (option B below), a focused test of the restored state comes first and fails without the fix.

## Inputs

- `temper/docs/plan/ui-history-story/notes/common.md`.
- `temper/docs/plan/ui-history-story/inputs/request.md`: the red and its one task.
- `skein/docs/design/ui-web.md`, 14.2: *awaits a control pressed*, the step this task uses.
- `temper/docs/design/client/browser.md`, 4.6, item 2: the story's filter, "its filter's address and pressed state kept across a reload and Back".
- `temper/docs/design/client/journeys.md`, 7: a selector names "never ... a part's position among its siblings".

## Owns and may change

**Owned:** `browser_pages.rs`. Nothing reads the kept name `pressed-filter`: the story's after-run reads take `call-line` alone, and `rg` finds no other reader.

**Options, each with its default.** Read the runs of Step 1, then:

- **A (expected): the story reads too soon or by place.** Every failure is at a `pressed(..)` keep, with no element, and its snapshot shows `Closed` (or, after Back, `All`) pressed, a Drafts button perhaps among them. Default: Step 2, no product change.
- **B: a real loss.** A snapshot shows the page past the reload with `Closed` found but not pressed, or past Back with `All` not pressed, after the story's `see` of it. Then first a row of the focused test that shows the loss: `arriving_at_chats_sets_its_address_filter` in `temper/crates/temper-client-domain/src/tests/pages.rs`, or, if the loss is the view's, `each_filter_selects_its_lines_and_presses_its_bound_button` in `temper/crates/temper-client-view/src/tests/chats.rs`. Then the fix in `temper/crates/temper-client-domain/src/pages.rs` or `temper/crates/temper-client-view/src/pages/chats.rs`, and Step 2 as well. Those four files are pre-approved for this case alone; run the package's focused tests and say so.
- **C: a failure at another step** (red-62's other message is unknown). If it is a read of this story's that comes too soon or by place, fix it the same way, keeping its check. Otherwise, don't fix it: list it under `## Left` with its step, message and log lines.
- **D: no failure in five runs.** Make Step 2's change anyway: the position rule holds, and the code shows the row moves. Say the runs didn't reproduce it.
- If a run after the edit fails at a new `Pressed` step, read its snapshot: a filter not pressed by the bound is B; anything else is C.

## Steps

1. Run the story alone, five times, before editing, reading each log by `grep` and `tail` around the failure (heavy prints its path; the harness writes the accessibility snapshot, a screenshot and the stderr tails on a failure):

   ```sh
   heavy cargo nextest run -p temper-client-browser-world --profile browser-temper --test browser_all \
     -E 'test(=browser_pages::history_keeps_the_chat_focus_and_filter_across_reload_and_back)'
   ```

   Check each run selected one test.
2. `browser_pages.rs`, `history_keeps_the_chat_focus_and_filter_across_reload_and_back`: the `pressed` closure takes the filter's name, not its place:

   ```rust
   let pressed = |name: &[u8]| Step::Pressed { find: Find::exactly_named(Role::Button, name), within: WITHIN };
   // ...
   Step::Person(PersonStep::Reload),
   see(Find::exactly_named(Role::Button, b"Closed")),
   pressed(b"Closed"),
   Step::Back,
   see(Find::exactly_named(Role::Button, b"All")),
   Step::Address(b"/chats".as_slice().into()),
   pressed(b"All"),
   ```

   One step for one step, so the stories' step numbers after it don't move.
3. Option B's test and fix, only if the runs showed it.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-browser-world --all-targets -- -D warnings`; `heavy cargo check -p temper-client-browser-world --tests`; `heavy cargo nextest run -p temper-client-browser-world --lib`. Then the story alone, three times, as in Step 1. The citations check (notes/common.md). Commit and push.

## Report

As the implementer's text says, and first the five runs before the edit and the three after: each outcome, and for each failure its step, message and the filter row the snapshot showed. Then which option the runs showed and why. If a Drafts button arrives in the filter row after the page is drawn, say so: it moves Closed under a person's pointer, and is a finding for after landing, not this task's.
