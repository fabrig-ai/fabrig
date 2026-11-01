# Part `pd` (jig's deletable project): progress

Designer of draft section 5, its testing in section 8, the jig half of 6.2, and contracts J1-J8.

## Specs

| id | state | needs |
|---|---|---|
| pd-acts | final | - |
| pd-vocabulary | final | pd-acts |
| pd-people-purge | final | pd-acts |
| pd-tasks-erase | final | pd-vocabulary |
| pd-accounts-uses | final | pd-vocabulary |
| pd-store-reads | final | pd-vocabulary |
| pd-test-connector | final | pd-vocabulary |
| pd-client-wire | final | pd-vocabulary, pd-test-connector |
| pd-client-view | final | pd-client-wire |
| pd-core-decide | final | pd-vocabulary |
| pd-core-erase | final | pd-core-decide, pd-tasks-erase, pd-people-purge, pd-store-reads |
| pd-core-release | final | pd-core-erase, pd-accounts-uses, pd-test-connector |
| pd-conformance | final | pd-core-release |

All twelve specs are final.

## For the lead: `pd-cancel` and the ids temper's tasks need

- **No separate `pd-cancel`.** J8's `Act::Cancel` is already decided in `pd-core-decide`
  (final, needs only `pd-vocabulary`; its test
  `the_cancel_act_cancels_any_live_task_of_its_project`). A separate task would write
  `jig-domain-core` (`routing/deployment_acts.rs`) beside `pd-core-decide`, and splitting
  `Cancel` out would change a final spec. **Recommendation:** `fe-world-remove` needs
  `pd-core-decide`. If you would rather split it, say so and I will write `pd-cancel`
  (about 45 minutes: core `Cancel` plus its test) and a replacement for `pd-core-decide` that
  needs it, as new ids.
- **The ids temper's reset needs:**
  - The decision (act and request, the row at `Ending`, the project closed at once, the
    cancel, `ProjectRemoved`) and its move to `Erasing`: `pd-core-decide`.
  - The `Erasing` stage: `pd-core-erase`.
  - The `Releasing` stage, the connector told (`Ask::ReleaseProject` sent, `ProjectReleased`
    awaited from every connector), the row erased at the end: `pd-core-release`.
  - The restart resume: each stage's own task (`Ending` in `pd-core-decide`, `Erasing` in
    `pd-core-erase`, `Releasing` in `pd-core-release`). A deletion runs whole, resumed at
    every stage, once `pd-core-release` lands.
  - Shapes temper matches: `pd-acts` (people acts, ask, outcome, row, grant words;
    temper's forced arms) and `pd-vocabulary` (`Ask::ReleaseProject` answered at once by
    temper's root, `Write::DropTranscript` routed to the store, `Event::ProjectReleased`).

## Design calls inside the contracts

- **Where the deletion row lives.** In the people child (`Stored::Deletion`, restored through
  every root's existing people restore arm, so no root changes for it), read by the core as
  `core.people.deletion(project)`; the core drives the stages.
- **Erasure through two raw store reads.** The store has no range erase, and an ended task's
  rows are not in memory. The core pages `reads::Scope::ProjectRows { project }` (a project's
  ended rows and decided person proposals, found by a filtered scan) and, per task,
  `reads::Scope::TaskRows { task }` (every row under the task's key prefixes), erasing what
  they return in bounded decisions; the tasks child's `Event::Erase { record }` (J6's
  tasks-child request) erases what the ended row names; the ended row goes last, so a crash
  resumes by scanning again. Both scopes pass through every root unchanged (the store's
  `Request::Read` carries the core's `reads::Range`): no new crossing shape.
- **Notes need no notes-child change.** The core pages the existing `Scope::Notes { project }`
  read and erases entries and lines; the notes child's cache names scopes no request names
  again.
- **Live ledgers are closed, not dropped** (`retire::close`: `Ledger` erased, `ClosedLedger`
  saved): no live ledger names the project afterwards, and its settled spend stays, as J4
  asks; `ClosedLedger` rows are the "settled spend totals" the referee allows.
- **The client leaves for `/`** (`Address::Inbox { project: None }`): jig's client has no
  projects-list address; the root's frame lists the projects (J7 "the projects list").
- **Retained answers naming a gone project restore** (people's and the core's restore checks
  relaxed), since J1 and J4 keep them until their retention.

## Temper files the jig tasks touch (for the lead's ordering with section T)

- `pd-acts`: `temper-engine-domain/src/{store.rs,route/demo.rs}`,
  `temper/tests/engine/domain/src/{roles.rs,roles_referee.rs}` (compiler-forced arms).
- `pd-vocabulary`: `temper-engine-domain/src/{route.rs,route/asks.rs,store.rs,decision.rs}`
  (ReleaseProject answered at once, `Write::DropTranscript`),
  `temper-service/src/{store/route.rs,clients.rs,clients/translation.rs}`,
  `temper/tests/client/src/translate.rs`, `temper/tests/one-process/src/referee/*`,
  `temper/tests/engine/domain/src/{store.rs,escalation_referee.rs,roles_referee.rs}`.
- `pd-client-wire`: `temper-client-tools/src/jig/render/*`, `temper-client-view/src/cards.rs`,
  `temper/tools/gallery/src/worlds.rs` (compiler-forced arms).
- `pd-core-release` runs temper-engine-domain's tests (restore relaxations), editing nothing.

## Task graph and critical path

```
pd-acts ──┬─ pd-people-purge ─────────────────────────────┐
          └─ pd-vocabulary ─┬─ pd-tasks-erase ────────────┤
                            ├─ pd-store-reads ────────────┤
                            ├─ pd-core-decide ── pd-core-erase ── pd-core-release ── pd-conformance
                            ├─ pd-accounts-uses ──────────────────┘ │
                            └─ pd-test-connector ───────────────────┘
                                     └─ pd-client-wire ── pd-client-view
```

Critical path: pd-acts (110) → pd-vocabulary (100) → pd-core-decide (120) → pd-core-erase
(120) → pd-core-release (110) → pd-conformance (120): about 680 implementer minutes. The
children (tasks, people, store, accounts, test connector) and the client run beside it and
finish before the core needs them.

## Contract requests

All five answered by the lead (yes to each recommendation or default); contracts.md J4, J7
and J8 now say so. Kept here for the record.

1. **People store version (J4, "people's store version moves by one").** jig's store design
   says the opposite: `jig/docs/design/protocol/store.md`, 5 (lines 382-386): "Until the first
   release, each family's version keeps its current number, and its schema, generated code and
   one set of golden bytes change in place. Versions move at a release." Recent people changes
   followed it (`jig-protocol-store/src/tests/settings_layout.rs:1`, "changes people 13 in
   place"); the layouts fingerprint is what refuses an older store. A bump to people v14 renames
   `people_v13` in about 40 files, among them temper's `temper-protocol-store`,
   `temper-forge-setup`, `temper-fake` and `tools/twin`, which section T's tasks also write.
   **Recommendation:** people v13 changes in place (schema, generated code, goldens, layouts
   fingerprint), as store.md 5 says. **Until answered** the store spec follows the contract as
   it stands (v14); it is written last of the surface so that an answer can still change it.
2. **jig's client wire version (J7, "one more client wire version").** Same rule in
   `jig/docs/design/protocol/client.md:432` and `:769` (client-v19 changes in place until the
   first release). A bump to v20 renames `wire::v19` in about 144 files of jig and temper.
   **Recommendation:** client-v19 changes in place. Until answered, the wire spec follows
   the contract (v20).
3. **J8's `AddKey { uses }` type.** `jig_domain_people::Act` cannot name
   `jig_domain_accounts::Use`: jig-domain-people does not depend on jig-domain-accounts. People
   already has the same type, `jig_domain_people::Use::{Party(u64), Project(u32)}`
   (`boundary.rs`, used by `AccountChange::SetUses`), which the core translates to accounts'
   `Use`. **Default taken:** `uses: Option<Box<[jig_domain_people::Use]>>`; temper names
   `people::Use::Project(project)`.
4. **J8 order of variants.** J1 appends `RemoveProject` after `Adopt` and last in
   `ActKind::ALL`; J8 adds `Cancel`. **Default taken:** `Act::{…, Adopt, RemoveProject,
   Cancel}`, `ActKind::{…, MakeProject, RemoveProject, Cancel}`, `ALL` in that order.
5. **Retained answers (J4's keep list).** Both a connector's `Stored::Acted` and a party's
   `Stored::Answer` naming the project are retained answers: both stay until their retention
   and answer as saved, then `Unknown`. **Default taken:** the referee allows both.
