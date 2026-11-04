# repository-copies-1: the plan

The foundation of `temper/docs/design/draft/repository-copies.md`, its section 10, step 1. The welcome,
Ready's copies and the "Add a repository" sheet are not this plan's.

**skein is changed, and stays generic.** `skein-forge` gains `Migrate` and `DeleteRepository`;
`skein-forge-setup` gains the plan `Copy`; the fake forge gains a migration route and a catalogue. None
names temper, a project or an adoption.

## Released so far

| Task | Part | Needs | What | Minutes |
|---|---|---|---|---|
| `migrate-request` | forgejo-requests | | the two requests of the dialect | 35 |

Later batches add the plan, the facts, the fake, then temper's store, request, engine, service, `ctl`
and fake backends.

## Critical path, as it stands

`migrate-request`, then the plan `Copy`, then temper's request. This section grows with each batch.

## What the cut decided, where the draft leaves it open

- **How the plan knows a repository is its own** (draft, 3, step 3): the item `Copying`, written by
  the plan before its `Migrate` and kept with the owner's record of the copy. A flag set by the owner
  at the run's start would let a crash between the start and the read adopt or delete a repository the
  plan never started.
- **Forgejo's words:** `Event::Said`, beside the stop, for `Migrate` alone.
- **A source with no branch** stops at `Empty`.
