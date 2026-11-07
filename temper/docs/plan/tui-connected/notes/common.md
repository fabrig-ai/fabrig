# tui-connected: what every task holds to

- **Focused and fuzzy alone** (`AGENTS.md`, 5; the user, 2026-11-06). Never
  `tools/tiers/run`, a `<tier>-<tree>` profile, `scripts/browser.sh`, a walk,
  a render check, a throwaway driver or a live run. A story for a slow tier
  is written and compiled (`heavy cargo check -p PKG --tests`), never run.
- **Nothing real in a test:** no real terminal or pseudo-terminal, no
  browser, no real engine in a focused or fuzzy test. The terminal runs on
  skein's simulated machine; the TUI's host runs in temper's client joint
  world; the binary's parts are unit-tested as functions.
- **Tests stay cheap.** A new case of one behaviour is a row of an existing
  table test (`skein/docs/foundation/testing-strategy.md`, 8); a new test is
  one your spec sketches. skein's focused suite has 30 CPU seconds and its
  fuzzy 120, temper's client and protocol worlds a combined guide of 8
  focused seconds (`temper/docs/design/domain/testing.md`): name your new
  tests' time in the report. Budgets and timeouts are the user's.
- **The contracts bind** (`temper/docs/plan/tui-connected/notes/contracts.md`):
  names, public types and signatures as written there. Inside them the
  choices are yours, and the report names those that matter.
- **`docs/design/` stays closed** but for the status entries your spec
  names: remove each **Not yet** entry, or the part of one, that your work
  closes, and nothing else. The design sections are written; what the code
  taught goes in the report.
- **A new crate** is a member of the root `Cargo.toml` and an entry of its
  `[workspace.dependencies]`, beside its siblings; its lint block comes from
  `python3 -I -B tools/lints/lints.py --set PROFILE PATH` after it is a
  member, and its `clippy.toml` (where `notes/contracts.md`, section 1, gives
  one) is a copy of the canonical variant in `tools/lints/clippy/`. Its
  `lib.rs` cites its design at module level.
- **The strict subset** in step crates (`skein/docs/foundation/programming-model.md`,
  10): no `std`, no indexing, no unchecked arithmetic, no `as`, no `unwrap`,
  every `match` naming every variant.
- **macOS stays cheap:** ordinary Rust here uses skein-shell's and
  skein-io's portable parts alone, nothing Linux-only, so it compiles there.
- **Other plans' files stay untouched:** `temper/crates/temper-client-view/src/pages/{terminal,chat,session}.rs`,
  `temper/crates/temper-client-view/src/tests/terminal.rs`,
  `temper/crates/temper-client-shell-web/temper.css` and
  `temper/crates/temper-client-domain/src/pages/terminal.rs` (look-round-17);
  jig's sign-in, the web's pages and `temper-client-domain` (tui-approval).
  This plan changes nothing in `temper-client-domain` or `temper-client-view`
  but the one export a spec names.
- **Generated files** (`docs/help/reference/*`, goldens, counted baselines)
  that your change makes stale are yours to regenerate with their own tool,
  never by hand; a golden this plan adds is written by hand, as the design
  says, and checked by its test.
- **A test's oracle or helper that your change's correct behaviour trips is
  yours to repair,** keeping what it checks for everything else, and named
  in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it
  once on the pristine base. If the base passes and your branch fails, it's
  yours.
- **Old-form counts only fall:** `python3 -I -B <tree>/scripts/old_forms.py`
  for each tree you touch; a raised count is yours to fix.
- **Citations:** `python3 -I -B tools/citations/citations.py --base origin/main`
  before pushing; a comment citing a design section cites one that exists.
