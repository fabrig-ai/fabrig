# client-binary: the plan

M1 of `temper/docs/design/draft/deployment-binaries.execution.md`: the
deployment's client, `temper-client`, a package and binary of its own that runs
the TUI and `ctl` by the code the box's `temper` runs (`inputs/request.md`;
the draft's sections 4 and 9.1). Design written by this cut:
`temper/docs/design/protocol/README.md`, section 4 (*The deployment's client,
`temper-client`*) and section 5 (the rows `crates/temper-client` and
`crates/temper-shell`, the shells' rows); `temper/docs/design/client/README.md`,
section 2 (the crate tree, `temper-client-shell-tui`'s `start`, *The clients'
start, one for two binaries*), 6.11 (*In the deployment's client*) and 6.12
(`temper-shell`'s `link_file::path`); their status files; the draft's status
(the section-4 lag narrowed, the release lag's `--version`, a **Design
departs**) and M1's state in the execution file.

## The shape

```
temper-shell                    new; what every binary of temper's starts from: lean, skein's and temper-docs alone
  directories                   the three directories' table and resolution, the private directories
  link_file                     the engine's link at rest: the engine writes it, the TUI reads it
  commands, rows, help          the table's types and how a binary's table prints, for its name; the shared rows; help's answers
temper-client-shell-terminal    + address: an engine's address or a sign-in link, for the TUI and temper open
temper-client-shell-tui         + start: the TUI's arguments and start, run(name, …)
temper-client-shell-cli         + start: ctl's start, run(…), its lines `temper ctl:` in both binaries
temper (the box)                reads all of the above; keeps its names for its readers; prints byte for byte as today
temper-client                   new package and binary: help, [ADDRESS], ctl; --help, --version (build script)
```

## Decisions

- **`temper-shell`, not a client crate.** The request puts the TUI's address
  and link-file helpers in "a client crate both binaries link". The link file
  is the engine's too (serve writes it, the TUI reads it), and so is a binary's
  table (M2's `temper-engine` needs one): a lean crate every binary links keeps
  the engine off the TUI's crates, as the draft's rule for shared code has it
  (section 4). It isn't a client crate by name, so the draft's status records
  it as **Design departs**, and M5's boundary test allows it. The TUI's address
  helpers and start do go into a client crate, `temper-client-shell-tui`.
- **Each start in its shell,** as smith's agent start is in
  `smith-agent-shell`: no new client crate. The box's `temper` keeps its
  modules (`tui`, `ctl`, `help`, `directories`, `commands`) as thin names over
  the moved code, so `lib.rs`' dispatch, every reader in `temper`, its tests and
  `temper/tests/real-loop` compile unchanged, and the in-flight `tui-box-front`
  meets small merges.
- **The binary's name is a parameter,** `"temper"` or `"temper-client"`, in
  every line a binary prints about itself (usage, refusals, the TUI's lines).
  The rows' prose, shared, names `temper`, as `ctl`'s own help and lines do: no
  behaviour of `ctl` changes (the request). The published file's name stays the
  user's (draft, section 5).
- **`--version` is a flag, not a row,** as `--help` is; `version` is no command
  of the client's. Its commit comes from the package's own build script, so only
  `temper-client` rebuilds when `HEAD` or the index moves. The box's `temper
  --version` stays `release.md`'s **Not yet**.
- **The story runs the box's `temper serve` as its engine:** `temper-engine`
  (M2) isn't on main. It lives in `temper`'s process tier, which finds
  `temper-client` beside `temper` and builds it first (`tools/tiers/packages.toml`).
  Its status entry tells M5 to switch it.
- **No compatibility kept:** `temper`'s internal APIs move freely; nothing
  stored or on the wire changes.

## What the client links (for M5's boundary test)

From today's closure of `temper-client-shell-tui`, `temper-client-shell-cli`,
`temper-docs` and skein's (`cargo tree -e normal`), the client links, beyond the
client crates, jig's client crates, skein's and `temper-shell`:

- `jig-protocol-accounts`, through `temper-client-shell-cli` (ctl's account
  calls), which brings `jig-protocol-config`, `jig-protocol-engine-clients`,
  `jig-protocol-engine-clients-http`, `jig-protocol-store`, `jig-domain-*` and
  `skein-oauth-issuer`: ctl's host reads jig's accounts protocol today; cutting
  it is a change of ctl's, which the request rules out.
- `jig-protocol-transcript` and smith's `smith-transcript`, below `jig-client`
  (turns decoded): by design (`temper/docs/design/client/README.md`, section 2,
  *below the client*).
- `temper-settings-form`, through `temper-client-tools`: the settings file's
  form, shared by the client and the engine by design.
- `temper-protocol-records`, through `temper-client-domain`, without its `forge`
  feature when the client is built alone (`client-table` checks it).

## Parts and tasks

| Part | Task | Crates (files) | Needs | Minutes |
|---|---|---|---|---|
| `local` | `shell-local` | temper-shell (new: `directories`, `link_file`); temper (`directories.rs`, `link_file.rs` gone, one `lib.rs` line) | | 75 |
| `ctl` | `ctl-start` | temper-client-shell-cli (`start`); temper (`ctl.rs`) | | 40 |
| `version` | `client-version` | temper-client (new: package, build script, `--version`) | | 75 |

Later batches (being written): `shell-table` (temper-shell's `commands`, `rows`,
`help`; temper's `commands.rs`, `help.rs`), `tui-start`
(temper-client-shell-tui's `start`, temper-client-shell-terminal's `address`;
temper's `tui.rs`, `open.rs`), `ctl-story` (the process tier's story,
compiled), `client-table` (temper-client's table, help and dispatch; the
status files closed).

## Overlaps

- **`tui-box-front`** (integration) changes `temper`'s `lib.rs`, `tui.rs`,
  `commands.rs`, `src/tests/tui.rs`, `src/tests/commands.rs` and adds
  `tui_front.rs`, which reads `tui`'s helpers. This plan's hunks there are small
  and apart from theirs: `shell-local` one `lib.rs` line; `shell-table`
  `commands.rs`' types and functions, not its rows but the three shared ones;
  `tui-start` `tui.rs` (its helpers move) and `src/tests/tui.rs`. Whichever
  lands second catches up: `tui_front.rs` reads the moved helpers by their new
  path, or through `temper`'s `tui` names (`notes/common.md`, *When
  `tui-box-front` has landed*). Their `temper-client-shell-tui` files (`host.rs`,
  `run.rs`, `limits.rs`, `tests.rs`, `journey/**`) are untouched: `start` is a
  new file there, with one `lib.rs` line.
- **Status files:** `client/README.status.md` and `protocol/README.status.md`
  are edited by several plans; this plan appends its entries at their ends.
- **M2 (`temper/ui`)** splits `temper-service`: this plan touches no service
  or engine crate.

## Critical path

As it stands: `shell-local` (75) → `shell-table` (90) → `client-table` (90):
about 4.3 hours from the first push, with `tui-start` beside `shell-table`, and
`ctl-start`, `client-version` and `ctl-story` beside them.
