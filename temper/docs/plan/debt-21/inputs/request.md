# Request: pay debt items, twenty-first round

From fabrig/debt, 2026-11-05 (the user's continuous tech-debt process; source of truth: each tree's
`docs/development/debt/`; `python3 -I -B tools/debt/debt.py` prints the index). Independent items. fabrig/ui has
priority over builder places (the user, 2026-11-05), so keep the plan small, with no temper client view, no
`temper.css` and no `jig-client-view`. Measure with inline `#[cfg(test)]` items split out and generated code counted apart.
New tests must earn their lines: a table or helper, not copies.

Every part, on its own branch:

- reads the item file whole, and the design text it cites;
- makes the change so the item's **Paid when** holds; if the item has a **Measure**, the command
  reads 0 (or the item's target) after;
- deletes any item it pays, in the same branch; updates any status entry the item names as closed by it, and any citation that breaks
  (`tools/citations/citations.py --all` stays at 0);
- writes acceptance searches narrowly: code that legitimately uses the same word for something else
  is exempted, not changed. Never pin an exact HEAD: fab merges the trunk in at each run's start.

**Behaviour stays unchanged.** If a test's visible events or
outcome change, stop and ask. Don't retime a test or rewrite an assertion to make a refactor pass.
Stored formats and breaking public interfaces stop and ask too; additions are fine.

**A test that pins a rule must be able to fail on that rule.** Break the rule by hand, watch the test
go red, restore it, and report what was broken and what failed.

**Keep new code in proportion.** Size caps in tasks are the cut's estimates, not user budgets. Compact
first. If a cap still can't be met without losing wanted behaviour, an answer call may raise it by up
to 25%, stating the measured size and why. Beyond that, ask fabrig/debt, the plan's owner, not the user. Budgets and timeouts stay the user's.

**All parts must be in the plan when the cut ends.**

## smith: the shell-cwd grader can't compile against its frozen tree

`smith/benchmarks/tasks/repository/shell-cwd/grader/src/lib.rs`, in `Harness::new`, calls `tools::valid(&limits)`.
Commit b556bc3ed5 (10-28) changed it from `tools::worst_case(&limits).is_some()` while renaming the live crate. The
grader builds against the frozen tree, shell-cwd-56c0c59. The frozen `limits.rs` is blob 2865805 (named by
`reference/solution.patch`). Both commits holding that blob export `worst_case` and have no `pub fn valid` anywhere in
`smith-domain-tools` (fabrig/debt checked 839ac73994 and 530f42dfef with `git grep`). So that grade fails for every
candidate, the reference included. Nothing has been built: no committed run shows it, the newest summaries are from 10-17.

Fix: restore `worst_case` on that line. Prove it with `cargo run -p smith-bench -- validate repository/shell-cwd` on a
machine holding the bundle. hel8, hetzner6 and hetzner8 hold none (checked 2026-11-05), and 56c0c59 isn't in this
repository, so its history was rewritten. `smith/benchmarks/README.md` says how to rebuild a bundle: try it from the
matching rewritten commit or an old repo under `~/src/fabrig/coord/old-repos*`. A rebuilt tree must match the manifest's
digest. If it can't be done on a builder, say "not validated" plainly in the report and record it in
`smith/docs/design/benchmarks.status.md`. The same validate also proves `fail-stop-names`' `grader-url` command.

## The D004 record

`jig/docs/development/debt/D004-ops-root-jig-wiring.md`'s first What line says rebuilding the test root is "deferred by the
user". That's wrong: the user hasn't decided, and it's fabrig/debt's recommendation. Change it to say so.

## The items

- `skein/docs/development/debt/D009-oauth-credentials-raw.md`
- `skein/docs/development/debt/D020-forge-connection-states.md`
- `jig/docs/development/debt/D017-unreachable-re-narrowing.md`
- `hearth/docs/development/debt/D009-worlds-as-kits.md`

Items may have changed since they were written: re-measure first. An item already paid gets deleted, saying what paid
it. skein D009 moves credentials to `Secret`, so it must keep them out of `Debug`, with a canary test. fabrig/hearth is
migrating hearth's memory model: stay out of `hearth-service-builder/src/lib.rs` and the machines' `MAX_OUT` forms.
