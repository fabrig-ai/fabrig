# sign-in-new-tab: the plan

A plan's sign-in moves to a new tab, and temper's page stays and waits (`temper/docs/design/draft/sign-in-new-tab.md`).

## Parts so far

- `listener-words`: the listener's fixed page (draft, 4). Task `listener-words`.
- `web-tab`: skein's tab (draft, 2). Tasks `web-tab` (the surface and the web half), `kit-tabs` (the browser kit reads a person's other tabs), and `web-tab-story` (Chromium runs it through the press's own path; needs both).

## To come

- `tab-root`: `tab-surface` is released. `sign-in-holds` makes the root ask and the departure go, then the view's refusal and the reload's link; their readers are being surveyed.
- The browser tier's story, and the wire's `Return` where nothing else sends it.

## Critical path

`web-tab`, `tab-surface`, `sign-in-holds`, then the browser tier's story.

## Overlaps

- `temper/docs/design/client/journeys.md` and `journeys.status.md`: the drafts `real-journeys` (`journeys-close`) and `first-usable-model` (`printed-command`) change them too.
- `temper-client-shell-web/src/journey/line.rs`, `replay.rs`: the draft `real-journeys` has `fields.rs`, `scan.rs` and `redact.rs` beside them, not yet landed. Whichever lands second takes the new lines' arms; `tab-surface` pre-approves them.
- `temper-client-view/src/words.rs`, `pages/connect_model.rs` and `temper-client-domain/src/route.rs`: fabrig/ui's `temper-client-*` work and `first-usable-model` have priority, so these hunks stay small.
