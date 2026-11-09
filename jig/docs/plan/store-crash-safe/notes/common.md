# store-crash-safe: rules every task keeps

- **The request** is `jig/docs/plan/store-crash-safe/inputs/request.md`:
  a deployment's engine, killed by SIGKILL after ignoring SIGTERM twice
  in one night, left a store whose next start panicked at
  `jig/crates/jig-domain-core/src/start/links.rs` with
  `Tasks(Live(199)) links missing People(Person(102))`: a resolution
  task's row restored, the person it names not. The evidence is outside
  the repository: the request's words are all there is. Read it for the
  why; your spec and the design sections it names are what you build.
- **What the cut found, and did not.** A read of the code found no path
  by which a kill leaves part of a decision durable: one decision is one
  skein-kv commit, applied to the map only once synced, and recovery
  keeps a whole prefix. A read of the code proves nothing. The two
  candidate families, a kill (`crash-rows`) and a writer that saves a
  link to a person the core does not hold (`writer-rows`), are each
  ruled in or out by rows; the link refusal (`link-refusal`) holds
  whatever the cause.
- **Failing test first.** Each fix is pinned by a row that fails on the
  trunk for the fault's reason before the fix and passes after it.
  Commit the failing row first, on its own, saying which candidate it
  shows; then the fix. The report quotes the failure seen before the
  fix.
- **Candidates are ruled in or out by test, never by assumption.** A
  spec that names candidates gives each a row; a row that passes on the
  trunk rules its candidate out and stays as a guard. The report says,
  per candidate, ruled in or out and by which row.
- **Fix at the lowest tier that shows it.** A fault a jig row shows in
  `skein-kv` is reproduced first as a row of skein's own crash tables
  (`skein/tests/kv/tests/suite/store.rs` or `disk.rs`) and fixed there;
  one in `jig-protocol-store` as a row of `jig/tests/store`'s crash
  tables. No stored format and no frame layout changes: a fix that
  needs one is a `Question:`, with the failing row pushed.
- **No figure changes.** Limits, bounds, deadlines and graces stay as
  configured; a row may set a world's own limits (a small snapshot bound
  in a fixture) to reach a case, never a shipped profile.
- **Tests stay cheap.** A new case of one behaviour is a row of an
  existing table test, or of the table your spec makes. Stories run on
  the simulated disk and clock; no test reaches a real host, no sleep.
- **Only the focused and fuzzy suites run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, `scripts/browser.sh`, a process or live tier
  story, a walk, a render check or a throwaway driver.
- **Every build and test through `heavy`,** one command at a time, with
  `TEMPER_BUNDLE=none` (the machines' cargo config sets it). `cargo fmt`
  builds nothing and runs directly.
- **Trees:** jig never names temper; skein names neither. A change to
  skein's `io`, `sim` or `world` interface goes to `hearth/core` first:
  none is planned here, and needing one is a `Question:`.
- **Status lines:** the design is written (`## Inputs` names it). A task
  removes only the **Not yet** entries its spec names, its lines alone.
  That departs from the implementer's text, which closes `docs/design/`
  to you: those entries are your task's.
- **Before calling a focused or fuzzy failure "outside the change,"** run
  it once alone on the pristine base. If it passes there and fails on
  your branch, it is your branch's.
- **Old forms:** `python3 -I -B <tree>/scripts/old_forms.py` for each
  tree you touch, once before pushing. A raised count is yours to fix.
- **Citations:** after committing, run
  `python3 -I -B tools/citations/citations.py --base origin/main` from
  the repository root. It reads HEAD, not the working tree.
- **Decide, don't ask.** A small edit just outside your file list (a
  compiler-forced arm, a call site's new field) is made and named in the
  report. A `Question:` is only for what truly blocks your acceptance.
