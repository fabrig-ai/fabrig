# tui-box-front: the account's rows

The landing's account (`temper/docs/design/draft/look-requests.md`, section
5): one row per frame change and per state the request names, each checked
against the trunk by task `account`, which writes the table in its commit's
message. The request: `inputs/request.md`, `inputs/request-commit.txt`, the
frames `inputs/box-front-01.txt` to `-05.txt`, `inputs/box-front-README.md`.
The look request has no overlay: its frames are hand-made text.

| Item | Where it should be on the trunk | Expected result |
|---|---|---|
| `temper box` in a terminal runs its TUI in front | `protocol/setup.md`, 7.6; `box-front`, `binary-front` | landed |
| `--no-tui`, a non-terminal stdin or stdout, `--echo steps`, or no first link: the box as today | 7.6; `binary-front`'s condition; the box's unchanged stories | landed |
| No starting screen: the box holds the TUI's connection until temper is up; the welcome is the first frame | 7.6 (the front spawned in the block's place); `client/README.md`, 6.17; `tui-welcome` | landed |
| Signed in with nothing to read or paste: a one-time secret on an inherited pipe | 7.1, 7.6; jig `protocol/client.md`, 8 (`local_secret`); `jig-sign-in`, `serve-sign-in`, `box-front`, `front-command`, `seam-surface`, `tui-seam` | landed |
| 01, the welcome: the link drawn whole, never opened; what it does; the browser's steps none started | 6.17; `tui-welcome` | landed; the frame elides the secret, the TUI draws it whole |
| 02, following the steps read-only; the step under way is `welcome::decision`'s | 6.17; `domain-front`, `tui-welcome` | landed; "connecting GitHub…" as 6.17 words the forge's row |
| 02, "✓ Signed in as the owner": the printed link used by a browser | jig `LinkSignedIn`; `client/README.md`, 4.6 `linked`; `serve-sign-in`, `domain-front` | landed; the protocol change named in `plan.md` |
| 03, the playground: "playground" in the header and the quiet line, the banner's words in the header's box | 6.17; `domain-front`, `tui-front-views` | landed |
| 04, quitting: "Stop the box? … stops with it.", "Stop the box", "Don't quit"; nothing running stops at once, draining | 6.17, 7.6; `tui-seam`, `tui-front-views`, `box-front` | landed; the head's words name what runs as 6.17 says |
| 05, a failing step on the quiet line; `/log` showing the box's recent lines | 6.17, 7.6; `box-lines`, `box-front`, `tui-front-views` | differs: the quiet line names the step as the box does (`forgejo-3 restarting`), and `/log` shows the box's own lines, time first, where the frame has hand-made words |
| A step failing for good: the box gives up naming the step, the TUI says so and stays open for `/log` | 6.17, 7.6 | landed |
| The light theme | the TUI's themes; 6.15 | landed: the new words take existing tokens, both schemes |
| The place: "box" or "box:PORT" | `tui-launchpad-2`'s `place` | landed if `place` is on the trunk; else deferred to `tui-launchpad-2`, tracked by its plan |
| Replaces: the printed block on the terminal (to the box's logs, the link to the TUI) | 7.6; `box-front` | landed |
| Replaces: a second terminal for `temper` | 7.6 | landed |
| A box over SSH | left out by the request | not the look's; left to the person |
| The journey of the box with the TUI in front | `client/tui/journeys.md`, 8.3 and 9; `journeys.status.md` | deferred: `again` can't drive `temper box` as it drives `temper` (the browser harness spawns the box on pipes and reads the printed link from stdout); tracked as **Not yet** in `journeys.status.md` |
| The render check | after landing | follows landing, owned by temper/tui's after-landing check |
