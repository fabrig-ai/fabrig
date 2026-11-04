# outward-names: contracts

The run's binding calls. A task builds to them; a spec that departs says so.

## 1. A script's build wrapper

A script that builds or tests (`temper/scripts/proof-live.sh`, `proof-live-github.sh`,
`proof-live-github-managed.sh`, `temper/tests/real-loop/tests/fixtures/{b0,b1}/regenerate.sh`,
`jig/scripts/live-sign-in.sh`) runs the plain command. A wrapper, when a caller wants one, comes
from the environment, in the tree's own variable:

| Scripts | Variable |
| --- | --- |
| temper's (`temper/scripts/*.sh`, the two `regenerate.sh`) | `TEMPER_WRAPPER` |
| jig's (`jig/scripts/live-sign-in.sh`) | `JIG_WRAPPER` |

- **Value:** one command, a name looked up on `PATH` or a path, with no arguments of its own.
- **Set and not empty:** each command the script ran through the build tool before (`$heavy
  cargo ...`, `heavy env ...`) is run by it, the command's own words following:
  `"$TEMPER_WRAPPER" cargo nextest run ...`, `"$TEMPER_WRAPPER" env TEMPER_BUNDLE=none ... cargo
  run ...`. A command the script ran plainly before stays plain
  (`jig/scripts/live-sign-in.sh`'s live test run).
- **Unset or empty:** the command runs plainly: `cargo nextest run ...`.
- **Shell form:** `${TEMPER_WRAPPER:+"$TEMPER_WRAPPER"} cargo ...` (and `exec` before it where
  the script execs). No word splitting, no `eval`, and it holds under `set -u`. The
  `# shellcheck disable=SC2086` lines that served the old unquoted form go where nothing else
  needs them.
- **No tool is looked for:** `heavy=$(command -v heavy || true)` and every use of `$heavy` go; no
  script refuses to run for want of a wrapper (`proof-live.sh rung-b0-offline` runs plain cargo
  when none is given).
- **The comment** that said "The machine's build queue where it has one, plain cargo where it
  hasn't" becomes, in the script's words: the variable, when set and not empty, names one
  command that runs each build command below; unset, cargo runs plainly.
- **Tests that stub the build** put an executable named `cargo` first on the `PATH` they give
  the script, where they put one named `heavy` before; its arguments are cargo's own, with no
  leading `cargo`. `temper/crates/temper/tests/suite/forge_land.rs`'s `launcher()` clears the
  environment, so no wrapper reaches a script unless a test sets one.

It has the shape of the tiers' wrapper, which a parallel step adds to `tools/tiers/` (its
`design.md`, 3): a command from the environment, plain when absent. Nothing here reads or names
the tiers' variable.

## 2. `regenerate.sh DATA`

`temper/tests/real-loop/tests/fixtures/{b0,b1}/regenerate.sh` take exactly one argument, the
recorded deployment's data directory, passed as the twin's `--data`. With none, or more than
one, each prints `usage: regenerate.sh DATA_DIRECTORY` to stderr and exits 2, before building
anything. There is no default path. Everything else they pass the twin (`--base`, `--out`,
`--cue`, `--replace`) stays byte for byte, and their header comments keep what they say about
the recording, with any scratch path said as "the walk's data directory".

## 3. The wording

Each name becomes the plain command or generic words, the same in every tree:

| The lab's name | Becomes |
| --- | --- |
| `heavy cargo X`, `heavy --gate cargo X` | `cargo X` |
| `heavy env A=B cargo X` | `env A=B cargo X` |
| a machine (`hel8`, `hetzner9`, `debian-16gb-hel1-1`) in a dated review | "the review machine" |
| a machine in a benchmark or measurement report or README | "the benchmark machine" (smith), "a builder" (elsewhere) |
| `~/src/fabrig/<x>` in a dated review | what it was: "a fresh clone", "the primary checkout", "the review machine's notes", "the review machine's coordination records" |
| `~/src/rust/tmp/<x>` in prose | "the scratch directory" or "a private directory outside any repository" |
| `/home/free/` as a script's default under the person's home | `$HOME/` (the same path for its owner, so behaviour holds) |
| `/home/free/` in recorded data | `/home/user/` (same length; `smith-names` decides whether the recording allows it) |
| `fab` in a dated review or debt item ("pre-fab pipeline", "fab's") | "the former pipeline" |
| `fab` elsewhere ("fab's gate list", "fab's `<draft>/integration`", "fab's records of a change") | the thing itself: "the gate's list", "a plan's shared branch (`<draft>/integration`)", "a runner's records of a change" |

A gate or lane distinction a README draws on a machine ("the build lane", "the gate lane")
becomes what it asks of the run: build anywhere; measure on an otherwise idle machine, one run
at a time.
