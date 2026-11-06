# sign-in-new-tab: the plan

A plan's sign-in moves to a new tab, and temper's page stays and waits (`temper/docs/design/draft/sign-in-new-tab.md`).

## Parts and tasks

- `listener-words`: the listener's fixed page (draft, 4). Task `listener-words`.
- `web-tab`: skein's tab (draft, 2).
  - `web-tab`: the three `Platform` methods and the web half.
  - `kit-tabs`: the browser kit reads a person's other tabs.
  - `web-tab-story`: Chromium runs it through the press's own path. Needs both.
- `tab-root`: temper's root, host and view (draft, 2, 3, 5).
  - `tab-surface`: the shapes, the shell's carrying, the recording's lines. Needs `web-tab`.
  - `sign-in-holds`: the press asks, the answer points or closes, the departure goes. Needs `tab-surface`.
  - After `sign-in-holds`: `moves-on` (the welcome moves on to Ready), `view-tab` (the refused note), `row-tab` (the accounts row), `journey-goes-on` (a recording goes on through a sign-in), `elsewhere-goes` (`Act::SignInElsewhere` goes).
  - `reload-link`: a reload keeps the address and the refusal. Needs `sign-in-holds`, `view-tab`, `row-tab`.
- `tab-browser`: `browser-story`, the temper tier's run on Chromium. Needs `sign-in-holds`, `moves-on`, `view-tab`, `kit-tabs`.
- `return-goes`: the family's `Return` leaves the wire, service and engine (draft, 5). Needs all of `tab-root`.

## Critical path

`web-tab`, `tab-surface`, `sign-in-holds`, `moves-on`, `browser-story`. `return-goes` ends it, and waits for the rest of `tab-root`.

## Every path that signs in

| Path | After this plan |
|---|---|
| `ConnectModel`'s way, real or fake | A new tab, and the page holds |
| The welcome's Model step, real or fake | A new tab, the page holds, then moves on to Ready |
| The accounts page's row | A new tab (`row-tab`); it holds its page, as it does today |
| A `--fake-backends` box | The same. Its issuer is `http` on loopback, which a tab follows only with a `loopback-navigation` bundle, the one that box's `--bundle` requires. Under a shipped bundle the tab closes and "Open it again" stays |
| The terminal and `temper ctl account connect` | Left as they are: they print the address |
| An owned Forgejo's sign-in link | Left as it is: a plain link, already `target=_blank` with `noopener` |
| A Google sign-in link | Left as it is: a plain external link |
| GitHub's App and install, start over, an authorization | Left as they are: they depart in their tab (draft, 6) |

## The journeys

No committed recording holds a sign-in, so none goes stale from the tab itself.

- `project-landing` moves: its walk signs in before its recording starts, and its `README.md` says "returns to Ready". Those words are for fabrig/ui.
- `chat-draft` moves the same way: it signs in before its recording starts, and its `README.md`'s words still hold.
- `welcome-set-up` stops at the Model step with "Continue with ChatGPT" unpressed, and `welcome-playground` signs in with the printed link alone. Neither moves.
- `return-goes` shifts the wire's enum tags, so recordings holding those bytes must be walked `again`, with no word changed. Which ones wasn't decoded in this cut; `welcome-playground` is inferred. The task reports each.

## Overlaps

- `temper/docs/design/client/journeys.md`, `journeys.status.md`, `journeys/**`: the drafts `real-journeys` (`journeys-close`) and `first-usable-model` (`printed-command`).
- `temper-client-shell-web/src/journey/line.rs`, `replay.rs`, `tests/suite/host.rs`, `tests/suite/host/walk.rs`: `real-journeys` (`scan`, `real-walk`, and `fields.rs`, `redact.rs` beside them). Whichever lands second takes the new lines' arms: `tab-surface` pre-approves them, and `journey-goes-on` keeps `real-walk`'s shape if it's there.
- `temper-client-protocol-wire/**`, `temper-service/src/family.rs`: `first-usable-model` (`printed-command`). `return-goes` comes last, and regenerates after it.
- `temper-client-view/src/words.rs`, `pages/connect_model.rs`, `temper-client-domain/src/pages.rs`, `route.rs`: fabrig/ui's `temper-client-*` work has priority, as does `first-usable-model`. These hunks are small, and named in each task.
- The look `sign-in-new-tab` itself has landed through `look-round-7`: `sign_in_wait` is its page, and this plan feeds it, changing no word but adding the tab's own line.

## What stays, named

Skein's `Request::Return` and jig's `return_to` stay as generic mechanisms with their own tests, and no shipping caller after `return-goes`.
