+++
id = "welcome-forge-busy-bar"
part = "welcome-forge-busy-bar"
kind = "feature"
needs = ["ready-account-email"]
files = [
  "temper/crates/temper-client-view/src/forge/connect.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-view/src/tests/model_forge.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/journeys/welcome-set-up/07.html",
  "temper/docs/design/client/journeys/welcome-set-up/08.html",
]
effort = "high"
minutes = 90
+++

# Land welcome-forge-busy-bar: a moving bar under the forge's set up

## Goal

Land the look request `welcome-forge-busy-bar`, commit `95532806d8`, whole: under the forge's line
in its pane, while the set up runs, an indeterminate bar; the line's details swap, the repository
first; done or failed, no bar; with reduced motion it stands still. The pane is the first landing's,
on the trunk.

The request's message, whole:

> Look request (temporary): a moving bar under the forge's set up
>
> The person wants the set up's line to show it is alive while it runs
> (2026-11-04): "Setting up: fetching" alone can sit unchanged for a
> minute while Forgejo downloads.
>
> - Under the forge's line in its pane, while the set up runs, a bar: a
>   3px track in the hairline colour, a third of it in ink sliding
>   across, over and over (1.4s). It is indeterminate: no step says how
>   far it is, so the bar shows that work goes on, not how much is left.
>   With reduced motion it stands still, a faint full track.
> - It is a progress node (skein's Role::Progress, `role="progressbar"`
>   with no value, which is indeterminate), named by the line's status
>   ("Setting up: fetching"), styled temper-flow-busy, after the line's
>   paragraph and before "Each step".
> - So that the bar sits right under the status, the line's details
>   swap: the repository first ("work/hello"), then "Setting up:
>   fetching".
> - Done, or failed, there is no bar.
>
> Look-request: welcome-forge-busy-bar
> Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>

Acceptance:

- `welcome-set-up/07.html` and `08.html`, reprinted, equal the request's pages as text, and
  `overlay.css` stays empty.
- Each of the commit's overlay rules is in `temper.css`, its `@keyframes` and reduced-motion rule
  among them.
- The render check of `notes/common.md` has run at 1440, 1000 and 420 with "Each step" open, once
  more with reduced motion, and on the under way, done and failed gallery states.
- Under way there is one bar, in the browser; done, failed, and on GitHub's waits there is none.
- The terminal prints no bar and says the line in its new order.
- Every commit carries `Look-request: welcome-forge-busy-bar`; the last holds the account.

## Inputs

- `temper/docs/plan/look-round-2/notes/common.md`: the standing rules. Read it first.
- `temper/docs/design/draft/look-requests.md`, sections 3 to 6.
- `temper/docs/design/client/README.md`, 3.5, The welcome: the forge's line, as the first landing
  left it, and the greenfield list near line 2514, which quotes its order.
- `temper/docs/design/client/browser.md`, the welcome's look.
- `skein/docs/design/ui.md`, 3.1, and `skein/docs/design/ui-text.md`'s line for a progress node.
- `git show 95532806d8`.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/docs/design/client/README.md`, 3.5, the greenfield list near line 2514, and the test list
  near line 4296: the line's order, "FORGE · ORG/NAME · CONNECTION", and the bar.
- `temper/docs/design/client/browser.md`, the welcome's look: a sentence for the bar.
- `temper/docs/design/client/mockups/STATUS.md`, 1.1: the pane's row says the bar and the order.
- `temper/tests/client/tests/suite/connect_forge.rs` and `forges_edges.rs`: the assertions that read
  the line, in its words' order only.
- `temper/tools/gallery/src/samples.rs`: only if a state needs it.

Options, each pre-approved:

- **The terminal has no bar.** `skein-ui-text` prints a progress node as `NAME: TEXT`, so a textless
  one reads `Setting up: fetching: ` under a line that says the same. The bar is the browser's alone
  (`Shell::Browser`), and the line says it in the terminal. The account has its row, "differs, and
  why".
- **GitHub's waits have no bar.** `connect::underway` is true for a GitHub forge waiting on an
  accepted installation, which waits on the person: no set up runs. The bar is
  `connect::underway(row) && row.kind != Kind::GitHub`, in one private `fn busy(row: &Row) -> bool`
  in `forge/connect.rs`. The account says so.
- **One order:** the swap applies to every state whose line holds both details, failed among them,
  since they share `Word::WelcomeForgeLine`. Done keeps its order.
- If the reprint's attributes on the bar come in another order than the request's page, the view's
  is right: the account says "differs, and why".

## Steps

1. **Read** `notes/common.md`, then `git show 95532806d8` whole.
2. **The token:** `look::FLOW_BUSY`, token 173, `b"temper-flow-busy"` at the end of `NAMES`' styles,
   after `temper-welcome-project`; `tests/look.rs`'s `FLOW_NAMES` gains it, its length with it. If
   another plan has taken 173 on the trunk, take the next free one.
3. **The swap:** in `welcome_line`, the details are the target, then the connection's words, in the
   paragraph's name and in its spans.
4. **The bar:** between the paragraph and `each_step`:

   ```rust
   if busy(row) && w.shell == Shell::Browser {
       b.open(Role::Progress);
       b.style(look::FLOW_BUSY);
       b.name(&forge::words::forge_connection(row));
       b.close();
   }
   ```

5. **The rules:** into `temper.css`, after the forge's state block, every rule of the commit's
   `overlay.css` that it adds, selectors and declarations as they stand, with the comment.
6. **The tests,** in `tests/model_forge.rs`:
   - `the_bar_sits_between_the_line_and_each_step`: `browser_page_view`; one `Role::Progress`, styled
     `FLOW_BUSY`, named by the connection's words, no text, after the paragraph, before the
     disclosure, inside the "Forge" region; for fetching, preparing, starting, `Connecting`, and a
     first repository making.
   - `done_failed_and_a_waiting_github_have_no_bar`.
   - `the_line_says_the_repository_before_the_status`: the name and the spans, under way and failed.
   - `the_terminal_says_the_status_with_no_bar`: `skein_ui_text::render` on `page_view`: the line's
     words in their order, and no `Setting up: fetching: `.

   In `tests/welcome.rs`, the expected names only (near 249, 324 to 339, 357). In
   `temper/tools/gallery/src/tests.rs`: `welcome-model-forge-under-way` holds `Role::Progress`
   "Setting up: preparing"; `-done` and `-failed` have it absent.
7. **Reprint,** then diff `07.html` and `08.html` against the commit's.
8. **Render check** (`notes/common.md`). The slider's `transform` moves, so compare it as no
   difference; run one more pass with `--force-prefers-reduced-motion`, where the faint full track
   stands still, and keep a screenshot of it (`07-1440-reduced-landed.png`).
9. **The terminal:** read the print of the three states; put it in the account.
10. **The design text:** as `Owns and may change` lists. `journeys/welcome-set-up/README.md` and
    `journeys.md`, 9, don't name the line's order: leave them.
11. **Account and push:** rows for each of the commit's overlay rules, the keyframes and the
    reduced-motion rule; the bar's node; the swap; done and failed; the narrow width; the terminal;
    GitHub's waits.

## Report

As `notes/common.md`, Your report, with the terminal's prints.
