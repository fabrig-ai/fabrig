+++
id = "panel-steps"
part = "change-panel"
kind = "feature"
needs = ["tiers-verification"]
files = ["temper/crates/temper-engine-domain-forge/src/panel.rs", "temper/crates/temper-engine-domain-forge/src/tests/panel.rs", "temper/crates/temper-client-protocol-wire/**", "temper/crates/temper-service/src/family.rs", "temper/crates/temper-service/src/tests/family.rs", "temper/crates/temper-client-domain/**", "temper/crates/temper-client-view/**", "temper/crates/temper-client-tools/src/change.rs", "temper/crates/temper-client-tools/src/tests/**", "temper/tests/client/**", "temper/docs/help/reference/ctl-change.md", "temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/verification.rs", "temper/crates/temper-engine-domain-forge/src/tests/verification.rs", "temper/crates/temper-protocol-store/schema/store-v11.schema", "temper/crates/temper-protocol-store/src/generated/v11.rs", "temper/crates/temper-protocol-store/golden/v11/**", "temper/crates/temper-protocol-store/src/records/verification.rs", "temper/crates/temper-protocol-store/src/tests/tiers.rs"]
effort = "high"
minutes = 240
+++

# The change panel after landing: verifying, red, cleared; `--until cleared`

## Goal

A change's panel, as `change show` and the change's page read it, goes on past `landed`:
`verifying` with the tiers that have not judged it, `red` with the tiers that blame it, and
`cleared` (or landed and uncleared); `change show TASK --until cleared` waits for it. The walk
reads a verdict this way.

Acceptance:

- `temper/docs/design/client/forge.md`, 2.1's rows for `verifying`, `red` and `cleared`, and
  `temper/docs/design/client/tools.md`, 5.4 (`landed` met by every step from `landed` on;
  `cleared` met at `cleared` or landed and uncleared) and 5.5 (`change show`'s following call:
  `--until landed` before it lands, `--until cleared` while verifying or red).
- The forge's panel (`panel.rs`) gives the steps from the change's state (`Landed` with a
  `Verification` row is `verifying`; `Red`; `Cleared` and `Uncleared`) and the tiers' rows: for
  each tier following its branch, its name and its verdict or its run in flight; for a red one,
  the run and the failing tests blamed, counted, or many. A `Landed` change with no row (no tier
  follows its branch) stays `landed`.
- The client wire (`schema/client-wire-v1.schema`) appends `verifying`, `red` and `cleared` to
  `Step` after its last variant, each with a record carrying the merge commit and the tiers'
  lines; generated code and goldens regenerated in place; the service's translation
  (`temper-service/src/family.rs`) fills them.
- `temper-client-tools`' `change show`: the `--until` set gains `cleared`; `landed` is met from
  `landed` on; the text prints the step's line as client/forge.md 2.1 words it (`step:
  verifying · TIER run N running`, `step: red · TIER run N · 2 tests`, `step: cleared · TIER run
  N`, `step: landed, uncleared`); the next call as tools.md 5.5 says; the JSON carries the same.
  Its notes text names `--until cleared`. `temper/docs/help/reference/ctl-change.md` is
  regenerated from the table by its test's write mode (`SKEIN_CODEGEN_WRITE=1`, test
  `every_ctl_group_page_is_what_the_table_renders` in package `temper`): this task's one
  `docs/help` file, a generated one.
- The client domain and view show the new steps where they show `landed` today
  (`temper-client-domain/src/forge/change.rs`, `temper-client-view`'s change page and delegation
  card), in the panel's words.
- Packages pass: `heavy cargo nextest run -p temper-engine-domain-forge --lib -E
  'test(/panel|views/)'`; `-p temper-client-protocol-wire`; `-p temper-service --lib -E
  'test(/family/)'`; `-p temper-client-tools --lib`; `-p temper-client-domain`;
  `-p temper-client-view`; `-p temper-client-world --test suite -E 'test(/change/)'`; `-p temper --lib -E
  'test(=tests::commands::every_ctl_group_page_is_what_the_table_renders)'`.

## Inputs

- `temper/docs/plan/reference-m8/notes/common.md`; `notes/contracts.md`, 2 and 4.
- `temper/docs/design/client/forge.md`, 2.1 and 2.5; `temper/docs/design/client/tools.md`, 5.4
  and the `change show` row of 5.5.
- Readers of today's `landed` step, by a survey: `temper-engine-domain-forge/src/panel.rs` (about
  34, 95, 137) and its tests `src/tests/{panel,views}.rs`; `temper-service/src/family.rs` and
  `src/tests/family.rs`; `temper-client-protocol-wire/src/tests/fixtures.rs`;
  `temper-client-tools/src/change.rs` (the flag about 35, notes about 40, conditions about 745
  and 754, the next call about 1148) and `src/tests/{change,landing}.rs`;
  `temper-client-view/src/tests/change.rs`, `tests/suite/change_page.rs`;
  `temper-client-domain/src/forge/change.rs`, `src/tests/{change,panel}.rs`,
  `tests/suite/change.rs`; `temper/tests/engine/domain/src/oracle/durable.rs`;
  `temper/tests/client/tests/suite/{change_panel,change_page,change_panel_edges}.rs`.

## Owns and may change

- Owns the files in the header for this wave, and the readers listed above for the new arms and
  goldens.
- Not this task's: the forge's `tiers.rs` and `verification.rs` (read their rows through the
  forge's own accessors; add a read-only accessor in `panel.rs` if one lacks), the concept help
  pages (`help-pages`).

Pre-approved options:

- If the client wire's `Step` cannot take a record per new variant within its bounds, carry the
  tiers' lines in one record shared by the three, and say so.
- If the web view has no place for the tiers' lines, show the step's name and its first tier's
  line, and list the rest as the report's `## Left`.

## Steps

1. The forge's panel steps and their unit tests.
2. The wire, the service's translation, regeneration.
3. `ctl`'s conditions, text, JSON and the regenerated reference page; then the client domain and
   view.
4. `cargo fmt`; clippy over every package you changed; the tests in Goal, counts reported.
5. Commit in steps that say why; push your branch.

## Report

As the implementer's text says, and also: the panel's lines as printed for each step; the
regenerated files.

## Answers

### 2026-11-02: Should tiers-verification repair the forge's compilation failures before panel-steps resumes, or should panel-steps repair `verification.rs`, `tiers.rs` and `tests/verification.rs`? (reference-m8/panel-steps-1, foreman reference-m8/panel-steps/foreman-1)

tiers-verification repairs them; panel-steps' boundary stands. The three errors (E0428 duplicate
`judged`, E0061 `tiers.rs`'s four-argument call, E0063 `TierRow` missing fields) come from
`origin/reference-m8/tiers-verification` as its first run pushed it, before `tiers-runs` landed
its durable tier procedures (`5065368876`, `f6c6737516`). The trunk has one `judged`, and the
merge brings in the stale branch. tiers-verification's second run, now running, owns
`verification.rs`, its tests and the call into `verification::judged` in `tiers.rs`, and is to
bring them onto today's trunk.

The next run starts only once tiers-verification has landed, or has pushed a branch whose
`heavy cargo nextest run -p temper-engine-domain-forge --lib` compiles. Before any edit it
`git fetch origin`, merges `origin/reference-m8/tiers-verification` again and runs the forge's
panel test to confirm the base compiles. If the base still fails to compile on errors in those
files, it stops with a Question naming them. It never edits `tiers.rs`, `verification.rs` or
`tests/verification.rs`. It reads verification rows only through the forge's accessors, adding
a read-only one in `panel.rs` if one is missing, as Owns says.

### 2026-11-02: Should tiers-verification retain the run that repaired each red tier, or may the panel name the repair commit instead? (reference-m8/panel-steps-2, foreman reference-m8/panel-steps/foreman-2)

The verification retains the repairing run; the panel names that run, as
`temper/docs/design/client/forge.md`, 2.1 says. The repair commit alone is not enough: the
change's result words need the run too (`notes/contracts.md`, 4: `Cleared by TIER run N, which
passed the failures blamed on it.`). tiers-verification has landed, so this task takes the
change, widened by the files now in its header; `notes/contracts.md`, 4 carries it as revision 1:

- `boundary.rs`: `TierVerification::repaired` becomes `Option<TierRepair>`, with a new public
  `#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct TierRepair { pub run: u64, pub tip:
  Commit }` (the run's number in the tier's sequence and its tip), re-exported where
  `TierVerdict` is.
- `verification.rs`: where `judged` sets `repaired`, it sets `TierRepair { run: verdict.number,
  tip: verdict.tip }`; every other reader keeps its meaning (`repaired.is_some()`). Nothing else
  in the file changes; `tiers.rs` stays untouched (tiers-reports owns it).
- The store, inside v11 (no new version: families 28 and 29 are this plan's own, not on main):
  in `store-v11.schema`, `record TierRepair { run: u64 tip: fixed 32 }` before
  `TierVerification`, whose field becomes `repaired: option TierRepair`; regenerate the code and
  goldens with `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test suite
  -E 'test(drift::)'`, then run it again without the variable; `records/verification.rs`
  translates both ways; `src/tests/tiers.rs`'s round trip carries a repair's run.
- `tests/verification.rs`: `later_green_repairs_red_without_replacing_its_verdict` also asserts
  the repair's run is run 2 after run 3 judges.
- More packages pass: `heavy cargo nextest run -p temper-protocol-store`; `-p temper-codecs-world
  --test suite -E 'test(drift::)'`; `-p temper-engine-domain-forge --lib -E
  'test(/verification/)'`.

The panel's cleared line for a repaired tier names `repaired.run`; for one cleared outright, its
verdict's run. The earlier answer's rule stands for everything else in `verification.rs` and
`tests/verification.rs`, and `tiers.rs` is still never edited.

## Rebase (2026-11-02, step merge, foreman reference-m8/panel-steps/foreman-3)

reconcile the v11 generated codec with trunk’s durable tier fields and this branch’s TierRepair.
The merge conflicts in `temper/crates/temper-protocol-store/src/generated/v11.rs`.
Trunk commit `906a6bb4a9` adds `TierProbeBranch`, `TierRow.probe`, and `TierFlight.report`.
Branch commit `03a1be1d31` adds `TierRepair { run, tip }` and changes `TierVerification.repaired` to `Option<TierRepair>`.
Preserve both sets of changes in the merged `schema/store-v11.schema`; keep version 11.
Resolve the generated codec to contain both changes, then regenerate it and `golden/v11/**` from the combined schema:
`SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'`
Run the same command without `SKEIN_CODEGEN_WRITE` to confirm committed output matches.
Retain the automatically merged tier-test and fuzzy-store changes from both sides.
Validate with `heavy cargo nextest run -p temper-protocol-store`.
No change to tier policy or `tiers.rs` is needed for this conflict.
