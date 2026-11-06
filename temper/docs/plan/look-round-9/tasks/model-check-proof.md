+++
id = "model-check-proof"
part = "model-check-proof"
kind = "feature"
needs = ["model-check-box"]
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/docs/design/client/mockups/STATUS.md",
]
effort = "high"
minutes = 70
+++

# The render check and the account of `model-check`

## Goal

Prove the landing against the request by rendering, fix what the render finds, bring the design text in
line, and carry the account in the landing's last commit. Every commit carries `Look-request: model-check`.

Acceptance:

- **The render check,** `temper/docs/design/draft/look-requests.md`, 4, recorded: identical, or each
  difference with its reason.
- **The screenshots** in `~/src/fabrig/look-round-9/model-check/` on your builder, outside any worktree, never
  committed: each page's `-request.png` beside its `-landed.png` at 1440, 1000 and 420, closed and with its
  disclosure open, and each state in words.
- **The account,** `temper/docs/design/draft/look-requests.md`, 5: in the last commit's message, one row per
  overlay rule, per page change and per state the message names, none left open.

## Inputs

- `temper/docs/plan/look-round-9/notes/common.md`.
- `temper/docs/design/draft/look-requests.md`, 3 (steps 5 to 8), 4, 5, 7.
- `temper/docs/design/client/README.md`, 3.5, *The try*.
- `temper/docs/design/client/README.status.md`: its **Not yet** on *The try*, this landing's.
- The request, as the note says, and `model-check-box`'s commits on the trunk.

## Owns and may change

Owns, as `files` lists them.

May also change, each for what the render finds and named in the account:

- `temper/crates/temper-client-view/src/pages/model_check.rs`, its tests, `look.rs` and `words.rs`.
- `temper/tools/gallery/src/**` and reprinted pages, regenerated.
- `temper/docs/design/client/README.md`, 3.5, *The try*, and `README.status.md`'s entry above: what the check
  taught you.

Options, each pre-approved:

- **A state no fact reaches:** flip `model_check::trial` or `last_try_failed` locally, print the gallery state
  named below, screenshot it, and restore; the commit never carries the flip.
- **A state with no gallery name:** a throwaway one, untracked or reverted before the push.

Leave: everything `notes/common.md` says is not built here, and `temper/docs/design/client/greenfield/`, whose
`app.js` still says "Checking Claude Code’s sign-in": its README keeps look requests' changes on top of it.

## Steps

1. **The pages:** from `0bfb77711b`, with the overlay at that commit, each against its gallery state under a
   flip, with the landed stylesheets:
   - `model-check/01.html` against `welcome-model-claude-checking`, `State::Running { at: 1, message: true }`;
   - `02.html` against `welcome-model-connected`, `State::Answered { seconds: 6 }`;
   - `03.html` against `welcome-model-claude-not-yet`, `State::Stopped { at: 2, .. }` with 03's why and
     `Answer::SignIn`.

   The Forge region is the world's: no journey reaches these pages, whose values were written by hand. Compare
   it against `welcome-model-forge-done`'s words, and say so.
2. **How,** at 1440, 1000 and 420 pixels wide, with each disclosure forced open: each element's computed style
   and box, then the screenshots for what styles miss. A difference is fixed, under the trailer, or has its
   reason. Include the "Each step" disclosure in 02, the acts' `align-items` and margins, and at 420 the muted
   words under the title, which the request's words name. "Sign in again" is a button where 03 has a link: its
   computed style and box against the link's.
3. **States in words,** beside the message's words, rendered at the three widths:
   - today's, no flip: `welcome-model-claude-checking`, `welcome-model-claude-not-yet`;
   - ChatGPT and a key, by flips with `First::Plan` and `First::Key`: running, answered, stopped;
   - a reason with no act, `answer: None`: "Try again", "All ways", "Continue anyway";
   - `ConnectModel`, by flips on `connect-model-signin-ready` and `connect-model-program-details`: the box in
     its card's place, its primary as today's;
   - Ready's Model line, flipping `last_try_failed` on `welcome-ready-adopted` and `welcome-ready-none`.
4. **The terminal,** `temper/docs/design/draft/look-requests.md`, 3, step 5: read the prints of the running,
   answered and stopped boxes (`skein_ui_text::render`, or the view's terminal tests): the same lines as text,
   no spinner, headings and order keeping their meaning. Say so in the account.
5. **Design text:** `temper/docs/design/client/mockups/STATUS.md`, its first run: what `mockups/` draws of the
   Claude check that the client no longer does. The journeys' `README.md` files,
   `temper/docs/design/client/journeys.md`, 9, and `temper/docs/design/client/README.md`'s section 1: read them;
   change only a line this makes false.
6. **Check the trailers and that the others stayed out:** `git log
   --format='%(trailers:key=Look-request,valueonly)' origin/main..HEAD | sort -u` names `model-check` alone on
   this draft's commits, and `git diff --stat origin/main...HEAD --
   temper/docs/design/client/journeys/overlay.css temper/docs/design/client/journeys/model-check` is empty. Then
   fmt, clippy on what you changed, its tests, the citations.
7. **The last commit** carries the render check's result and the table in its message, above its trailers; push.

At the least, these rows defer or differ, each with its home:

| Item | Result |
|---|---|
| The trial: its task and charter, how it selects its model, what ends it, listed nowhere, who pays | deferred, to `temper/docs/design/draft/model-check.md`, 2 and fabrig/temper; `draft/model-check.status.md` |
| Running at its later steps, live | drawn by `model_check::status`; the fact deferred, the draft's 3, `model_check::trial` the one function it feeds |
| "one short message" | differs: said only once a message goes, so not over today's host check; the draft's 2 turns it on |
| Answered, "answered in N s", "Each step" | drawn; the fact deferred, the draft's 3; a model usable shows today's status |
| It does not move on by itself | deferred, the draft's 3: `welcome_program_progress` still moves to Ready; `README.status.md` |
| A stop past the first step and its reason in the person's terms | drawn; the fact deferred, the draft's 4 (`RunWait`) |
| `WelcomeClaudeNotYet`'s words | differs: "Sign in again, then try again." for "Run the command above, then press I’ve signed in.", the box standing in the panel's place |
| "Sign in again" | differs: a button, where 03 has a link, its address being the page it stands on |
| A reason with no act | "All ways"; differs: "Continue anyway" stands after it, the draft's 4, it keeps no one out |
| The list's accessible name and hidden state words | differs: as Forge's list has them, which the terminal reads |
| ChatGPT or a key | drawn; the fact deferred, the draft's 2 and 3 |
| `ConnectModel` | drawn in its card's place; the fact deferred, the draft's 2 |
| Ready's "didn’t answer when tried" | drawn by `ready_note`; the fact deferred, the draft's 4, `model_check::last_try_failed` |
| Leaving or "‹ Back" cancels a trial | deferred, the draft's 2 |
| The Forge region in 01 to 03 | the world's: no journey reaches these pages, whose values were written by hand |

## Report

The table, the render check's result, the screenshots' directory and the builder (`hostname`), and what
`notes/common.md`, Your report, asks.
