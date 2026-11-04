# Request: one lint regime across the trees

From fabrig/debt, 2026-11-04 (the user's continuous tech-debt process; source of truth: each tree's
`docs/development/debt/`). The user approved one plan for this on 2026-11-03: `jig` D019, `skein` D036,
`temper` D011 and `hearth` D010 are one job across the trees.

Every part, on its own branch:

- reads the item file whole, and the design text it cites;
- makes the change so the item's **Paid when** holds; if the item has a **Measure**, the command
  reads 0 (or the item's target) after;
- **deletes the item's file and its row** in that tree's `docs/development/debt/README.md`, in the
  same branch;
- updates any status entry the item names as closed by it, and any citation the deletion breaks
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
to 25%, stating the measured size and why. Beyond that, ask. Budgets and timeouts stay the user's.

**All parts must be in the plan when the cut ends.**

## The job

Items: `jig/docs/development/debt/D019-lint-config-copied.md`,
`skein/docs/development/debt/D036-lint-blocks-by-hand.md`,
`temper/docs/development/debt/D011-manual-map-expectations.md`,
`hearth/docs/development/debt/D010-lint-config-copied.md`.

Cargo can't give this directly: a member shares only the root's `[workspace.lints]`, all or nothing, and
has no named tables. debt-1 found moving test crates to `workspace = true` would trip about ten thousand
sites. So:

1. **A few named profiles** (step, harness, binary, as skein's D036 proposes), one written definition
   each, and a gate check in `tools/` that holds every package in the four trees to one of them. A
   manifest keeps its block, but the check makes it identical to its profile's. The check runs in the
   queue's precheck alongside `tools/citations/citations.py`. It should stay small: no new dependencies.
2. **Move every manifest onto a profile.** No lint gets stricter or looser by accident: list in the
   report each lint whose level changes for any crate. A crate's real exception is a scoped
   `#[expect(..., reason = ...)]`. No manifest allows a `disallowed_*` lint.
3. **`wildcard_enum_match_arm` in test crates:** skein's first review (decision D2) and D036 ask to allow
   it in the harness profile, as `testing-strategy.md` says tests are ordinary Rust. Do that.
4. **`clippy.toml` files** come in as few variants as profiles, each with a true header.
5. **temper D011:** one crate-root `#![expect(clippy::manual_map, reason = ...)]` per step crate, as jig
   did, with one wording, replacing the 126 per-item ones.

All four items' **Paid when** hold, and all four items go. Their README index rows go too.
