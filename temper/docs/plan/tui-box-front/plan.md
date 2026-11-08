# tui-box-front: the plan

The user's look request `tui-box-front` of 2026-11-08 (`inputs/request.md`,
its commit and hand-made frames 01 to 05): `temper box` in a terminal runs
its services with the TUI in front. Design written by this cut:
`skein/docs/design/shell.md`, 6.3; jig's `jig/docs/design/protocol/client.md`,
4.2 and section 8; `temper/docs/design/protocol/setup.md`, 7.1 and 7.6 (with
pointers in 7.2 and 7.3, 13.1 and 15); `temper/docs/design/protocol/README.md`,
4; `temper/docs/design/client/README.md`, 4.6, 4.8, 6.11, 6.13, 6.14, 6.16,
6.17 and 11; `temper/docs/design/client/tui/journeys.md`, 3.2, 4.1, 5, 8.3
and 9; their status files; the drafts' status (`draft/tui.status.md`,
`draft/box-link-first.status.md`).

## The protocol changes

- **jig's clients' component** (`jig/docs/design/protocol/client.md`, 4.2
  and section 8): the startup may give the first local link's secret,
  `local_secret`, as it gives the printed link's; and the component says
  `LinkSignedIn` to the startup each time the printed link signs a person in.
- **temper's forges document** (`temper/docs/design/client/README.md`, 4.6):
  `Forges` gains `linked`, whether the printed link has signed a person in
  since the engine started: the smallest fact for the welcome's "✓ Signed in
  as the owner". The engine sets it at `LinkSignedIn`, and every client
  already watches the document from the start. It is memory, as the links
  are: false after a restart, so `welcome::signed_in` also counts the
  welcome's decision past its first step, or a project.
- **serve's standard input under the box** (`protocol/setup.md`, 7.1): 32
  bytes as today, or 64, the printed link's first secret then the local
  link's.
- **The box's front** (`protocol/setup.md`, 7.6): `temper internal tui --box
  http://LISTENER`, a child of the box: the local link's URL on fd 3, the
  box's lines (`link`, `log`, `restarting`, `running`, `gave-up`) on fd 4,
  its stdout and stderr into `tui.log`.
- **temper box's flag** `--no-tui` (`protocol/README.md`, 4).
- **The TUI's recording** (`client/tui/journeys.md`, 3.2): the start fact
  `front` and the record `box-line`.
- **skein-shell** (`skein/docs/design/shell.md`, 6.3): `open_terminal` and
  `take_foreground`.

## Changes to the box's code and help (temper/ui's seam 5)

`temper-box`: a module `front` (the lines), the host's front (spawn,
descriptors, serve's 64 bytes, `tui.log`, its end as the stop, the stop and
give up with it), the terminal's lines to `box.log` and the front while it
runs. The binary: `box_command.rs` and `fake_backends.rs` (`--no-tui`, the
front's condition, the local link drawn, the foreground taken back in the
drain hook), `commands.rs` (the box's row, `internal tui`), `lib.rs`,
`tui_front.rs` (new). Help: `temper/docs/help/box.md`, `tui.md`,
`reference/commands.md`. With `--no-tui`, `--echo steps`, or stdin or stdout
not a terminal, the box runs byte for byte as today; every process-tier and
focused box story starts the box on pipes, so none changes.

## Decisions

- **The box is the parent, the TUI its child.** The box already owns its
  children, their pipes and its stop; the TUI as its child keeps the box's
  refusals on the terminal before anything, its exit status, and its stop
  when the TUI ends. The child opens `/dev/tty` and takes the terminal's
  foreground itself (no change to skein-io's spawn); the box takes it back
  in its loop's drain hook once the child has ended.
- **The TUI signs in by a link the box hands it,** the local link's first
  secret, drawn by the box and given to serve beside the printed link's;
  never the printed link (which the welcome shows for the browser), never
  argv, the environment or a file. The TUI's sign-in from the engine's link
  file takes it as its first read, so retries and fallbacks are today's.
- **The front's address is the clients' listener,** never a configured
  public origin, which isn't on loopback.
- **`Told::BoxLine`,** not `Told::Boxed`: `tui-launchpad-2` adds a unit
  `Told::Boxed`.
- **The journey is not recorded** (the request's escape): `again` runs the
  box under the browser harness on pipes and reads the printed link from its
  stdout; with the TUI in front the box must run in the pseudo-terminal and
  the link is the TUI's alone. `client/tui/journeys.md`, 8.3 designs what
  `again` needs, and `journeys.status.md` keeps it **Not yet**.
- **Formats changed before v0.1.0, no migration:** the forges document's
  `linked`, serve's stdin, jig's settings, the TUI recording's `front` and
  `box-line`.

## Parts and tasks

| Part | Task | Crates (files) | Needs | Minutes |
|---|---|---|---|---|
| `foreground` | `skein-foreground` | skein-shell, skein's world | | 60 |
| `jig-sign-in` | `jig-sign-in` | jig-protocol-engine-clients (+ forced arms in jig and temper) | | 75 |
| `linked` | `forges-linked` | temper-client-protocol-wire (+ `linked: false` literals) | | 45 |
| `linked` | `serve-sign-in` | temper (serve's stdin), temper-service (`local_secret`, `linked`) | `jig-sign-in`, `forges-linked` | 75 |
| `tui-front` | `tui-surface` | temper-client-tui (`BoxLine`, state, events, words, tokens, `work_runs`, stubs) | | 120 |
| `tui-front` | `domain-front` | temper-client-domain (`linked`, `welcome::signed_in`, the playground read again) | `forges-linked` | 75 |
| `tui-front` | `tui-welcome` | temper-client-tui (`view/welcome.rs`, the dispatch) | `tui-surface`, `domain-front` | 120 |
| `tui-front` | `tui-front-views` | temper-client-tui (`view/front.rs`, launchpad's header, foot, lists, notes) | `tui-surface` | 130 |
| `tui-front` | `seam-surface` | temper-client-shell-tui (`Start.front`, `Report::Front`, `run::Front`; literals) | `tui-surface` | 40 |
| `tui-front` | `tui-seam` | temper-client-shell-tui (host, run, journey records) | `seam-surface` | 130 |
| `tui-front` | `tui-redact` | temper-client-shell-tui (`journey/redact.rs`, `scan.rs`) | `tui-seam` | 45 |
| `tui-front` | `world-front` | temper-client-world (the joint world's front stories) | `tui-seam`, `tui-welcome`, `tui-front-views`, `domain-front`, `serve-sign-in` | 120 |
| `box-front` | `box-lines` | temper-box (`front.rs`) | | 45 |
| `box-front` | `box-front` | temper-box (the host's front) | `box-lines` | 150 |
| `box-front` | `front-lines` | temper-box (the lines sent, the terminal's lines rerouted) | `box-front` | 120 |
| `box-front` | `front-command` | temper (`tui_front.rs`, `internal tui`) | `seam-surface`, `skein-foreground`, `serve-sign-in` | 90 |
| `box-front` | `binary-front` | temper (`--no-tui`, the condition, the local link, the foreground back, the pty story) | `box-front`, `front-command`, `tui-seam`, `serve-sign-in`, `skein-foreground` | 140 |
| `box-front` | `box-help` | `temper/docs/help/box.md`, `tui.md` | `binary-front` | 45 |
| `account` | `account` | status files; the account in its commit | `world-front`, `box-help`, `tui-redact`, `front-lines` | 45 |

`tui-welcome` and `tui-front-views` share `temper-client-tui` in one wave, a
stated departure from one writer per crate: `tui-surface` lays every name,
word, token and stub file both need, and their files are disjoint. So do
`tui-seam` and `front-command` in time, in different crates
(`seam-surface` fills the binary's literal).

## Overlaps

- **`tui-launchpad-2`** (integration) changes the same TUI files
  (`view/launchpad.rs`, `view/foot.rs`, `view.rs`, `state.rs`, `look.rs`,
  `theme.rs`, `words.rs`, `host.rs`, `run.rs`, `journey/**`,
  `temper/crates/temper/src/tui.rs`) and 6.12 to 6.15 of the client's
  design: expect a catch-up. This plan keeps its views in new files
  (`view/welcome.rs`, `view/front.rs`, `tui_front.rs`) and its hunks in the
  shared ones small. Whichever lands second: both append look tokens after
  `tui-command`; `Told::Boxed` (theirs) and `Told::BoxLine` (ours) both stay;
  both rewrite 6.13's *The seam keeps its shape*; their start fact `box` and
  ours `front` both follow `local`; in front the TUI is on a box, so the
  binary sets their `boxed` (or the view reads `front() || boxed()`), and
  the place reads "box" once their `place` is on main (until then, as
  today; the account says so and `README.status.md` keeps it).
- **`look-ask-ways`:** `forges-linked` fills `linked: false` in
  `temper/tools/gallery/**` and some `temper-client-view` tests: the field
  alone.
- **`jig-sign-in`'s** forced arms touched `temper-service/src/clients.rs`
  and `temper/tests/client/src/component.rs`: one arm and one field each.
- **Status files** (`client/README.status.md`, `protocol/setup.status.md`):
  several tasks each delete their own bullet; expect small merges there.

## Critical path

`tui-surface` (120) → `seam-surface` (40) → `tui-seam` (130) →
`binary-front` (140) → `box-help` (45) → `account` (45): about 8.7 hours
from the first push. Beside it: `box-lines` (45) → `box-front` (150) →
`front-lines` (120); `seam-surface` → `front-command` (90), done before
`tui-seam`; `forges-linked` → `domain-front` → `tui-welcome` →
`world-front` (with `tui-seam`).
