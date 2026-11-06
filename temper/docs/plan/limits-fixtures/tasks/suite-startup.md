+++
id = "suite-startup"
part = "temper-suite"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/tests/suite/startup.rs",
  "temper/crates/temper/tests/suite/forge.rs",
  "temper/crates/temper/src/**",
  "temper/crates/temper-init/src/**",
  "temper/docs/plan/limits-fixtures/figures/suite-startup.md",
]
effort = "high"
minutes = 75
+++

# suite-startup: what `temper::suite`'s `startup` and `forge` tests pay to start

## Goal

`temper::suite`'s `startup` reads 5.3 CPU s over 89 tests at a 26 ms median, and `forge` 2.1 s over 31 at 44 ms, over a floor of 6 ms. Many of them run the real `temper` binary, and so pay what a person's box pays for a command. Profile the `temper` process and the test around it, and fix what's worth fixing, in the product first. This is the request's smallest row: stop early.

**Acceptance:**

- `figures/suite-startup.md` gives `Ir` before and after, by test name, for the cheapest and a median test of `startup::`, and of `forge::`, summed over their processes. Each says what it built before its first assertion, and how much is the `temper` process.
- The causes table has every cause over the stop rule: fixed, or with what fixing it would take. **For this binary, the stop rule is a gain under about 0.3 CPU s** across the two rows.
- A product change has its own focused test, and changes no behaviour: the same words, the same exit codes, the same files.
- Every test of `startup::` and `forge::` passes, and `temper`'s other focused tests pass.

## Inputs

- `temper/docs/plan/limits-fixtures/notes/common.md`: the rules, how to profile, your figures
- `temper/docs/plan/limits-fixtures/inputs/request.md`, items 1 and 3 and `For every task of this plan`
- `skein/docs/foundation/testing-strategy.md`, 8 and 8.1
- `temper/docs/design/domain/testing.md`, 2.6: what the process tier holds, so that you can see which of these belong where they are. They stay where they are in this plan.

## Owns and may change

- **Owns:** a test's own setup lines in `temper/crates/temper/tests/suite/startup.rs` and `tests/suite/forge.rs`; product code in `temper/crates/temper/src/**` and `temper/crates/temper-init/src/**`, where the profile puts the cause; `figures/suite-startup.md`.
- **Leave to `limits-heavy`:** `tests/suite/box_listeners/**`, `tests/suite/forge_land.rs` and `src/fake_backends/**`.
- **May change, small and only if the profile puts a cause there:** `temper/crates/temper/tests/support/mod.rs` (`Scratch`), `tests/support/owned.rs` and `tests/support/configuration.rs`. `model-catalogue-2/process-fixtures` is changing `tests/support/**`, so keep a change there to lines that save something, and name them.
- **May change, where the profile puts the cause, small, with a focused test:** `temper/crates/temper-service/src/startup.rs` and `src/startup/**`, `jig/crates/jig-protocol-config/src/**`, `skein/crates/skein-json/src/**`. `model-catalogue-2/stock-models` is changing `temper-service/src/startup*` and `model-catalogue-2/config-models` is changing `jig-protocol-config`, so keep a change there small.
- **Options, each pre-approved:**
  - **If the cost is before `main`'s first line** (the loader relocating a large binary), that's no test's and no setting may change: a line in the causes table, with its share, and leave it.
  - **If a command does work its arguments don't need** (a whole configuration parsed or a stock built for a command that needs neither, as `temper --help` or a refusal), do it only where it's needed: a person's box pays it too.
  - **If `temper_init::files` is the cost,** look at what `temper init` builds; `service-fixture` may also have changed it, so read the trunk's version first.
  - **If a test's own script or file setup is the cost** (`support::owned::scripted` writes scripts and `/bin/sh` runs them), that's what these tests check, unless a test has no use for a piece: a line in your causes table.
  - **If nothing's over the stop rule,** say so, with the counts: that's a good outcome here, and your figures are the report.

## Steps

**From reading, not found (`notes/common.md`):**

- `tests/suite/startup.rs` has 89 tests and 17 lines naming `skein_scratch::paths::binary("temper")`, so some of its tests run a process or two, and others check a value with no process at all: read which is the median.
- `tests/suite/forge.rs` builds `Scratch` and `support::owned::scripted`, runs `temper forge prepare` (`tests/support/owned.rs:47`), and that runs `/bin/sh` scripts.
- `temper/crates/temper/src/main.rs` installs the fail-stop handler, then goes through `temper::configure_with` and `temper::run`.

**How to measure:** add `--trace-children=yes` to the tracer (`notes/common.md`), and use `-p temper --test suite`. Each `%p` file is one process, and a test's figure is their sum, listed by `cmd:`. **Callgrind sees no kernel time,** and much of these tests' cost is the kernel's: `exec`, page faults and files. So a row's kernel share is `unknown`, unless `strace` is on the machine, when `--tracer 'strace -c -f'` on two tests gives their system calls: say which.

**The work:**

1. Choose the tests and take the before figures, at your branch's start.
2. Read the annotations of the test's process and each `temper` process. Say what's the loader, the configuration, `temper init` and the test.
3. Fix the largest cause over the stop rule. Run the narrowest test (`heavy cargo nextest run -p temper --test suite -E 'test(=NAME)'`), and read its `Ir` again.
4. Repeat until the stop rule.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`, and `<tree>/scripts/old_forms.py` for any other tree you touched; `heavy cargo clippy -p temper --all-targets -- -D warnings`, naming any other package you changed.
6. `heavy cargo nextest run -p temper`, and `-p temper-init` if you changed it. A change in another crate also runs that crate's own tests, and its readers by `cargo tree -i PKG --depth 1`, with `heavy cargo check -p PKG --tests` for those in another tree. `temper`'s focused tests are all this runs: its process tier isn't yours to run.
7. Write the figures, commit and push.

## Report

The counts before and after, the causes fixed and not fixed, and what's kernel time or `unknown`.
