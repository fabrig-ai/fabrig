# tui-remote: execution

2026-11-18. What of [tui-remote.md](tui-remote.md) gets built, and in
what order. It designs nothing: each milestone writes its own design
from the draft as it runs (`DESIGN.md`, section 3). A milestone that
meets a question the draft leaves open takes its section 10 default, or
the nearest thing, and says so in the draft's status.

## 1. In one page

- **Three milestones,** M1 to M3 (section 3): the TUI at a name, signed
  in by a typed code; approval across the internet, guarded, with the
  loopback return; and the TUI's one click.
- **M1 and M2 run side by side:** M1 is the TUI's and jig's native
  shell's; M2 is jig's engine, its client wire and the web's approval
  page. M2's change to the approval's messages touches the shared
  client crates M1 links, so whichever lands second takes the other's
  shape.
- **M3 follows both:** the TUI's listener and its one-click ask, over
  M2's messages, on M1's screen.
- **Then dev.fabrig.ai:** once M2 has landed, its redeploy, and
  `approval` in its clients' configuration if it isn't on, each on the
  user's word. Approval stays off there until M2 is deployed: M1 alone
  hardens nothing.
- **Then the user takes over:** they use the TUI from their laptop, and
  the next parts are drafted from what they ask.

## 2. How a milestone runs

- **Its entry here is its input,** with `tui-remote.md` its source.
- **Its design and its build together,** landed whole: the design
  documents' sections it needs (`tui-remote.status.md`'s lags), then the
  code and its focused and fuzzy tests (`tui-remote.md`, section 9).
- **After landing,** the user tries it on their laptop. Nothing waits
  for that.
- **Its state** is section 3's table, kept current. When all three have
  landed, `tui-remote.status.md` holds no lag, and this file goes with
  the draft (`DESIGN.md`, section 3).

## 3. The milestones

| | Milestone | Executes (`tui-remote.md`) | Needs | State |
|---|---|---|---|---|
| M1 | The TUI at a name | 3, 4, 5.2, 5.5, 6, 7 | | landed |
| M2 | Approval across the internet | 5.3, 5.4, and 5.1's engine and page | | landed |
| M3 | One click | 5.1's terminal | M1, M2 | landed |

### M1. The TUI at a name

- **Executes:** `tui-remote.md`, 3 (the address, kept, the place named,
  the addresses opened), 4 (HTTPS in jig's native shell, the name
  resolved at start), 5.2 and 5.5 (the sign-in screen at a name, by a
  typed code, Enter its one key), 6 (a directory per deployment, and the
  move from today's) and 7 (a build refused).
- **Design:** `../client/tui/README.md`, 1, and `../client/tui/directory.md`, 1, and `../client/tui/host.md`, 1;
  `../client/tui/pages/README.md`, 3; `../client/tui/pages/chat.md`, 1;
  `../client/tui/pages/sign-in.md`, 1 and 2; `../client/sign-in.md`, 1
  and 1.1; jig's `protocol/client/shells.md`, 1.
- **Builds:** `temper https://NAME` and its refusals; jig's native shell
  over skein's TLS client with the machine's roots; the binary's
  resolving at start; the TUI's directory per origin and the move; the
  sign-in screen at a name with no paste, which until M3 asks a typed
  code alone and, with an opener, opens the tab at `https://NAME/terminal`
  with no code; the screen for a build
  refused; the place in the header and the rule.
- **After landing:** `temper https://dev.fabrig.ai` from the user's
  laptop reaches the sign-in screen, and signs in by a typed code once
  M2 is on dev.fabrig.ai.
- **Left:** signed out, a build refused shows only once signed in
  (`tui-remote.status.md`, 7): a follow-up once M2 has landed.

### M2. Approval across the internet

- **Executes:** `tui-remote.md`, 5.4 (the ask's kind and source, two
  waiting per source, the loopback return's grant and take), 5.3 (one
  press for a loopback return alone, the code typed for a public ask
  without one on every origin, the source said), and 5.1's page and
  engine (the grant answered once, the browser sent to `127.0.0.1` at
  the ask's port).
- **Design:** jig's `protocol/client/sign-in.md`, *A sign-in approved by
  a browser*; jig's `protocol/client/default-http.md`, 3 and 4; jig's
  `protocol/client/wire.md`, 2.1 and 2.2; jig's
  `client/domain/client/README.md`, 4.1 and 9; jig's
  `protocol/configuration-engine.md`, 1, and `protocol/configuration.md`, 8; jig's `protocol/client/README.md`,
  10 and 12; `../protocol/sign-in.md`, 2.1 and 2.3;
  `../client/pages/shared-behaviour.md`, 1, *The approval page*;
  `../client/web/addresses.md`, 1.
- **Builds:** the engine end's source address, from `X-Forwarded-For`
  by the public name and the peer otherwise; the ask's kind; the
  per-source count; the deciding browser's source beside the ask's; the
  loopback return in the ask, the grant in the
  decision's answer and the take; `ApprovalRead`'s source, kind and wall
  time; the approval page's three rules and its hand-off to the
  loopback.
- **After landing:** on dev.fabrig.ai, a typed-code ask from the laptop
  shows its source on the page, and a `/terminal?code=` link for it
  opens to the typed field.

### M3. One click

- **Executes:** `tui-remote.md`, 5.1's terminal: the listener on
  `127.0.0.1`, the ask with its loopback return where there is an
  opener, the take with the grant, the redirect back, Enter for a typed
  code instead.
- **Design:** `../client/sign-in.md`, 1.1; `../client/tui/pages/sign-in.md`,
  3 and 4; `../client/tui/host.md`, 1, and `../client/tui/README.md`, 4;
  `../client/tui/journeys.md`, 3.2 and 5.
- **Builds:** the TUI's one-shot listener on skein-io; the one-click ask
  and its lines; the take by grant; the listener's closing.
- **After landing:** `temper https://dev.fabrig.ai` from the user's
  laptop signs in with one press in the browser.
