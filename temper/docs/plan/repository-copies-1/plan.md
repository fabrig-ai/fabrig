# repository-copies-1: the plan

The foundation of `temper/docs/design/draft/repository-copies.md`, its section 10, step 1. The welcome,
Ready's copies and the "Add a repository" sheet are not this plan's.

**skein is changed, and stays generic.** `skein-forge` gains `Migrate` and `DeleteRepository`;
`skein-forge-setup` gains the plan `Copy`; the fake forge gains a migration route and a catalogue. None
names temper, a project or an adoption.

## The design, written first

- `skein/docs/design/forge-setup.md`, 4.3, 4.4, 4.10 (new), 6.3, 6.4, 8, 9, and its status.
- `skein/docs/design/fake-forge.md`, 2.6 (new), 4, 6, 7, and its status.
- `temper/docs/design/draft/repository-copies.status.md`: the entries these close, and what departs.

## Tasks released so far

| Task | Part | Needs | What | Minutes |
|---|---|---|---|---|
| `migrate-request` | forgejo-requests | | the two requests of the dialect | 35 |
| `throwaway-migrations` | copy-facts | | a migration's setting, patience, kill and restart | 35 |
| `fake-copy` | fake-migration | | the fake's domain copies and deletes | 70 |
| `copy-surface` | copy-plan | `migrate-request` | the public shapes, every reader compiling | 40 |
| `copy-machine` | copy-plan | `copy-surface` | the plan's steps | 60 |
| `copy-words` | copy-plan | `copy-surface` | Forgejo's words, `TimedOut` | 45 |
| `copy-facts` | copy-facts | `migrate-request`, `throwaway-migrations` | the facts, recorded | 75 |
| `fake-migrate` | fake-migration | `migrate-request`, `fake-copy` | the route and the catalogue | 60 |
| `copy-story` | copy-story | `copy-machine`, `copy-words`, `fake-migrate` | the plan on the fake | 50 |

Later batches add temper's store, request, engine, service, client, `ctl` and fake backends.

## Critical path, as it stands

`migrate-request`, `copy-surface`, `copy-machine`, `copy-story`: about 185 minutes, with `fake-copy` and
`fake-migrate` (130) and `copy-facts` beside it. This section grows with each batch.

## What the cut decided, where the draft leaves it open

- **How the plan knows a repository is its own** (draft, 3, step 3): the item `Copying`, written by
  the plan before its `Migrate` and kept with the owner's record of the copy. A flag set by the owner
  at the run's start would let a crash between the start and the read adopt or delete a repository the
  plan never started.
- **Forgejo's words:** `Event::Said`, beside the stop, for `Migrate` alone.
- **A source with no branch** stops at `Empty`.
- **A copy past its deadline** stops `TimedOut`; `Abort`, `Remove` and every other plan stay `Down`.
- **The fake's catalogue is the protocol layer's,** since the domain's `Config` is `Copy`, and a copy
  is called on its source, since a call needs a repository that exists.
- **A deleted repository's slot stays, marked gone,** in the fake's domain: some 124 sites assume a slot
  lives.

## For the user to weigh

- **`[git.timeout] MIGRATE`:** Forgejo's own bound on a migration, whose default the surveyor recalled
  as 600 s, from memory, unchecked. temper's owned `app.ini` sets none, so a large copy may stop
  with Forgejo's words well before `copy_deadline` (30 minutes). The draft says "its settings are
  Forgejo's defaults", so the plan keeps them (`skein/docs/design/forge-setup.md`, section 9).
- **A live migration deleted:** from memory, Forgejo's migration runs on after its client goes, so the
  next run's delete may fall on a live migration. `copy-facts` records what happens; if it is not safe,
  4.10's step 3 needs a revision, which the plan will carry as a new task.

## Overlaps with other drafts

- `skein/tests/forge/conformance/facts.txt`: `macos-cheap`. `copy-facts` only adds lines and directories,
  by a focused mode. Its recording stands at the pin on its base: `macos-cheap` moves it to 17.0.1, and
  a recording at 17.0.0 reruns unchanged.
