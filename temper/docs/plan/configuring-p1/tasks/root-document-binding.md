+++
id = "root-document-binding"
part = "documents-temper"
kind = "feature"
needs = ["jig-documents-surface", "forge-document-gather"]
files = ["temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/crates/temper-engine-domain/src/assemble.rs", "temper/crates/temper-engine-domain/src/route/now.rs", "temper/crates/temper-engine-domain/src/limits.rs", "temper/crates/temper-engine-domain/src/link.rs", "temper/crates/temper-engine-domain/src/budget.rs", "temper/crates/temper-engine-domain/src/store/settings_bytes.rs", "temper/crates/temper-engine-domain/src/tests.rs", "temper/crates/temper-engine-domain/src/tests/documents.rs", "temper/crates/temper-engine-domain/src/tests/resume_fixture.rs", "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs", "temper/crates/temper-engine-smith/src/start.rs", "temper/crates/temper-engine-smith/src/tests.rs", "temper/crates/temper-service/src/limits.rs", "temper/crates/temper-service/src/limits/names.rs", "temper/crates/temper-service/src/startup/checks.rs", "temper/crates/temper-service/src/tests/**", "temper/tests/engine/domain/src/walking.rs", "temper/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 150
+++

# temper: the root binds each planned document to the forge, and titles its section

## Goal

When jig's core plans a run's brief with a charter's documents, temper's root gives each to
its forge before the gather, as a section of its own kind, budget and title, and the run reads
`Document OWNER/NAME PATH` sections (temper's `temper/docs/design/protocol/configuration.md`,
13.7; `temper/docs/design/domain/agent.md`, 4.1; jig's `jig/docs/design/domain/engine.md`,
17.9, "planned with the brief"). Small's brief holds the core's own sections and two documents.

Acceptance:

- **Binding.** `route/now.rs` passes `BriefCorePlanned`'s `documents` to `finish_brief_plan`
  (`assemble.rs`), which, before its forge sections, for each `PlannedDocument` (index `i`):
  takes a `brief_connectors` row (its kind `ForgeBriefKind::Document(title words)`), rewrites
  the plan's matching section (same connector and core token) to the row's token, kind
  `DOCUMENT_KINDS + i` and limit `budgets.document`, and, when the name is a forge branch,
  sends `forge::Event::PlanDocument { section, document }` with the name's host, owner,
  repository and branch (its segments joined by `/`), the path and the heading
  (`temper/docs/plan/configuring-p1/notes/contracts.md`, section 3). A name that is no forge
  branch is answered `brief::GatherEvent::Missing` when the brief asks to gather it, the forge
  not asked.
- **Kinds and titles.** `ForgeBriefKind::Document(Box<[u8]>)`; `ForgeBriefKind` and `BriefKind`
  lose `Copy` and their uses take references or clones. `GatherPlaced::Connector` of a document
  gives the row's kind; `GatherPlaced::Missing { kind, .. }` with `kind >= DOCUMENT_KINDS` gives
  `Document(title of document kind - DOCUMENT_KINDS of that brief's task)`: the root keeps each
  planned brief's document titles from the binding to its `Complete` or its abandonment, within
  `brief.briefs × tasks.charter_documents`, and none outlives its brief. `temper-engine-smith`'s
  `section` titles a document `Document ` and its words: `OWNER/NAME PATH`, or
  `OWNER/NAME PATH, "HEADING"`; `Document PATH` for a name that is no forge branch. The title's
  bytes count wherever a section's bytes are held (`budget.rs`'s `section_bytes`).
- **Room.** The root's required forge sections (`plan_home_access`, `make_forge_room`) replace
  optional *core* sections only, never a document; a plan with no such room is not ready, as
  today. (With 8 sections the core's required sections, at most three, two documents and the
  forge's three required sections fit.)
- **Limits.** `BriefBudgets.document`, 4,096 in `small` (`temper-service/src/limits.rs`, with a
  comment saying what it bounds), override `root.brief.budgets.document` (`limits/names.rs`);
  `root.brief.sections` 8 in `small` (`forge.brief_sections` 32 still holds `briefs × sections ×
  2`). Each `BriefBudgets` literal gains `document` (the profile, `resume_fixture.rs`,
  `temper/tests/engine/domain/src/walking.rs`); in-code fixtures keep their sections.
- **The start.** temper's service start refuses limits whose `root.brief.sections` cannot hold
  `jig_domain_core::CORE_SECTIONS` and the charters' documents
  (`jig_domain_core::brief_holds_documents`), naming `root.brief.sections` and the tasks limit's
  override path, beside `startup/checks.rs`'s other limit checks; if jig's
  `check_effective_limits` already refuses it once `jig-documents-config` lands, add only temper's
  path naming and the test.
- **Pricing.** `store/settings_bytes.rs` prices a charter as `size_of::<Charter>()` plus
  `jig_domain_core::charter_owned_bytes`, at both of its sites.
- Tests:
  - `temper-engine-domain`, a new `src/tests/documents.rs` (declared in `src/tests.rs`):
    `finish_brief_plan` with two planned documents sends two `PlanDocument`s with the parsed
    fields and rewrites the plan's two sections (token, kind 16 and 17, limit 4,096);
    `a_missing_document_section_is_titled_with_its_document`; a name that is no forge branch is
    `Missing` at its gather and sends no `PlanDocument`; a repair delegate's Pull and reason
    sections replace optional core sections and keep both documents; no title or row is left
    after `Complete` or an abandonment.
  - `temper-engine-smith`, `src/tests.rs`: the three title forms.
  - `temper-service`, `src/tests/`: Small's `document` budget and 8 sections; the override; a
    start with `root.brief.sections` 7 refused naming both limits, 8 admitted.
  `heavy cargo nextest run -p temper-engine-domain --lib -E 'test(/documents::/)'` passes, its
  count in the report; then the three crates' lib tests.
- **Readers' tests (a change of meaning).** Raising Small's sections lets the core plan its
  optional sections (notes, plan, attempts) into temper's briefs: run
  `heavy cargo nextest run -p temper-engine-domain-world -p temper-system-world -p temper-service`
  and `-p temper-engine-smith`; an assertion that counted a brief's sections or bytes and moved
  only because the core's optional sections now fit is updated and listed; any other failure is
  a `Question:`. Memory figures that move with the sections are regenerated where pinned
  (`SKEIN_CODEGEN_WRITE=1`), each with its old and new value.
- The root's sentence leaves `temper/docs/design/protocol/configuration.status.md`: delete
  "13.7: the root plans no document section, `root.brief.budgets` has no `document`, and
  `root.brief.sections` is 3." and nothing else.

## Inputs

- `temper/docs/plan/configuring-p1/notes/common.md` and `notes/contracts.md`, sections 1 and 3.
- `temper/docs/design/protocol/configuration.md`, 13.7 and section 7 (the small profile's
  brief); `temper/docs/design/domain/agent.md`, 4.1 (the sections' titles);
  `temper/docs/design/domain/root.md`, 7.2 (a connector's brief section);
  jig's `jig/docs/design/domain/engine.md`, 17.9.
- Old code, for reference: the old temper branch's `assemble.rs::bind_documents` rewrote jig's
  plan-local tokens into `brief_connectors` rows and made `plan_home_access` evict only
  `Planned::Core` optional sections; it never gathered (it answered `Missing`), and its
  `forge_brief_budget(Document)` was the task's budget, which `document` replaces.

## Owns and may change

- Owns the files in the header.
- Readers of `ForgeBriefKind`/`BriefKind` and `BriefSection` the loss of `Copy` forces, in any
  temper file: the reference or clone alone, listed. Fixture JSON documents that set
  `root.brief.sections` (`cluster/engine.json`, `github/engine.json`, the real loop's
  `landing.rs`): only if one no longer starts, raised to 8, listed.
- Read only: the forge crate (`forge-document-gather`'s), jig's crates.
- Pre-approved: if a title past smith's run bytes would not fit, cut the title's path and
  heading to fit and say how; if the brief's abandonment reaches the root by a path that does not
  free its titles today, free them where the root drops the brief's rows, and say where.

## Steps

1. The kind, the titles' keeping, the binding, the Missing mapping, the room rule.
2. smith's titles; the limits, the override, the start check; the pricing.
3. The tests; run them alone, then the three crates' lib tests, then the readers' tests above.
4. The status sentence.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-engine-domain -p temper-engine-smith -p temper-service
   -p temper-engine-domain-world --all-targets -- -D warnings`; the citations check.
6. Commit (why: a charter's documents reach a run's brief as their own titled sections); push
   your branch.

## Report

As the implementer's text says, and also: every assertion and figure that moved with its values,
where the titles are kept and freed, and the tests' count.

## Answers

- 2026-11-02. *May the titles' storage go in `temper-engine-domain/src/domain.rs`, which
  the files omitted?* Yes: `domain.rs` is now in the header. Add one field to `Domain`, a map
  keyed by the brief's task holding its planned documents' titles (each entry at most
  `tasks.charter_documents` titles), made with capacity `brief.briefs` at `Domain`'s
  construction beside `brief_connectors`, and its emptiness joined to the idle check beside
  `brief_connectors.is_empty()`. The binding inserts the entry; `Complete` reads it for a
  `Missing` document kind and removes it; an abandonment removes it where the brief's rows are
  dropped. Nothing else in `domain.rs` changes; report the field and where it is freed.
