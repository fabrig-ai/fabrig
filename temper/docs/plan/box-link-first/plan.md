# box-link-first: the plan

From `temper/docs/plan/box-link-first/inputs/request.md` and `temper/docs/design/draft/box-link-first.md`. `walk = false`, as the request says. The design it became is `temper/docs/design/protocol/setup.md`, 7.1, 7.2, 7.5, 13.1 and 13.4, and `jig/docs/design/protocol/client.md`, section 8. The contracts are `notes/contracts.md`.

## What is already built

The draft's section 4 stands in the code today. `temper box` opens every step's listener before any step (`temper/crates/temper/src/box_command.rs`, `open_listeners`), refuses a taken port before anything starts, holds the socket and gives it to each spawn through `LISTEN_FDS`, restarts included. So the plan adds no socket handling. It prints the block after that point, and the tests hold it there.

## Parts and tasks

| Part | Task | Needs | Does |
|---|---|---|---|
| `jig-given-secret` | `jig-link-secret` | none | `Settings.link_secret`: the component takes a given first secret, and says no `Link` |
| `box-help` | `box-help` | none | `temper help box`, `tier`, `party` and the `box` row, and the tests that pin them |
| `box-terminal` | `box-own-log` | none | lifecycle moves to `box.log`; the terminal keeps errors, all timed |
| `box-terminal` | `box-first-block` | `box-own-log` | block before any step and after the logs move; the secret on serve's first stdin alone |
| `serve-takes-secret` | `serve-takes-secret` | `jig-link-secret` | `internal serve --box` reads stdin; `Named.link`, `Named.public`, `first_link_url` |
| `box-wiring` | `box-draws-link` | `serve-takes-secret`, `box-first-block` | draws the secret, builds the URL; first-run and `--fake-backends` lines |
| `box-readers` | `box-readers-log` | `box-draws-link` | every reader of the moved words reads `box.log` |
| `box-readers` | `box-readers-early` | `box-readers-log` | harnesses wait for `serve_listening`, not the link |
| `box-stories` | `box-link-stories` | `box-readers-early` | the real binary's stories |

## Critical path

`box-own-log`, `box-first-block`, `box-draws-link`, `box-readers-log`, `box-readers-early`, `box-link-stories`: about 395 implementer-minutes in order. `jig-link-secret` then `serve-takes-secret` run beside the first two, and `box-help` beside everything.

## What the cut decided

- **The channel:** serve's standard input, a pipe of its own on its first spawn, 32 raw bytes and a close. A restart's stdin is the null device, so it draws its own and a spent secret is never replayed. The existing box pipe was rejected: it is read asynchronously, after the component has started.
- **Serve says no `Link` for a given secret,** so serve's stdout holds only fresh links, and the box needs no filter.
- **A fresh link's blank line goes on stderr.** The draft's words can read as both on stdout, but its own "the link alone on stdout" and `temper help` want stdout to hold links alone. Recorded in the draft's status as a departure.
- **The block waits for the last run's logs to move,** a few file operations. So whoever reads the link reads this run's `box.log` and `serve.log`, which removes the stale-log race for every harness. It still prints before any step.
- **A clients' listener at port 0** isn't held (7.1), so it has no port to print. Its block is written at serve's first link. A printed link not enabled prints no block. A link step keeps its handoff. No production document does the last.
- **The terminal's error lines are always timed:** the block comes first, so nothing is left untimed.
- **`box.log` holds what left the terminal, not errors,** as the draft's section 5 says. If it can't be written, it is named once and its lines go to the terminal.
- **The test that a secret stays out of argv and the environment** reads the `Spawn` the box asks for, at the lowest tier. `AGENTS.md`, 3 forbids reading `/proc/*/environ`.
- **No harness bound changes value:** a bound that ran from the link now runs from `serve_listening`, so existing stories keep today's timing.

## macOS

The box's story on macOS stays green if `box-readers-log` and `box-readers-early` hold. By reading, not by a run (the workflow runs only on a schedule, on main):

- **Handing the socket on:** nothing new. The box hands macOS the same listeners it does today, by the same `Way::Given` and `LISTEN_FDS`. `SO_ACCEPTCONN` answers `ENOPROTOOPT` there, and `skein-shell/src/listener_calls.rs` lets that pass (`skein/docs/design/shell.md`, 6.6), so serve takes what it is given. The box made that socket itself or kept `temper init`'s, so it is listening.
- **The new pipe:** `skein-shell/src/darwin/process.rs` and `linux/process.rs` both make pipes at a chosen child descriptor and fill the rest from `/dev/null`. Serve's stdin is such a pipe, like the `answers` pipe macOS already uses. Serve's read is a plain blocking `std::io` read.
- **What would have broken:** the one story reaches `first_repository.rs` (`Forge N (Forgejo): started`) and `process.rs`'s `First run:` reader. `box-readers-log` moves both, and runs the story on Linux.

## Overlaps with other drafts

None avoidable. No task touches ui-final-walk's files (`notes/common.md` lists them). `temper/docs/design/client/greenfield/app.js` shows the old terminal in a mock: left alone, since it is fabrig/ui's. debt-25 is still being cut, so its overlap is unknown.
