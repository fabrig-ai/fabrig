# box-front (hand-made)

Hand-made frames, not a recording, at 80 by 24 in the dark theme. They show `temper box` run in a terminal with the TUI in front of it, as the user sketched it on 2026-11-08: the TUI is a small front end for the box, started by it, and quitting it stops the box. The values are written in by hand.

- `01`: a brand-new box: the welcome, from the first moment. The box hands out its sign-in link before temper is up, and holds the TUI's connection until it is, as it does the browser's. So there's no starting screen. The TUI prints the link whole (elided here) and never opens a browser. Under it, the browser's welcome steps, none started.
- `02`: the link opened. The TUI follows the browser's welcome steps, read-only. Setup never runs in the TUI, and the forge is set up there (a Forgejo fetched, GitHub connected, or the playground).
- `03`: the playground chosen in the browser. The TUI works with it as with any project, and says it's the playground in the header, in the box under it, and on the quiet line.
- `04`: Ctrl+C while work runs. Quitting asks first, as a list under the box. With nothing running, it quits and stops the box at once.
- `05`: a step of the box failing. The quiet line says "forge restarting", and `/log` notes the box's recent log.

Once the welcome is done, the TUI goes to the project's launchpad, as `../launchpad-chat/` shows. In box mode, the header names the place as "box", or "box:PORT" away from the default port.

Not drawn: `temper box --no-tui`, which runs as today with no TUI. Nor does it draw a box over SSH, which is left to the person.
