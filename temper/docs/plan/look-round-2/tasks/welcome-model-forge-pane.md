+++
id = "welcome-model-forge-pane"
part = "welcome-model-forge-pane"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/pages/welcome.rs",
  "temper/crates/temper-client-view/src/forge/connect.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
  "temper/crates/temper-client-view/src/tests/model_forge.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/journeys/welcome-set-up/07.html",
  "temper/docs/design/client/journeys/welcome-set-up/08.html",
  "temper/docs/design/client/journeys/welcome-set-up/README.md",
]
effort = "high"
minutes = 150
+++

# Land welcome-model-forge-pane: the forge's state in a pane beside the Model step

## Goal

Land the look request `welcome-model-forge-pane`, commit `87cddc4453`, whole: the forge's state
leaves the top of the Model step for a region of its own, a pane at the right from 1280 px, under
the choices when narrower, its line stacked, in every state the message names, at every width, in
the browser and the terminal. No journey reaches its done state, and no test prints the Model step
in the terminal today: this task brings both in.

The request's message, whole:

> Look request (temporary): the forge's state in a pane beside the Model step
>
> On the Model step the forge's line and its "Each step" sat above the
> heading, in a green box, which pushed the step's heading and choices
> down from where the Forge step draws them. The person wants them where
> the Forge step has them, with the choices given the room, and the
> forge's state to the side.
>
> - The forge's state is a region of its own (`aside`, named "Forge",
>   class temper-welcome-status) after the choices and before the foot,
>   in the page's order.
> - From 1280px wide it is a pane at the right of the 680px column, its
>   top at the heading's, 272px wide, a hairline on its left, nothing
>   else around it. Narrower, it stays in the column after the choices,
>   a hairline above it.
> - Its line is stacked: a mark, the forge's name in ink (strong), then
>   each detail on a line of its own, muted (temper-flow-muted). Under
>   way the mark is a quiet ring and the details are "Setting up:
>   fetching" and "work/hello", then "Each step" as before, its list in
>   the pane. The green box and the line's "·" separators go.
> - Done, which no journey reaches (the fake box's set up never ends),
>   takes the same shape: a green ✓, "Forgejo on this machine", then
>   "connected as temper" and "work/hello made, in no project yet", each
>   on its own line, with no "Each step" (as today). The ✓ moves from the
>   line's text to the style's mark, as the checklist's does.
> - The rules this replaces are temper.css's "The forge's state above the
>   Model step" and their narrow-width nth-child rules, which a landing
>   removes with them.
>
> Look-request: welcome-model-forge-pane
> Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>

Acceptance:

- `welcome-set-up/07.html` and `08.html`, reprinted, equal the request's pages as text, and
  `overlay.css` stays empty.
- Each of the overlay's rules is in `temper.css`, selectors and declarations as the overlay has
  them, and the rules listed under Steps are gone.
- The render check of `notes/common.md` has run on `07.html` and `08.html` at 1440, 1000 and 420,
  "Each step" open, and on the done, under way and failed states through the gallery; its result is
  in the account.
- The terminal prints "Forge:" after the choices, with the line one readable run of words, in the
  three states.
- Every commit carries `Look-request: welcome-model-forge-pane`; the last holds the account.
- The design text says what the page now does.

## Inputs

- `temper/docs/plan/look-round-2/notes/common.md`: the standing rules. Read it first.
- `temper/docs/design/draft/look-requests.md`, sections 3 to 6: the procedure.
- `temper/docs/design/client/journeys.md`, 7, for the request, and 8 for a stale recording.
- `temper/docs/design/client/README.md`, 3.5, The welcome: the Model step's forge line and its seam.
- `temper/docs/design/client/browser.md`, 2.5's welcome look, and 7.2: the gallery's states.
- `git show 87cddc4453`.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/docs/design/client/README.md`, 3.5: the Model step's forge line and its seam. The line's
  words and states stay; its place and shape change. The test list near 4296, if it says where the
  line reads.
- `temper/docs/design/client/browser.md`: the sentence "the forge's line on Model is a pale green
  box"; 7.2's table gains the gallery's three states.
- `temper/docs/design/client/mockups/STATUS.md`, 1.1, "Where the client does otherwise": a row for
  the pane. The mockup is not edited, as its neighbours say.
- `temper/docs/design/client/journeys.status.md`: its one **Not yet** entry is stale, the journey
  being recorded and printed; drop it and say the status has no gap.
- `temper/tests/client/tests/suite/connect_forge.rs` and `forges_edges.rs`, in the assertions that
  read the line, only for its shape.
- `temper/crates/temper-client-shell-terminal/src/tests/`: no change is expected; a golden the
  line reaches may be rewritten by `SKEIN_CODEGEN_WRITE`, and say so.

Options, each pre-approved:

- If `temper/tools/gallery`'s `FirstRun { step: 7 }` leaves the welcome at `Step::Model`, so that no
  done state draws: leave the worlds alone, draw the done state on a hand-made page for the render
  check (`notes/common.md`), keep the under way and failed gallery states, and say so in the report.
- If a page's node bound in `temper-client-view/src/limits.rs` falls short: raise the derived count
  by what the pane adds, with its comment, and say so. It's derived, not a budget.
- If `.temper-welcome-status`'s two base rules (`temper.css`, 976 and 977) change no computed value
  in the render check, keep them: the request doesn't name them.

## Steps

1. **Read** `notes/common.md`, then `git show 87cddc4453` whole.

2. **The region.** In `pages/welcome.rs`, `Step::Model`, after `connect_model::body` and before the
   foot, and gone from before the heading:

   ```rust
   if let Some(row) = crate::forge::connect::personal(root) {
       b.open(Role::Complementary);
       b.style(look::WELCOME_STATUS);
       b.name(&w.say(Word::WelcomeForgeStep, &[]));
       crate::forge::connect::welcome_line(b, root, w, row);
       b.close();
   }
   ```

   `look::WELCOME_STATUS`'s doc comment says "beside" what it now is.

3. **The line.** `forge/connect.rs`, `welcome_line`, one shape for every state, through one private
   helper:

   ```rust
   /// The forge's line (client/README.md, 3.5): one paragraph named by its words. The browser
   /// stacks its name and details by style; the terminal has neither, so says them on a line.
   fn stacked(b: &mut Builder, w: &Words, line: &[u8], forge: &[u8], details: &[&[u8]], done: bool)
   ```

   - The paragraph's name is the line's words as today: "FORGE · DETAIL · DETAIL". `done` adds
     `look::FLOW_DONE`.
   - `Shell::Browser`: one `Role::Text` with `Run::Strong` for the forge's name, then for each
     detail a `Role::Text` styled `look::FLOW_MUTED`. No "✓", no "·" in its text.
   - `Shell::Terminal`: the paragraph's one text, the line's words, with "✓ " before them when
     `done`: `✓ Forgejo on this machine · connected as temper · work/hello made, in no project yet`.
   - Under way, failed and stopped: details are the connection's words, then "ORG/NAME" when
     `welcome_target` gives one, in today's order, then `each_step`, as today.
   - Done: `WelcomeForgeDoneDetail` splits into two words, "connected as LOGIN" and "ORG/NAME made,
     in no project yet", each a detail; a forge with no first repository has its one detail,
     `forge_connection(row)`, as today. No "Each step".

4. **The rules.** Into `temper.css`, in the place of "The forge's state above the Model step", every
   rule of the commit's `overlay.css`, selectors and declarations as they stand, with the block's
   comment, its `@media (min-width: 1280px)` rule among them. `#app > .temper-welcome { position:
   relative; }` is one of them. Removed:
   - the old block, lines 1317 to 1322;
   - in the narrow media block, 1387 to 1389: the `margin-bottom`, which would beat the overlay's
     margin, and the two `nth-child` rules;
   - 978, `.temper-welcome-status > .temper-flow-done::first-letter`: with no "✓" in the text, it
     would colour the forge's name's first letter.

5. **The tests.** New file `temper/crates/temper-client-view/src/tests/model_forge.rs`, its `mod`
   line in `tests.rs`, built on `connect_forge`'s row helpers (`rows`, `first_rows`, `made`,
   `connected`, `github_rows`):

   - `the_forge_is_a_region_after_the_choices_and_before_the_foot`: in document order, the level-1
     heading, the cards, `Role::Complementary` "Forge" with `WELCOME_STATUS`, the "‹ Back" link;
     nothing of the forge's before the heading; on a way's Details too.
   - `under_way_the_browser_stacks_the_line`: `browser_page_view`; the name whole; the text nodes
     the strong forge, then two `FLOW_MUTED` details; "Each step" inside the region.
   - `done_is_marked_by_style_alone`: `FLOW_DONE`, no "✓" and no "·" in any text of the paragraph,
     two details, no "Each step".
   - `a_connected_github_has_its_one_detail`, and `failed_keeps_its_shape_and_its_steps`.
   - `the_terminal_says_the_line_after_the_choices`: through `skein_ui_text::render`, on `page_view`,
     for under way, done and failed: "Forge:" after the heading and the cards, the line's words in
     one line, indented under it; done with its "✓ "; "Each step" where the state has it.

   In `tests/welcome.rs`, the helpers `before_model`, `model_forge_line` and `line_before_model`
   read the paragraph in the "Forge" region, renamed for it, and each test keeps its assertion of
   the line's name. `model_ticks_the_first_repository_made` reads the new runs. Nothing else in that
   file moves.

6. **The gallery.** In `samples.rs`, three states at `Step::Model { way: None }`, one small helper
   beside `owned_sample`: `welcome-model-forge-under-way` (`FirstRun` step 6),
   `welcome-model-forge-done` (7), `welcome-model-forge-failed` (9). Each has its `Holds` row in
   `tests.rs`: `Part::Named` for `Role::Complementary` "Forge", and "Each step" present or absent.
   Their names are plain, since no mockup draws them (`browser.md`, 7.2).

7. **Reprint** (`notes/common.md`), then diff `07.html` and `08.html` against the commit's.

8. **Render check** (`notes/common.md`, all of it), with `heavy cargo run -p temper-client-gallery --
   ~/src/fabrig/look-round-2/welcome-model-forge-pane/gallery` for the states in words. Look at the
   done state at 1440 and 420 yourself: a green ✓, the forge's name in ink, two muted lines, no
   green first letter.

9. **The terminal:** read the print of the three states, and put it in the account.

10. **The design text:** `README.md` 3.5, `browser.md`, `journeys/welcome-set-up/README.md`'s
    `07.html` line ("its forge's line and each step above the plans"), `mockups/STATUS.md`,
    `journeys.status.md`. `journeys.md`, 9, says nothing of where the line sits, so it stays.

11. **Account and push:** `notes/common.md`, Your checks and Commits. The account has a row for each
    of the overlay's rules, its media rule among them, for the region, the line's markup and the page's
    order, for under way, done, failed, a forge with no repository and the narrow width, and for the
    terminal.

## Report

As `notes/common.md`, Your report: the account whole, the render check's result, the builder and
directory of the screenshots, and the terminal's three prints.
