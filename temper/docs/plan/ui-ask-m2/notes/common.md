# ui-ask-m2: rules every task follows

- **Read first:** fabrig's `AGENTS.md`, the tree's `AGENTS.md` (`jig/` or
  `temper/`, as your spec's files are), your spec, and
  `temper/docs/plan/ui-ask-m2/notes/contracts.md`, which binds the shapes
  that cross tasks. Where a spec and the note differ, the note wins; say
  so in the report.
- **The request:** `temper/docs/plan/ui-ask-m2/inputs/request.md`, read
  whole (it is short). Its `For every task of this plan` list binds every
  task; the lines below repeat what bites here.
- **Only focused and fuzzy run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, a browser, process or live tier, or
  `scripts/browser.sh`. A test of those tiers your change must touch is
  edited and compiled (`heavy cargo check -p PKG --tests`), never run; say
  so in the report. No walk, render check, screenshot or throwaway driver:
  the plan is complete when it lands.
- **Tests stay cheap.** A new case of a behaviour is a row of an existing
  table test where one fits; a new test only where none does, beside its
  siblings' fixture. temper's focused and fuzzy suites have 30 and 120 CPU
  seconds in the gate; a fuzzy test of this plan runs at most 64 seeds.
- **Fakes, never the real thing:** jig's core world (`jig/tests/core`,
  package `jig-core-world`) and its fixtures
  (`jig/tests/core/tests/settings/support.rs`); the engine's domain world
  (`temper/tests/engine/domain`, package `temper-engine-domain-world`),
  whose asking chat is set up in `tests/suite/configuring.rs`; the
  forge's own fixtures
  (`temper/crates/temper-engine-domain-forge/src/tests/fixtures.rs`).
- **One frame, temper's words:** an answer names numbers with their
  names, times in UTC, money in the client's one unit, and never a key, a
  digest, a cursor or a command (`temper/docs/design/domain/agent.md`,
  8.4.1, *How an answer reads*).
- **Design files are the cut's,** with one departure each spec states: the
  task deletes the **Not yet** entries it closes from the status files it
  names, and touches no other design text.
- **Help pages:** `temper/docs/help/` is closed but where a spec names a
  page; a page your change pushes past its byte bound is condensed, losing
  no fact, never its bound raised.
- **Generated files your change makes stale** (`temper/docs/help/reference/*`,
  goldens, prints, counted baselines) are yours to regenerate with their
  own tool, in the same task, and to name in the report. On a catch-up,
  regenerate rather than hand-merge.
- **A test's oracle or helper that the correct new behaviour trips** is
  yours to repair in the same task, keeping what it checks for everything
  else, named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run
  it once on the pristine base. If the base passes and your branch fails,
  it is yours.
- **Decide, don't ask.** A stored format, a protocol or a public interface
  may change freely: no compatibility. A small edit just outside your files
  (a re-export, a `mod` line, a doc comment, a golden) is made and named in
  the report. A `Question:` is only for what truly blocks your acceptance.
- **Not to touch:** `temper/docs/design/**` and `jig/docs/design/**` but
  the status entries your spec names, and every other task's owned files
  below.

| Task | Owns |
|---|---|
| `person-surface` | jig's `NamedAction::PersonRead`, `Now::PersonAnswer` and the core's `reads/person.rs` until `person-reads`; each forced arm of `Now::PersonAnswer` |
| `person-reads` | `jig-domain-core`'s `reads/person.rs` and the read pipeline's person destination; `jig/tests/core`'s `person_reads` stories |
| `see-surface` | `temper-engine-domain`'s `route/see.rs` address grammar and `SeeAnswer`'s new variants and records, until `see-route` |
| `see-route` | `temper-engine-domain`'s `route/see.rs`, `route/see/`, `route/now.rs`'s person arm, `route/asks.rs`'s `Ask::Read`, the flight in `host_route.rs` |
| `see-render` | `temper-engine-smith`'s rendering of `SeeAnswer::Person` for `me`, `project`, `goals`, `tasks`, `task N`, `task N tree`, `inbox`, and its test table |
| `see-render-runs` | the same, for `task N activity`, `task N turn A.T`, `change N`, `changes` |
| `see-page` | `temper-engine-smith`'s table of addresses, the reference page `see`, the declaration's description; `SeeAnswer::NotYet`'s removal |
| `see-stories` | the engine domain world's `configuring/asking_person.rs` and the helpers it adds |
| `see-fuzzy` | the engine domain world's `fuzzy/see.rs` |
