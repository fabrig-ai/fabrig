+++
id = "change-by-rule"
part = "change"
kind = "feature"
needs = ["change-requires"]
files = ["temper/crates/temper-engine-domain-forge-change/src/boundary.rs", "temper/crates/temper-engine-domain-forge-change/src/domain.rs", "temper/crates/temper-engine-domain-forge-change/src/tests.rs", "temper/crates/temper-engine-domain-forge-change/src/tests/**", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs", "temper/tests/engine/domain/tests/suite/forge_routes/light_rules.rs", "temper/tests/engine/domain/src/forge_routes.rs", "temper/docs/design/domain/forge.status.md"]
effort = "high"
minutes = 90
+++

# change-by-rule: a change checked and updated only where its landing branch's rules ask

## Goal

The change procedure waits for CI at its head and updates it from its base only where the
rules covering its landing branch ask CI passed and up to date (`Domain::requires`, from task
`change-requires`). Under rules asking neither, a change merges at its head once the forge
finds it mergeable, with no CI wait, no repair for CI, no update first or before a run; and
one the forge finds in conflict, or whose merge it refuses for a conflict, is held as one past
its resolutions (`Hold::Resolutions`, which the root holds as `Procedure`), making none. A
branch no rule covers keeps today's waits, so every existing story is unchanged.

Acceptance:

- `temper_engine_domain_forge_change::Facts` gains `pub updates: bool`, doc "Whether its
  landing branch's rules ask up to date (domain/forge.md, 8.7)"; `first` sends a conflict
  (`mergeable` failed) to `resolving_due` where `updates`, and to `hold(change,
  Hold::Resolutions)` otherwise; nothing else in the step changes.
- The forge's `change_facts` (`temper-engine-domain-forge/src/domain.rs`) reads
  `d.requires(repository, &row.base)` for the change's repository and landing branch at each
  step, never storing it, and fills: `ci` and `base_ci` as `Passed` where the rules ask no CI
  (as it does today for a repository without CI, `has_ci`); `contains_base` as `Passed` where
  they ask no up to date; `updates` from `up_to_date`.
- The two stories below pass; every story of `forge_routes` and the forge-change crate's
  tests pass unchanged.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.2 steps 3 and 6 and the paragraph "Lazy, too, for
  runs"; 8.7 (the bullets "Enforced, and waited for" and "What the procedure asks of the
  rules"); section 10 (the bullet "By its landing branch's rule").
- `temper/docs/design/domain/forge.status.md`: its **Not yet** entry "8.2 steps 3 and 6, 8.7,
  10", which this task removes.
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C5.

## Owns and may change

Owns the files in the header.

- forge-change `boundary.rs`: the field; `domain.rs`: `first`'s conflict arm. `landing`'s
  conflict goes back to `First` as today, which then holds where the rules ask no update.
  The tests' `facts()` literals (`src/tests.rs`, `src/tests/turn.rs`) gain `updates: true`.
- forge `domain.rs`: `change_facts` alone (keep the hunk to it: the file is changed on plan
  reference-m8's trunk too). Where the repository's own check gates stand for CI (a
  repository without CI, 8.3), leave them as they are.
- Stories: `temper/tests/engine/domain/tests/suite/forge_routes/light_rules.rs`, new, with its
  `#[path = "forge_routes/light_rules.rs"] mod light_rules;` in `suite/forge_routes.rs`.
- Fixture: `temper/tests/engine/domain/src/forge_routes.rs`, one builder at the end of
  `impl World`'s public methods, beside `with_forge_client` (see `## Answers`); the file is
  changed on plan reference-m8's trunk too, so no other hunk.
- Options: if a repair's or a resolution's lazy update reads `contains_base` elsewhere than
  through `Facts`, make it follow `updates` too and say so. If `Hold::Resolutions` at zero
  resolutions trips an assertion (a count, a panel figure), keep the hold and fix the
  assertion's reading of it, naming it in the report.

## Steps

1. The field, the arm and the facts, as above.
2. Tests in the forge-change crate (`src/tests.rs`, beside the `first` ones):
   - `a_conflict_under_rules_asking_no_update_holds_the_change`: at `First`, `mergeable`
     failed, `updates: false`: `Held { why: Hold::Resolutions }`, no effect, no delegate.
   - `a_conflict_under_rules_asking_up_to_date_is_resolved`: the same with `updates: true`
     gives today's resolving.
3. Stories in `light_rules.rs`, the world's deployment's landing rules set as
   `tests/suite/settings/b2.rs` sets them (`config.landing.deployment`), one rule on `forge`,
   open, `ci: false`, `up_to_date: false`, no gates, `enforce: true`:
   - `a_change_under_rules_asking_neither_merges_at_its_head`: a chat's `change`
     (`change_tool::change`); the producer pushes; the fake forge's CI at the head stays
     pending, and main moves on so the head lacks its tip; the change merges at exactly its
     head with no update effect and no repair delegate.
   - `a_change_under_rules_asking_neither_is_held_when_its_merge_conflicts`: as above, but
     the fake forge reports the pull request not mergeable once settled: the change is held
     (`Phase::Held`, procedure), with no resolution delegate and no update effect.
4. Narrow: `heavy cargo nextest run -p temper-engine-domain-forge-change --lib -E
   'test(/a_conflict_under_rules_asking/)'` (2), then `heavy cargo nextest run -p
   temper-engine-domain-world --test suite -E 'test(/light_rules::/)'` (2). Then the change
   of meaning's readers: `heavy cargo nextest run -p temper-engine-domain-forge-change`,
   `heavy cargo nextest run -p temper-engine-domain-forge`, `heavy cargo nextest run -p
   temper-engine-domain --lib`, `heavy cargo nextest run -p temper-engine-domain-world --test
   suite -E 'test(/forge_routes::/)'`.
5. Remove the **Not yet** entry. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-engine-domain-forge-change -p temper-engine-domain-forge -p
   temper-engine-domain-world --all-targets -- -D warnings`. Commit ("Check and update a
   change only where its landing branch's rules ask") and push.

## Report

As the implementer's text says. The words of the conflict's hold are task `change-words`'s:
say what the hold reads today.

## Answers

2026-11-02. May the task add a hook to `temper/tests/engine/domain/src/forge_routes.rs` for
the world's landing rules, or should the stories drive `Driver` instead of
`change_tool::change`? Add the hook; the stories stay as Step 3 writes them. The file joins
the header. The hook is one public builder, on `with_forge_client`'s pattern and placed
right after it: `#[must_use] pub fn with_landing_deployment(mut self, rules:
Box<[engine::LandingRule]>) -> Self`, doc "Replace the deployment's landing rules before
the world's first input.", asserting as `with_forge_client` does that the world has not
started (`env.now == Time::ZERO`, `!root.ready()`), setting `restart_config.landing.deployment`
and rebuilding `root` from a copy of `restart_config`, so a restart keeps the rules. No
other hunk in the file: no change to `configured_settings`, `StoryLimits` or any
constructor. The stories build their world as `change_tool`'s stories do, then
`.with_landing_deployment(...)` with Step 3's rule, its `connector` the one the world's other
rules use (0 outside the stock worlds). If the bootstrap store's rows carry the landing rules
so that rebuilding the root alone does not reach the forge, stop and say so (`Question:`)
rather than widening the hook.

## Rebase (2026-11-02, step merge, foreman reference-m1/change-by-rule/foreman-2)

resolve forge.status.md by preserving both branches’ removal of completed gaps.
The merge conflicts in `temper/docs/design/domain/forge.status.md`.
Target-side commit `4d7d1123b9e4fe17d212679200ab05d9be99dc18` (“Build a plan's changes as one batch”) removes the **Not yet** 8.1 entry about landing branches, dependencies and several changes.
This task removes the adjacent **Not yet** entry “8.2 steps 3 and 6, 8.7, 10” about checking and updating by landing rules.
Resolve by deleting both entries, retaining all surrounding status text.
`temper/tests/engine/domain/src/forge_routes.rs` merged automatically; preserve both the target’s `job: None` in `change_world_funded` and this task’s `with_landing_deployment` builder.
No behavior change is needed for this refusal.
