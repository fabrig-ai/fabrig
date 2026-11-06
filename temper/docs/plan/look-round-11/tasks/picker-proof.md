+++
id = "picker-proof"
part = "proof"
kind = "feature"
needs = ["picker-menu"]
files = [
  "temper/docs/design/client/journeys/chat-draft/**",
  "temper/docs/design/client/journeys/welcome-playground/**",
  "temper/tests/client-browser/tests/browser_all/browser_model_picker.rs",
  "temper/tests/client-browser/tests/browser_all/main.rs",
]
effort = "high"
minutes = 120
+++

# The reprints, the browser story, the render check and the account of `model-picker`

## Goal

Reprint the journeys that print the composer, write the picker's browser story, prove the landing against the request by
rendering, fix what the render finds, and carry the account in the landing's last commit. Every commit carries
`Look-request: model-picker`.

Acceptance:

- **The journeys:** `chat-draft` and `welcome-playground`, and any other the reprint changes, regenerated with `cargo run -p
  temper-client-journeys -- again NAME` through `heavy`; `temper-client-journeys`' tests green. A re-record that stops on "a key mark
  caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the
  recording driver and say so. `model-check` is re-recording `chat-draft` too: a conflict at catch-up is regenerated, not merged.
- **The browser story** `the_model_picker_at_the_composer`, compiled by `heavy cargo check -p temper-client-browser-world --tests`,
  **not run**: one menu open at 420, a model pressed on a started chat, an effort pressed.
- **The render check,** `temper/docs/design/draft/look-requests.md`, 4, recorded: identical, or each difference with its reason.
- **The screenshots** in `~/src/fabrig/look-round-11/` on your builder, outside any worktree, never committed: each page's
  `-request.png` beside its `-landed.png` at 1440, 1280, 1000 and 420 wide, the menu open and closed, and each state in words.
- **The account,** `temper/docs/design/draft/look-requests.md`, 5: in the last commit's message, one row per overlay rule, per page
  change and per state the message names, none left open, saying that 07 and All models' "new" and link stay for step 2.

## Inputs

- `temper/docs/plan/look-round-11/notes/common.md`.
- `temper/docs/design/draft/look-requests.md`, 3 (steps 5 to 8), 4, 5, 7.
- `temper/docs/design/client/journeys.md`, 8.
- `temper/docs/design/client/README.md`, 3.5, *The work's account and model*.
- `temper/docs/design/draft/model-catalogue.status.md`: the deferred rows' home.
- The request, as the note says, and `picker-menu`'s commits and report on the trunk.

## Owns and may change

Owns, as `files` lists them.

May also change, each for what the render finds and named in the account:

- `temper/crates/temper-client-shell-web/temper.css`; `temper/crates/temper-client-view/src/choice/picker.rs`, `choice.rs`,
  `pages/session.rs`, `look.rs`, `words.rs`, and their tests.
- `temper/tools/gallery/src/**`, `temper/tools/journeys/src/**` (a mark's wait), and reprinted pages, regenerated.
- `temper/docs/design/client/README.md`, 3.5, 5.5, `README.status.md`'s entries, `browser.md`, 7, and `mockups/STATUS.md`: what
  the check taught you. The journeys' `README.md` files and `temper/docs/design/client/journeys.md`, 9: a line this makes false.
  `temper/docs/design/draft/model-catalogue.status.md`: a deferred row.

Options, each pre-approved:

- **Going to 420 in the story:** if `skein_fake_person_browser`'s steps hold no way to narrow the window, the story opens the
  menu at the tier's own size and the report says so; the render check is what proves 420. Build no tool for it.
- **A state with no gallery name:** a throwaway one, untracked or reverted before the push.
- **A setup that no longer walks** (`temper/docs/design/client/journeys.md`, 8.2) stops `again`, naming its line: edit it; no
  recording changes for it.

Leave: what `notes/common.md` says isn't landed; `temper/docs/design/client/greenfield/`, whose pages look requests go on top of.

## Steps

1. **Reprint** by `temper/docs/design/client/journeys.md`, 8. Find the rest with `cargo run -p temper-client-journeys` through
   `heavy`, whose diff names each journey whose pages change.
2. **The story,** a new `browser_all/browser_model_picker.rs` with a `mod` line in `main.rs`, on the tier's helpers as
   `browser_pages.rs` has them, so that it stays off `browser-budget`'s lines there. On the playground's scripted model, or a
   fake box where it needs a model with efforts: open "Model", see the note; press a model, see the closed line change; press
   an effort, see it change. Keep the `details.temper-model[open]` `Keep`s `byline_steps` has.
3. **The pages:** from `47c23ff6ed`, with the overlay at that commit, each against its gallery state with the landed
   stylesheets:

   | Page | Gallery state |
   |---|---|
   | `model-picker/01.html` | `chat-model-open`, closed |
   | `02.html` | `chat-model-open` |
   | `03.html` | `chat-model-both` |
   | `04.html` | `chat-model-all` |
   | `05.html` | `configure-model-open`, and closed |
   | `06.html` | `chat-started-model-open` |

   These pages are hand-made: their values were written by hand, and no reprint equals them.
4. **How,** at 1440, 1280, 1000 and 420 pixels wide, with the menu open and closed: each element's computed style and box, then
   the screenshots for what styles miss. A difference is fixed, under the trailer, or has its reason. Include: the composer's
   centre with the thread's gutters, with no overflow hidden; the menu over the conversation, not clipped, at 420 its row's
   whole width, its long list scrolling; the hint gone on a phone, the picker kept; the tick and the raised effort; Enter in "Find
   a model" sends nothing; a press outside and Escape fold it.
5. **States in words,** beside the message's words, rendered at the widths: `chat-model-two-accounts`, `chat-model-sign-in-again`,
   `chat-model-fixed`, `chat-started-model-changing`, `chat-started-model-refused`, and no model yet, `chat-new`'s state with none.
6. **One press, in a browser:** walk a started chat on a fake box, by a throwaway driver: a press applies with no other press,
   the closed line follows, and the turn after it runs that model. Say what ran, or why it couldn't.
7. **The terminal,** `temper/docs/design/draft/look-requests.md`, 3, step 5: read the prints of a new chat, a chat, Configure temper
   and the states: the same list as lines, effort beneath; headings and order keeping their meaning. Say so in the account.
8. **Design text:** read `temper/docs/design/client/README.md`, 3.5 and 5.5, `browser.md`, 7, the journeys' `README.md` files,
   `journeys.md`, 9, and `mockups/STATUS.md`; change only a line this makes false. `README.status.md`'s **Not yet** on *The work's
   account and model* is gone by now: if not, remove it. If `temper/docs/help/` says anything of the byline or "Change model", put
   it right.
9. **Check the trailers and that the others stayed out:** `git log --format='%(trailers:key=Look-request,valueonly)'
   origin/main..HEAD | sort -u` names `model-picker` on this draft's commits, and `git diff --stat origin/main...HEAD --
   temper/docs/design/client/journeys/overlay.css temper/docs/design/client/journeys/model-picker` is empty. Then fmt, clippy on
   what you changed, its tests, the citations.
10. **The last commit** carries the render check's result and the table in its message, above its trailers; push.

At the least, these rows defer or differ, each with its home:

| Item | Result |
|---|---|
| Settings, Models (07): its tab, list, switches, "Read again", "Add a model by name", a line sending a change of default to Configure temper | deferred, to `temper/docs/design/draft/model-catalogue.md`'s steps 2 and 3; `model-catalogue.status.md` |
| "Couldn't read OpenAI's list · 2 hours ago", "estimated prices" | deferred, the same; both are 07's |
| "new" on All models (04) | deferred, step 3: nothing reads a provider's list; `model-catalogue.status.md` |
| All models' link | differs: goes to Accounts, and says "Accounts, in Settings" where 04 says "Choose what the list shows, in Settings", since no page chooses that yet; `model-catalogue.status.md` |
| `/* Settings' list of models. */`'s rules | deferred with 07 |
| "All models" on the stock | landed, but drawn only where a configured model has `picker` false: all seven have it true until step 2 |
| "Manage usage", which the byline's panel held | differs: on Accounts and an agent's task's line, since 03 draws none in the menu |
| "T2" | differs in the terminal alone: its heading says it |
| `.temper-model > summary` | differs: its name hides as the byline's did, as the view's disclosure has a name and a description where the page has `aria-label` |
| An agent's task's page, no page drawn | the same picker beside its line, which keeps its account; `README.md`, 5.9 |
| The first run of a model its provider refuses | landed as today: its failure in the conversation |
| Which models, the default order, the list in the store | not the look's, `model-catalogue.md`; landed by `model-catalogue-1` or steps 2 and 3 |

## Report

The table, the render check's result, the screenshots' directory and the builder (`hostname`), that the story was compiled and not
run, and what `notes/common.md`, Your report, asks.
