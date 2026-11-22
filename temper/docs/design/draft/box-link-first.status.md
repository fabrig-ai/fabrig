# box-link-first: status

2026-11-19. Where the design lags or departs from [box-link-first.md](box-link-first.md).

- **Design departs:**
  - 2: `../protocol/setup/box-supervision.md`, 1: the block's link is at the link's page, `http://127.0.0.1:8367/sign-in?secret=...`, where this draft has the link route, `/jig/v1/sign-in/link`, "as today": a link at the route still signs in (`../client/README.md`, section 4). The user's request of 2026-11-18, "a more terse URL".
  - 2: `../protocol/setup/box-supervision.md`, 1: the block's footer says "It signs you in as this temper's owner, once, within 2 hours." and "Opened after that, it prints a fresh one here.", where this draft has "once and for a short while" and "Opened again once spent": an expired link draws a fresh one too, and it lives 2 hours (jig's `protocol/client/sign-in.md`). The user's, 2026-11-06, walking a real box.
  - 2: `../protocol/setup/box-supervision.md`, 1: a fresh link's blank line is on stderr, the link on stdout, where this draft's words can read as both on stdout: its own "the link alone on stdout, the other lines on stderr" and `temper help commands`' "its stdout" want a script reading stdout to read links alone. A terminal shows the same. Decided by the plan's cut.
  - 2, 3, 4: `../protocol/setup/box.md`, 1, and `../protocol/setup/box-supervision.md`, 1: a clients' listener at port 0 is not held, as 7.1 has it, so the box has no port to print: serve draws its own first secret and the block is written at its first link. The default document's port is fixed (8.2), so a person's box never meets this. Decided by the plan's cut, to keep 7.1's rule and leave such a document working.
  - 2, 5: `../protocol/setup/box-front.md`, 1: in a terminal, `temper box` runs the TUI in front of it: the block's link goes to the TUI alone and the block's other lines to the box's log, and while the TUI runs the errors this draft keeps on the terminal go to the box's log and the TUI's `/log`, as do `--fake-backends`' lines; the terminal shows the block and the errors as `box-link-first.md`, section 2 has them in `temper-box` alone or with `--echo steps`, or off a terminal. The user's look request `tui-box-front`, 2026-11-08.
- **Open:**
  - 6: a progress page; the time of day on error lines (`../protocol/setup/README.md`, 10).
