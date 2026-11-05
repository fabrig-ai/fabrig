# debt-22: the twenty-second debt round

Two independent parts (`inputs/request.md`): the spot review of debt-20's jig part, and skein's state items D021 and D022. No temper client view, no `temper.css`, no `jig-client-view`. This file grows with each push.

## Parts and tasks

| Source | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| review 1, 2, 4 | `spot-review` | `tasks-pins` | | high | 60 | `word_bytes`' dead term; `remap_models`' test; `FinishRequested`'s refusal; one `LIMITS` |
| review 1, 2 | `spot-review` | `tasks-families` | | high | 60 | family table's columns; all eight wake categories |
| review 1, 2 | `spot-review` | `core-tables` | | high | 60 | subject table's columns and always-true assert; act table's loop; `history_copy`'s test |
| review 4 | `spot-review` | `store-copy` | | medium | 30 | `Write::Copy`'s tests; `store.md`'s sentences are in this plan's commit |
| review 1, 6 | `spot-review` | `people-tables` | | medium | 45 | `Settled` rows, aliases, `named_project` |
| review 5, 6 | `spot-review` | `core-small` | | medium | 30 | `settings::moves`; `control` takes its row |
| review 3 | `spot-review` | `separate-slots` | `tasks-pins`, `tasks-families` | high | 60 | `MessageFamily::Separate` |
| skein D022 | `oauth-states` | `oauth-client-phase` | | high | 60 | the client's `Phase` holds its exchange; pins first |
| skein D022 | `oauth-states` | `accounts-record-states` | | high | 60 | `Lending`, `Refresh`, `Notice`; pins first |
| skein D022 | `oauth-states` | `accounts-entries` | `accounts-record-states` | medium | 75 | one `AccountEntry` per account |
| skein D022 | `oauth-states` | `oauth-states-close` | `oauth-client-phase`, `accounts-entries` | medium | 15 | the item goes; status keeps what's left |
| skein D021 | `forgejo-states` | `answer-measure` | | medium | 25 | `size.rs` checks row limits, then asks `crate::measure`; pin first |
| skein D021 | `forgejo-states` | `small-plan-states` | | medium | 25 | `artifact`, `start` |
| skein D021 | `forgejo-states` | `turn-states` | | medium | 30 | `turn` |
| skein D021 | `forgejo-states` | `contents-states` | | high | 60 | `contents` |
| skein D021 | `forgejo-states` | `actions-states` | | high | 60 | `actions` |
| skein D021 | `forgejo-states` | `writes-states` | | high | 75 | `writes`; pins first; `error` keeps its answers |
| skein D021 | `forgejo-states` | `reads-delegates` | | medium | 30 | five `Option<Plan>`s become one `Delegate` |
| skein D021 | `forgejo-states` | `reads-states` | `reads-delegates` | high | 120 | `reads`; pins first; no `Rows` |
| skein D021 | `forgejo-states` | `plan-states-close` | the seven above | medium | 10 | the item goes; status keeps what's left |

990 implementer-minutes in all.

## What was decided

- **All six findings are right but one clause.** Each was checked against the code at this plan's base. `capacity.rs`'s `project` doesn't share `named_project`: it answers `Some(DEPLOYMENT)` for three families the others answer `None` for, so a helper wouldn't shorten it. The review's "act table" is `jig-domain-core/src/tests/acts/classifiers.rs`.
- **`remap_models`' test goes further than the review's.** A `Models::remap` test alone wouldn't catch dropping `proposal`, `recurring` or `History`, which is what the finding complains of. `tasks-pins` adds a short table over those arms, at most 25 lines; if an arm doesn't fit, the request lets it go unpinned, and the report names it.
- **`store.md`'s two sentences are the cut's, not an implementer's.** They state what `commits.rs` does today. The store's `words` tests read that file, and `store-copy` runs them.
- **`word_bytes`' dropped term leaves a const assertion.** The reminder being shorter than the notice is why the term never wins, so that fact stays checked by the build.

- **D022's item is stale in one sentence, and its behaviour stays.** It says `SignIn`, `Refresh` and `Revoke` are dropped outside `Idle` with no terminal. The code answers `Failed { Limit }` or `Revoked { confirmed: false }`, as `skein/docs/design/oauth.md`, 3 says, and `oauth-client-phase` pins it.
- **A record's flags become three fields, not 32 variants.** `skein/docs/foundation/programming-model.md`, 5.4 asks for one enum per lifecycle and never one for their product: `Lending`, `Refresh` and `Notice`. `rejection` goes, since it's `Some` exactly when `rejected`.
- **D022 leaves `exchange.rs`.** The item's **Where** names it and its **Paid when** doesn't, and the user wants this plan small. The status keeps `pending_grant`, `aborted` and `file_active` without the item, as debt-21 did for `GitHub`, and says neither machine has one handler per cell.
- **`entries` isn't prefilled,** as `names` was: that would raise `Component::new`'s heap. A merged list's growth may still move a heap ceiling, which goes to the user with measurements.

- **D021's own-enum forms are 18 in the plans, not the item's 35.** The review counted all of `forgejo/`, whose `client/` and `server/` the item's **Where** doesn't name, and which stay. Tuple scrutinees read 0 in the plans already. The reads' five `Option<Plan>`s and 56 stage comparisons, which the lint can't see, are in the acceptance too.
- **"The dialect isn't asked again" has no site in the plans.** No plan reads a dialect. The cut reads the request's clause as what's asked twice here: the call, read 51 times, and the repository row, re-asserted 19 times. A state holds them, and the call is matched once. The one literal re-ask is `match (side, &mut entry.dialect)` in `skein-forge-connection/src/component/calls.rs`, outside the item's **Where**; `Side::GitHub` needs `gh` from the entry either way, so a smaller change wouldn't remove it. It's left, and named in the report.
- **`size.rs` keeps its row limits.** `measure::answer` alone would answer a size for an `Item` with 9 labels at `rows` 8, where `size::answer` answers `TooLarge`, and that's reachable from Forgejo: it would change a refusal. So `within` counts and `measure` measures, and that's one measure. `reads::size::answer` stays public, as the connection's tests read it. `dequeued` now counts, and no Forgejo path sets it.
- **Unreachable row cells change spelling.** A row kind the state's exchange can't decode answers `InvalidAnswer`, where `reads.rs` has `TooLarge` or a cut. The exchange's decoder picks the row kind, so the wire can't reach these and no test feeds them; `Rows` can't go otherwise. If a test sees it, the task stops and asks. This is the one place the cut changes an answer, and only in cells nothing reaches.
- **`start.rs`'s `ended` goes too,** though it has no `Stage`: a flag is what the request names, and it costs about 6 lines.
- **`forge.md` names no private stage.** Its table cited `reads::Stage::Read` and `contents::Stage::File`, and now names the reads themselves, true before and after this part. `forge.status.md`'s entry stays until `plan-states-close`.
- **Both status entries keep "one handler per cell".** Neither item's **Paid when** asks it. The statuses say so rather than dropping that departure with the item.

## Seen, left

- Accounts' `has_work` answers true for a `due` record with no `earliest`, including while its exchange runs, when `fire` then does nothing. It reads like a busy loop for the owner. Read, not run; changing it changes behaviour, so it's for fabrig/debt.

## Overlaps

- D021 and D022 meet no other draft: `brief-failure/forge-answer`'s changes in `forgejo/` are in `client/` and its fixtures, not `plans/`.
- Inside this plan: `jig-domain-core/src/settings/tests.rs` has `core-tables` and `core-small`, on different tests; `jig-domain-tasks/src/tests.rs` has `tasks-families` and one line of `tasks-pins`. `accounts-record-states` and `accounts-entries`, and `reads-delegates` and `reads-states`, hold their files in order.
- `brief-failure/retry-reason` holds `jig/crates/jig-domain-tasks/src/**`, and changes `boundary.rs`, `stored/fit.rs` and `tests/revisions.rs` on its trunk: none of them in the lines these tasks touch, by its diff. fab's merges carry it.

## Critical path

`reads-delegates`, `reads-states`, `plan-states-close`, 160 minutes. `accounts-record-states`, `accounts-entries`, `oauth-states-close`, 150. `tasks-pins` or `tasks-families`, `separate-slots`, 120.
