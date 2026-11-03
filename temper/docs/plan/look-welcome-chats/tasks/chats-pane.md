+++
id = "chats-pane"
part = "pane"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/pages/chats.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/choice.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/**",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/*.txt",
  "temper/docs/design/client/journeys/welcome-playground/02.html",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/journeys.status.md",
]
effort = "high"
minutes = 120
+++

# The chats page's pane, the prompt beside it

## Goal

Draw the chats page as the look requests ask, for every project: a pane at the head of the main
holding the new chat's bar, the playground's start, the chats under the level-1 heading "Chats"
and the playground's banner; beside it the composer alone, its line and chooser under it; no
accounts' banner in the playground's project while the playground runs. Then reprint the
journeys, so `welcome-playground/02.html`'s main equals the requests' but where this spec says
(`temper/docs/design/client/README.md`, 3.5, 5.1, 5.6; `temper/docs/design/client/journeys.md`,
7). The banner's switcher and "Set up temper" are the next task's (`banner-switcher`).

Acceptance:

- The chats page's main is the tree of `temper/docs/plan/look-welcome-chats/notes/contracts.md`,
  "The chats page's tree", for every project: no pagebar, no level-2 "Start a chat", no `PAGE`
  style on its main, the playground's invitation region gone, "Scripted model" gone from the
  chatbar.
- On the Chats address, the playground's banner (`frame::playground_banner`'s region
  "Playground") is the pane's last child and is drawn nowhere else on the page; on every other
  address it is drawn where it is today.
- No accounts' banner (`choice::banner`) while the person is in the playground's project and the
  playground runs or is leaving; elsewhere as today.
- The page's title is "Chats · temper" (its main's level-1 heading).
- `heavy cargo run -p temper-client-journeys` reprints, and `git diff HEAD 7fba5d0fe3 --
  temper/docs/design/client/journeys/welcome-playground/02.html` then holds only: the banner (the
  next task's), `<h1`/`</h1` where the requests have `<h2`/`</h2` for "Chats", the suggested
  requests' slot after the chooser, and the order of a tag's attributes where the printer orders
  them otherwise. Any other hunk is a defect of this task, or a question.
- The tests below pass; the status entries below are gone.

## Inputs

- `temper/docs/plan/look-welcome-chats/notes/common.md` and `notes/contracts.md` ("The style
  tokens", "The chats page's tree").
- `temper/docs/design/client/README.md`, 3.5: the `Chats` row, "The new chat's bar", "The chats
  list"; 5.1: "the accounts' banner", "the playground's banner", "the main"; 5.6: "The
  playground's start", "The context line"; 11: "The playground's suggested requests".
- `temper/docs/design/client/journeys.md`, 5.1 (the reprint), 6 (its tests), 7.

## Owns and may change

- `temper/crates/temper-client-view/src/pages/chats.rs`: `main`, `start`, `list`, `suggestions`.
  `main` gains the forge's bindings, to draw the playground's banner in the pane:

  ```rust
  pub(crate) fn main(
      b: &mut Builder,
      root: &Domain,
      words: &Words,
      bindings: &mut Bindings,
      forge: &mut crate::forge::bindings::Bindings,
      text_limit: u32,
      times: &mut Times,
  )
  ```

  The pane is `Role::Complementary`, named by a new word "Project", styled `look::SIDEBAR`, the
  main's first child (the title readers take the first `h1` inside `main`). In it, in order:
  - the new chat's bar, `look::CHATBAR`, its one slot "New chat" and its status: in the
    playground's start (`playground_start` is `Some`), today's three words; else
    `Word::PlaygroundStarting` while `root.forge().playground_starting()`; else `Word::Ready`
    while `ready_to_start(root)`; else no status node;
  - in the playground's start, the pane's context line: a new private `fn context` in
    `chats.rs` drawing `look::CONTEXT` > paragraph > `look::REPOSITORY` text OWNER/NAME
    (`Word::PlaygroundRepository`), under the same conditions as `chat::context` (the playground
    read, running, its project the open one); `chat::context` stays as it is, for a chat's page;
  - in the playground's start, once it failed, the alert and "Try again"
    (`Act::RetryPlayground`), as `suggestions` draws them today;
  - `list`'s heading slot, its heading now level 1 and `Word::Chats` ("Chats"), its filters, the
    list (still named "Your chats") or its empty paragraph, "More closed chats";
  - `crate::frame::playground_banner(b, root, words, forge)`, unchanged.

  Then, the main's next children, only while `me` and a project are there, as today: the
  `look::ENTRY` slot (its text area gains `b.placeholder` of a new word "Describe a change"; the
  hint's words become "enter to start · shift+enter for newline"), then
  `crate::choice::composer(...)`, then, in the playground's start alone, a slot styled
  `look::INVITATION` holding today's list "Suggested requests" of cue buttons with today's
  disabled rule. The region "What shall we change?", its heading and its two paragraphs go, with
  their words (`PlaygroundHeading`, `PlaygroundRequest`, `PlaygroundScript`) if nothing else
  uses them; a phrase id goes with its word and is not reused.
- `temper/crates/temper-client-view/src/root.rs`: on the Chats address the playground's banner is
  not drawn before the main (the chats page draws it); `chats::main`'s call passes
  `&mut view.forge`.
- `temper/crates/temper-client-view/src/choice.rs`, `banner`: returns also while the open
  project is the playground's (`root.forge().playground()`'s `project`) and the playground runs
  or is leaving (`playground_running()`, `leaving()`).
- `temper/crates/temper-client-view/src/look.rs`: `SIDEBAR`, token 158, `temper-sidebar`
  appended to `NAMES.styles` at its index (if 158 is taken on the trunk, the next free number,
  and say so).
- `temper/crates/temper-client-view/src/words.rs`: the two new words, the hint's words, the
  removed words, each with its phrase id as the file keeps them.
- `temper/crates/temper-client-view/src/tests/**`: the tests below.
- `temper/crates/temper-client-shell-terminal/src/tests/golden/*.txt`: regenerated
  (`SKEIN_CODEGEN_WRITE=1` on `render.rs`'s golden test). `first_run.txt` moves;
  `chat_delegation_*.txt` move only where the accounts' banner's rule reaches them (a chat's page
  in the playground's project); any other golden moving is a question.
- `temper/docs/design/client/journeys/welcome-playground/02.html` (and `skein-ui.css`,
  `journey.js` beside it if the reprint rewrites them identically): by the reprint alone.
- `temper/docs/design/client/README.status.md`: the two sentences "3.5, 5.1, 5.6: the chats
  page's pane, ... every main." and "5.1: no accounts' banner in the playground's project ...
  `choice.rs` draws it there." go; "5.1: the switcher and ..." stays, for `banner-switcher`.
  `journeys.status.md`: its 9 entry
  goes; the file then says `No gap.`.

Reader files, edits allowed (each only for this change; list them in the report):

- `temper/tools/gallery/src/tests.rs`: the holds of `chats-first-run-5`, `chats-first-run-1` and
  `chats-empty` that read the invitation's heading or the level-2 "Your chats" read the level-1
  "Chats" and the slot of suggested requests instead.
- `temper/tests/client/tests/suite/open_chat.rs` (`:57`, `:61`) and `suite/chat.rs` (`:92`):
  `Find::named(Heading 2, "Your chats")` becomes the level-1 "Chats"; the list keeps its name.
- `temper/tests/client-browser/tests/browser_all/browser_task_pages.rs` (`:30`-`:32`): the
  invitation's `h1` becomes "Chats"; its press of a suggested request stays.
  `browser_all/browser_pages.rs:64`: the selector's `[aria-label='Your chats']` becomes
  `[aria-label='Chats']`. These are the browser tier's: check they compile (`heavy cargo check -p
  temper-client-browser-world --tests`), never run them.

Pre-approved, with their defaults:

- **Tests that encode the old page** (the survey's list: `tests/playground_start.rs` `:9`,
  `:85`, `:105`, `:224`, `:371`; `tests/playground.rs` `:7`, `:118` if it moves, `:312`;
  `tests/chats.rs` `:286`, `:459`, `:484`, `:727`, `:835`; `tests/choice.rs:1291`): rewrite each
  to the design's page, keeping what it proves where it still holds (an order, a disabled cue,
  the ways' scripted provider left out, now on a page where the accounts' banner shows, a
  project other than the playground's). A test whose subject is gone (the invitation's words) is
  removed, and the report says which.
- **`Act::Suggest` and the cues stay** in `temper-client-domain`: the suggestions are drawn.
- **A test that reads a Region "Playground" before the main** on a page other than Chats stays
  as it is; on Chats it finds the region inside the pane.
- **The `INVITATION` token stays**, styling the suggestions' slot.
- If `chats::main` then holds more than clippy's argument bound, a scoped
  `#[expect(clippy::too_many_arguments, reason = "...")]` on it.

## Steps

1. Read the requests' tip page and the diff (`notes/common.md`), and the survey above.
2. The view: `chats.rs`, `root.rs`, `choice.rs`, `look.rs`, `words.rs` as above. Narrow test
   first: `heavy cargo nextest run -p temper-client-view --lib -E 'test(/playground_start/)'`.
3. The view's tests, as above; then `-p temper-client-view` whole.
4. The terminal golden: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p
   temper-client-shell-terminal -E 'test(/golden/)'`, then without it; read its diff.
5. The reprint: `heavy cargo run -p temper-client-journeys`; read `02.html`'s diff against the
   requests' (acceptance); `heavy cargo nextest run -p temper-client-journeys`.
6. The readers: `-p temper-client-gallery`, `-p temper-client-shell-web`, `-p
   temper-client-world` (`temper/tests/client`), and the browser tier's compile.
7. fmt, clippy on the packages you changed, old forms, citations, the status entries.
8. Commits: the view and its tests; the golden and the reprint, its message ending with these
   trailer lines, one each, in this order:

   ```
   Look-request: playground-without-account-banner
   Look-request: terminal-chats-experiment
   Look-request: chats-left-pane-experiment
   Look-request: chats-pane-head-experiment
   Look-request: pane-playground-note-experiment
   Look-request: pane-note-after-chats-experiment
   Look-request: pane-repository-only-experiment
   ```

   and, in its body, why the print still differs from the requests' page: the level-1 "Chats"
   (the requests' title is the main's level-1 heading's), the suggested requests kept (the
   user's word, `README.md`, 11), the attribute order, the banner (`banner-switcher`). Push.

## Report

`notes/common.md`'s and the implementer's text's. Beside them: the tests rewritten and removed,
each in a line; the reader files edited; `02.html`'s remaining diff against the requests' page,
hunk by hunk with its reason.
