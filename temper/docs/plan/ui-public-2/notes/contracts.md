# ui-public-2: the shapes this plan binds

Both tasks build to these. Both schemas change in place: they stay `public-v1`, family version 1. Nothing stores a public record (the public index is memory, jig's `domain/people.md`, 13.9), and the server and its bundle ship together, so a change here needs no compatibility (the request: a protocol may change freely).

## jig's `public-v1.schema` (task `public-records`)

```
enum PublicPhase { waiting, idle, due, preparing, claimed, running, backing_off, active, closing, held, done, failed, cancelled }
record PublicLine {
    task: u64
    title: text 256
    title_cut: bool
    parent: option u64
    goal: bool
    priority: option u32
    executor: PublicExecutor
    phase: PublicPhase
    hold: option PublicHold
    made: u64
    changed: u64
    active: option u64
    ended: option u64
    spent: option u64
    delegates: PublicCounts
}
```

- `budget` goes. `spent` is `Some` exactly when `goal` is true, and holds what it holds today: `numbers.spent + numbers.spent_below`.
- `PublicPhase::ended` goes, because nothing would make it. `tasks::Phase::Ended(ending)` makes `done` from `Ending::Done`, `failed` from `Ending::Failed` and `cancelled` from `Ending::Cancelled`, whether or not a cancelled task keeps a result.
- Every other record and enum in the schema stays as it is. In particular `title`, `title_cut` and what a title is don't change: titles aren't this plan's.

## temper's `public-v1.schema` (task `public-page`)

Only these records change. Each loses its forge addresses and keeps the rest of its fields, in their order:

```
versioned record PublicChange { task: u16 head: option fixed 32 step: PublicStep repair: option PublicRepair merge: option fixed 32 tiers: u32 hold: option PublicHold ci: PublicCi gates: list 64 PublicGate repairs: PublicUsed resolutions: PublicUsed updates: PublicUsed way: PublicWay in_landing: bool }
record PublicQueue { changes: list 64 PublicQueued }
record PublicGateProof { number: u64 kind: PublicGateKind blocks: bool proof: PublicProof }
record PublicMember { task: u16 base: option fixed 32 candidate: option fixed 32 landed: option fixed 32 gates: list 8 PublicGateProof }
```

The schema then holds no `bytes` or `text` field at all.

**Why the addresses go, not only the links.** The forge's state doesn't hold whether a repository is private. `Reached.private` (`temper/crates/temper-engine-domain-forge/src/boundary.rs`) is read in the list of what can be adopted, and no repository row keeps it. So the request's ruling lands on its last branch: drop the links. A field no page reads would still hand a private repository's owner and name to anyone reading the record.

## What a page shows (task `public-page`)

Binding words, from the request and `temper/docs/design/client/browser.md`, 8.3:

| Where | Words |
|---|---|
| under the board's title | `What temper is doing on this project, as it happens. Read-only.` |
| a live goal's second line | `running · for 14 minutes · 126 spent` |
| a live task's second line | `running · for 14 minutes` |
| an ended goal's second line | `landed · 2 hours ago · 126 spent` |
| the task page's own row | `an agent · running · for 14 minutes`, and its spend after them if it is a goal |
| a goal's ending, from its phase | `done` reads `landed`; `failed` reads `failed`; `cancelled` reads `ended` |
| a task's ending, from its phase | `done` reads `done`; `failed` reads `failed`; `cancelled` reads `ended` |
| a board with no goals | `No goals yet.` |
| no landing yet | `Nothing has landed yet.` |
| no public project | `No public projects yet.` |
| a change's commit | `Its change: 789ada1`, the first seven digits, plain text |

- The goal's mapping is the signed-in Goals page's (`temper-client-domain/src/pages/goals.rs`, `state`), with "ended" where that page says "Closed", as the request rules.
- A change's commit is its `merge` where it has one, otherwise its `head`. A landing's member's is its `landed`, otherwise its `candidate`. With neither, the words don't show. A member's `base` isn't shown.
