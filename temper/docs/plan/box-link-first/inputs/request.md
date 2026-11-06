# box-link-first: `temper box` prints its link at once, and nothing else

The user asked for this to be built and landed now, on 2026-11-05. The spec is `temper/docs/design/draft/box-link-first.md`: read it whole. It settles the design; this request adds only what the plan needs.

## In short

- Before any step starts, the box prints exactly the block in its section 2, the link on stdout, then only fresh links (a blank line and the link) and errors, in today's words with their time of day.
- The box draws the first secret, and gives it to `temper serve`'s first start. jig's sign-in component takes a given secret instead of drawing one (`jig/docs/design/protocol/client.md`, section 8).
- The box binds `application.clients.listener` before anything starts, and hands the socket to serve, keeping it across serve's restarts, so a browser opening the link early waits on the connection.
- First run's line, `Logs:`, `Starting`, the groups' lifecycle, `Stopping` and `Stopped`, and `Ctrl-C stops temper.` move to `DATA/box/logs/box.log`. `--fake-backends` keeps its `box:` lines after the block. `--echo` steps are unchanged.
- Update `temper/docs/design/protocol/setup.md`, 7.2, jig's section 8, and `temper help box` with it.

## What the plan must hold

- **A secret stays out of argv and a child's environment:** stdin or an inherited descriptor only (`AGENTS.md`, 3). A test checks it. The draft's secret given to serve is used once, and a restart of serve doesn't replay a spent one.
- **Linux and macOS both:** the box's story on macOS is green on main, and must stay so. Handing a listening socket on: `SO_ACCEPTCONN` was a macOS fault here once; say how this behaves there. The macOS job runs on a schedule, so a break there shows late: read it through.
- **Everything else that reads the box's terminal output:** `temper/tests`, the system tier, `temper box` stories, and the site's words if it's in this repository (`AGENTS.md`, 2's "read by tests"). Find them all. A live tier or a walk's brief that greps for `Logs:` or `Starting` must move with this: name each in the report.
- **A port already in use** is an error before the block prints: no link is shown that can't work. If the draft says otherwise, follow it.
- **A failed first start:** the error prints, in today's words, after the block. The link stays printed; say what the person's next step is, in the words the draft has.

## Tests

A failing test first in the lowest tier for each: the block's exact bytes and that nothing else reaches the terminal; a secret given is the first link's; the socket is bound before the block prints, and held across a restart of serve; the secret isn't in argv; a browser's early connection waits and then signs in.

## Size

jig's one surface change, and temper. Hearth's manifests name no jig crate. fabrig/ui has priority: leave its files alone, and expect catch-ups. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). A protocol change beyond the draft is a `Question:`. Formats change in place before v0.1.0.

## Walk

`walk = false` for the plan: the user reads the terminal themselves, and fabrig/qa's box walks will meet it on their next run.
