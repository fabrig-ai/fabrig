+++
id = "restore-stories"
part = "restore"
kind = "feature"
needs = []
files = ["temper/tests/engine/domain/tests/suite/settings/kinds.rs", "temper/tests/client/tests/suite/settings_tools.rs", "temper/docs/design/protocol/configuration.status.md", "temper/docs/design/client/tools.status.md"]
effort = "medium"
minutes = 60
+++

## Goal

Four restore stories of temper's kinds run again in the focused suite.
They were ignored waiting on `restore-request`'s `Restore` admission by
people and its `ForgetReplaced` on history pruning; both are on main
since (`jig/docs/design/domain/engine.status.md`, 17.10:
`routing/party/people.rs`, `settings/restart.rs`, `forget_replaced`;
temper's `temper-engine-domain/src/route/kinds.rs` answers
`connector::Ask::ForgetReplaced` with `forge.forget_replaced`).

Acceptance:

- These lose their `#[ignore]` and pass in the focused suite:
  - `temper/tests/engine/domain/tests/suite/settings/kinds.rs`
    (package `temper-engine-domain-world`, binary `suite`):
    `a_landing_rule_and_the_forges_charter_changed_then_restored`,
    `a_restore_leaves_out_a_deleted_projects_rule`,
    `forgetting_erases_the_forges_replaced_rows`;
  - `temper/tests/client/tests/suite/settings_tools.rs` (package
    `temper-client-world`, binary `suite`):
    `each_kind_through_ctl_proposed_shown_accepted_and_restored`
    (`temper/docs/design/client/tools.md`, 11.2).
- Each that fails is fixed by its cause (`notes/common.md`), against
  `jig/docs/design/domain/engine.md`, 17.10 for the restore itself,
  `temper/docs/design/protocol/configuration.md`, 13.2 for what temper's
  forge keeps and restores of its kinds, and `tools.md`, 11.2 and 5.3's
  `revision restore REVISION [REASON]` row for the client story.
- The status entries that say the stories are ignored, or that whether
  they pass unignored is unknown, are updated to match (below). This
  departs from the implementer's text, which closes `docs/design/` to
  you: the entries are this task's.
- `heavy cargo nextest run -p temper-engine-domain-world -p
  temper-client-world` passes, and the packages you change in the product
  pass their own focused tests and their direct dependents'.

## Inputs

- `temper/docs/plan/ignored-engine/notes/common.md`: every rule there
  holds.
- `jig/docs/design/domain/engine.md`, 17.10 (Restoring a version) and
  its status entry for 17.10 in `jig/docs/design/domain/engine.status.md`
  (read it; it is not yours to change: jig's eleven restore stories stay
  as they are).
- `temper/docs/design/protocol/configuration.md`, 13.2, and the 13.2
  clause of `configuration.status.md`'s long **Not yet** entry.
- `temper/docs/design/client/tools.md`, 5.3 (the `revision restore`
  row) and 11.2, and the 11.2 clause of
  `tools.status.md`'s **Not yet** entry.
- `skein/docs/foundation/testing-strategy.md`, 8 (an ignored test is a
  finding its owner runs until it's fixed).

## Owns and may change

- Owns: the four tests above (their `#[ignore]` lines first; their
  bodies only where the design has moved, citing the section in a
  comment where the change is not obvious), and the two status entries
  below.
- `temper/docs/design/protocol/configuration.status.md`: the clause of
  the long **Not yet** entry that begins `13.2: the engine's domain
  world's three restore stories of temper's kinds` and ends `(...
  temper-engine-domain-forge/src/tests/kinds.rs`).`: if all three pass,
  remove that clause (both its sentences), its words alone, leaving the
  rest of the entry as it stands. If one stays ignored, the clause names
  it and the missing design section in place of `whether they pass
  unignored is unknown`.
- `temper/docs/design/client/tools.status.md`: in the **Not yet** entry,
  11.2's clause `the joined story of a settings file of each kind
  proposed, shown, accepted and restored (... ), ignored pending the
  restore request path;`: removed if the story passes, its words alone
  (keep the list's punctuation whole); otherwise it names the missing
  section in place of `the restore request path`.
- The worlds' helpers, if a story needs a capability they lack:
  `temper/tests/engine/domain/src/kinds.rs`,
  `temper/tests/engine/domain/src/forge_routes.rs` (an added method
  only), and `temper/tests/client/src/` (an added helper; avoid
  `fixtures.rs`, `component.rs`, `tui.rs` and `front.rs` there, which the
  `tui-box-front` plan is changing; if one of them must change, keep the
  edit to added lines and name it in the report).
- The product, where a story shows it wrong against its design:
  `temper/crates/temper-engine-domain/src/route/kinds.rs`,
  `temper/crates/temper-engine-domain-forge/src/kinds.rs` and its
  `src/tests/kinds.rs`, `temper/crates/temper-client-tools/src/jig/**`
  (rendering of a restore's outcome or refusal). A fault in jig
  (`jig/crates/jig-domain-core/src/settings/**`,
  `jig/crates/jig-domain-core/src/routing/party/**`) is reproduced first
  by a row or test in `jig/crates/jig-domain-core/src/tests/settings/restore.rs`,
  then fixed there.
- Options, each pre-approved:
  - If a story fails only because a figure its fixture sets is now short
    (`limits.people.requests`, `settings.history`, a request count the
    joint's `Seen` reads take), set the fixture's figure to what the
    story needs, with its comment saying why. That is the story's own
    fixture, not a configured figure.
  - If an expected text has moved with the design (a rendered line of
    `revision restore`, `proposal show` or `revision show`), take the
    words the design and the tool's own goldens
    (`temper-client-tools/src/tests/golden/restore_proposed_json.txt` and
    its siblings) now give, and say so.
  - If a story's revision numbers moved because the fixture's import
    takes another release, read the revision from the store as the story
    already does (`kinds::revision`), never a new constant.
  - If the cause is a feature larger than this task, follow the
    still-ignored rule in `notes/common.md`.

## Steps

1. Remove the four `#[ignore]` lines; commit. Run each test alone (count
   1) and record its first failure or its pass.
2. For each failing test, read the failure's log, find its cause in
   the design order above, fix it in the lowest tier that shows it, and
   rerun the test alone.
3. Update the two status clauses to match what holds.
4. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py` (and
   jig's, if you changed jig); `heavy cargo clippy -p
   temper-engine-domain-world -p temper-client-world [-p each product
   package you changed] --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-engine-domain-world -p
   temper-client-world [-p each product package you changed and its
   direct dependents]`.
5. `python3 -I -B tools/citations/citations.py --base origin/main` after
   committing (it reads HEAD), then push.

## Report

The implementer's report, with: for each of the four tests, passed as
it stood, or its first failure, cause and fix; its time from nextest;
the status clauses as changed; any test left ignored and its notes file.
