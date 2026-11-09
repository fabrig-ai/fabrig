# agent-binary-closure: the plan

deployment-binaries M4, the agent's binary, is carried onto main in one
commit (`inputs/request.md`). Meeting M2's code there, it turns two
boundary tests red, and nothing else: `tests/boundary/tests/binaries.rs`'s
`temper-engine` row and `tests/boundary/tests/closure.rs`'s `temper-agent`
row. This plan makes both pass by moving code, adds no exception to either
row, and moves `temper-agent`'s commit to `temper-version`.

## What the cut found

- **The engine's row:** all nine excluded smith crates
  (`smith-agent-document`, `smith-agent-profile`, `smith-events`,
  `smith-host-domain`, `smith-protocol-{channel,events,llm,machine,program}`)
  come through one edge, `temper-fetch-shell` -> `smith-agent-document`,
  whose only use is the launcher's two calls to
  `program::private_directory` in `place`. Without that edge, the engine's
  closure keeps only the row's four named exceptions among excluded
  crates (checked by `cargo tree` on a throwaway manifest edit).
- **The agent's row:** all sixteen jig crates come through
  `temper-shell` -> `jig-protocol-config`, whose only use is
  `Directories::jig()`. Without that edge, the agent's temper crates are
  exactly the row's five and it holds no jig crate (checked the same way).
- `Directories::jig()` has 35 call lines, in `temper-engine-shell` (7) and
  the box's `temper` (28, its `src/` and `tests/`). `place` is called by
  the box alone (`temper/src/claude_code.rs`).

## Decided

- **Item 1: the launcher gets a crate of its own, `temper-launcher`,**
  rather than moving the `private_directory` call to the box. The
  launcher, the placing of its links and the substitution are agent-side
  glue the box, a worker's commands (M3) and the agent link, never the
  engine; with them in their own crate, the forge's setup links
  `temper-fetch-shell` as the fetch's host alone, and the engine links no
  launcher code at all. Moving only the call would leave the links'
  privacy check apart from the code that writes the links, and M3's
  worker would need it again in a crate it shares with the box. The cost
  is one more crate in the agent's allowed temper crates, not an
  exception: the launcher is "temper's thin agent glue"
  (`deployment-binaries.md`, section 4).
- **Item 2: `jig_directories` lives in `temper-engine-shell`,** the crate
  both callers link (the box links it already), and the box's
  `temper::directories` re-exports it, as it does `program_environment`.
  M3's worker, which won't link the engine's shell, finds its own home
  for it then.
- **Item 3:** `temper-version` joins the agent's allowed temper crates;
  release.md 3 has every binary's package link it, and it links nothing.
- **Item 4, the two closure tests' shapes:** the design now says what is:
  `protocol/README.md`, 4.1 names both tables, `binaries.rs` by the crates
  a row excludes (the engine's) and `closure.rs` by those it allows (the
  agent's), and 4.4 is the second shape. The gap left, one test of one
  shape for every binary, is the draft's, written as a **Design lags**
  in `draft/deployment-binaries.status.md` and left to M5, which owns the
  dependency rule's test; the README status's **Not yet** line for it
  goes.
- **The carried M4's statuses cited `fdc26f71c6` and PR #78,** which are
  no objects of this repository. The temper statuses this cut touched
  now say M4 is carried onto main by this plan. smith's two statuses that
  cite it (`smith/docs/design/shell.status.md`,
  `protocol/limits.status.md`) are under `smith/docs/design/**`, which
  tests read, and are left as they are.

## Design written by this cut

- `temper/docs/design/protocol/README.md`: 4.1 (*What it links is held by
  a test*, two shapes), 4.3 (*Its arguments* 1 and 3, *One run*, *It
  links*), 4.4 (its opening and the agent's row: `temper-launcher`,
  `temper-version`), section 5 (the crates' list and the rows
  `crates/temper`, `crates/temper-agent`, `crates/temper-fetch-shell`, a
  new `crates/temper-launcher`, `crates/temper-engine-shell` with
  `jig_directories`, `crates/temper-shell` with no jig crate).
- `temper/docs/design/protocol/setup.md`: 6.6 (*Its launcher*) and
  section 11 (the tree's `temper-fetch-shell`, `temper-launcher`,
  `temper-agent` and `temper`; *The forge's setup links no launcher*).
- `temper/docs/design/README.md`, section 9: `temper-launcher`.
- Statuses: `protocol/README.status.md` (three **Not yet**, one per task;
  the present paragraph), `protocol/setup.status.md`, `release.status.md`,
  `README.status.md`, `STATUS.md`; the drafts' `agent-binary.status.md`
  and `deployment-binaries.status.md` (the launcher's crate in the
  departures, the two shapes as a lag), and the execution file's M4 row.

## Parts and tasks

| Part | Task | Needs | Effort | Minutes |
|---|---|---|---|---|
| `engine-closure` | `launcher-crate`: `temper-launcher` from `temper-fetch-shell`'s launcher; the readers' paths; the forge scan's exemption path; the agent row gains `temper-launcher` | — | high | 60 |
| `agent-closure` | `shell-without-jig`: `Directories::jig()` out of `temper-shell` into `temper_engine_shell::jig_directories`, 35 call lines; `temper-shell` without `jig-protocol-config` | — | medium | 45 |
| `agent-commit` | `agent-version`: `temper-agent --version` from `temper-version`; its build script gone; the agent row gains `temper-version` | `launcher-crate` | medium | 25 |

The shapes they share are `notes/contracts.md`; the standing rules
`notes/common.md`.

## Critical path

`launcher-crate` (60) -> `agent-version` (25): about 85 implementer
minutes. `shell-without-jig` runs beside them. Both rows are green on the
trunk once `launcher-crate` and `shell-without-jig` have merged;
`agent-version` keeps them green with `temper-version` in the agent's row.

## Overlaps

- Between this plan's tasks: `agent-version` waits for `launcher-crate`
  because both edit `temper-agent/Cargo.toml` and `closure.rs`'s agent
  row. `launcher-crate` and `shell-without-jig` share only `Cargo.lock`
  (different packages' entries) and `protocol/README.status.md`, whose
  entries the cut placed apart.
- With the drafts beside this one: `transcript-window`'s files are
  `temper-service`, `temper-engine-domain` and jig's; none is this plan's.
  The queued drafts named no files. `shell-without-jig` edits 14 of the
  box's `temper` files at their `.jig()` lines alone; a queued plan that
  edits the same lines would meet it there.
- `temper-shell` is the shared crate of temper/ui's and temper/tui's
  binaries: `shell-without-jig` removes its `Directories::jig()` and its
  `jig-protocol-config` dependency. A later caller in those trees uses
  `temper_engine_shell::jig_directories`, or, in a crate that doesn't link
  the engine's shell, `jig_protocol_config::Directories::given`.

## Not in this plan

- `temper-client`'s own build script (`TEMPER_CLIENT_COMMIT`), a recorded
  departure of M1's (`protocol/README.status.md`, 4 and 5): the request
  names `temper-agent` alone.
- Any new Cargo feature, any figure (the request).
