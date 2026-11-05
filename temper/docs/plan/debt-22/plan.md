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

D021 is still being cut.

## What was decided

- **All six findings are right but one clause.** Each was checked against the code at this plan's base. `capacity.rs`'s `project` doesn't share `named_project`: it answers `Some(DEPLOYMENT)` for three families the others answer `None` for, so a helper wouldn't shorten it. The review's "act table" is `jig-domain-core/src/tests/acts/classifiers.rs`.
- **`remap_models`' test goes further than the review's.** A `Models::remap` test alone wouldn't catch dropping `proposal`, `recurring` or `History`, which is what the finding complains of. `tasks-pins` adds a short table over those arms, at most 25 lines; if an arm doesn't fit, the request lets it go unpinned, and the report names it.
- **`store.md`'s two sentences are the cut's, not an implementer's.** They state what `commits.rs` does today. The store's `words` tests read that file, and `store-copy` runs them.
- **`word_bytes`' dropped term leaves a const assertion.** The reminder being shorter than the notice is why the term never wins, so that fact stays checked by the build.

- **D022's item is stale in one sentence, and its behaviour stays.** It says `SignIn`, `Refresh` and `Revoke` are dropped outside `Idle` with no terminal. The code answers `Failed { Limit }` or `Revoked { confirmed: false }`, as `skein/docs/design/oauth.md`, 3 says, and `oauth-client-phase` pins it.
- **A record's flags become three fields, not 32 variants.** `skein/docs/foundation/programming-model.md`, 5.4 asks for one enum per lifecycle and never one for their product: `Lending`, `Refresh` and `Notice`. `rejection` goes, since it's `Some` exactly when `rejected`.
- **D022 leaves `exchange.rs`.** The item's **Where** names it and its **Paid when** doesn't, and the user wants this plan small. The status keeps `pending_grant`, `aborted` and `file_active` without the item, as debt-21 did for `GitHub`, and says neither machine has one handler per cell.
- **`entries` isn't prefilled,** as `names` was: that would raise `Component::new`'s heap. A merged list's growth may still move a heap ceiling, which goes to the user with measurements.

## Overlaps

- `brief-failure/retry-reason` holds `jig/crates/jig-domain-tasks/src/**`, and changes `boundary.rs`, `stored/fit.rs` and `tests/revisions.rs` on its trunk: none of them in the lines these tasks touch, by its diff. fab's merges carry it.

## Critical path

`accounts-record-states`, `accounts-entries`, `oauth-states-close`, 150 minutes, so far. `tasks-pins` or `tasks-families`, `separate-slots`, 120.
