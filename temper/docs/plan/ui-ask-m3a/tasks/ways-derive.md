+++
id = "ways-derive"
part = "paths"
kind = "feature"
needs = ["ways-surface"]
files = ["temper/crates/temper-engine-domain-ways/src/path.rs", "temper/crates/temper-engine-domain-ways/src/path/**", "temper/crates/temper-engine-domain-ways/src/tests/**", "temper/crates/temper-engine-domain-ways/src/tests.rs"]
effort = "high"
minutes = 180
+++

## Goal

Write `derive`, the one derivation of a branch's path
(`temper/docs/design/domain/forge.md`, 8.13), over the inputs the root
gathers, with its focused table of the look request's three paths and its
fuzzy test, so that Ask temper's page, `see ways` and the views draw the
same stops.

Acceptance:

- `temper_engine_domain_ways::derive` follows 8.13's order step for step,
  its words, opens and cards those of `notes/contracts.md`, section 2.
  Pure and total: any inputs give a path of at most `STOPS` stops ending
  at a `Bar` (past `STOPS`, the stops before the last bar are cut so the
  landing branch's bar ends it), each card within `FACTS`, `WHO`, `NAME`,
  `LABEL` and `VALUE`, each stop's words within `WORDS`.
- A focused table test, a row a case, each a `derive` over built inputs
  and the expected list of `(actor, words)` in order:
  - over a project with one repository with CI, a tier following main and
    a tier following goal branches, landing by queue, the three stock ways'
    rules laid (build them in the test as 13.10's table has them; `laid`
    and `items` are the surface's): Agents review gives "you ask, produce,
    check, review, repair, check, update, resolve, merge, verify, main";
    You approve "you ask, produce, check, repair, check, you approve,
    update, resolve, merge, verify, main" (the reader holding role 0);
    Draft first, kind 7's way a template with its own goal branch run by
    charter 5, "draft, plan, produce ×n, merge, check, repair, goal
    branch, check, update, resolve, merge, verify, main". These are the
    look request's three pages (`notes/common.md`, *The look request*)
    but for "verify" on the first two (`plan.md`, *What was decided*);
    the test's comment says so;
  - the same with no CI and no tier, a repair drawn only where a check
    or a gate precedes it: Agents review "you ask, produce, review,
    repair, update, resolve, merge, main"; You approve "you ask, produce,
    you approve, update, resolve, merge, main"; Draft first "draft, plan,
    produce ×n, merge, goal branch, update, resolve, merge, main";
  - by turn, a repository with CI (none runs on a pushed head) and no
    tier: Agents review with a test gate named `unit` "you ask, produce,
    review, repair, prepare, resolve, prove, turn, main", and with no test
    gate "you ask, produce, review, repair, prepare, resolve, turn, main";
    You approve with `unit` "you ask, produce, you approve, prepare,
    resolve, prove, repair, turn, main";
  - an approval of role 1 the reader doesn't hold, asking 2 people: "a
    maintainer approves ×2"; an advisory gate's card; a rule that doesn't
    enforce adds its gate and nothing else; a deployment rule (scope 0)
    opens `Rule { kind: 1, .. }`; a project with no repository draws the
    `branch` rules alone over `main`;
  - cards: the review stop's card of the stock (who, name, Model, Can,
    Asked to), the merge's, the approval's, main's bar.
- A fuzzy test (the crate's `tests/` or the forge world's fuzzy suite, as
  the crate's siblings keep theirs; at most 64 seeds): random rules (kind
  1 and 2, `forge` and `branch` patterns, CI, up to date, gates,
  approvals, enforced or not), random CI, tiers, `by` and start; every
  path ends at a bar, holds at most one "repair", has no stop of `Update`,
  `Merge`, `Prepare`, `Prove` or `Turn` whose actor is not `Code`, and is
  the same on a second call.
- Every test you add bites (`What each implementer is told`).

## Inputs

- `temper/docs/plan/ui-ask-m3a/notes/common.md`; `notes/contracts.md`,
  sections 1 and 2 (binding).
- `temper/docs/design/domain/forge.md`, 8.13 (the order), 8.3 and 8.9
  (gates; the rules' gates enforced or not), 8.7 (what a rule asks), 11.1
  (a goal's branch); `temper/docs/design/domain/landing.md`, sections 3
  and 8 (the turn); `temper/docs/design/domain/after-landing.md`, section
  2 (what a tier follows); `temper/docs/design/protocol/configuration.md`,
  13.3 and 13.10.
- How the forge judges a rule's coverage: `covers_branch`
  (`temper-engine-domain-forge/src/gates.rs`), `held_gates` and
  `judge_held_landing` (`standing.rs`); the panel's CI rule,
  `repository.ci && requires(...).ci` (`panel.rs`).

## Owns and may change

- **Owns:** `derive` and whatever private modules it takes under
  `temper-engine-domain-ways/src/path/`; the crate's tests.
- **Neighbour edits:** `src/lib.rs`'s `mod` lines; `src/path.rs`'s
  `derive` body and its private helpers. The public shapes are the
  contract's: changing one is a question.
- **Options, pre-approved:** if `covers_branch` needs a repository the
  inputs lack (none adopted), judge `branch` patterns past four
  placeholder segments as the forge does and skip `forge` patterns, as
  8.13 says. If the wire lowered `Path`'s stops below 32 (task
  `ways-wire`'s report on the trunk), take its figure for `STOPS`.

## Steps

1. Read 8.13 and contracts, section 2 whole; read the forge's coverage
   and gate selection to agree with them.
2. Write `derive` segment by segment: who starts, CI's check, gates, one
   repair, approvals, the queue's or the turn's steps, tiers, the bar.
3. The focused table and the fuzzy test; break the rule once (give the
   update an agent's actor) to see the fuzzy test fail, and restore.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-ways
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-engine-domain-ways`, focused and `--profile fuzzy`. Commit and
   push.

## Report

As the implementer's text asks; give the table's rows as the paths'
words, one line each.
