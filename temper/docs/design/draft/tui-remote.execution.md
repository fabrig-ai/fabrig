# tui-remote: execution

2026-11-18. What of [tui-remote.md](tui-remote.md) gets built, and in
what order. It designs nothing: each milestone writes its own design
from the draft as it runs (`DESIGN.md`, section 3). A milestone that
meets a question the draft leaves open takes its section 10 default, or
the nearest thing, and says so in the draft's status.

## 1. In one page

- **Two milestones,** M1 and M2 (section 3): the TUI at a name, and
  approval across the internet.
- **They run side by side:** M1 is the TUI's and jig's native shell's;
  M2 is jig's engine side and the web's approval page. Neither waits for
  the other.
- **Then dev.fabrig.ai:** once both have landed, its redeploy, and
  `approval` in its clients' configuration if it isn't on, each on the
  user's word. M2 lands on dev.fabrig.ai before approval is on there.
- **Then the user takes over:** they use the TUI from their laptop, and
  the next parts are drafted from what they ask.

## 2. How a milestone runs

- **Its entry here is its input,** with `tui-remote.md` its source.
- **Its design and its build together,** landed whole: the design
  documents' sections it needs (`tui-remote.status.md`'s lags), then the
  code and its focused and fuzzy tests (`tui-remote.md`, section 9).
- **After landing,** the user tries it on their laptop. Nothing waits
  for that.
- **Its state** is section 3's table, kept current. When both have
  landed, `tui-remote.status.md` holds no lag, and this file goes with
  the draft (`DESIGN.md`, section 3).

## 3. The milestones

| | Milestone | Executes (`tui-remote.md`) | Needs | State |
|---|---|---|---|---|
| M1 | The TUI at a name | 3, 4, 5.1, 6, 7 | | not started |
| M2 | Approval across the internet | 5.2, 5.3 | | not started |

### M1. The TUI at a name

- **Executes:** `tui-remote.md`, 3 (the address, kept, the place named,
  the addresses opened), 4 (HTTPS in jig's native shell, the name
  resolved at start), 5.1 (the sign-in screen at a name), 6 (a directory
  per deployment, and the move from today's) and 7 (a build refused).
- **Design:** `../client/tui/README.md`, 1 to 3;
  `../client/tui/pages/README.md`, 3; `../client/tui/pages/chat.md`, 1;
  `../client/tui/pages/sign-in.md`, 1 and 2; `../client/README.md`, 4
  and 4.1; jig's `protocol/client/README.md`, 9.
- **Builds:** `temper https://NAME` and its refusals; jig's native shell
  over skein's TLS client with the machine's roots; the binary's
  resolving at start; the TUI's directory per origin; the sign-in
  screen's approval at a name, with no paste; the screen for a build
  refused; the place in the header and the rule.
- **After landing:** `temper https://dev.fabrig.ai` from the user's
  laptop reaches the sign-in screen, and signs in once M2 is on
  dev.fabrig.ai.

### M2. Approval across the internet

- **Executes:** `tui-remote.md`, 5.2 (the ask's source, one per source,
  the source kept and read) and 5.3 (the code typed off loopback, the
  source said).
- **Design:** jig's `protocol/client/sign-in.md`, *A sign-in approved by
  a browser*; jig's `protocol/client/default-http.md`, 3;
  `../protocol/sign-in.md`, 2.1; `../client/pages/README.md`, 2, *The
  approval page*; `../client/web/README.md`, 2.3.
- **Builds:** the engine end's source address, from `X-Forwarded-For`
  by the public name and the peer otherwise; one live approval per
  source by the public name; `ApprovalRead`'s source; the approval
  page's code typed off loopback, and its "from ADDRESS".
- **After landing:** on dev.fabrig.ai, a terminal's ask from the laptop
  shows its source on the page, and a `/terminal?code=` link opens to
  the typed field.
