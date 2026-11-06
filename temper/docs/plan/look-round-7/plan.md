# look-round-7: the plan

From `inputs/request.md`: land the look request `sign-in-new-tab` (`c725deec96`), by
`temper/docs/design/draft/look-requests.md`, sections 3 to 5, the view's and stylesheet's part alone.
fabrig/temper builds the behaviour (`temper/docs/design/draft/sign-in-new-tab.md`) after this, and
takes this look by catch-up. The trunk is `look-round-7/integration`.

**The check before the cut:** on `origin/look-requests`, `chat-work` (`look-round-6`'s), then
`sign-in-new-tab`, then `configure-temper-own-way`, held for the user. Only the middle one lands.

The design is cut ahead of the tasks: `temper/docs/design/client/README.md`, 3.5 (`ConnectModel`'s
*While it waits*, *From another computer*, *The paste*; the welcome's card and *The sign-in under
way*), 5.5's Choose and section 11; `browser.md`, 7.2; `README.status.md`'s new **Not yet**;
`draft/sign-in-new-tab.status.md`.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `sign-in-wait-page` | | high, 110 | the words, one waiting function for both pages, the rules, the entry and `ModelStep::Paste` gone, the gallery, the reprint |
| 2 | `sign-in-wait-proof` | 1 | high, 70 | the render check, the screenshots, the mockups' status, the account |

## Critical path

`sign-in-wait-page`, `sign-in-wait-proof`: ~180 implementer-minutes, then the trunk's end check.

## What the cut settled

- **The words lead the behaviour for a while.** The card says "in a new tab" and the note "in the new
  tab" while a press still leaves in its own tab. The request asks this order; `README.status.md` says
  so, and the account names it.
- **The domain's way to hold a page stays.** `ModelStep::Paste` and the card's entry go, as the request
  says, but `Act::SignInElsewhere` and `ConnectModel::elsewhere` stand with no entry in the view: they
  are the behaviour plan's to take, and they keep the paste's tests and gallery states reaching a page
  that doesn't leave. Until that plan lands, a browser on another computer presses Continue, leaves,
  comes back by hand, and finds the note's folded paste.
- **The view has no fact of a tab refused.** Its drawing lands, behind `sign_in_wait::tab_refused`,
  false: the one function the behaviour plan feeds. The render check flips it locally.
- **The terminal** can't say "in the new tab": it reads "ChatGPT’s sign-in", draws no link, and prints
  the address by `Notice::Leave`, as today.
- **A page come back** holds no address, so it draws no "Open it again" (the draft's 3 gives a reload
  its source). What it can't honestly draw it leaves out.
- **`ConnectModel`'s come-back** draws the same note and folded paste, where it drew the paste unfolded
  under "Go forward to that page": the request's "the same note, link and disclosure" and "a reload
  while it waits: the same page".
- **`greenfield/`** stays: its README keeps look requests' changes on top of it. `mockups/STATUS.md`
  is the proof task's.
- **The card's words on `ConnectModel`** ("In a browser · about a minute") stand: the request names the
  welcome's line alone.

## Overlaps it could not avoid

- `temper-client-view/src/words.rs`: `ui-walk-large/home-true`, `ui-walk-large/needs-proposal`.
- `temper-client-shell-web/temper.css`: `ui-rename-focus/rename-row`, in another block; this plan's rules
  go under `ui: welcome`.
- `temper/tools/gallery/**`: `ui-walk-large/home-true`, `ui-final-walk/fix-look`; regenerate on a catch-up.
- `temper/docs/design/client/journeys/**`: `ui-walk-large/needs-proposal`, `proposal-projects`; reprint on a
  catch-up.
- `look-round-6`, `chat-reading-a` and `chat-reading-d` are cuts in flight, their files unknown; `ui-final-walk`'s
  merged `fix-flow` and `fix-look` touched the view and domain but stand on their own trunk.

No stored format, protocol or public wire form changes. `ModelStep` is the domain crate's own, its
readers the view and its tests.
