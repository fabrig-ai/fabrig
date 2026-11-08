# tui-box-front: the plan

The user's look request `tui-box-front` of 2026-11-08 (`inputs/request.md`,
its commit and hand-made frames 01 to 05): `temper box` in a terminal runs
its services with the TUI in front.

## The protocol changes

- **jig's clients' component** (`jig/docs/design/protocol/client.md`, 4.2
  and section 8): the startup may give the first local link's secret,
  `local_secret`, as it gives the printed link's; and the component says
  `LinkSignedIn` to the startup each time the printed link signs a person in.
- **temper's forges document** (`temper/docs/design/client/README.md`, 4.6):
  `Forges` gains `linked`, whether the printed link has signed a person in
  since the engine started, the smallest fact for the welcome's "✓ Signed in
  as the owner": the engine sets it at `LinkSignedIn`, and every client
  already watches the document from the start. It is memory, as the links
  are: false after a restart, so the TUI also counts the welcome's decision
  past its first step, or a project, as signed in.
- **serve's standard input under the box** (`protocol/setup.md`, 7.1): 32
  bytes as today, or 64, the printed link's first secret then the local
  link's.
- **The box's front** (`protocol/setup.md`, 7.6): `temper internal tui --box
  http://HOST:PORT`, a child of the box with the local link's URL on fd 3 and
  the box's lines (`link`, `log`, `restarting`, `running`, `gave-up`) on fd 4.
- **skein-shell** (`skein/docs/design/shell.md`, 6.3): `open_terminal` and
  `take_foreground`, since a child starts in a process group of its own and
  skein-io gives a child no descriptor of its parent's terminal.

## Decisions

- **The box is the parent, the TUI its child.** The box already owns its
  children, their pipes and its stop; the TUI as its child keeps the box's
  refusals on the terminal before anything, its exit status, and its stop
  when the TUI ends. The child opens `/dev/tty` and takes the terminal's
  foreground itself (no change to skein-io's spawn); the box takes it back
  once the child has ended.
- **The TUI signs in by a link the box hands it,** the local link's first
  secret, drawn by the box and given to serve beside the printed link's,
  never the printed link (which the welcome shows for the browser), never
  argv, the environment or a file. The TUI's existing sign-in from the
  engine's link file takes it as its first read, so its retries and
  fallbacks are today's.

## Parts and tasks so far

| Part | Task | Crates | Needs | Minutes |
|---|---|---|---|---|
| `foreground` | `skein-foreground` | skein-shell, skein's world | | 60 |
| `jig-sign-in` | `jig-sign-in` | jig-protocol-engine-clients (+ forced arms in jig and temper) | | 75 |
| `linked` | `forges-linked` | temper-client-protocol-wire (+ `linked: false` literals) | | 45 |
| `linked` | `serve-sign-in` | temper (serve's stdin), temper-service (`local_secret`, `linked`) | `jig-sign-in`, `forges-linked` | 75 |

The box's side (`protocol/setup.md`, 7.6) and the TUI's (`client/README.md`,
6.17) are being designed; their tasks follow in the next pushes.

## Overlaps

- `forges-linked` fills `linked: false` in `temper/tools/gallery/**` and some
  `temper-client-view` tests, which `look-ask-ways` is changing: the field
  alone, as the compiler forces.
- `jig-sign-in`'s forced arms touch `temper/crates/temper-service/src/clients.rs`
  and `temper/tests/client/src/component.rs`: one arm and one field each.

## Critical path

To be written with the box's and the TUI's tasks.
