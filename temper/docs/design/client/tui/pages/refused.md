# A build refused

## 1. A build refused

(`../../../draft/tui-remote.md`, section 7.) The person keeps the
TUI's build at the deployment's, for now (the user, 2026-11-18); when
the engine refuses it anyway, the TUI says so, in place of a TUI that
waits for nothing. Illustrative, at a name:

```
╭─ temper ──────────────────────────────────────────────╮
│ dev.fabrig.ai runs another build of temper.           │
│ This one is temper 0.0.0 (b948f40d6df1).              │
│ Start the build dev.fabrig.ai runs, then try again.   │
╰───────────────────────────────────────────────────────╯

 ● Refused                                Ctrl+C to quit
```

- **When:** out of front, the engine refuses the TUI's version
  (`../../../draft/redeploys.md`, 8): `jig-client`'s link out of date,
  `link().state` `Link::OutOfDate`, jig's own version not served, or an
  owner its link view names as not served, `link().not_served` not
  empty. Signed out or signed in, at a name or on loopback, alike. In
  front of a box, the box runs the TUI's own build, and out of date
  there stays the welcome's (`../box.md`).
- **In place of every screen:** the sign-in screen, the launchpad, a
  chat, a talk and a place not yet started give way to it, whatever
  list or question stood. It is drawn as the sign-in screen is
  (`sign-in.md`, 1): a framed top, `tui-top`, titled "temper", holding
  its lines, and the bottom row, the link's state at its first column
  and "Ctrl+C to quit" at its right. No header, box or quiet line.
- **Its lines,** the TUI's own words:
  - "PLACE runs another build of temper.", PLACE the place as the rule
    above the box names it, `place(origin, boxed)` (`chat.md`, 1, *The
    place*): `dev.fabrig.ai`, `127.0.0.1:8367`, or "box" on a box;
  - "This one is temper VERSION.", VERSION `Start.version`, as the
    header's first line has it (`README.md`, 3), the line left out where
    it is empty;
  - "Start the build PLACE runs, then try again."
- **The link's state** reads "Refused" (`tui-down`), the TUI's word, in
  place of the view's "Out of date" (`README.md`, *The link's state*).
- **Nothing more is sent:** from the report in which the link reads
  refused, the host steps the root no further and asks the native shell
  nothing: no request among `Duties.asked`, no approval asked, no tab
  opened, no engine's link read. A key does nothing but Ctrl+C, which
  closes, as the input's end and the signals do (`../README.md`, 4).
  The screen stands until the TUI closes: jig-client's out of date
  stays (jig's `client/domain/client/README.md`).
- **No version logic of the TUI's own:** the screen draws what
  `jig-client` reports, and the hello's versions and fingerprints stay
  `jig-client`'s, the same code for every client
  (`../../../draft/tui-remote.md`, 8).
- **Its tests:** the view's rows for each line, at a name, on loopback
  and on a box, and with an empty version; the host's: the link out of
  date and a family not served each draw it, signed out and signed in,
  and no duty asks anything after it; golden frames at 80 by 24, dark
  and light.
