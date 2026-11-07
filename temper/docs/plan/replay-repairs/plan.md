# replay-repairs: what the first real goal on a fresh box tripped over

The six faults of `inputs/request.md`, each a task, cut so that the independent ones go first. This file grows with each push.

## Parts and tasks

| Part | Task | Tree | What it makes true | Request item |
|---|---|---|---|---|
| `refusal-at` | `settings-at` | jig | an out-of-shape settings refusal's `at` is the failing item's position | 6 |
| `room` | `reference-room` | temper | `room.json` carries `root.journal.run_bytes` 65,536, its README's commands merge it, and says what a charter must fit | 5 |

Items 1 to 4 (the restart panic, the review's pull files, the turn's lost input, the delivery's silent timeout) are being surveyed; their tasks follow in the next push.

**Critical path, so far:** none; both tasks need nothing and run at once.

## What this cut decided

- **`at` counts from 0, as a batch's members do** (`jig/crates/jig-domain-core/src/routing/party/create.rs`), the last item naming the failing setting, and 0 when none does (a whole-set check failing on a setting the change leaves alone; a restore). `jig/docs/design/domain/engine.md`, 17.2 now says so. temper's own translation refusal (`temper-service/src/clients/translation.rs`) counts from 1; it is another tree's, left as found and named in the report.
- **`root.journal.run_bytes` goes in `room.json`, as the request says,** in a new `application` member merged over the document as `limits.json` is. `temper/docs/design/protocol/configuration.md`, 13.9 now says `room.json` holds it beside the agent's and the workers' room.

## Overlaps with the drafts beside this one

- `settings-at` changes `jig/crates/jig-domain-core/src/routing/settings/dispatch.rs`, which `model-catalogue-3/core-reads` changed (merged into its trunk, not yet landed): the file the request names, so unavoidable.
- `jig/docs/design/domain/engine.status.md` and `temper/docs/design/protocol/configuration.status.md` take one entry each, which the tasks remove; `model-catalogue-3/list-close` globs `**/*.status.md`.
