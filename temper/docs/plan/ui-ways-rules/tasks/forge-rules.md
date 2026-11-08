+++
id = "forge-rules"
part = "rules"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain-forge/src/settings.rs", "temper/crates/temper-engine-domain-forge/src/gates.rs", "temper/crates/temper-engine-domain-forge/src/kinds.rs", "temper/crates/temper-engine-domain-forge/src/standing.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/tests/kinds.rs", "temper/crates/temper-engine-domain-forge/src/tests/settings.rs", "temper/crates/temper-engine-domain-forge/src/tests/standing.rs", "temper/crates/temper-engine-domain-forge/src/tests/requires.rs", "temper/crates/temper-engine-domain/src/route/kinds.rs", "temper/crates/temper-engine-domain/src/route/host_route.rs", "temper/crates/temper-engine-domain/src/tests/kinds.rs", "temper/tests/engine/forge/tests/suite/main.rs", "temper/tests/engine/forge/tests/suite/fuzzy/branch_patterns.rs", "temper/tests/engine/domain/tests/suite/agent_reads.rs", "temper/docs/design/protocol/configuration.status.md", "temper/docs/design/domain/forge.status.md"]
effort = "high"
minutes = 130
+++

# The forge holds every project's given rule and a project's rule naming a branch

## Goal

The mechanism of `temper/docs/design/draft/stock-ways.md`, 3.1 and 3.2,
in the forge and the root, built to `notes/contracts.md`, sections 1 and
2: a rule of kind 2 may begin at `branch` and then covers that branch in
every repository of its project, those adopted later among them; and a
rule given to the forge as kind 2 of scope 0 is every project's rule of
its number, held while the project sets none of its own, a removal or a
reset going back to it. The binary still gives kind 1's rule 1: the swap
is a later task's (`project-stock`).

Acceptance:

- `settings::judge` admits kind 2's `["branch"]` patterns, exact and open,
  and refuses `Refusal::Pattern` for the same patterns as kind 1, as rows
  of `judge_accepts_rules_within_their_bounds` and
  `judge_refuses_each_fault_by_name` (`src/tests/settings.rs`); the
  existing kind 1 row whose first segment is `branch` stays refused.
- `covers_branch` exists as the contract has it, and every held-rule
  reader in `standing.rs` and `settings::requires` matches through it.
- **The given rule per project and its reset** (one table test in
  `src/tests/kinds.rs`, beside `a_change_of_rules_is_judged_kept_and_held`):
  given `(2, 0, 1)` (the contracts note's section 4 value) beside the
  file's other stock, with projects 1 and 2: `landing_rules(1)` and
  `landing_rules(2)` each hold it as their rule 1 with their own scope;
  `give_settings_defaults` answers no state for it; project 1's rule 1 set
  to a rule asking CI alone is project 1's and not project 2's; reset, and
  in another row removed, project 1's rule 1 is the given one again, and
  `setting(2, 1, 1)` answers it; a project with only a rule 2 of its own
  holds the given rule 1 beside it; a deleted project's settings erased
  leave no held entry. The change from given to own and back wakes the
  project's live changes as `a_change_of_rules_wakes_the_live_changes_it_covers`
  checks it.
- **A branch pattern over two repositories, one adopted after the rule**
  (`src/tests/standing.rs`, beside `project_rules_add_to_the_deployments`):
  project 1 with repository A sets its rule 2, `["branch"]` exact `main`,
  enforced, with gate 2 blocking; then repository B is adopted into
  project 1; `held_gates` and `held_requires` of B's `main` hold gate 2,
  of B's `dev` do not, and of `main` of a repository of project 2 do not.
- **Fuzzy** (`temper/tests/engine/forge/tests/suite/fuzzy/branch_patterns.rs`,
  under the suite's `mod fuzzy`, so its name starts `fuzzy::`): for 64
  seeds, random repositories and random branch names (segments of 1 to 3
  parts, bytes drawn to include `/`, short and long parts, and a name
  with an empty part), random `last` values exact and open (an empty open
  among them): a `["branch"]` pattern covers a branch of repository R
  exactly when the full pattern `["forge", R's host, owner, name, "branch"]`
  with the same `last` covers it; the full pattern covers no branch of
  another repository; the `["branch"]` pattern covers a branch of every
  repository alike; a name with an empty part is covered by neither.
- The root: `covers_repository` is true for a rule beginning at `branch`,
  as a row of the root's or the domain world's existing approval test
  (below); a run's `settings landing-rule 1` read of a project answers
  the given rule where jig knows no setting of that number, as a row in
  `temper/tests/engine/domain/tests/suite/agent_reads.rs`, whose world
  gives its own forge stock (add `(2, 0, 1)` to it).
- The two **Not yet** entries you close are deleted: in
  `temper/docs/design/protocol/configuration.status.md` the bullets
  starting `13.2, 13.3: a pattern of kind 2` and `13.3, 13.4: the forge
  holds no rule given`, and in `temper/docs/design/domain/forge.status.md`
  the bullet starting `8.9: a project's rule 1 given`.
- Clippy clean, fmt clean, the focused tests and the fuzzy test below
  green.

## Inputs

- `temper/docs/plan/ui-ways-rules/notes/common.md` and `notes/contracts.md`,
  sections 1 and 2 (binding).
- `temper/docs/design/protocol/configuration.md`, 13.2 (the bullets *A
  project's rule may name a branch* and *Judged by the forge*), 13.3 (the
  first, third and fourth bullets) and 13.4 (the bullet *A project's rule
  1 is given to the forge*).
- `temper/docs/design/domain/forge.md`, 8.9 (*The root checks approval
  eligibility*, *It holds them*).
- `temper/docs/design/draft/stock-ways.md`, 3.1 and 3.2, for why.

Departures from the implementer's text, and why:

- **You delete three status entries,** named above, once your acceptance
  holds. Touch no other design text. The cut wrote the design and these
  entries for you to close.

## Owns and may change

Owns:

- `F/settings.rs` (`F` is `temper/crates/temper-engine-domain-forge/src`):
  `judge`'s first-segment check, `requires`' matching.
- `F/gates.rs`: `covers_branch`, and `covers` as its helper.
- `F/kinds.rs`: `setting`, `setting_default`, `give_settings_defaults`,
  `rebuild_settings`, `landing_rules`, `keep_settings`' wake, and the
  holding of every-project rules beside `HeldRules` (a field of
  `HeldRules` or of `Domain`, your choice).
- `F/standing.rs`: its four readers of `landing_rules`.
- `F/lib.rs`: the `covers_branch` re-export.
- `F/tests/kinds.rs`, `F/tests/settings.rs`, `F/tests/standing.rs`,
  `F/tests/requires.rs`: the rows above, and the call sites of
  `landing_rules`' new signature.
- `temper/crates/temper-engine-domain/src/route/kinds.rs`:
  `covers_repository`.
- `temper/crates/temper-engine-domain/src/route/host_route.rs`: `settings`'
  stock answer, widened to kind 2.
- `temper/crates/temper-engine-domain/src/tests/kinds.rs`: rows.
- `temper/tests/engine/forge/tests/suite/fuzzy/branch_patterns.rs` (new)
  and its `mod` line in `temper/tests/engine/forge/tests/suite/main.rs`.
- `temper/tests/engine/domain/tests/suite/agent_reads.rs`: the world's own
  forge stock gains `(2, 0, 1)` and one row.
- The status entries above.

Reader and neighbour files (edits allowed as stated):

- Any other caller of `landing_rules` the compiler names: the call site
  only. The cut's survey found only `standing.rs`.
- `temper/tests/engine/domain/tests/suite/forge_routes/approval_identity.rs`:
  one row in `approval_rules_count_known_forge_people_with_the_required_role`
  for a project's rule beginning at `branch`, if the root has no unit
  test of `covers_repository` to take the row.
- Tests that pin `HeldRules`' fields, where you add one: the literal only.

Pre-approved options:

- **Where every-project rules are kept:** a `Box<[Held]>` beside
  `HeldRules`' `deployment`, filled by `rebuild_settings` from the given
  stock, is the default. If the forge's restore or its rows need them as
  stored `Default` rows to survive a restart before the stock is given
  again, store them as the forge stores other defaults today and say so.
- **`settings::requires`:** if a signature change (a repository and a
  branch in place of a resource name) reads better than matching a name,
  make it, with its callers, and say so.
- **The root's run read:** if `host_route.rs` reaches the forge's
  `setting` through a helper `route/tools.rs` already has, use it; if the
  only clean way is a line in `route/tools.rs`, make it and name it.
- **Adopting B after the rule:** through the forge's adoption, as
  `src/tests/adoption.rs` adopts; if the standing tests' fixture can't
  reach it cheaply, insert B's repository row as `standing.rs`' fixture
  inserts A's, after the rule is kept, and say so.

## Steps

1. Read `F/settings.rs`, `F/gates.rs`, `F/kinds.rs` (`judge_settings` to
   `rebuild_settings`), `F/standing.rs`, `F/domain.rs`'s
   `branch_resource_name`, and `R/route/kinds.rs`' `covers_repository`
   and `approval_for_projects`, `R/route/host_route.rs`' `settings`.
2. Sketch, to the contract:

   ```rust
   // F/gates.rs
   pub(crate) fn covers(pattern: &GatePattern, name: &[Box<[u8]>]) -> bool {
       let name = match pattern.segments.first() {
           Some(first) if first.as_ref() == b"branch" => match name.get(4..) {
               Some(tail) => tail,
               None => return false,
           },
           Some(_) | None => name,
       };
       /* today's prefix and last rule over `name` */
   }
   #[must_use]
   pub fn covers_branch(pattern: &GatePattern, repository: &Repository, branch: &[u8]) -> bool {
       crate::domain::branch_resource_name(repository, branch).is_some_and(|name| covers(pattern, &name))
   }
   // F/settings.rs, judge
   let first_ok = first.as_ref() == b"forge" || (kind == Kind::ProjectRules && first.as_ref() == b"branch");
   ```

3. Write the judge rows and the given-rule table; run them red on the
   base where they can be (`heavy cargo nextest run -p
   temper-engine-domain-forge --lib -E 'test(/kinds::/)'`; check the
   count), then build `setting`, `landing_rules` and the holding to green.
4. Write the two-repository row, the root's rows and the run-read row;
   then the fuzzy test. A test you add to pin a rule must bite: break the
   rule once by hand (drop the `branch` tail, hold the given rule for a
   project with a rule 1 of its own, answer none on a reset), see the row
   fail for that reason, restore; the report names each.
5. Narrow runs: `heavy cargo nextest run -p temper-engine-domain-forge
   --lib`; `-p temper-engine-domain --lib -E 'test(/kinds::/)'`;
   `heavy cargo nextest run -p temper-engine-forge-world --profile fuzzy
   -E 'test(/^fuzzy::branch_patterns::/)'`; `-p temper-engine-domain-world
   --test suite -E 'test(/agent_reads::/) | test(/approval_identity::/)'`.
6. **A change of meaning runs its readers' tests:** `setting` answers
   more, and `landing_rules` changes shape. Run, focused, the packages
   `temper-engine-domain-forge`, `temper-engine-domain`,
   `temper-engine-forge-world`, `temper-engine-domain-world` and
   `temper-client-world` (`-E 'test(/settings_tools::/)'` for the last),
   and `heavy cargo check -p temper-service -p temper --tests`.
7. Once, at the end: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`
   (lower with `--write` only for what you removed); `heavy cargo clippy -p
   temper-engine-domain-forge -p temper-engine-domain -p
   temper-engine-forge-world -p temper-engine-domain-world --all-targets
   -- -D warnings`; `python3 -I -B tools/citations/citations.py --base
   origin/main`. Commit (`ui-ways-rules: a project's rule names a branch,
   and every project holds its given rule`, the body saying why: a way
   is data naming no repository, and a project's way must be able to
   loosen the stock) and push.

## Report

As the implementer's text says: the first line, the commit pushed, each
run with its count, each rule broken by hand and what failed, the choices
you made inside the sketch and the options above, and the minutes.
