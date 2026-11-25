# Upgrades

Draft, 2026-10-23. How a deployment moves to a newer temper and keeps
its history: its people, projects and accounts, its tasks and goals,
live and ended, its landings and what agents wrote. It comes from the
investigation of 2026-10-23 (Codex, read-only) and a session with the
user that day, whose turns shaped it: the records before the formats on
disk, and changes carried with the build, as an application migrates
its database. It crosses skein, smith, jig and temper on purpose; an
owner splits it. The oldest store a build opens is dev.fabrig.ai's,
settled at the floor (`archive/store-floor.md`, section 3; 4.1 below).
Commands' spellings are placeholders.

## 1. In one page

- **The registry and jobs** (section 4). A registered family's schema
  changes in place: an added field takes a default (`= value`), a
  renamed part its former name (`was`), and a change no plan derives
  brings its owner's written job. No version or level moves and no
  map patch is kept.
- **The records first, the formats on disk later.** As PostgreSQL's own
  upgrade between versions of its files (`pg_upgrade`) is apart from an
  application's migrations, an upgrade here has two layers: the formats
  on disk, skein-kv's frames and snapshot and the transcripts' framing,
  and jig's and temper's records in them. This draft is the second; the
  first is left as it is (6.5).
- **Cheap, because schemas are read at run time** (section 5).
  The store holds the texts of the layouts its values use in its
  registry. Plans derived from those texts convert each value as it is
  read, without a rewrite at open. Current writers write the build's
  layout.
- **The floor is dev.fabrig.ai's settled store** (4.1). A store without
  the floor or below it is refused before any upgrade is planned or
  written, naming bridge build `3d1c717d379e` to open it through.
- **A copy is upgraded, never the store in use** (section 6): when a
  layout needs registering or a job is pending, a start makes a new
  generation of the records' and the secrets' maps, registers the
  layouts, runs the jobs, opens the copy as any start does, and switches
  to it by one atomic replacement. The previous generation is kept,
  and going back is supported until the new one's first commit of its
  own (3.2).
- **Transcripts are never rewritten** (section 7): their frames are
  patched as they are read. An agent resumes its conversation across
  a compatible change; a breaking change of the turns' wire follows
  `archive/redeploys.md`'s bridges and breaking declarations.
- **Every stop settles effects first** (section 8). Their identities,
  keyed outcomes and recovery survive the store's changes.
- **The engine alone may be upgraded while workers keep their runs**
  (section 9), when their wire is compatible and the window fits their
  restart wait or grace.
- **The configuration is never rewritten** (section 10): each release
  reads its own members and the previous release's.
- **The gate holds the policy** (section 12): registered changes use
  defaults, former names or jobs, not repository-kept store history.
  The floor's fixture opens and serves; an older store is refused.
  Wires keep their patches and bridges, and formats outside the
  registry stay fixed at their versions.

## 2. What an upgrade carries

| Part | Where | Its formats' owners | Across an upgrade |
|---|---|---|---|
| the records | the deployment's `store/` | jig's component and families; temper's families | layouts registered and jobs run in a copy; values converted as read (section 6) |
| the secrets | the deployment's `secrets/` | jig's records; temper's forge records; skein's OAuth records | copied at their fixed versions, every number kept (6.3) |
| the transcripts | the deployment's `transcripts/` | jig's frames, smith's turns in them | kept as written, shared by the generations, patched as read (section 7) |
| the configuration | the configuration directory: `temper.json`, `agent.json`, charters' instructions | the operator's | read as written (section 10) |
| small files | the data directory: each forge's setup `state`, the box's records, the demo's, a worker's secret and digest | temper | read as section 10 says |
| an owned Forgejo | its forge's directory | Forgejo, and temper's preparation | its own upgrade, at its preparation (section 11) |
| program logins | each worker's programs directory | each program's | kept as they are; a login a program no longer reads is signed in again |
| a worker's root | each worker's data directory | temper | rebuilt at will: checkouts and workspaces |
| what temper wrote on a forge | the forge | temper's connector | read in every version (section 11) |
| a client's kept requests | the person's browser or shell | jig's client | as jig's client says (7.4) |
| the cache | the cache directory | — | never carried |

The directories are `../protocol/configuration/README.md`'s (`../protocol/configuration/directories.md`); what
the store holds is jig's `protocol/store/README.md` (4.4) and
`../protocol/store.md`'s (`../protocol/store.md`, section 2).

**What must keep its identity:** the deployment's id, which names its
branches and keys' markers on the forge (`../protocol/forge/README.md`, section
3); the installation's id; every number the domain gives (tasks,
attempts, messages, sources, accounts, issued program names), so that
none is given twice; and the commit position, which the deployment's
header and projections name, and which the upgrade carries on through
ordinary commits (6.2). A worker holds no authoritative state of its own: what it
acknowledged is in the engine or on the forge.

## 3. At start

### 3.1 Every start

- **It reads the levels and the floor first.** jig's store, as it
  opens (jig's `protocol/store/opening.md`, 1), compares the format
  row with the binary's levels and versions (4.2), then checks the
  floor before reading the registry or planning any upgrade (4.1).
- **At the binary's levels,** it reads the registry and plans every
  layout the store holds to the binary's. A new layout to register
  or a pending job upgrades a copy (section 6), printing a line per
  job with its counts. Otherwise it opens in place, converting values
  as they are read (jig's `protocol/store/layouts.md`, 4 and 5).
- **Every shape with a store:** `temper serve`, an engine alone and
  `temper box` (`../protocol/cluster.md`). A worker alone
  keeps no store; its small files are read as section 10 says.
- **Refused before any upgrade is written,** naming why (jig's
  `protocol/store/opening.md`, 1.8):
  - a store ahead of the binary: nothing runs backwards;
  - a store without the floor or below it, behind the binary's
    levels, or holding values without a registry: start bridge build
    `3d1c717d379e` on it and let it open, then this build;
  - a store from before generations, with an older format, an
    undecodable component row or no levels: refused by name, naming
    `3d1c717d379e` too, but not promising it translates forms it also
    refuses (4.1);
  - a layout a value may still hold with no derived plan or written
    job, naming its family, value type, layout and why.
  Opening the maps may already have run skein-kv's recovery; no
  refusal makes a generation or seeds a registry.
- **A job that refuses a record, or a copy that does not open** (6.4),
  stops the start with the old generation active, saying why, and exits
  non-zero; the next start begins again from a fresh copy.
- **Rehearsed:** `temper upgrade --check`, by the new binary, on a copy
  such as last night's backup, works in a scratch directory and
  switches nothing. After the same floor and format checks, it decodes
  every record of every family before planning: registered values by
  their layout's text in the store, other records by their current
  format, and frames by the frames' texts. It reports the first that
  does not decode, with its key or transcript and offset, or the count
  decoded; the jobs and their counts and time, the whole time, a
  refusal or the restores whole, and whether the workers may stay
  (section 9). What follows the switch is not rehearsed (jig's
  `protocol/store/opening-rehearsal.md`, 1). Its measurements tell the operator
  whether the service manager's start timeout covers the upgrade.

### 3.2 Going back

- **Before the new generation's first commit of its own:** `temper
  upgrade --back` makes the previous generation active again, and the old
  binary starts on it. Nothing has happened since the stop, so nothing is
  lost or made twice. This is for a new binary that upgraded and then
  failed to start. The manifest records the commit position at the end
  of the restores, before the switch; the generation's own first
  commit moves past it: every start
  commits, the wire versions it speaks first (section 9), and every
  effect is committed before it is made.
- **Transcripts are shared** (section 6): a frame the new version wrote
  past the old generation's committed lengths, never committed, is cut
  at the old one's open, as after a crash (jig's `protocol/store/transcripts.md`,
  4).
- **After that commit,** going back is restoring a backup. The new
  version may have acted on the forge, and the old one, restored, may act
  again; an owned Forgejo migrated since needs its dump restored with it
  (`../protocol/setup/owned.md`, 3). The tool refuses `--back` and says so.

### 3.3 Backups

A backup is a copy of the data directory with the deployment stopped
(jig's `protocol/store/README.md`, section 11; `../protocol/setup/README.md`, section
10). The previous generation is a copy of the maps alone, not of the
transcripts, the forge or the programs' logins. Online backups stay open
(section 15).

## 4. Layout changes

### 4.1 The floor

The floor is dev.fabrig.ai's store as store-floor's F0 settled it
(`archive/store-floor.md`, section 3): the oldest store a build opens. The
store's floor row carries its number and the build that wrote it;
its first commit writes it, and a reopen leaves it alone (jig's
`protocol/store/records.md`, 3). A store without that row or below the
build's floor is refused before planning, registration, jobs or the
rehearsal's decoding, naming bridge build `3d1c717d379e`: start that
build on it and let it open, then this one. A store behind the binary's
levels or holding values without a registry gets the same way forward.

Nothing keeps code, texts or fixtures to upgrade stores below the
floor. The floor moves only when a landing drops a whole older form;
that landing designs how its upgrade writes the new floor and names
its bridge. After dev.fabrig.ai runs a job, a small landing drops its
code; its done row stays inert and its name is never reused
(`archive/store-floor.md`, section 6; jig's `protocol/store/jobs.md`, 1).

A store from before generations, with an older format, an undecodable
component row or no levels is refused by name too. Its refusal names
`3d1c717d379e` without claiming that build translates it: those forms
are refused there too (jig's `protocol/store/opening.md`, 1.8).
Starting afresh is the operator's choice, moving the store aside, not
an upgrade deleting it.

The wires keep their own rule: a changed layout moves its version and
brings its bridge, or is declared per-build in the bridge inventory
(`archive/redeploys.md`, sections 3 and 4.2). A format outside the registry
stays fixed at its version; a change designs and brings, in the same
landing, how dev.fabrig.ai's store gets there (`archive/store-floor.md`, 5).

### 4.2 Levels

- **Each owner holds its level explicitly:** jig 6 and temper 14.
  These are comparisons at open, not lengths of patch arrays and not
  a sequence of pending work. A store behind them is refused (4.1).
- **The format row** names the levels beside the families' versions
  (jig's `protocol/store/records.md`, 3), written by the first commit
  and by an upgrade's last commit. Neither a registry change nor a
  job moves them.
- **Registered families keep their versions.** A value's head names
  its value type's layout id in the store's registry; its texts and
  fingerprint say what it holds (jig's `protocol/store/layouts.md`,
  3 and 4). There is no landed store patch to renumber or keep.
- **Outside the registry,** the component's own rows, the secrets'
  store and the transcripts' frames stay at their current versions
  (4.1). Wires keep their version moves, patches and bridges.

### 4.3 What a change is

- **Derived:** a registered family's added field takes a default
  (`= value`); a renamed declaration or field names its former name
  (`was`). The open derives plans from the store's layout texts to
  the binary's text; each value is converted as it is read. No patch
  is written, no version moves and no eager walk rewrites the map
  (jig's `protocol/store/layouts.md`, 4 and 5).
- **Written:** a change no plan derives, a change of meaning, a field
  computed from others, or a change of keys is its owner's job at
  open (jig's `protocol/store/layouts.md`, 6; `jig/docs/design/protocol/store/jobs.md`,
  4). A written conversion receives the values at that layout as
  stored, with their texts from the registry, and puts them at the
  binary's layout. It keeps no old generated codec.
- **Wires:** operations over generic values (section 5), derived by
  the patch command or written where derivation cannot supply the
  meaning. They keep their texts, patches and bridges, not the store's
  jobs (`archive/redeploys.md`, 3 and 4).

What every change keeps:

- **No invention.** An added field's value must be true of every old
  record: a field old records hold no evidence for is an `option`, and
  none for them. Unknown is not zero: smith's usage became optional so
  that a turn with none is not one that cost nothing. A written job may
  refuse a record, naming it, which stops the start (3.1).
- **No number given twice.** Every number and counter is kept; the next
  number given after the upgrade is one no record holds.
- **No outcome repeated or forgotten.** Keyed requests' answers,
  connectors' keyed outcomes and decisions are kept, so a request sent
  again after the upgrade is answered as before.
- **Indexes follow their records:** a job that changes what an index
  derives from rebuilds it from its records (jig's `protocol/store/loads.md`,
  4).

### 4.4 Values nested in another owner's

- **Opaque bytes, said what they hold.** A record of another family is a
  `bytes N` field, decoded by its own family's code (skein's `data/codec/README.md`,
  section 3): temper keeps jig's refusals as `refusal: bytes 517`. Its
  schema says what such a field holds, another family's type or pieces of
  a type of its own, so conversion reaches it wherever it is held.
- **Part of the holder's layout.** Its fingerprint resolves each
  held record's layout by that family's text in the registry. A held record's
  change converts with its holder; the held record's head stays its
  family's version (jig's `protocol/store/layouts.md`, 1 and 3).
  There are no upstream patch levels to interleave. Jobs run in the
  sets' order, upstream first, and within each set its own (jig's
  `protocol/store/jobs.md`, 1).
- **A nested type that grows** widens its holder's bound in the landing
  that brings it. The holder's registered layout is planned as any
  other; it brings no store patch.

### 4.5 The tooling

- **For registered families,** generation writes the current schema's
  code and golden bytes. The open's layout derivation uses the store's
  texts; the author supplies defaults, former names or a written job
  (4.3). It keeps no schema history in the repository.
- **For wires,** `cargo run -p skein-codegen -- patch <crate>` writes
  the forward patch and its tests and keeps the version's text; their
  bridges follow `archive/redeploys.md`, section 3. Formats outside the
  registry are fixed at their versions (4.1).
- **The gate holds each rule** (section 12), not a pending map patch
  or a frozen registered text.
- **Its aim:** the common change, a field added, costs its author the
  field's default and a review of the current golden bytes: minutes.

## 5. Schemas at run time

- **Texts in the store.** The registry holds each layout a value may
  use, its text and its held texts. Only the current schema is
  generated into code; no repository-kept registered history is
  needed (jig's `protocol/store/layouts.md`, 3).
- **Generic values.** skein-codec reads a schema and decodes bytes of it
  into a tree of generic values (records, variants, integers, bytes,
  lists, options), and encodes a tree back, with the generated code's
  checks: each value within the schema's ceilings, text that is UTF-8,
  tags it knows, no bytes left over. A `secret` stays one, its bytes
  never printed.
- **Fingerprints of the layout:** a digest of what the bytes are (types,
  order, tags, bounds), not of names or comments. Former names let a plan match
  a renamed part. The registry names each value type's layouts by
  fingerprint, and each layout its texts by digest.
- **Plans** derive from the stored layout to the binary's. A layout
  with no plan needs a written job or refuses the open before any
  upgrade is written, naming what cannot convert (4.3), unless no
  value holds it and it retires.
- **Where it runs:** at open to plan the layouts; at the store's read
  edges to convert each value, and in a written job over the copy
  (section 6). Frames keep their read-time patcher (section 7).
  It is step code: a value's work is bounded by its schema's ceilings
  (`skein/docs/foundation/programming-model.md`, 6.3).
- **Retirement:** a layout no value holds retires at an open that
  writes, with each text no other row names. A log-held layout is
  not retired by the map's census (jig's `protocol/store/layouts.md`,
  5.4). Code for a job goes after dev.fabrig.ai has run it (4.1).
- **Small:** about 1,750 lines of jig's store schemas and 190 of
  temper's at 2026-10-23, in a language of a handful of types (skein's
  `data/codec/README.md`, section 3).

## 6. Upgrading a copy

```
<deployment>/
  active                   the active generation's number
  generations/
    7/  store/  secrets/  manifest
    8/  store/  secrets/  manifest
  transcripts/             shared by every generation
```

- **jig's deployment directory holds them** (jig's `protocol/store/README.md`,
  section 2): its open reads `active` and opens that generation's maps,
  and a first start makes generation 1. The generations keep that
  layout across upgrades.
- **The manifest** names the generation upgraded from, the levels before
  and after, each family's count before and after, and the commit
  position at the end of the restores, before the switch.

### 6.1 The steps

When a layout needs registering or a job is pending, from the active
`A` to `A + 1` (jig's `protocol/store/opening.md`, 2):

1. **Plan:** after the format and floor checks, the registry's held
   layouts to the binary's, the sets and bounds checked, and the
   pending jobs found. A refusal comes before any upgrade is written.
   `A` closes; a generation before `A` is dropped, since going back
   from the new one is to `A`.
2. **Link and copy** the active generation's `store/` and `secrets/`
   into the next one's, synced. Snapshots, older log segments and task
   logs may be linked; the newest segment, other map files and all
   secrets are copied. A file system that refuses hard links copies
   those files instead; other faults stop the start. Nothing writes
   a shared map file again; a task log is appended only past
   `A`'s extent, which `A` reads as garbage. Transcripts stay shared.
3. **Open the copy** as any start opens a store (6.2).
4. **Register and run jobs:** commit the binary's new layouts and the
   retirements; then run each pending job in order, in pages within
   the commit bounds. A job puts and erases a moved key together.
   Its last page commits its done row. No eager patch walk rewrites
   other values. The seal, the upgrade's last commit, writes the
   format row's levels and versions, the layouts and the deployment's
   header at that commit. It leaves the floor row alone (4.1).
5. **Open** and restore the copy as any start does (6.4), making no
   effect and writing no transcript before the switch.
6. **The manifest and switch:** once the restores are whole and their
   commits durable, write the manifest, then replace `active` by io's
   durable replace, expecting the digest read from `A` (skein's
   `runtime/io/README.md`). One rename, never two standing for one.
   The restart goes on over the copy, still open; its wires are its
   first commit after the switch, before anything leaves the process.

- **A crash before the switch** leaves the old generation active and
  untouched by the upgrade; the next start removes the partial one
  and begins again from a fresh copy. After the switch, the new one
  is whole. Map recovery at open is still skein-kv's (3.1).
- **Room:** on disk, what the generations do not share: the copied
  newest segment and secrets, and the registry's and jobs' commits.
  As the new generation snapshots and writes, the maps may stand
  twice; without hard links they do from the start. In memory, one
  map at a time, and a job's page and answer within its room (jig's
  `protocol/store/jobs.md`, 3), before the engine serves.

### 6.2 skein-kv as it is

- **The copy opens as any store does:** skein-kv recovers its snapshot
  and the log's longest valid prefix (skein's `data/kv.md`, section 3), and
  jig's store checks its format row and the deployment's header (jig's
  `protocol/store/opening.md`).
- **The registry and jobs use ordinary commits** through jig's store,
  each within skein-kv's bounds. The upgrade's last commit seals the
  header at its commit position (6.1). Numbering carries on from the
  old generation's, the header's check holds at the next open, and
  every reference to an earlier commit still names it.
- **Nothing new of skein-kv.**

### 6.3 Secrets

- **Copied whole,** in their own map (jig's `protocol/store/secrets.md`):
  jig's credentials' digests and LLM accounts' records, temper's forge
  records, and skein's OAuth records. These formats are outside the
  registry, fixed at their current versions (`archive/store-floor.md`, 5).
  A change brings how dev.fabrig.ai's store gets there in its landing,
  not a promise to read older stores.
- **Their numbers kept:** sources', accounts' and the carried values'
  counter, so that the start's sweep, which erases the secrets no record
  names, erases nothing it should keep.

### 6.4 Opened before the switch

The check opens the upgraded copy before the switch (jig's
`protocol/store/opening.md`, 2): the live ranges and the ended read
positions decode under the binary's codecs within the deployment's
limits; the secrets' files are private to the service's user. The
restart restores and links what it loaded before asking for the switch
(jig's `domain/engine/restart.md`). A link it cannot make refuses the
start, naming the record and what it links to; no effect leaves before
the switch. Nothing judges a writer's domain rules again.

The cross-record rules are oracles over every writer, a cold restart
closing every world story (jig's `domain/engine/world.md`, section 1).
Registry changes and jobs are proved over sample stores (13); the
counts per family, before and after, stay in the manifest.

### 6.5 The formats on disk

skein-kv's frames and snapshot, at version 1 since it was written (skein
288c5423), and the transcripts' framing, a frame's length at both ends
and its checksum (jig's `protocol/store/transcripts.md`, 1), are left as they are.
The gate fingerprints them as it does every kept format (`upgrades.md`, section 12), so
a change is seen. Changing one needs an upgrade of its own, a tool of
its owner's, as `pg_upgrade` is PostgreSQL's (section 15). The
transcripts' frame patcher stays; no map patch or fresh-window mark
is part of it (`archive/store-floor.md`, 5).

## 7. What survives

### 7.1 Records

Everything recorded: people, roles, projects and their policies,
sign-ins, LLM accounts and their sources, issued program names; tasks,
live and ended, with their activity, goals, proposals, decisions and
notes; requests' keys and answers within their retention; temper's
repositories adopted, its landings, its forges' records and keyed
outcomes; the deployment's and installation's ids.

### 7.2 Transcripts and agents

- **Patched as read.** A frame keeps the versions it was written in,
  jig's for its fields and smith's for its turn. jig's transcripts' reads
  (jig's `protocol/store/transcripts.md`, 3) apply the patches between to each
  frame's fields and turn, so the engine hands out every turn in the
  current version, to a client or a run. Old frames are never rewritten,
  so the places the map keeps for them stay true.
- **Agents resume** their windows across a change of smith's turns.
- **Every live task keeps its definition,** its inbox, its holds and its
  place in its queue.
- **Runs end with a full upgrade,** as at any stop: the workers stop
  first, their runs stopped once the engine has every turn and answer
  (`../protocol/cluster.md`, 4), and the tasks retry after the start.
  An upgrade of the engine alone keeps them (`upgrades.md`, section 9).

### 7.3 Sign-ins

- **People stay signed in:** a credential's digest is a record, kept.
- **LLM accounts stay signed in;** one expired or revoked since is
  signed in again, as at any start.
- **Program logins** are the programs' own, kept on their hosts.

### 7.4 Clients

- **Pages reload.** The bundle is the new binary's; a page of the old
  version learns from the stream's hello that its versions are not
  served, and holds its requests unsent until it reloads (jig's
  `client/domain/client/watches.md`, 2).
- **A kept request of another version is dropped,** and the person told
  (jig's `protocol/client/small-store.md`, section 1). Its outcome may have been
  committed before the stop, so the notice says it may have been made.
  Patching them too is open (`upgrades.md`, section 15).

### 7.5 What is lost

- What retention erased before the upgrade (jig's `protocol/store/growth.md`,
  1.1; temper's bounded `Landed`, `../protocol/store.md`, section 7). A
  deployment that keeps history long sets its retention first.
- A run's unsaved state on its worker, in a full upgrade, as at any
  stop.
- Nothing from a refused store is erased: a store below the floor
  stays as it was, to open through the named bridge or move aside
  before starting afresh (4.1).

## 8. Effects in flight

- **Every stop settles first.** Told to stop, before it stops taking
  commits (jig's `domain/engine/restart.md`, 5), the engine starts no effect,
  places no run and steps no procedure, answers busy a run's call that
  would start an effect, and commits the outcomes of the effects in
  flight within their deadlines. Then it stops as today. Past a bound, it
  stops unsettled, naming what remains. A second signal aborts the wait,
  as jig's service does (jig's `protocol/service.md`, section 5).
- **Every stop, not only an upgrade's,** since an operator does not
  always know beforehand, and the box's person never does. Its cost is a
  stop that waits up to an effect's deadline.
- **An unsettled store recovers as after any stop.** Its registered
  records pass through the same read-edge conversion, and a job keeps
  the in-flight effects recoverable, with their keyed outcomes and
  identities (4.3). There is no pending map patch or effect-family
  patch admission. The new version recovers each effect by its key
  (jig's `domain/engine/restart.md`, step 5), whose version the
  connector reads (section 11).

## 9. The engine alone

Replaced by `archive/redeploys.md` (the user, 2026-11-12): when the workers may
stay, judged by each wire's fingerprints and floor rather than its
number (`archive/redeploys.md`, 5.1); the settled stop and the window within
the workers' grace (5.2, 5.3); and the workers following in turn
(section 7). A breaking change of the turns' wire is declared there
(4.5); it is not a pending store patch or a fresh-window mark in the
store.

## 10. Configuration and small files

- **The configuration is never rewritten.** It is the operator's, which
  they may keep in their own tools. Each release reads its own members
  and the previous release's: a member added is optional, with a
  default; a member removed or renamed is accepted for one release with a
  warning naming its replacement, then refused as unknown, as today
  (`../protocol/configuration/README.md`, section 2). A deployment that skips a
  release may find a removed member refused, by name.
- **Settings in the store shrink it** (`configuring.md`, section 3):
  charters, templates, policies and rules in the store, where the
  registry and jobs carry their changes; the configuration keeps what
  a start fixes.
- **Small files outside the generation,** a forge's setup `state`, the
  box's records, the demo's, a worker's secret and digest: each carries
  its version, is read in every version it has, and is
  written in the current at its next write. None is written before the
  new generation's first commit, so going back (3.2) finds them as they
  were.

## 11. The forge

- **An owned Forgejo upgrades at its preparation,** as today
  (`../protocol/setup/owned.md`, 3): a dump before the migration, pins that
  refuse a downgrade. A temper release may move its pins; the box
  prepares at its start, a service deployment by `temper forge prepare`
  before the engine's start. Going back past a migration needs its dump.
- **What temper wrote on the forge outlives every version:** its keys'
  markers in what it creates, the key's first byte its version
  (`../protocol/forge/README.md`, section 3), and the branches in the
  deployment's namespace. The connector reads every version of a
  marker, and the deployment's id is kept, so the namespace and the
  markers stay its own.
- **The forge is no backup.** Its issues and pull requests cannot rebuild
  the engine's records; the generations are the record.

## 12. The policy and the gate

- **Registered changes use the registry or a job** (4.3), in the
  landing that changes the schema: an added field's default, a
  renamed part's former name, or the written conversion. Versions
  and levels stay. No map patch, old registered text or old generated
  codec is kept in the repository (`AGENTS.md`, 2).
- **The gate checks:**
  - current generated code and golden bytes; the registry's generic
    conversions and the jobs' outcomes in their owners' focused tests;
  - the floor's fixture opening and serving, and the named refusal
    of a store without the floor (section 13);
  - the wires' kept texts, patches and bridges, by their own rule;
    the boundary's patches check covers wires and formats outside the
    registry, not registered map families (`archive/store-floor.md`, 4.4);
  - the formats outside the registry, fixed at their current versions,
    and the formats on disk (6.5): a changed fingerprint fails unless
    its landing brings the designed way for dev's store to reach it.
- **Meaning is reviewed.** A change of what a value means may move no
  layout: its review, its written job and a scenario hold it.
- **The floor keeps the cost bounded:** there is no chain to squash,
  baseline to catch up or frozen store text to mutate. A job's code
  goes after dev.fabrig.ai runs it (`archive/store-floor.md`, 6); generic
  conversion at read stays for the next change.

## 13. Proving it

On the existing tiers (skein's `testing-strategy.md`, section 2;
`../testing/tiers.md`, section 1), storage mechanics tested once, below
the domains:

- **Step tests:** a registry change's default and former name read at
  the binary's layout, and a written job's conversion, refusal and
  done row. Generic values against the generated code: any value of a
  current schema decodes to the same tree by both, and encodes back to
  the same bytes. The frames' patcher keeps its own fixtures.
- **A fixture at the floor,** which opens and serves, its identities,
  counters, policies, outcomes and turns kept. A store without the
  floor is refused by name, naming bridge build `3d1c717d379e`;
  nothing upgrades it (`archive/store-floor.md`, 4.4). No golden store below
  the floor is promised an upgrade.
- **A story through an upgrade,** in the one-process world
  (`../testing/tiers.md`, 1.5): a project, a person and an account; a
  goal and a change through a landing; a chat paused and a task
  mid-run; settle, stop, start the new binary with a registry change
  and a job, and go on. The referee: the promised history equal before
  and after, no effect made twice, kept requests answered as before,
  the next numbers colliding with none, and the task going on. No
  owner's level moves.
- **Power loss** on skein's crashing disk at every step of the copy,
  the registry's and jobs' commits, the seal, the check, the manifest
  and the switch: the active generation is wholly the old or wholly
  the checked new, and the next start begins again from a fresh copy.
- **Faults:** a full disk, a short write, a failed sync, a malformed
  record, a job refusing a record, a layout with no plan, a limit too
  small, and recovery after an unsettled stop.
- **The engine alone:** the engine upgraded while a worker's run goes on,
  adopted at its hello; a window past its restart wait or grace ending
  the run.
- **The process tier** (`../testing/process-tier.md`): stop, start the new
  binary and go back, on fakes, with an owned Forgejo's migration as its
  tests have it.

## 14. Who builds what

| Owner | What |
|---|---|
| skein | skein-codec: schemas read at run time into generic values and encoded back; layouts' fingerprints and derived plans; what a `bytes` field holds; the wires' patch command and checks. Nothing of skein-kv |
| smith | its turns' current format, and the turns' wire's kept texts, patches and bridges |
| jig | the explicit levels and floor; registry planning and conversion at read; the jobs' runner; the copy, seal, check and switch; generations and `active`; transcripts patched as read; the settled stop's core; the engine's wire versions kept; its families' current schemas and jobs |
| temper | its families' current schemas and jobs; `temper upgrade --check` and `--back`, `temper versions`; the configuration's previous release; small files; every marker's version read; the settled stop's connector |
| all | the floor's fixture and refusal; the story through a registry and job upgrade; power loss; the process drill; removal of jobs once dev.fabrig.ai runs them |

The full upgrade and the engine alone share the store's mechanism;
the wires and workers' lifecycle are `archive/redeploys.md`'s.

## 15. Open questions

- **When the first release is.** Its date is open; the floor does not
  wait for it (4.1).
- **Old turns patched on every read.** Default: as read; a task's file
  rewritten once if reading old turns proves slow.
- **The client's kept requests,** patched by the same patches, which
  the client's binary holds. Default: dropped with a notice (7.4).
- **Online backups.** Default: a copy with the deployment stopped (3.3).
- **The formats on disk** (6.5). Default: unchanged; a tool of their
  owner's when one first changes, run before the records' upgrade, as
  `pg_upgrade` runs before an application migrates.
- **Rolling upgrades,** an engine and workers of different wires:
  `archive/redeploys.md`, from 2026-11-12.
