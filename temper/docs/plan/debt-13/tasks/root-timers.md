+++
id = "root-timers"
part = "root-timers"
kind = "feature"
needs = []
files = ["skein/crates/skein-lib/src/env.rs", "skein/crates/skein-lib/src/tests/env.rs", "jig/crates/jig-domain-core/src/routing/timers.rs", "jig/crates/jig-domain-core/src/tests/routing/deadlines.rs", "jig/examples/ops/domain/src/domain.rs", "jig/testing/jig-test-domain/src/lib.rs"]
effort = "medium"
minutes = 30
+++

# The wall's expiry and the decision's header, once

## Goal

`wall_due` is the same four lines in both roots, and it reads nothing but `Env`, so it's lib's. Both roots spell the
same seven lines to pick the decision's deployment header, around an `unreachable!`, so that's the core's.

Acceptance:

- `rg -n 'fn wall_due' jig` finds nothing; `Env::wall_due` is in `skein-lib/src/env.rs` with a test.
- `rg -n 'commit_header|cold_header' jig/examples/ops/domain/src jig/testing/jig-test-domain/src/lib.rs` finds nothing.
- `rg -n 'a deployment header is a save' jig` finds nothing.
- `heavy cargo nextest run -p skein-lib --test ... ` (find the binary that holds `src/tests/env.rs`), `-p
  jig-domain-core -E 'test(deadlines)'`, `-p jig-ops-world`, `-p jig-test-domain` and `-p jig-core-world` pass.
- The same commit numbers: `Core::commit_header` and `cold_header` keep their public signatures, for temper's root.

## Inputs

- `temper/docs/plan/debt-13/notes/common.md`
- `temper/docs/plan/debt-13/notes/contracts.md`, 4
- `jig/docs/development/debt/D004-ops-root-jig-wiring.md`
- `skein/docs/design/lib.md`, 9
- `jig/docs/design/domain/engine.md`, 5.4
- `jig/docs/design/domain/root.md`, 3

## Owns and may change

- **Owns:** `skein-lib/src/env.rs` and `src/tests/env.rs`; `jig-domain-core/src/routing/timers.rs` and its tests.
- **May change:** `ops`'s `domain.rs` and the test root's `lib.rs`: the `wall_due` function and its calls, and the
  header block in the decision. Those lines alone: `route.rs`, `link.rs` and `admission.rs` are other tasks'.
- **Not to touch:** `timers_due`, which differs by root. `temper-engine-domain/src/decision.rs`, which spells the
  same header block: a branch stays in its tree, and the report names it under `## Left`.
- **Options:**
  - If the core's type at base isn't named `Core`, the method goes on the type that holds `commit_header`.
  - If `Env::wall_due` needs `L: ...`, it doesn't: it reads `now` and `wall`. If a clippy lint asks for `const`, take it.

## Steps

1. `skein-lib/src/env.rs`, the body as both roots have it:

   ```rust
   /// Whether a wall-clock expiry has come, on this step's clocks.
   #[must_use]
   pub fn wall_due(&self, expiry: Wall) -> bool {
       let remaining = expiry.as_nanos().saturating_sub(self.wall.as_nanos());
       self.now.saturating_add(Duration::from_nanos(remaining)) <= self.now
   }
   ```

   Tests in `src/tests/env.rs`: past, exactly now, a nanosecond ahead, and `now` at its saturation. Break it by hand
   (`<` for `<=`) and see it red; restore.
2. `jig-domain-core/src/routing/timers.rs`:

   ```rust
   /// The deployment row this decision writes, if any (engine.md, 5.4).
   #[must_use]
   pub fn decision_header(&mut self, wrote: bool) -> Option<Record>;
   ```

   `Counters` gains the step that makes the record, so `commit_header` wraps it in `Write::Save` and
   `decision_header` returns it, with no `unreachable!`. Dirty and clean, wrote and not: a test in
   `src/tests/routing/deadlines.rs`, each case's `Some` or `None` and the commit number it took. Break `wrote` by
   hand and see it red; restore.
3. Both roots:

   ```rust
   if let Some(header) = domain.core.decision_header(domain.wrote) {
       write(domain, &mut decision, Write::Save(Record::Core(header)));
   }
   ```

   The test root's field is `decision_wrote`. `wall_due(env, due)` becomes `env.wall_due(due)`, and the private
   functions go.
4. `cargo fmt`, `heavy cargo clippy -p skein-lib -p jig-domain-core -p jig-ops-domain -p jig-test-domain --all-targets
   -- -D warnings`, the tests above, commit, push.

## Report

What changed, the commit, what ran with counts, the rules broken and what failed, the net lines, and `## Left` for
temper's copy.
