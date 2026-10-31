# component-kit-3: moving a host onto io's stage and the world surfaces

Every `ss-*` and `ws-*` migration cites this note beside `notes/common.md`.
It moves the hosts of its crates onto `skein_io::stage::{up, down}`
(`skein/docs/design/draft/component-kit.md`, 3.3), and, in the same files,
onto the surfaces of task `ws-shell` (3.5). Read both as merged first:
`skein/crates/skein-io/src/stage.rs`, `skein/crates/skein-shell/src/drive.rs`
(`Host::next_mechanism_deadline`, the provided `next_deadline`, `Dispatch`)
and `skein/testing/skein-world/src/referee.rs` (the defaults).

## The io stage

- A host's up-pass io stage (`resume` while ready, `up` for each completion,
  `fire` while due, in any spelling) becomes one `skein_io::stage::up(...)`;
  its down-pass loop over io's requests becomes `skein_io::stage::down(...)`.
  The stage checks the kernel's room before each step (`MAX_OUT_*`), so a
  site's own room checks go with the loop; a site without them gains them.
- `stage::up` reads the time from `env.now`: make sure the env the site
  passes carries this iteration's time.
- **Order.** The stage keeps io.md 2's order (the ready list, then
  completions, then deadlines). A site with another order (completions
  first) takes the stage's. If a test then fails because it pinned the old
  order, read it: an assertion of the order io.md 2 forbids is updated, and
  the report names it; anything else is a `Question:`.
- **Between steps.** A site that handles io's events between single io
  steps (after each `resume`) handles them after the stage instead, unless
  the handling feeds io before its next step; then that is a `Question:`.
- **One step alone.** A test that meters or asserts a single io step by
  itself (a heap ceiling per `resume`, an assert between one `resume` and
  the next) keeps calling `skein_io::resume` there: name each such site in
  the report, which leaves it in the `io_resume` count.
- A test that drives io by hand to a point and asserts (`while io.is_ready()
  { resume }`, then an assert) calls `stage::up` with an empty completions
  queue at that point.

## The world surfaces

- **Referee:** an `act` that does nothing, a `next_deadline` that returns
  `None` and an `overdue` that returns `None` go: the defaults are the same.
- **Host deadlines:** a `next_deadline` that merges the host's layers with
  `self.next_policy_deadline()` becomes `next_mechanism_deadline`, the same
  merge without the policy term; the provided `next_deadline` adds it back.
  A `next_deadline` without the policy term is renamed likewise (the
  earlier of a set and its subset is the set's). If a host's `next_deadline`
  leaves out a deadline its `next_policy_deadline` returns, keep its
  override and say so: the provided one would add a wake.
- **Dispatchers:** an `enum Process` (or another enum of hosts) whose
  `impl Host` forwards each method to the variant it holds becomes
  `impl skein_shell::Dispatch for Process` with `host` and `host_mut` (the
  inherent accessors many already have move into it), and its `impl Host`
  goes. A wrapper whose `Host` adds behaviour to a method (a cut, a restart)
  keeps its impl and the report names it.
- Use `skein_lib::earliest` for any merge of optional deadlines the move
  touches.

## Counts and checks

- From `skein/`: `python3 -I -B scripts/old_forms.py --write`. `io_resume`
  falls by each site moved; `forwarded_host` by each dispatcher moved.
- `cargo fmt`; `heavy cargo clippy -p PKG ... --all-targets -- -D warnings`
  for the packages owned; `heavy cargo nextest run -p PKG ...` for them.
  A test crate has no dependents; a `testing/` or `crates/` package also
  checks its direct dependents with `heavy cargo check -p <each> --tests`
  (`cargo tree -i PKG -e normal,dev --depth 1`).

## Report

Each site moved (file, before and after in one line), each site left and
why, the referee methods and host deadlines removed or renamed, the
dispatchers moved, the counts before and after, and the tests run with
their counts.
