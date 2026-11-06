# look-round-12: the plan

Lands the look request `model-check-pane` (`ba044d7413`), by `temper/docs/design/draft/look-requests.md`, sections 3 to
5. It reverses part of `look-round-9`. The cut wrote the design: `temper/docs/design/client/README.md`, 3.5, *A model
usable*, *A way's page*, *The try*, *Ready*; `browser.md`, 2.5 and 7.2; `README.status.md`; `mockups/STATUS.md`;
`STATUS.md`; and `draft/model-check.status.md`, whose **Design departs** records the user's word against the draft's 2 to 4.

## Parts and tasks

| Part | Task | Needs | Minutes | What |
|---|---|---|---|---|
| `ready-pane` | `ready-pane` | | 150 | domain, view, stylesheet, gallery, tests |
| `pane-proof` | `pane-proof` | `ready-pane` | 90 | reprints, render check, screenshots, account |

Critical path: `ready-pane`, then `pane-proof`. Two tasks, as the request asks; the second proves what the first lands.
One slice, no walk: `pane-proof` walks the way in a real browser for the flash.

## What the cut decided

- **Where the check lives on Ready:** `ConnectModel` stays on `Welcome { step: Ready }` while its `check` isn't `Idle`, and
  goes as today otherwise. `connect_model::open` doesn't change, so a held sign-in's return drops it as today. No new state
  on `Pages`.
- **Leaving Ready doesn't stop the try,** at no cost: each host reports once a program is declared
  (`jig/docs/design/domain/accounts.md`), and the client stops watching. So nothing keeps the page past Ready.
- **`model_counts`,** in `pages/welcome.rs`, is "an account just declared counts as usable": `decision`, `go_on`, Ready's
  heading and act, the steps' tick. `model_step` keeps `model_usable`, so that a way's page isn't *A model usable* during a
  check.
- ***A model usable* stays,** with its "Continue": ‹ Back from Ready reaches it, and without it that page has no way on. The
  request's "Continue" is the try's answered one. The other is `ui-welcome`'s.
- **A refused declare or check** would otherwise leave Ready saying "Model none yet" with no reason, the way's page being
  gone: its words go under Model's line, from the page Ready keeps.
- **A key moves on once added and usable,** not at its press: a key can be refused, and its way's page says why. A sign-in's
  end is `sign-in-new-tab`'s move.
- **"Sign in again" is a link again,** as 04 draws it, since it now leaves the page it stands on; `Act::SignInAgain` goes.
- **A stored format, a protocol or a public interface:** none changes. `model_counts` and `ConnectModel.tried` are the
  domain's new surface; the gallery's two names move.

## Overlaps with drafts beside this one

Not avoidable: the request moves these lines, and it's `priority`. Each is a catch-up, and `notes/common.md` says how.

- **`sign-in-new-tab`,** on its trunk: `route.rs`'s `welcome_program_progress` (renamed, and the sign-in's move added),
  `domain.rs`'s two calls, `pages/connect_model.rs`, `temper/tests/client/tests/suite/connect_model.rs`,
  `welcome_model.rs`. The one real conflict: its branch and this plan's both change that function.
- **`first-usable-model`,** its `printed-command`: the view's `pages/connect_model.rs`, `journeys/welcome-set-up/**`,
  `journeys/welcome-playground/**`, `journeys/project-landing/**`. Prints are regenerated.
- **`ui-walk-3`,** its `fix-look` and `fix-flow`: `temper.css`, `temper-client-view/src/**`, `temper-client-domain/src/**`,
  `temper/tools/gallery/**`. Rules go under the marker; `names.txt` is regenerated.
- **`chat-reading-c`,** its `reading-close`: `journeys/**`, `journeys.md`, `browser.md`, `mockups/STATUS.md`,
  `client/README.md` and its status, `STATUS.md`, the gallery. Its `reading-browser`: `temper.css`.

## Left out

Nothing of the request is dropped. Deferred, named in the account and in `README.status.md`: the trial and what its facts
feed; Home's line; the accounts row. Those are `temper/docs/design/draft/model-check.md`'s, by the request.
