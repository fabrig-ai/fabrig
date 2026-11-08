+++
id = "ctl-story"
part = "story"
kind = "feature"
needs = ["client-version"]
files = [
  "temper/crates/temper/tests/suite/client_binary.rs",
  "temper/crates/temper/tests/process.rs",
  "temper/crates/temper/tests/support/ctl.rs",
  "tools/tiers/packages.toml",
  "tools/tiers/design.md",
  "temper/docs/design/protocol/README.status.md",
]
effort = "high"
minutes = 60
+++

# The story: `temper-client ctl` signs in to an engine and reads `me`

## Goal

Section 10 of `temper/docs/design/draft/deployment-binaries.md` asks a story that
signs `temper ctl` from `temper-client` in to an engine and reads `me`
(`temper/docs/design/protocol/README.md`, section 4, *Its story*). Write it in
`temper`'s process tier and compile it; **it is not run in this plan**: it runs
after landing, when `temper-client` dispatches `ctl` (`client-table`). Its
engine is the box's `temper serve`, since `temper-engine` (M2) isn't on main;
its status entry tells M5 to switch it.

Acceptance:

1. **`Call::by`** in `temper/crates/temper/tests/support/ctl.rs`
   (`notes/contracts.md`, section 10): `pub fn by(self, program: &Path) -> Self`,
   the same call run by `program` as `PROGRAM ctl DIRECTORY ARGUMENT…`, its stdin
   the call's (the link, for `Call::sign_in()`); every other call is run as
   today, `Call::command`'s with no `ctl` and no directory. Doc: "Run the call by
   another binary's `ctl`, as the deployment's client runs it (protocol/README.md,
   section 4)."
2. **The story,** `temper/crates/temper/tests/suite/client_binary.rs`, included by
   `tests/process.rs` beside its other `#[path = "suite/…"]` modules:
   `ctl_from_the_deployments_client_signs_in_to_an_engine_and_reads_me`: a
   `support::Scratch`, `support::configuration(…, &support::Forge::Unreached)`,
   the client's file as contracts, section 10, says (asserted a file, the message
   naming `cargo build -p temper-client --bin temper-client`), then
   `support::ctl::run(Story::new(…, vec![Step::Run(Call::sign_in().by(&client)),
   Step::Run(Call::new(&["me", "--json"]).by(&client))]))`; asserts both calls
   exit 0 (stderr in the message), `me`'s JSON names the party's number
   (`json(me)["party"]["number"]` a number), and the scratch removed.
3. **The tier builds the client first:** `tools/tiers/packages.toml`' `[binaries]`
   `process-temper` lists `{ package = "temper-client", bin = "temper-client" }`
   after `smith-fake-claude`'s; `tools/tiers/design.md`, 2.1, its sentence on
   `process-temper`'s list names it too, as the story's sibling binary.
   `python3 -I -B tools/tiers/test_run.py` passes.
4. **It compiles:** `heavy cargo nextest run -p temper --test process --no-run`
   builds; the story is listed by `heavy cargo nextest list -p temper --test
   process -E 'test(/client_binary::/)'` (one test) and **not run**.
5. **Status:** in `temper/docs/design/protocol/README.status.md`, the bullet
   `- **Not yet:** 4: `temper-client`'s story, …` becomes `- **Departs:** 4:
   `temper-client`'s story runs the box's `temper serve` as its engine
   (`crates/temper/tests/suite/client_binary.rs`), because `temper-engine` isn't
   built; M5 switches it to `temper-engine`
   (`../draft/deployment-binaries.execution.md`, M5).` If `temper-engine` is on
   your base, start it in place of `temper serve` as the cluster's sibling story
   starts it, and delete the bullet instead. No other design text.

## Inputs

- `temper/docs/plan/client-binary/notes/common.md`; `notes/contracts.md`,
  section 10.
- Design: `temper/docs/design/protocol/README.md`, section 4 (*The deployment's
  client*, *Its story*); `temper/docs/design/draft/deployment-binaries.md`,
  section 10; `tools/tiers/design.md`, 2.1.
- The code: `temper/crates/temper/tests/process.rs` (its `#[path]` modules, and
  `an_adopt_cut_by_a_signal_is_made_once`, a story of `Call::sign_in()` and `me`);
  `temper/crates/temper/tests/support/ctl.rs` (`Call`, `Call::command`,
  `Calls::start`'s arguments, about line 664, `Story::new`, `run`, `json`);
  `temper/crates/temper/tests/support/mod.rs` (`Scratch`, `configuration`,
  `Forge::Unreached`); `temper/crates/temper/tests/suite/admission_edges.rs` (a
  configuration with `Forge::Unreached`); `tools/tiers/packages.toml`.

## Owns and may change

Owns the new story file, its `mod` lines in `process.rs`, `Call::by` and what it
needs inside `support/ctl.rs`, the `packages.toml` entry, the one sentence of
`tools/tiers/design.md`, 2.1, and the one status bullet. Not yours: the
`temper-client` package (`client-table` gives it `ctl`), any other story.

Options, each decided:

- **`Call`'s fields:** a new field for the binary is fine; `program`'s meaning
  (no `ctl`, no directory) stays as it is.
- **Unused-code lints** in the process binary for `Call::by` read by one story:
  none expected, since the story reads it; else a scoped `#[expect]`.

## Steps

1. `Call::by`, the story, the `mod` line. `heavy cargo nextest run -p temper
   --test process --no-run` until it builds.
2. The tiers' entry and sentence; `python3 -I -B tools/tiers/test_run.py`.
3. The status bullet; `python3 -I -B tools/citations/citations.py --base
   origin/main`.
4. Once: `cargo fmt`; `heavy cargo clippy -p temper --all-targets -- -D
   warnings`; `heavy cargo nextest list -p temper --test process -E
   'test(/client_binary::/)'`. Run no story.
5. Commit ("The deployment's client's ctl story, on the box's serve, compiled")
   and push.

## Report

`Done:`, `Failed:` or `Question:` first, then the files, that the story compiled
and was not run (it runs after landing), whether `temper-engine` was on your base,
the runs and their counts, anything left.
