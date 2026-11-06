# look-round-9: the plan

From `inputs/request.md`: land the look request `model-check` (`0bfb77711b`), by
`temper/docs/design/draft/look-requests.md`, sections 3 to 5, the view's and stylesheet's part alone.
fabrig/temper builds the behaviour (`temper/docs/design/draft/model-check.md`) after `first-usable-model`,
and takes this look by catch-up. The trunk is `look-round-9/integration`.

**The check before the cut:** on `origin/look-requests`, `sign-in-new-tab` (`look-round-7`'s), then
`model-check`, then `configure-temper-own-way` (`look-round-8`'s). Only the middle one lands here.

The design is cut ahead of the tasks: `temper/docs/design/client/README.md`, 3.5 (*A way's page*'s Claude and
the new *The try*); `browser.md`, 7.2; `README.status.md`'s new **Not yet**; `draft/model-check.status.md`.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `model-check-box` | | high, 100 | the words, one box and one fact function for both pages, the rules, the note and alert gone, `Act::SignInAgain`, the gallery |
| 2 | `model-check-proof` | 1 | high, 70 | the render check, the screenshots, the mockups' status, the account |

## Critical path

`model-check-box`, `model-check-proof`: ~170 implementer-minutes, then the trunk's end check.

## What the cut settled

- **What is honest today:** the host's check is the first step, so `Declaring | Checking` draws the box
  Running at its first step and `NotYet` draws it Stopped there. Every other state is drawn and tested,
  and reached by nothing: `model_check::trial` is the one function the behaviour plan feeds, and
  `last_try_failed`, false, the other. The proof flips them locally, as `look-round-7` did `tab_refused`.
- **"one short message" isn't said over the host's check,** since no message goes then. `State::Running`'s
  `message` is false until the draft is built. The request's 01 says it at step 2, a state the proof renders.
- **`WelcomeClaudeNotYet`'s second sentence changes,** to "Sign in again, then try again.": the box takes the
  panel's place, so there's no command above and no "I’ve signed in" to press. Its first sentence, which a world
  reads, stands.
- **"Sign in again" needs an act.** Its address is the page it stands on, so a link would do nothing:
  `Act::SignInAgain` returns the check to `Idle` and brings the panel back. It's a button, where 03 has a link.
  The act is the domain's own, on no wire.
- **A reason with no act keeps "Continue anyway".** The request's words say "'Try again' and 'All ways'"; the
  draft's 4 says a failure keeps no one out. Taken as the act's place alone.
- **The list is `flow::progress`'s:** an accessible name and hidden state words the pages lack, as Forge's list
  has them, because the terminal reads them; a new `Mark::Stopped` styles `temper-welcome-check-failed`.
- **WHO on a key is the plan's name:** "ChatGPT answers", as the request and the draft have it, not "OpenAI".
- **It still moves on by itself.** `welcome_program_progress` is behaviour: a usable model goes to Ready until the
  draft is built, and the account names it. A model usable shows today's status with no trial behind it.
- **`greenfield/`** stays: its README keeps look requests' changes on top of it. `mockups/STATUS.md` is the
  proof task's.

## Chained behind `look-round-7`

It changes the same Model step and is landing now. No plan key orders plans, so the tasks start from this
trunk and take it by catch-up, keeping its words; `notes/common.md` draws the line between the two. The design's
edits merge cleanly with its trunk as of this cut (`8a460e18e7`). If `model-check-box` starts first, the catch-up
falls on `welcome_model.rs`, `connect_model::body`, `words.rs` and `look.rs`.

## Overlaps it could not avoid

- `temper-client-view/src/pages/welcome_model.rs`, `connect_model.rs`: `look-round-7/sign-in-wait-page`.
- `temper-client-view/src/words.rs`, `look.rs`: `look-round-7/sign-in-wait-page`, `ui-walk-large/needs-proposal`,
  `look-round-6/*`, `chat-reading-a/chat-live`; numbers start 40 above the highest in use.
- `temper-client-view/src/tests/**`: the same; this plan's tests go in a new file.
- `temper-client-shell-web/temper.css`: `look-round-7`, `look-round-6/*`, `ui-rename-focus/rename-row`; this plan's
  rules go under `ui: welcome`.
- `temper/tools/gallery/src/tests.rs`: `look-round-7`, `look-round-6/call-line`; regenerate on a catch-up.
- `temper/docs/design/client/README.status.md`: `look-round-7`, `look-round-6/*`, `chat-reading-a/*`; this plan's entry
  sits mid-list so that it merges.
- `look-round-8`, `first-usable-model`, `plainer-panels` and `ui-owner-menu` are cuts in flight, their files unknown.

No stored format, protocol or public wire form changes. `Act::SignInAgain` and `flow::Mark::Stopped` are their crates'
own, read by the view and its tests.
