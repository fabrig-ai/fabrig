# repository-copies-1: the plan

The foundation of `temper/docs/design/draft/repository-copies.md`, its section 10, step 1. The welcome,
Ready's copies and the "Add a repository" sheet are not this plan's.

**skein is changed, and stays generic.** `skein-forge` gains `Migrate` and `DeleteRepository`;
`skein-forge-setup` gains the plan `Copy`; the fake forge gains a migration route and a catalogue. None
names temper, a project or an adoption.

## The design, written first

- skein: `skein/docs/design/forge-setup.md`, 4.3, 4.4, 4.10 (new), 6.3, 6.4, 8, 9;
  `skein/docs/design/fake-forge.md`, 2.6 (new), 4, 6, 7.
- temper: `temper/docs/design/domain/forge.md`, 4.5 (new); `temper/docs/design/protocol/setup.md`, 6.9
  (new), 7.5, 8.3; `temper/docs/design/protocol/store.md`, 2, 3, 5;
  `temper/docs/design/protocol/configuration.md`, 7, 12.1; `temper/docs/design/client/README.md`, 4.6, 8;
  `temper/docs/design/client/tools.md`, 5.3.
- Their status files, with **Not yet**; `temper/docs/design/draft/repository-copies.status.md`: the
  **Design lags** of sections 3, 5, 6 and 10 closed, the **Design departs** below added.

## Parts

- **forgejo-requests:** `Migrate` and `DeleteRepository` in the dialect.
- **copy-plan:** the plan's surface, its steps, its words and deadline.
- **copy-facts:** the throwaway for a migration, and the conformance's facts.
- **fake-migration:** the fake's copy and delete, its route and catalogue.
- **copy-story:** the plan on the fake's route.
- **making-records:** the wire, the address's check, the store family.
- **making-engine:** the connector's decisions, the root's act.
- **making-service:** the plans run, one writing run at a time.
- **making-client:** the client's facts, `repository make` and `copy`.
- **fake-backends:** two sources on the `fake forgejo` peer.
- **copy-end-to-end:** the slice with `walk = true`; the walk's brief is `walk.md`.

## Tasks

| Task | Part | Needs | What | Minutes |
|---|---|---|---|---|
| `migrate-request` | forgejo-requests | | the two requests of the dialect | 35 |
| `throwaway-migrations` | copy-facts | | a migration's setting, patience, kill and restart | 35 |
| `fake-copy` | fake-migration | | the fake's domain copies and deletes | 70 |
| `copy-surface` | copy-plan | `migrate-request` | the public shapes, every reader compiling | 40 |
| `copy-machine` | copy-plan | `copy-surface` | the plan's steps | 60 |
| `copy-words` | copy-plan | `copy-surface`, `copy-machine` | Forgejo's words, `TimedOut` | 45 |
| `copy-facts` | copy-facts | `migrate-request`, `throwaway-migrations` | the facts, recorded | 75 |
| `fake-migrate` | fake-migration | `migrate-request`, `fake-copy` | the route and the catalogue | 60 |
| `copy-story` | copy-story | `copy-machine`, `copy-words`, `fake-migrate` | the plan on the fake | 50 |
| `making-wire` | making-records | `copy-surface` | the request, the rows, `source.rs` | 50 |
| `making-family` | making-records | `copy-surface` | byte 35, `forge.makings` | 55 |
| `making-decide` | making-engine | `making-wire`, `making-family` | the connector's decisions and states | 70 |
| `making-root` | making-engine | `making-decide` | scope, the connector's act, the grant | 70 |
| `making-run` | making-service | `making-root`, `copy-machine`, `copy-words`, `fake-migrate` | the service's pull and the plans | 75 |
| `making-facts` | making-client | `making-wire` | the client's rows and ask | 30 |
| `repository-tools` | making-client | `making-facts` | `repository make`, `copy`, `--until adopted`, help | 70 |
| `fake-box` | fake-backends | `fake-migrate` | two sources, their printed lines | 35 |
| `copy-walk` | copy-end-to-end | `making-run`, `repository-tools`, `fake-box`, `copy-story` | the process test and the restart | 60 |

## Critical path

`migrate-request`, `copy-surface`, `making-wire`, `making-decide`, `making-root`, `making-run`, `copy-walk`:
35 + 40 + 50 + 70 + 70 + 75 + 60 = 400 minutes of implementer time. `copy-machine`, `fake-copy`,
`fake-migrate` and `repository-tools` run beside it; `copy-facts` is outside it, but what it records can
add a task.

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

- **Names:** the row is `RepositoryMaking`, the family and its `Stored` and `Key` variants alone being
  `Making`: the wire and the forge's boundary each hold a `Making` already (a forge a request makes or
  names). The word "address" is taken twice as well, so the check is `source`.
- **The adoption is jig's connector act** (`Act::Adopt` under `engine.acts`), not a party's request as the
  playground's is: a party's needs a sign-in that a long copy can outlive and that no restart keeps. It
  costs temper a route it did not have, and `"adopt"` in the grant `temper init` writes. Other documents
  must add it; a document without it keeps its copy and its row says so.
- **A `home` in a project with a home is refused at admission,** not after a 30-minute copy.
- **The row holds no party:** the connector is not told one and nothing reads it (draft, 5.2).
- **A row with no `into` is erased once made:** nothing is left for it to say.
- **`Empty` on a managed forge** uses the record's `first.label`, or else `docker`, the default the client
  gives; `temper-host` on an owned one.
- **`--until adopted`** reads the row gone with its forge still listed, since a row is erased at adoption
  and a failed one stays. Acts took no `--until` before; these two alone do.
- **The sources are on the `fake forgejo` peer,** a Forgejo elsewhere: an owned Forgejo is real, and cannot
  reach `.invalid`. So the walk connects that peer as its admin, and the project it needs comes from
  `--first`, ctl having no tool that makes one.
- **The walk:** only on the fake backends. A copy from a real source would break "no test reaches a real
  host", and the request's "on a real box" is left to the person. A restart mid-copy is a world's story,
  since a fake peer's memory lasts one launch.

## Left out, for later

- **A failed row's dismissal** (draft, 5.2): no request dismisses one, and it comes with the sheet that
  shows it. Until then, asking for the name again replaces a failed row, and failed rows count against
  `forge.makings`.

## For the user to weigh


- **`[git.timeout] MIGRATE`:** Forgejo's own bound on a migration, whose default the surveyor recalled
  as 600 s, from memory, unchecked. temper's owned `app.ini` sets none, so a large copy may stop
  with Forgejo's words well before `copy_deadline` (30 minutes). The draft says "its settings are
  Forgejo's defaults", so the plan keeps them (`skein/docs/design/forge-setup.md`, section 9).
- **A live migration deleted:** from memory, Forgejo's migration runs on after its client goes, so the
  next run's delete may fall on a live migration. `copy-facts` records what happens; if it is not safe,
  4.10's step 3 needs a revision, which the plan will carry as a new task.

## Overlaps with other drafts

- **`to-begin-store`** is on the same enums and lines of `temper-engine-domain-forge`,
  `temper-protocol-store` and the wire's schema. `making-wire` and `making-family` need `copy-surface`
  for that reason alone, which lets it land first. `Dismissed` stays before `Making`, byte 34 before 35.
  A catch-up is likely.
- **`look-round-3`:** `temper-client-domain`'s `forge.rs` takes one `mod` line; `pages/` is untouched.
  `temper/docs/design/client/README.status.md`: this plan adds its own line.
- **`debt-11`, `debt-10`:** `temper-service`'s `setup.rs`, `setup/runtime.rs`, `route.rs`, `family.rs`. New code
  goes in `setup/making.rs`.

- `skein/tests/forge/conformance/facts.txt`: `macos-cheap`. `copy-facts` only adds lines and directories,
  by a focused mode. Its recording stands at the pin on its base: `macos-cheap` moves it to 17.0.1, and
  a recording at 17.0.0 reruns unchanged.
