# ui-cli-c1: rules every task keeps

- **The request** is `temper/docs/plan/ui-cli-c1/inputs/request.md`:
  milestone C1 of `temper/docs/design/draft/cli.execution.md`, the
  command line's conventions and their mechanism. C1 **adds and moves
  nothing**: no binary moves, no variable or command is renamed, no
  feature goes, no table is given to a binary. Renames and moves are C2
  to C8's. Where something doesn't follow the conventions yet, it gets
  an exception or a scoped `expect` naming the milestone that removes
  it, by the map in `notes/contracts.md`, section 4.
- **The design you build** is `skein/docs/foundation/cli.md` (the
  conventions), `skein/docs/design/shell.md`, 6.5 and 15 (skein-shell's
  settings and table) and `tools/lints/design.md`, 4 (the environment
  ban). The draft `temper/docs/design/draft/cli.md` is the why; build to
  the design.
- **The shapes that cross tasks** are `notes/contracts.md`: binding.
- **Made-up names in tests' data take no tree's prefix**
  (`skein/docs/foundation/cli.md`, 9, check 5): a test's variable is
  `DEMO_CONFIG`, never `TEMPER_CONFIG`, `SKEIN_X` or `JIG_X`.
- **Only the focused and fuzzy suites run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, `scripts/browser.sh`, a browser, process or
  live tier story, a walk, a render check or a throwaway driver.
- **Tests stay cheap:** a new case of one behaviour is a row of an
  existing table test. `fabrig-boundary` runs on every change
  (`.config/gate/always.txt`): a check there reads what it reads once,
  and runs well under a second.
- **Every build and test through `heavy`,** with `TEMPER_BUNDLE=none` (the
  machines' cargo config sets it). `cargo fmt`, `cargo metadata` and
  `python3 -I -B tools/lints/lints.py` build nothing and run directly.
- **Clippy may be red outside your task while the environment ban
  lands:** once `env-ban` has merged, a crate whose reads another task
  of this plan covers (`env-jig-hearth`, `env-temper-src`,
  `env-temper-tests`) fails `disallowed_methods` until that task merges.
  A `disallowed_methods` finding on a read you didn't write isn't yours:
  name it under `## Left` and go on.
- **Before calling a focused or fuzzy failure "outside the change",** run
  it once alone on the pristine base. If the base passes and the branch
  fails, it's the branch's.
- **Decide, don't ask.** A small edit just outside the file list (a
  re-export, a `mod` line, a dependency line, a regenerated golden) is
  made and named in the report. A `Question:` is only for what truly
  blocks the task.
