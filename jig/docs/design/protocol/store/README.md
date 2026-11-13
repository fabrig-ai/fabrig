# The store

Provisional, 2026-10-12, revised 2026-10-28. What lies beneath the
engine's store vocabulary: skein's key-value store, embedded in the
engine's process, holds every record jig's core and the application
keep, transcripts are files beside it, and secrets are in a second
store. jig ships the component that does this, `jig-protocol-store`; the
application hands it its own records as bytes it encodes. The domain's
half (commits, loads, what is kept) is `../../domain/engine/README.md`, section 5,
and what the rest of the protocol layer relies on is `../README.md`,
section 9. Read it with skein's `data/kv.md`. What is still open is listed in
section 11.

## 1. In one page

- **skein-kv, in the engine's process.** The engine's protocol layer owns
  `jig-protocol-store`, a component that owns two skein-kv stores (skein's
  `data/kv.md`): the store, with every record jig and the application keep,
  and the secrets' store. Keys are ordered bytes, values are bytes, both
  in memory, made durable by an append-only log and snapshots. There is
  no store process, no wire and no SQL: the engine is the store's only
  client.
- **One component, two owners of records.** jig's records (the core's
  and its children's) reach the component typed, in the core's
  vocabulary, and the component encodes them. The application's records
  (its connectors' and its own children's) reach it as key and value
  bytes the application encodes, under the application's tag. Both go
  into one commit (section 3).
- **Keys sort as the domain's keys do,** written with skein-kv's key
  writer, with a tag per owner, then per family. Every range the domain
  asks for maps to a short, fixed list of intervals (section 4).
- **Records from schemas,** with skein's generator: a codec per family,
  versioned, golden bytes and fuzzing. A format row names the versions,
  each owner's patch level and the deployment; a store of another
  version or deployment refuses to open, and before the first release
  nothing is translated (`records.md`).
- **Generations.** The maps live in numbered generations under the
  deployment's directory, `active` naming the open one. A store behind
  the binary's patch levels is patched at open in a copy, checked, and
  switched to by one durable rename; a crash anywhere leaves one whole
  generation active (`README.md`, section 2, and `opening.md`, section 2 to 4).
- **One decision, one commit, the same number.** Each of the root's
  commits is one skein-kv commit with the same number. Group commit folds
  bursts into one sync. A failed or refused commit stops the engine
  (section 5).
- **Loads are answered in the step that asks,** from memory, in bounded
  pages: the restart's live ranges, parties' reads with their joins, a
  child's pages, single records (`loads.md`). Only a turn's bytes, read
  from its transcript, take io.
- **Ended tasks are found through indexes,** the tasks child's: by their
  requester party, and by their project for those flagged listed, each
  by label and newest ended first. Live tasks are the tasks child's
  memory, never the store's to list (`loads.md`, 3, 4).
- **Transcripts are files, one per task,** each turn a frame written and
  synced before the commit that accepts it. The map keeps where each
  task's current window starts, so a resumed run reads only its window
  (`transcripts.md`).
- **Secrets in a second store,** in a directory only the engine's user
  may read, written by components alone: minted credentials' digests,
  and LLM accounts' records and keys, each kept before it is used
  (`secrets.md`).
- **Growth is bounded by retention and fill:** the domain forgets an
  ended task once read, or shrinks it past a horizon on its timers, by
  the task's retention class, and refuses new work above a high-water
  mark of the map (section 6).
- **The real store in every world:** the domain worlds run the component
  over skein-kv's in-memory mode, with an in-memory stand-in for
  transcripts' files, and the store's own world crashes it on skein's
  simulated disk (section 9).

## 2. Where it sits

```
the application's root          commits (jig's writes typed, its own wrapped), loads, restart steps
        │ the root's store vocabulary, through the protocol layer's routing
        │   (the application's records encoded by its own codecs on the way down, decoded on the way up)
jig-protocol-store              keys and records to bytes; ranges to intervals; parties' reads and joins;
        │                       transcripts' files; secrets
        │ skein-kv's vocabulary (skein's data/kv.md, section 2)      │ io's file requests (skein's runtime/io/files.md)
skein-kv ×2                     the store; the secrets' store
        │ io's file requests
skein-io                        files: the ring, or the simulator's disk
```

- **A component of the engine's protocol layer,** step code under the
  subset (skein/docs/foundation/programming-model.md, section 10), with its limits, worst case,
  deadlines and most outputs per entry point. It depends on skein-lib,
  skein-kv, io's file vocabulary, skein-codec and its generated codecs,
  and the core's vocabulary (its keys, records, ranges and reads). It
  depends on no application's crate, and links no smith crate: turns are
  bytes to it.
- **On disk,** in the deployment's directory (`../configuration.md`,
  section 3), which nothing else writes. The two maps live in numbered
  generations; `active` names the one a start opens, and an upgrade
  patches a copy into the next and switches to it (`opening.md`, 2). Transcripts
  are shared by every generation:

```
<deployment>/
├── active          the active generation's number: its decimal digits and a newline
├── generations/
│   ├── 7/          the previous generation, kept until the next switch
│   │   ├── store/      skein-kv: snapshot, log segments
│   │   ├── secrets/    the secrets' store: the directory 0o700, its files 0o600
│   │   └── manifest    what made this generation (below)
│   └── 8/          the active one, as 7
└── transcripts/    one file per task, in directories by the task's number
```

- **A first start** is a directory with no `active`, and none of
  `store/`, `secrets/` and `transcripts/` at its top (the layout before
  generations, below). A `generations/` that holds only a `1/` whose `store/` and
  `secrets/` hold no file, beside at most its manifest or the manifest's temporary
  (`manifest.skein-<16 hex>`), is a first start a cut stopped before `active`
  named it (the stores' first open, which makes their files,
  comes after `active`): it is removed, with any temporary io's durable replace
  left beside `active` (`active.skein-<16 hex>`; skein's `runtime/io/files.md`, 2).
  A `generations/` that holds anything else, with no `active`, is
  refused, `Generations::Unnamed`, and left as it is: a generation
  whose `active` was lost, which the operator names again from a
  backup. The start then makes `generations/1/` with its empty
  `store/` and `secrets/` (each directory durable as io makes it), and
  generation 1's manifest; then writes `active`, `1`, by io's durable
  replace expecting no file there (`Store` with `Absent`); then makes
  `transcripts/`. The maps are empty: the root's first commit writes
  the format row (`records.md`, 3).
- **Strays.** `active` names a generation only once it is whole
  (`opening.md`, 4), and at most it and its previous (`active` less one) stand:
  every other entry of `generations/` is a stray, a copy a crash left
  before its switch or a generation a crash left in its drop, and every
  open removes them once the active generation has opened and nothing
  refuses (`opening.md`, 1). Nothing reads the previous generation; it is what an
  operator goes back to, until the active one's first commit of its
  own (temper's `draft/upgrades.md`, 3.2).
- **The manifest,** a file per generation, written by io's durable
  replace into the generation's directory before `active` names it,
  and never changed: a record of the component's own schema
  (`Manifest`, beside the format row in the component's family, `records.md`, 1,
  added to `component-v3.schema` in place, as the levels are: `records.md`, 3),
  encoded and versioned as every record. It names:
  - `generation`, the generation's number, and `from`, the one it was
    patched from, none for a first start's;
  - `before` and `after`, each owner's levels (`records.md`, 3), none before its
    baseline; a first start's before levels are all none, and its after
    levels the binary's;
  - `counts`: each kept family's count of records before its patches
    and after, one per family the binary's sets keep, in their order (a
    first start's each 0 and 0; none with no sets). Only kept families
    are walked: every other record is copied byte for byte, and no patch
    adds or removes a record, so the check before the switch refuses a
    count that moved (`opening.md`, 2, step 5);
  - `store` and `secrets`, the last commit of each map when patching
    ended: the store's, which the deployment's header names (section 5),
    and the secrets' store's; for a first start, 0 and none respectively;
  - `written`, the wall-clock time it was written, `env.wall` in its own
    unit.

  Nothing at open reads it: it is the operator's record of what an
  upgrade did, and what temper's `upgrade --check` and `versions`
  print (temper's `draft/upgrades.md`, 3.1).
- **A directory in the layout before generations** (`store/`, `secrets/` or
  `transcripts/` at its top, and no `active`) is a store from an older
  build, made before generations. Generations never have the first
  two at the top, and a first start makes `transcripts/` last, after
  `active`, so no cut leaves it alone; a new store beside it would
  number its tasks again over its files. Open refuses it, `OlderBuild`
  with `Older::Directory`, before anything is written, and leaves it
  untouched (`records.md`, 3). Before the first release nothing is translated:
  the operator stops the service, moves `store/`, `secrets/` and
  `transcripts/` out of the deployment's directory together (its
  transcripts belong to its tasks), and starts again, which is a first
  start. The refusal's text says so, naming
  the directory:

  > `<deployment>` holds a store from an older build, in the layout
  > before generations (`store/`, `secrets/` or `transcripts/` at its
  > top), and this start has changed nothing in it. Before the first
  > release a store is not translated, so every start refuses this one
  > until it is moved. To start afresh, stop the service, move
  > `store/`, `secrets/` and `transcripts/` out of `<deployment>`
  > together, and start again: nothing is deleted, and the next start
  > makes a new, empty store. The new store has no tasks, sign-ins,
  > accounts or keys, and every client signs in again.

- **`OpenRefusal::Format`** refuses another version of a family (`records.md`, 3),
  putting the family's lower-case name in place of <family>:

  > `<deployment>` holds a store whose <family> family is at another
  > version than this build's, and this start has changed nothing in it.
  > A store from before the baseline is not translated, so every start
  > refuses this one until it is moved. To start afresh, stop the service,
  > move `active`, `generations/` and `transcripts/` out of `<deployment>`
  > together, and start again: nothing is deleted, and the next start
  > makes a new, empty store. The new store has no tasks, sign-ins,
  > accounts or keys, and every client signs in again.

- **`OpenRefusal::Deployment`** refuses a format row naming another
  deployment than its header (`records.md`, 3):

  > The store in `<deployment>` names another deployment than its header
  > does, so its files come from two deployments, and this start has
  > changed nothing in it. Restore `<deployment>` whole from one backup,
  > or, to start afresh, stop the service, move `active`, `generations/`
  > and `transcripts/` out of `<deployment>` together, and start again:
  > nothing is deleted, and the next start makes a new, empty store.

- **`OpenRefusal::Numbers`** refuses a recovered last commit other than
  the one the header records (section 5):

  > The store in `<deployment>` does not end at the commit its header
  > records, so a file was lost, cut or restored apart from the others,
  > and this start has changed nothing in it. Restore `<deployment>` whole
  > from one backup, or, to start afresh, stop the service, move `active`,
  > `generations/` and `transcripts/` out of `<deployment>` together, and
  > start again: nothing is deleted, and the next start makes a new,
  > empty store.

- **`OpenRefusal::Failed`** stops an open when a file fails or a store
  cannot recover (`opening.md`):

  > A file in `<deployment>` could not be read or written, or its store
  > could not be recovered, while this start opened it, and the start
  > stopped there. Check the disk's space and the directory's permissions,
  > and start again. If the next start fails the same way, restore
  > `<deployment>` from a backup.

- **Each `Older` names its own layout's fix,** in a sentence its
  refusal's text carries whole:
  - `Directory`: "To start afresh, stop the service, move `store/`,
    `secrets/` and `transcripts/` out of `<deployment>` together, and
    start again: nothing is deleted, and the next start makes a new,
    empty store."
  - `Unrecorded`, `Layout` and `Unleveled`: "To start afresh, stop the
    service, move `active`, `generations/` and `transcripts/` out of
    `<deployment>` together, and start again: nothing is deleted, and
    the next start makes a new, empty store."
- **A backup** copies, with the engine stopped, the deployment's
  directory whole. Online, which stays open (section 11),
  it would copy `active`, then the generation `active` names, then
  `transcripts/`, in that order, and rest on recovery's rule: no switch
  happens while the engine serves (patching is open's, `opening.md`, 2), and a
  transcript's frames past the committed length the copied map names
  are cut at the next open (`transcripts.md`, 4). The previous generation needs no
  copy for a backup of the deployment as it runs.
- **Opened before the root starts** (`opening.md`; `../service.md`,
  section 3).

## 3. The boundary

What the component takes and answers, as the engine's protocol layer
routes it. The core's side is `../../domain/engine/README.md`, section 5 and `../../domain/engine/restart.md`;
the root wraps it, and the protocol layer's routing unwraps it, encoding
the application's part with the application's codecs.

| Down, to the component | Up, from it |
|---|---|
| **commit** n: jig's writes, typed (the core's saves and erases of its records and its children's, a copy of the bytes under one of its keys to another, and a task's transcript dropped whole); the application's writes, each a key's bytes and a value's bytes, or an erase | **committed** n, with the store's fill (6.2); **failed** n; or **FrameTooLarge** n, with task, full encoded bytes and page limit (section 5) |
| **restore** a restart step's range: jig's (the core's live families, `loads.md`, 2), or the application's, as byte intervals under its tag | jig's records, typed, then done; the application's rows as bytes, then done |
| **load** a page of a child's range: jig's, or the application's intervals | the page's rows, typed or as bytes, and a cursor if more remain |
| **read** a party's read: its scope, position, direction, rows and bytes (`../../domain/engine/README.md`, 5.3) | the page, each row with its joins, and its cursor; or busy, or gone |
| **get** a record by key: jig's or the application's | the record, or none |
| **transcript**: a task's current window, from the turn the core names as its start; a turn by its task, attempt and number; a page of a task's turns from either end | the turns, with their fields and bytes, and a cursor; or unread |
| **secrets**: keep, replace, erase, get and load, by owner and number; carry a value under a drawn number; adopt a carried value into its source's row (`secrets.md`) | kept, stale, erased, the value or none, a page, or the carried number and digest |

- **Turns are records of the core's** (`../../domain/engine/README.md`, 5.4): the
  core saves each accepted turn, keyed by its task, attempt and number,
  with its spend, the last message it read, its time, whether it opens a
  window, and its bytes. The component keeps the bytes and those fields
  in the task's transcript file, and the transcript's places in the map
  (`transcripts.md`). The core never erases a single turn: a task's transcript
  goes whole, by retention (6.1).
- **Reads see what is durable.** A load, a read, a get or a transcript
  sees every commit answered before it was asked, and nothing that is
  not durable (skein's `data/kv.md`, section 2). Each load of the core's is
  held until every commit made before it is answered
  (`../../domain/engine/README.md`, 4.2 and 5.3), so it sees what the decisions
  before it wrote.
- **One terminal per request,** tagged with the asker's token
  (skein/docs/foundation/programming-model.md, 4.4).

## 4. Keys

### 4.1 The encoding

- **Bytes sort as the domain's keys do.** Every key is written with
  skein-kv's key writer (skein's `data/kv.md`, 2.1) so that byte order equals
  the order Rust derives for it: a variant's index as a tag, then its
  fields in order; numbers big-endian at fixed width, narrower ones
  widened; an absent option before a present one; a byte string escaped
  and terminated, so a shorter one sorts before its extensions; a list of
  byte strings as each string marked, then an end mark.
- **Keys never hold payload:** numbers, small fixed identities, and
  names within their limits.
- **Written out, not generated.** skein's generator writes values, whose
  length-prefixed bytes do not sort (skein's `data/codec/README.md`, section 3), so
  each family's key has a small encoder and decoder of its own, beside
  its codec. A property test holds each encoder to the derived order on
  generated keys, and each decoder to the encoder.

### 4.2 Tags

The first byte of every key names its owner:

| Tag | Owner | Then |
|---|---|---|
| 0 | the component's own rows | its family: the format row (`records.md`, 3), transcripts' places (`transcripts.md`, 1) |
| 1 | jig | the child's tag, in the order of the core's key: the core's own, then each child's; then the family's key; and one row of the component's, the installation's identifier, at its reserved key (4.4) |
| 2 | the application | its own key bytes, which it encodes by the same rule, so that bytes sort as its keys do |

- **The application's bytes stay under its tag.** The component prefixes
  them, and refuses an application's key or interval it would have to
  write outside it, which is a bug, as a contract break.
- **A child's keys are one interval,** and each family one interval
  within it, since the encoding follows the keys' nesting.

### 4.3 Ranges to intervals

The domain asks for ranges as predicates over its keys; the store reads
intervals. So:

- **each range maps to a short, fixed, ordered list of intervals,** in the
  component, and a page that exhausts one interval with room left goes on
  into the next in the same step, since skein-kv answers loads at once;
- **families loaded together are adjacent:** within each child, the
  families a restart loads come first, and history after them
  (`../../domain/engine/README.md`, 5.5), so each child's live part is one interval;
- **a test checks each mapping** against the range's predicate over
  generated keys: every key the predicate admits lies in an interval, and
  every key in an interval is admitted; and a restart's key's place
  (`loads.md`, 2), the domain's over its typed key and the component's over its
  bytes, names the interval that holds it, over a request of several
  ranges as over one.

### 4.4 jig's families

What the store keeps of jig, by owner. "At start" families are read by
the restart (`opening.md`), the others by a load, a read or a get when
wanted. The families' keys are the children's; the order within each
child puts live first.

| Owner | Family | Key | Loaded |
|---|---|---|---|
| component | the format row | none: one row | at open |
| component | the installation's identifier, at its reserved key in the accounts' family (01 04 03): drawn by the shell at each start (`../configuration.md`, section 7) and kept by the store's first commit, beside the format row (`records.md`, 3), so that each later start's draw is ignored | none: one row | at open, handed up as the store opens, never by the restart's ranges (`loads.md`, 2) |
| component | transcripts' places: each task's committed length and its current window's start; each attempt's first turn | task; task, attempt | at open, the run proofs' tasks' lengths (`transcripts.md`, 4); the rest when read |
| core | the deployment: its id, its numbers (the next project's among them, past every project seeded or made), the last commit | none: one row | at start |
| core | settings (`../../domain/engine/settings.md`, 7): the defaults the newest revision was made with, a record per setting; the changes over them, a record per setting changed (its value whole, or its removal, with the revision that made it); the held revisions' records; the charter versions held revisions read and the newest does not; pending proposals' sets; of an application's kinds, each setting's state and size and each item's edit and size, never the value, which its connector's family keeps (`../../domain/engine/settings.md`, 5) | under the core's family tag 8: kind, then scope (0 for the deployment, a project's number), then number, for defaults and changes; revision; charter, then the revision that made it; proposal | at start, after the deployment's row, before every other family of the core's |
| core | settings' history: the records of revisions no longer held, each with its number, the revision it was made against, when, who decided it, the proposal it came from, whether it moved running work, and its items with the values they set and what each replaced (`../../domain/engine/settings.md`, 10), an application's with their sizes alone (`../../domain/engine/settings.md`, 5); within the history limit, the oldest erased by the decision that makes a newer one | under the core's family tag 9: revision | on demand |
| core | goals' projections (feed, plan rows, pending milestones), run proofs, calls committed since a live task's last turn, live agent tasks' conversations (their first attempt, their current window's start) | a goal's, then its own; task; task, attempt, completion, position; task | at start |
| core | turns (in transcripts' files, `transcripts.md`), terminals, decisions on proposals and escalations | task, attempt, turn; task, attempt; proposer, proposal; task, revision | on demand |
| tasks | live tasks, each with its revision and its claim's (`../../domain/engine/settings.md`, 3), stubs naming ended results, the funders' live periods, writer slots, pools' slots, pending proposals | task; task; funder; slot; proposal | at start |
| tasks | placed holds (`../../domain/tasks/lifecycle.md`, 10): each one's owner, mode, scope, reason, what lifts it and its subscription, its revision and the tasks it exempts; a lifted one kept while a task it held is still held by it, erased with the last one's release | hold | at start, with the live tasks |
| tasks | ended tasks, past periods, decided proposals, history, milestones, activity | task, then a position or revision; funder, period; proposal | on demand |
| people | parties, sign-ins (no secret), roles, projects' names, whether each is public (`../../domain/people/public-reading.md`, 2), and whole policies (their ceilings, escalation roles and overrides of templates among them), seeded and made alike, read positions over results and over live conversations, answered keys within the retention, a party's or, for the deployment's acts, a connector's (`../../domain/people/acts.md`, 3), including typed policy-ceiling refusals (holder, dimension and optional scalar bound and requested values), program declarations and checks, their declared names, and sign-out/removal answers naming unconfirmed configured workers with their original failure reasons; each project being deleted, with the step its deletion has reached and the grace it gives its conversations (`../../domain/people/acts.md`, 2), at most one per project the deployment has room for, erased with the deletion's last step; each deleted project's mark, its number alone, kept past its deletion, at most as many as the projects the deployment has room for, the lowest number's erased to make room (`../../domain/people/acts.md`, 2, *Marked deleted*) | as the people child keys them; a live conversation's position by party, then task; a deletion by its project; a mark by its project | at start |
| people | ended conversations' read positions (`loads.md`, 4) | task, then party | at start, by the census, every row (`loads.md`, 2); otherwise on demand |
| accounts | issued program names, including retired ones, with their source number and original holder and desired sign-out (key tag 4, after the installation's reserved tag); the next number the accounts child gives, below which it gives none again; LLM accounts: each one's holder, LLM provider, subject and display name, its uses, and what its last list reads gave, where it has any: the catalogue's numbers as a bit each, when its last good read was, when its last attempt ended, why that failed, and how many names took no number (`../../domain/accounts.md`, 3.7); their sources, each at its place in its holder's order, with its kind, its name for a program's, and its hold: limited, with its reset as a wall time where it has one, refused, or moving (a signed-in source whose record is out with a move, or back and not yet refreshed: `../../domain/accounts.md`, section 6). The rest of a source's state is heard again at each start, and none is kept per host (`../../domain/accounts.md`, 3.5); what each account's LLM provider says of it, a record per account of at most 24 facts (key tag 5, after the issued names; `../../domain/accounts.md`, 3.7) | source for an issued name; none: one row; account; account, source; account for its facts | at start, issued names before live account/source records, an account's facts after them |
| notes | entries, and an index line per entry, under its scope | name; scope, name | when a scope is in use; entries on demand |
| tasks | the results' index, the inbox's census's (`loads.md`, 2): a row per result of a task a party requested, at its position among that party's results, past the tasks child's live families | party, position | at start, by the census, past each party's read position over results |
| tasks, people | indexes for parties' reads (`loads.md`, 4): each party's inbox by time; ended tasks by requester party and by project; ended delegates; a subtree's delegates | the index's own, newest first where it reads so: an ended index's by party or project, then label, ended at reversed and task | on demand |

Children that keep nothing durable (the views, the fleet, authority,
briefs) have no family. What a run reports as it goes (its text, its
activity) is not stored: each committed turn holds what the run said
and did. Nor is a program's login on a host, which its host reports
(`../llm.md`, 5.5), or a sign-in under way, which a restart ends
(`../../domain/accounts.md`, section 5). An account's values are in the
secrets' store (`secrets.md`).

## 5. Commits

- **Numbers are shared.** The journal numbers the root's commits, and
  skein-kv numbers its own from one, across restarts. Every root commit
  is one skein-kv commit, and nothing else writes the store but
  patching, before the root starts, whose last commit writes the header
  with its own number (`opening.md`, 2), so the numbers are equal whenever the
  root runs. The component checks this as it submits, and stops on a
  mismatch. At start, skein-kv's recovered last commit must equal
  the commit the deployment's header names, which every writing decision
  saves; a store that disagrees refuses the start.
- **Writes become operations** in order: a save a put of its encoded key
  and record, an erase an erase, a copy a put under its second key of the
  bytes the map holds under its first, the application's bytes under its
  tag. The store's writes name no domain's operation: which key a copy
  reads and which it writes are the domain's (the core's for a settings
  revision that enters history). A copy reads the map as its commit found
  it, so a save of its first key earlier in that commit is not what it
  copies; a first key the map doesn't hold fails the commit.
  A key written more than once in one commit becomes one operation, its
  last write, which leaves the map as the writes in order would:
  skein-kv applies a commit whole (skein's `data/kv.md`, section 7), so
  nothing sees the writes between.
  A turn's save becomes a frame in its task's file, written first
  (`transcripts.md`, 2), and puts of its place in the map.
- **In order, whatever waits.** A commit with turns goes to skein-kv once
  their files are synced, and the commits after it queue behind it, so
  numbers reach skein-kv in order. The wait is one sync, and only for the
  commits behind a turn.
- **Group commit.** While a sync is in flight, commits queue, and the next
  write carries them all (skein's `data/kv.md`, section 4). The root's bound
  on commits in flight is checked at start against skein-kv's queue and
  queued bytes, and the component's queue behind transcripts' syncs, so
  the store never answers a commit busy.
- **Committed n** goes up when skein-kv answers that commit, which makes
  every earlier one durable too, with the store's fill (6.2).
- **A failure stops the engine.** skein-kv answers failed for every commit
  in flight when a write or a sync fails or stalls, and a failed write or
  sync of a transcript fails its commit the same way. The component
  answers failed, the root stops (`../../domain/engine/README.md`, 5.1), and the
  process ends; the next opens the store, which recovers. The engine
  never carries on over a store that recovered beneath it.
- **A refusal is a failure.** Past the checks at start, a commit skein-kv
  refuses (too large, the map full) would be a decision already made and
  not kept, so it stops the engine as a failure does. Section 6 keeps
  the map from filling.
- **An oversized transcript frame refuses the whole commit before data
  IO.** Each actual encoded current frame, including all metadata, payload
  and its wrapper, must fit the store's page byte bound `P`; equality is
  admitted. All frames and map operations are prepared before any of that
  commit's data is written, so a later oversized turn leaves none of the
  refused batch on disk. Its one terminal is
  `FrameTooLarge { number, task, bytes, limit }`, where `bytes` is the full
  encoded frame length and `limit` is `P`. There is no additional `Failed`
  for that number. This terminal comes first, followed by the
  failures of earlier pending commits, and the component stops as for an
  ordinary commit failure. Previously durable data remains recoverable.
  The event is strictly in-process: no journal record, stored family or
  wire message carries it. An application feeds the rejected number to its
  uncommitted-commit policy and may report the named cause.
- **A commit's operations are bounded,** as skein-kv applies each commit
  in one step (skein's `data/kv.md`, section 7). Where removing a record would
  erase an unbounded number of rows (a transcript's attempts' places,
  `transcripts.md`, 5), its commit erases the row that names the rest, so nothing reads
  them from then on, and the rest, orphans, go in the commits that
  follow: the component adds a bounded number of their erases to each
  commit it makes for the root, within that commit's operations, until
  none is left. At open it finds any a restart left by the rows they
  name being gone. Every commit stays one decision's, numbered as the
  root numbers it.

## 6. Growth

### 6.1 Retention

Retention is the domain's: mechanisms on its timers, tested in its
worlds, with horizons the deployment's configuration gives. What becomes
of an ended task is its retention class, set from its template when it
was made (`../../domain/tasks/README.md`, section 3):

- **A task kept** stays to the deployment's retention horizon, and whole
  while none is set.
- **Ended tasks shrink** past a horizon: one commit erases a task's full
  records (its record, history, activity, terminals, its ended read
  position) and saves a summary, so what parties read of old work comes
  from summaries. Its rows in the ended indexes stay, and a task query
  joins its summary (`loads.md`, 3).
- **Transcripts go** past a longer horizon, whole, by their task (`transcripts.md`, 5).
  A live task keeps its transcript, whatever its age.
- **Answered keys expire** past the people child's retention.
- **A task forgotten once read** (`../../domain/people/README.md`, section 5) goes
  in one decision: its record, its rows in the ended indexes, its
  result's update and its ended read position (`loads.md`, 4).
- **A forgotten task's remains go** at retention's next pass, whatever
  their age: its activity, history, terminals and transcript, and its
  replies' rows in its requester's updates (`loads.md`, 4). Nothing reads them,
  and the requester's pages of updates skip those rows until then.
- **A deleted project's tasks go whole** (`../../domain/people/acts.md`, 2),
  whatever their retention class and age, once every task of the
  project has ended: each task in decisions of its own, one task at a
  time, its rows first, a page at a time (its history, activity and
  terminals, with its replies' rows in its requester's updates), then
  its transcript dropped (`transcripts.md`, 5) with its rows in the ended indexes and
  its ended read position, and its record last, so that a deletion a
  restart cut finds the task's record and erases what is left of it.
  No summary is saved.

### 6.2 Fill

- **The store reports its fill** with each commit's answer: the map's
  bytes against its budget (skein's `data/kv.md`, section 7).
- **Above a high-water mark** of that budget (both are limits,
  `../limits.md`), the domain refuses new work at its
  entrances: new tasks, new sign-ins. Work under way goes on,
  and a refusal is never a failed commit. The deployment's document
  shows the fill.
- **Transcripts' disk** is the deployment's: retention bounds it, and its
  directory's free space is the operator's.

### 6.3 The census

What the store holds and what its log writes, per keyspace, so that a
deployment's owner sees where the store grows and what an upgrade would
walk (temper's `draft/store-shape.md`, 8). skein-kv counts (skein's
`data/kv.md`, 2.2); the component names the keyspaces and reports the
count up, and nothing of it is stored.

- **The rule:** the component gives every skein-kv store it opens for
  the map (the active generation's and a patched copy's, never the
  secrets' store) the census rule of its tags (4.2): a key under tag 0
  counts by its first two bytes, the component's family; under tag 1 by
  three, jig's child and family (`01 01 0a`, the tasks child's
  milestones); under tag 2 by two, the application's first key tag
  (`02 0c`); any other first byte by one.
- **Names,** `OWNER.FAMILY`, ASCII lower-case letters, digits, `.` and
  `-`, at most 48 bytes: `component.format`, `tasks.milestones`,
  `core.terminals`, `temper.changes`. The component's and jig's come
  from a table beside the component's key encoders (`keys`), one name per
  tag they write; the application's from a table it hands the component
  with its patch sets, its own name and a name per first key tag it
  writes (`Store::keyspaces`). A prefix no table names has an empty name,
  and readers show its bytes. A test holds each table to its key
  writers: every tag written has one name, and no two names repeat.
- **The report,** `StoreCensus` (the core's vocabulary, beside
  `StoreFill`): the last commit counted (`commit`), the commit the log's
  period began at (`since`, skein's `data/kv.md`, 2.2), and a row per
  keyspace in prefix order, each its prefix, its name, its rows, keys'
  and values' bytes, its heads with their rows, and its puts, erases and
  bytes written since `since`. The component sends it up as
  `Event::Census`:
  - once the store has opened, after `Opened`: what recovery counted, a
    patched generation's after its patches;
  - then after a commit's answer, once `CENSUS_EVERY` (60 s of the
    component's clock) has passed since the last report, so never while
    no commit is answered.
  The fill goes on with every commit's answer (6.2).
- **Up to the core:** the root hands each report to the core as the
  core's `Event::Census`, keeping the last one while it starts as it
  keeps the fill; the core keeps the last report and marks the
  deployment's document, which carries it
  (`../../client/domain/contract/documents.md`, 1). So a deployment
  document's watchers hear of the census at most once a minute, and a
  reader sees it as of its `commit`.
- **Its cost** is skein-kv's: a search of at most 128 keyspaces per
  operation applied, and a census of at most 129 rows, the empty
  prefix's included, of at most 8 heads each; the component's report
  copies it once a minute.

## 7. The application's part

- **Its families,** under its tag (4.2), keyed by its own encoder by the
  same rule, encoded by its own codecs, with its store version (`records.md`, 2).
- **Its restart ranges** as byte intervals under its tag, each live
  interval of each connector, served in the restart's step for that
  connector (`../../domain/engine/restart.md`).
- **Its loads and gets** by byte intervals and keys, answered with bytes.
- **Its indexes,** as records of its own, by the same rule (`loads.md`, 4).
- **Its secrets,** under its tag in the secrets' store (`secrets.md`).
- **What it writes is in the same commits** as jig's: one decision, one
  commit, whoever's records it holds.

## 8. Limits and the worst case

- **The component's limits:** both stores' (skein-kv's, their maps'
  budgets among them), the fill's high-water mark, the open files,
  the frames' largest bytes (a turn's, from the domain's limits), the
  queue of commits behind transcripts' syncs, the reads in flight and the
  joins a page may make.
- **Three file drivers,** one for each store (skein's `data/kv.md`, section
  12) and one for transcripts, so a turn's sync never queues behind a
  store's write, nor a store's behind a turn's.
- **The worst case** is both stores' (skein's `data/kv.md`, section 7: each
  map's budget, its queued commits and its buffers, which follow its
  chunk target and the largest row it holds), the component's buffers
  and tables, and the three drivers' (skein's
  `runtime/io/files.md`). Patching's peak (`opening.md`, 2:
  one map open, one machine per kept family within its own map's record
  room, the walk's page and commit buffers at the larger of the two
  maps' bounds, since one walk serves both, the copy's chunk, and the
  closing walk's run proof decoded by the binary's codec beside its
  bytes) is the
  larger of it and serving's, since the root never serves while it
  patches. The sets are the shell's; their kept families determine the
  patch machines' capacities. Fixed scenarios check measured heap peaks
  against named ceilings (skein/docs/foundation/programming-model.md, 6.3).
  Serving's holds the frames' machines (`transcripts.md`, 6), at most two, each its
  `Patcher::workspace_bound`, and none for sets with no kept text: their
  bytes the frames' bounds and their nodes the most their kept texts
  allow (`transcripts.md`, 6). The tests price smith's real turn's machine against
  serving's (`tests/frame_memory.rs`), from a copy of smith's schema
  checked against smith-transcript's generated bounds.
- **Whole files:** the manifest (section 2) is read and written whole,
  and at its bounds, 64 kept families each named in 64 bytes, it is 5464
  bytes (88, and 84 for each family). The store's file driver's
  whole-file bound is at least that, checked at startup with the worst
  case, or the start is refused naming it; a test world's drivers are
  made with such a bound (skein-io's `file_layer::Limits`, its `whole`).
- **A page read whole:** a transcript frame of up to the page is read
  back in one read, so each file driver reads at least the page, checked
  at startup, or the start is refused naming it; a test world that
  raises its page raises its drivers' read with it.
- **Checked whole at startup** with the root's and every component's
  (`../limits.md`).

`jig_protocol_store::host` hosts the three file drivers around the store for any engine: the drivers, their kernel hand-offs and their adoption and close. An application's service routes its own keys and records to it and writes no driver of its own.

## 9. Testing

Each within jig's suites' budgets.

- **The real store in the domain worlds.** The worlds run the component
  over skein-kv's in-memory mode, in place of a fake of the map, so every
  story runs the real keys, codecs, intervals, paging and joins. No
  world fakes the store's component: the real one runs in each
  (skein/docs/foundation/testing-strategy.md, section 4).
- **The restart oracle** (`../../domain/engine/README.md`, section 9): every
  world story ends with a cold restart over its store that restores
  with every link made and reads back what it wrote. The cross-record
  rules a restore once checked are proved there, writer by writer,
  never re-judged at a start.
- **Faults come from the world.** The world holds commits between the
  root and the component, in order: it delays them to make the store
  slow, and fails one to stop the engine. A crash keeps what reached the
  store and loses what the world held.
- **Transcripts' files have a stand-in** in the domain worlds: an
  in-memory file system answering the component's file requests, with
  the same rule after a crash. A crash keeps both in-memory stores and
  the stand-in, and the next engine's component is made over them and
  opens as `opening.md` says.
- **The store's world,** `tests/store`: the component over skein-kv and
  files on skein's simulated, crashing disk, a scripted root committing,
  loading, reading and appending turns, crashed at every file operation
  and sync. Its referee: after recovery, the last acknowledged commit, or
  that and one whole uncertain one, never part of one; no turn readable
  past its commit and none missing before it; numbers that match the
  header; pages equal to a model's.
- **The core's world on the simulated disk,** for a decision's
  durability across families (`../../domain/engine/README.md`, 5.1 and `../../domain/engine/restart.md`): the
  core world's directory-backed store (`jig-test-store-files`), killed
  at a chosen file operation of a chosen commit, and the engine started
  again through its link pass. Its rows: a kill at each operation of a
  decision's commit (after each record's bytes, mid-frame, between
  families), for a decision that makes a task and the person task
  linking a person, and one that makes a person and a task linking
  it; a second kill at each operation of the restart that follows the
  first, and of the commits after it; a kill at each step of a
  snapshot with commits racing it; a kill with several commits in
  flight. Its referee: the start opens, never refused and never a
  panic, with the last acknowledged decision, or that and one whole
  uncertain one, never part of one.
- **Generations and patching,** in the store's world: a fixture owner's
  patch sets over a store at an older level, patched, checked, opened
  `pending`, switched only when its root asks, never before, and a
  root that ends before asking leaving the old generation active; a
  fixture family's flight range holding a record under a pending patch
  refused `InFlight`, naming it, before any write, and the same record
  under a patch of another family patched past it; each
  refusal before any
  write; a second start refused `Busy` while one holds the lock; the
  copy's secrets private; a cut at every write, sync and rename of the
  first start, the copy, the patch commits, the manifest, the switch
  and the drop, each
  run from its own start, the referee finding one whole generation
  active, the other strays removed, and its records the patched or the
  old, never a mix.
- **The transcripts' framing's golden bytes,** as skein-kv keeps its
  own (skein's `data/kv.md`, section 9): a task's file of frames made from
  fixed turns (a window turn, a turn at the frame's bound, two
  attempts), kept beside the component's tests and never regenerated
  by the test that reads it; the writer's file compared with it byte
  for byte, and the reader walking it from both ends to the same turns.
  A change to the framing (`transcripts.md`, 1: the frame's length at both ends and its
  checksum, around the body) fails it, naming the format: from
  the first release on, it needs an upgrade of its own before the
  records' patches (temper's `draft/upgrades.md`, 6.5). A change to the
  core's `TurnRecord` or smith's turn is the patches' and their goldens'
  instead.
- **Rehearsing and going back** (`opening.md`, 5), in the store's world over
  fixture sets: a rehearsal reporting each patch and the restores whole,
  `A` still active and its maps unchanged, and the next start removing
  the scratch; one meeting a refusal, reporting it in a start's words;
  going back after a patched start that failed before its first commit
  of its own, the next start opening `A` as it was; refused once the
  patched generation has committed, with no previous generation, and
  while another process holds the lock, each changing nothing.
- **The deployment's layout's golden bytes,** as the framing's: a first
  start's directory and a patched start's, kept as their listings (each
  name, each file's kind and mode) with `active`'s bytes (a generation's
  number in decimal, no leading zero, and a newline) and the manifest's
  record, never regenerated by the test that reads them. A change to a
  name, to `active`'s form or to where a generation keeps its maps fails
  it, naming the layout: from the first release a later build reads this
  layout as it finds it (temper's `draft/v0.1.0.md`, section 5). The
  manifest's fields are the component's family's, which patches carry.
- **Transcripts patched as read,** most of it at the lowest layer. Step
  tests in `jig-protocol-store` over `jig-test-patches-smith` (and a
  fixture body family in `jig-test-patches-jig`): frames written at each
  fixture version, read by every kind of `transcripts.md`, 3, each handing on the bytes
  its patches' goldens give (skein's `data/codec/command.md`, 3) and the file's
  bytes unchanged; bounds judged on patched sizes; each halt, a
  `NoSite` and a turn past its kept texts among them, an unread frame; a
  version before a machine's first or past its last unread; a site up
  and back on a read, a commit waiting behind it, and a stray `Site`
  refused in each phase; the frames' checks refusing at a serving open;
  no machine with sets that keep no text; each
  machine's worst case against its measured peak; over skein-io's fault
  points, a cut at each step of a read changes no file. Fuzzed: a frame's decode with
  machines, over random fixture frames. One story in the store's world:
  a fixture store behind, patched with a fresh-marked patch, its live
  windows closed in the copy, a cut at each closing commit run from its
  own start, and after the switch a window read patched and a closed one
  found closed, going back finding them as they were. The core's side
  (a closed window starting fresh with its tail, counted as a
  transition) is the core's tests' (`../../domain/engine/executors.md`, 2), in its
  world, whose store closes the window by the walk itself: smith's
  fixture turn kept at one text, and a patch marked fresh that changes
  no family. Generation, upgrade, check and frame scenarios run in the focused
  `jig-store-world::suite` binary, with short cases as data rows and a few
  cuts chosen for what a cut there can lose (`../../domain/testing.md`, 9). Their
  whole sweeps are under `fuzzy::` in that binary. The copy's
  mutation cuts belong to `fuzzy::patching::every_patch_write_cut_recovers_all_records_and_no_stray`;
  `fuzzy::upgrades::cuts_across_check_and_switch_recover_whole_maps` starts
  at the seal's sync and cuts every operation through the check and switch.
  Frame and mark scenarios kept above step tests prove durable files or
  proofs across restart; decoder-only duplicates stay in the step tests.
- **Step tests:** each key encoder against the derived order; each
  range's intervals against its predicate; each codec both ways, with
  golden bytes; refusals at open (format, deployment, sizes, numbers).
  Direct transcript pages either advance through a nonempty prefix or
  terminate `Unread` when no turn fits. Private phase tests refuse a
  closing run proof outside its decode limit, a mismatched key and proof,
  and a malformed run-proof key with `Check::Load` naming the offending
  key: the active generation and source stay unchanged, and the next
  start sweeps the abandoned copy. The closing-write fault and crash
  stories hold the switch's order.
- **Fuzz targets** for each family's decoder and a frame's.
- **On the real ring,** the reference binary (`../service.md`, section 7)
  killed around each phase of a turn's commit and started again. A killed
  process keeps the page cache, so this checks integration, not power
  loss (skein's `data/kv.md`, section 9).

## 10. What it owes and is owed

The domain owes 10.1 to 10.9; skein owes 10.10, and the application
15.11.

### 10.1 Live before history

In each child's keys: the tasks child's history and ended tasks after
its live families (stubs among these), decided proposals apart from
pending ones, and past funding periods apart from live ones (4.3);

### 10.2 The restart reads live work only

The inbox's census reads unread results from the results' index and
ended conversations from the family of ended read positions, not every
ended task, joining each to its record by a point read; the census hands
each ended read position up with its record, and the people child keeps
none (`loads.md`, 2; `../../domain/people/README.md`, section 5);

### 10.3 Indexes for parties' reads

Each party's inbox by time; ended tasks by requester party, and by
project for those flagged listed, each keyed by label and ended at
reversed, the flag read from the task, never from a template; each
task's ended delegates; a subtree's delegates for its history (`loads.md`, 3, 4
);

### 10.4 Turns as the store keeps them

Saved with whether each opens a window, never erased one by one, a
task's transcript dropped whole; a resume reads the current window, from
the turn the core keeps as its start (`transcripts.md`, 3; `../../domain/engine/README.md`, 5.4, and `../../domain/engine/executors.md`, 2
);

### 10.5 Fill

With each commit's answer, and new work refused above the high-water
mark (6.2);

### 10.6 Retention

By each task's retention class, summaries of ended tasks kept,
transcripts dropped, answered keys expired, and the remains of tasks
forgotten once read collected, each on the domain's timer (6.1); a task
forgotten once read erased in one decision (6.1; `../../domain/people/README.md`,
section 5);

### 10.7 Keys for single reads

A question's line and its answer's, an escalation's hold, and a task's
proposal's row in its proposer's history, each named by its position in
its task's record, as a reply's is, an answer's with the task whose
activity holds it, so a read finds it by key, never by a scan (`loads.md`, 3); and
each open question its task asked, by its number, answering party,
logical addressed party and question activity position
(`QuestionCredit`, `records.md`, 1), so a person's answer is credited to its asker,
a move carries the question to the asker's new requester under the same
number, and an answering task's end lifts its asker's keep with a notice
(`../../domain/tasks/lifecycle.md`, 8; `../../domain/people/README.md`, section 5);

### 10.8 Read positions over conversations

A live conversation's in the people child's family, loaded at start;
moved, by the decision that ends its task, to the family of ended read
positions, read on demand; erased with the task (`loads.md`, 4, `README.md`, 6.1).

### 10.9 The accounts' family

The next number; each account with its holder, LLM provider, subject,
display name and uses; each source with its place in its holder's order,
kind, name for a program's, and its hold, limited with its reset,
refused or moving; each account's facts, at most 24, a record apart that goes
with its account (`../../domain/accounts.md`, 3.7); restored at start by fit and
link, a set past its limit refusing the start and a link the start cannot make
a panic (`../../domain/accounts.md`, 3.6), and facts whose account is gone dropped
as they are read; a source's record or key named by the
source's number, and erased when the source goes (4.4, `secrets.md`;
`../../domain/accounts.md`, 3.5). The installation's identifier is the
component's (`README.md`, 4.4, `records.md`, 3).

### 10.10 What skein owes

io's `OpenWrite` and `Cut` (skein's `runtime/io/files.md`), for transcripts.md, 2 and 4. skein-kv serves as it is: its in-memory mode, its key writer, its
fill. Its accounts driver takes the owner's store as keeper, made with
the records it holds (skein's `oauth/accounts.md`, 4).

### 10.11 What the application owes

Its keys' encoders, its records' codecs, its store version, its live
intervals, and its indexes, by the rules of section 7.

## 11. Open questions

- **Upgrades** are a pass at opening that patches a copy and switches
  to it (`README.md`, section 2, and `records.md`, section 3, and `opening.md`, section 1 to 4). Still open:
  - values nested in another owner's family as they run across owners'
    levels (skein's `data/codec/README.md`, section 10);
  - pieces whose items' conversion stops, a site or a held field in
    reach, are refused `Pieces` (skein's `data/codec/patches.md`, 3, and
    `data/codec/README.md`, 10): a patch of smith's `Hit` (`Found.hits`,
    `pieces Hit`) or of jig's own packed lists (`records.md`, 1) is
    opened piece by piece while their items convert without a site;
  - the core's records walked by a smith patch: the core's `Row` reaches
    `TurnRecord`, which holds smith's turn, so each smith patch runs the
    core's identity plan over every core record, though no row in the
    map holds a turn; a frames' root kept out of `Row`'s `Value` would
    spare that walk;
  - smith's machine is priced for the most nodes its kept turn allows
    (`transcripts.md`, 6): from the baseline on, about 249 MB per serving store
    (`Patcher::workspace_bound` at 1,054,764 nodes), though a typical
    turn takes a few hundred nodes. Its tables grow in bounded steps
    (skein's `data/codec/patches.md`, 7), so a store holds what its largest turn
    needed, not the price; but it keeps that room until the machine
    goes, so one large turn keeps it large (skein's `data/codec/README.md`, 10,
    "Shrinking");
  - keys a patch moves (`opening.md`, 3), left to later work: a line for them
    in skein's patch text, and the key function's answer up and back;
  - the free space checked before the copy (`opening.md`, 2);
  - a gate holding each patch's `after` to at least the pin it lands
    on (`opening.md`, 3);
  - a copy made by the file system (a reflink, or btrfs's snapshot:
    skein's `runtime/io/files.md`) in place of one written file by file;
  - what a patch's working buffers pay (`opening.md`, 2): the copy reads and writes
    a file in pieces of skein-kv's `Limits::chunk`, the row ceiling, not
    its chunk target (skein's `data/kv.md`, 5 and 7), and the switch's digest
    check reads `active` into a buffer of the drivers' whole read, which
    that ceiling sets (`host::max_read`). So a patch pays about 60 MB on
    standard however small its rows; a deployment's budgets meet this
    (skein's `data/kv.md`, section 12).
- **The header by its own level's text.** A jig behind decodes the header
  by the binary's core text, which every store holds until jig's first patch
  moves the core; that patch brings the kept text's decode with it.
- **Backups:** copying the deployment's directory whole while the engine
  is stopped works. An online backup would copy `active`, the generation
  it names and `transcripts/`, in that order (section 2), and rest on
  recovery's rule, and on skein-kv keeping each segment the copy has
  yet to read.
- **Finding a turn faster:** a row per turn in the map, if walking an
  attempt's frames proves slow for parties' reads.
- **Values on disk,** with only keys in memory, if a deployment outgrows
  its map's budget after retention (skein's `data/kv.md`, section 12).
- **Search** over tasks, notes and transcripts: an index of the domain's,
  or a file-based one beside the store.
- **Several engines:** the store is one process's; the log of commits is
  what would be replicated.
- **One remap of a mask:** `jig-domain-authority`, `jig-domain-tasks` and
  `jig-domain-people` each own a `Models` with the same `remap` (`records.md`, 1), since
  step code in jig has no generics. Sharing them takes one type the three
  name.
