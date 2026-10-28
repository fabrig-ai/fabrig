# Upgrades

Draft, 2026-10-23; 6.4 removed by the user, 2026-10-27. How a deployment moves to a newer temper and keeps
its history: its people, projects and accounts, its tasks and goals, live
and ended, its landings and what agents wrote. It takes
`first-release.md`'s section 6 further, from the investigation of
2026-10-23 (Codex, read-only;
`~/src/rust/tmp/roadmap-3/store-upgrades.md`) and a session with the
user that day, whose turns shaped it: the records before the formats on
disk, and a patch landed with each change, as an application migrates
its database. It crosses skein, smith, jig and temper on purpose; an
owner splits it. Nothing here is adopted, and nothing changes today's
development: the baseline is the first release (skein's `codec.md`,
section 5), and until then the rules of section 4.1 hold. Commands'
spellings are placeholders.

## 1. In one page

- **Patches, as an application migrates its database** (section 4).
  Every landing that changes what is stored brings a patch beside it.
  The store records each owner's patch level, as a table of applied
  patches does; a start that finds the store behind applies the patches
  between, in order, before anything else (section 3).
- **The records first, the formats on disk later.** As PostgreSQL's own
  upgrade between versions of its files (`pg_upgrade`) is apart from an
  application's migrations, an upgrade here has two layers: the formats
  on disk, skein-kv's frames and snapshot and the transcripts' framing,
  and jig's and temper's records in them. This draft is the second; the
  first is left as it is (6.5).
- **Cheap, because schemas are read at run time** (section 5).
  skein-codec decodes the bytes of any kept schema into generic values,
  so an old version is kept as its schema's text, not as code. Most
  patches are operations a tool derives from two schemas: a field added
  with its value, a field dropped, an integer widened. A change every old
  value survives byte for byte needs none, until a landed patch names
  the family's version (4.3).
- **Any store from the baseline on upgrades to any later `main`,** so a
  deployment may follow `main`. The baseline is the first release;
  until it, versions stay fixed and schemas change in place (4.1;
  skein's `codec.md`, section 5).
- **A copy is patched, never the store in use** (section 6): a start
  copies the records' and the secrets' maps into a new generation,
  patches the copy, opens it as any start does, and switches to it by
  one atomic replacement. The previous generation is kept, and going back is
  supported until the new one's first commit of its own (3.2).
- **Transcripts are never rewritten** (section 7): their turns are
  patched as they are read. An agent resumes its conversation across a
  change, unless a patch says the change breaks it; then its next
  attempt starts from a fresh brief.
- **An effect in flight stays with the version that started it**
  (section 8): every stop settles first, and a start refuses only when a
  pending patch changes a family that holds an effect still in flight.
- **The engine alone may be upgraded while workers keep their runs**
  (section 9), when their wire is unchanged and the window fits their
  grace. Built second.
- **The configuration is never rewritten** (section 10): each release
  reads its own members and the previous release's.
- **The gate holds the policy** (section 12): a stored change with no
  patch fails it, and so does a landed patch or a kept schema edited;
  golden stores of the baseline and of the last release are patched
  into the candidate.

## 2. What an upgrade carries

| Part | Where | Its formats' owners | Across an upgrade |
|---|---|---|---|
| the records | the deployment's `store/` | jig's component and families; temper's families | patched in a copy (section 6) |
| the secrets | the deployment's `secrets/` | jig's records; temper's forge records; skein's OAuth records | patched in the copy, every number kept (6.3) |
| the transcripts | the deployment's `transcripts/` | jig's frames, smith's turns in them | kept as written, shared by the generations, patched as read (section 7) |
| the configuration | the configuration directory: `temper.json`, `agent.json`, charters' instructions | the operator's | read as written (section 10) |
| small files | the data directory: each forge's setup `state`, the box's records, the demo's, a worker's secret and digest | temper | read in any version from the baseline on (section 10) |
| an owned Forgejo | its forge's directory | Forgejo, and temper's preparation | its own upgrade, at its preparation (section 11) |
| program logins | each worker's programs directory | each program's | kept as they are; a login a program no longer reads is signed in again |
| a worker's root | each worker's data directory | temper | rebuilt at will: checkouts and workspaces |
| what temper wrote on a forge | the forge | temper's connector | read in every version (section 11) |
| a client's kept requests | the person's browser or shell | jig's client | as jig's client says (7.4) |
| the cache | the cache directory | — | never carried |

The directories are `../protocol/configuration.md`'s (section 12); what
the store holds is jig's `protocol/store.md` (4.4) and
`../protocol/store.md`'s (section 2).

**What must keep its identity:** the deployment's id, which names its
branches and keys' markers on the forge (`../protocol/forge.md`, section
4); the installation's id; every number the domain gives (tasks,
attempts, messages, sources, accounts, issued program names), so that
none is given twice; and the commit position, which the deployment's
header and projections name, and which patches carry on, being ordinary
commits (6.2). A worker holds no authoritative state of its own: what it
acknowledged is in the engine or on the forge.

## 3. At start

### 3.1 Every start

- **It reads the levels first.** jig's store, as it opens (jig's
  `protocol/store.md`, section 10), reads the patch levels in the format
  row and compares them with the binary's (4.2). Equal, it opens as
  today. Behind, it patches a copy (section 6), printing a line per patch
  and what it counted, then opens the new generation as today.
- **Every shape with a store:** `temper serve`, an engine alone and
  `temper box` (`../protocol/README.md`, section 15). A worker alone
  keeps no store; its small files are read as section 10 says.
- **Refused before anything is written,** naming why:
  - a store ahead of the binary: there are no patches backwards;
  - a store from before the baseline, which is made again, as today
    (4.1);
  - an effect in flight in a family a pending patch changes (section 8).
- **A patch that refuses a record, or a copy that does not open** (6.4), stops
  the start with the old generation active, saying why, and exits
  non-zero; the next start begins again from a fresh copy.
- **Rehearsed:** `temper upgrade --check`, by the new binary, on a copy
  such as last night's backup, patches into a scratch directory and
  switches nothing. It reports how long patching took, what it refused
  and whether the workers may stay (section 9), so an operator knows
  beforehand whether a service manager's start timeout needs raising.

### 3.2 Going back

- **Before the new generation's first commit of its own:** `temper
  upgrade --back` makes the previous generation active again, and the old
  binary starts on it. Nothing has happened since the stop, so nothing is
  lost or made twice. This is for a new binary that patched and then
  failed to start. The manifest records the commit position patching
  ended at; the generation's own first commit moves past it: every start
  commits, the wire versions it speaks first (section 9), and every
  effect is committed before it is made.
- **Transcripts are shared** (section 6): a frame the new version wrote
  past the old generation's committed lengths, never committed, is cut
  at the old one's open, as after a crash (jig's `protocol/store.md`,
  8.4).
- **After that commit,** going back is restoring a backup. The new
  version may have acted on the forge, and the old one, restored, may act
  again; an owned Forgejo migrated since needs its dump restored with it
  (`../protocol/setup.md`, 6.3). The tool refuses `--back` and says so.

### 3.3 Backups

A backup is a copy of the data directory with the deployment stopped
(jig's `protocol/store.md`, section 16; `../protocol/setup.md`, section
16). The previous generation is a copy of the maps alone, not of the
transcripts, the forge or the programs' logins. Online backups stay open
(section 15).

## 4. Patches

### 4.1 Before the baseline

The baseline is the first release (skein's `codec.md`, section 5).
Until it, each encoding keeps its current version: changes a reader
would notice change the schema, generated code and one set of golden
bytes in place (`../protocol/README.md`, section 9). A reader still
refuses another version and nothing is translated (jig's
`protocol/README.md`, section 7). A version names no one layout yet, so a
store from an older build is started afresh when its families change:
no patch could tell those layouts apart (`../protocol/store.md`, section
4). No promise reaches back before the baseline, and nothing in this
draft asks anything of a change landed before it. Versions move at a
release, not at a landing; `first-release.md` proposes v0.1.0.

### 4.2 Levels

- **Each owner keeps a sequence:** jig's patches numbered from the
  baseline, temper's likewise, and smith's and skein's for what of
  theirs is kept (turns, results, OAuth records). A patch changes one or
  more of its owner's families and prepares each one's next released
  version.
- **The format row is the table of applied patches:** each owner's level
  beside each family's version (jig's `protocol/store.md`, 5.3), written
  by the patching's last commit.
- **Versions move at a release,** so a released version names one
  layout. Before the baseline, schema changes keep the current number
  (`../protocol/README.md`, section 9; skein's `codec.md`, section 5).
- **A landed patch never changes.** Until it lands, its author edits it
  with the schema; once on `main`, deployments may have run it. Two
  branches that each add jig's patch 41 meet at `land`, whose rebase and
  gate refuse the second until the tool renumbers it (4.5).

### 4.3 What a patch is

Operations over generic values (section 5), for each family it changes,
in its owner's crate beside the schemas. By kind:

- **None needed:** a change every old value survives byte for byte. A
  field or variant renamed, a variant appended, a bound widened, a
  variant marked `boxed` (skein's `codec.md`, section 3). The family
  keeps its version and no level moves, while its kept text is not
  frozen. Once a landed patch names the family's version, its kept text
  is frozen: an appended variant, a widened bound or an added
  declaration also takes a patch the command writes, and a version move.
  A rename or a dropped declaration stays none needed. This is the
  user's rule (skein's `codec.md`, 12.2 step 3), and it applies from
  the first release on, since no version moves before then (4.1).
- **Derived:** a field added, with the value every old record takes; a
  field dropped; an integer widened; a type wrapped in `option`;
  variants reordered. The tool writes these from the two schemas, asking
  the author only for an added field's value.
- **Written:** a change of meaning, or a field whose value is computed
  from others, as a function over the generic values; and a change of
  keys, since jig's keys are written out by hand, not generated from
  schemas (jig's `protocol/store.md`, 4.1).

What every patch keeps:

- **No invention.** An added field's value must be true of every old
  record: a field old records hold no evidence for is an `option`, and
  none for them. Unknown is not zero: smith's usage became optional so
  that a turn with none is not one that cost nothing. A written patch may
  refuse a record, naming it, which stops the start (3.1).
- **No number given twice.** Every number and counter is kept; the next
  number given after the upgrade is one no record holds.
- **No outcome repeated or forgotten.** Keyed requests' answers,
  connectors' keyed outcomes and decisions are kept, so a request sent
  again after the upgrade is answered as before.
- **Indexes follow their records:** a patch that changes what an index
  derives from rebuilds it from its records (jig's `protocol/store.md`,
  7.4).

### 4.4 Values nested in another owner's

- **Opaque bytes, said what they hold.** A record of another family is a
  `bytes N` field, decoded by its own family's code (skein's `codec.md`,
  section 3): temper keeps jig's refusals as `refusal: bytes 517`. Its
  schema says what such a field holds, another family's type or pieces of
  a type of its own, so a patch of that type reaches it wherever it is
  kept.
- **In the order the pins made.** Each of temper's patches records jig's
  level when it was written, which temper's pin of jig fixed; patching
  runs jig's patches up to that level before it, and jig's remaining ones
  after temper's last. jig's patches record smith's and skein's levels
  likewise.
- **A nested type that grows** widens its holder's bound at the repin
  that brings it, which moves no bytes and needs no patch.

### 4.5 The tooling

- **The gate finds the change** (section 12): a schema whose layout
  differs from its latest kept version's with no patch fails it, naming
  the command.
- **The command** (`cargo xtask patch`, a placeholder) diffs the schema
  against the latest kept one. A change that needs no patch replaces the
  kept text. Otherwise it keeps the old schema's text as its version,
  writes the next patch with the derived operations and a skeleton for
  the written ones, and writes its test: the family's old golden bytes
  through the patch, its result reviewed once and kept as a golden of
  its own.
- **Renumbering:** the same command renumbers a patch that `land` found
  taken.
- **Its aim:** the common change, a field added, costs its author the
  field's value and a review of one golden: minutes.

## 5. Schemas at run time

- **Kept as text.** Each family's schema at every version from the
  baseline on is kept as its text in its owner's crate; only the current
  one is generated into code, as today (skein's `codec.md`, section 4).
- **Generic values.** skein-codec reads a schema and decodes bytes of it
  into a tree of generic values (records, variants, integers, bytes,
  lists, options), and encodes a tree back, with the generated code's
  checks: each value within the schema's ceilings, text that is UTF-8,
  tags it knows, no bytes left over. A `secret` stays one, its bytes
  never printed.
- **Fingerprints of the layout:** a digest of what the bytes are (types,
  order, tags, bounds), not of names or comments, so a rename moves none.
  The gate compares them (section 12).
- **The diff** compares two schemas' layouts into 4.3's kinds: none
  needed, derived operations, or a written patch, naming each field.
- **Where it runs:** in the store's open, over the copy (section 6), and
  in jig's transcripts' reads (section 7). It is step code: a value's
  work is bounded by its schema's ceilings, and a patch's by its
  operations (`programming-model.md`, 6.3).
- **Small:** about 1,750 lines of jig's store schemas and 190 of
  temper's at 2026-10-23, in a language of a handful of types (skein's
  `codec.md`, section 3).

## 6. Patching a copy

```
<deployment>/
  active                   the active generation's number
  generations/
    7/  store/  secrets/  manifest
    8/  store/  secrets/  manifest
  transcripts/             shared by every generation
```

- **jig's deployment directory holds them** (jig's `protocol/store.md`,
  section 2): its open reads `active` and opens that generation's maps,
  and a first start makes generation 1. The layout is the baseline's
  from the start, so no upgrade ever moves today's.
- **The manifest** names the generation patched from, the levels before
  and after, each family's count before and after, and the commit
  position patching ended at.

### 6.1 The steps

1. **Plan:** the levels, the patches between them and their order
   across owners (4.4). What section 3.1 refuses is refused here, before
   anything is written.
2. **Copy** the active generation's `store/` and `secrets/` whole into
   the next one's, synced.
3. **Open the copy** as any start opens a store (6.2).
4. **Patch:** for each patch in order, walk its families' ranges; decode
   each record by the schema of its version, apply the operations, encode
   it by the next, and write it back. A record whose key moves is put at
   its new key and erased at its old in the same commit. Each versioned
   record names its family's version, so one already patched is passed
   over. The levels and the format row are the last commit's.
5. **Open** the copy as any start does (6.4).
6. **Switch:** `active` replaced by io's replacement sequence: written
   beside, synced, renamed over, the directory synced (skein's
   `io.md`). One rename, never two standing for one.
7. **Drop** the generation before the previous one.

- **A crash before the switch** leaves the old generation active and
  untouched; the next start removes the partial one and begins again
  from a fresh copy. After the switch, the new one is whole.
- **Room:** on disk, the maps twice, not the transcripts. In memory, one
  map, where a record whose key moves is held twice until its old key is
  erased: at most about twice the map's budget, before the engine serves.

### 6.2 skein-kv as it is

- **The copy opens as any store does:** skein-kv recovers its snapshot
  and the log's longest valid prefix (skein's `kv.md`, section 3), and
  jig's store checks its format row and the deployment's header (jig's
  `protocol/store.md`, section 10).
- **Patches are ordinary commits** through jig's store, each within
  skein-kv's bounds and each writing the deployment's header, as every
  commit does. So the commit numbering carries on from the old
  generation's, the header's check holds at the next open, and every
  reference to an earlier commit still names it.
- **Nothing new of skein-kv.**

### 6.3 Secrets

- **Patched as the records are,** in their own map with its own commits
  (jig's `protocol/store.md`, section 9): jig's credentials' digests and
  LLM accounts' records, temper's forge records, and skein's OAuth
  records by skein-oauth's patches.
- **Their numbers kept:** sources', accounts' and the carried values'
  counter, so that the start's sweep, which erases the secrets no record
  names, erases nothing it should keep.

### 6.4 Opened before the switch

Removed by the user (10-27): no check runs before the switch. The copy
is opened as any start opens a store, decoding each record under its
codec and fitting the deployment's limits, and the start links what it
loaded (jig's `domain/engine.md`, section 6): nothing is judged, and a
link that cannot be made, a writer's bug, panics (10-28). The checks
are tests, written when a schema changes: each patch proved over sample
stores (13), and the cross-record rules as oracles over every writer, a
cold restart closing every world story (jig's `domain/engine.md`,
section 15). The counts per family, before and after, stay in the
manifest.

### 6.5 The formats on disk

skein-kv's frames and snapshot, at version 1 since it was written (skein
288c5423), and the transcripts' framing, a frame's length at both ends
and its checksum (jig's `protocol/store.md`, 8.1), are left as they are.
The gate fingerprints them as it does every kept format (section 12), so
a change is seen. Changing one after the baseline waits for an upgrade of
its own, a tool of its owner's, as `pg_upgrade` is PostgreSQL's (section
15).

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
  (jig's `protocol/store.md`, 8.3) apply the patches between to each
  frame's fields and turn, so the engine hands out every turn in the
  current version, to a client or a run. Old frames are never rewritten,
  so the places the map keeps for them stay true.
- **Agents resume** their windows across a change of smith's turns.
- **Unless a patch says otherwise.** A smith patch that changes what a
  conversation means to an agent says that windows across it start
  fresh. Patching then closes each live window, and the next attempt
  starts from a brief carrying the transcript's tail, as jig starts one
  past the resume limit (jig's `domain/engine.md`, 7.2): a transition,
  not a failed attempt.
- **Every live task keeps its definition,** its inbox, its holds and its
  place in its queue.
- **Runs end with a full upgrade,** as at any stop: the workers stop
  first, their runs stopped once the engine has every turn and answer
  (`../protocol/README.md`, 15.4), and the tasks retry after the start.
  An upgrade of the engine alone keeps them (section 9).

### 7.3 Sign-ins

- **People stay signed in:** a credential's digest is a record, kept.
- **LLM accounts stay signed in;** one expired or revoked since is
  signed in again, as at any start.
- **Program logins** are the programs' own, kept on their hosts.

### 7.4 Clients

- **Pages reload.** The bundle is the new binary's; a page of the old
  version learns from the stream's hello that its versions are not
  served, and holds its requests unsent until it reloads (jig's
  `client/domain/client.md`, 8.2).
- **A kept request of another version is dropped,** and the person told
  (jig's `protocol/client.md`, section 10). Its outcome may have been
  committed before the stop, so the notice says it may have been made.
  Patching them too is open (section 15).

### 7.5 What is lost

- What retention erased before the upgrade (jig's `protocol/store.md`,
  11.1; temper's bounded `Landed`, `../protocol/store.md`, section 7). A
  deployment that keeps history long sets its retention first.
- A run's unsaved state on its worker, in a full upgrade, as at any
  stop.
- A store from before the baseline, whole: it is made again (4.1).

## 8. Effects in flight

- **Every stop settles first.** Told to stop, before it stops taking
  commits (jig's `domain/engine.md`, 6.1), the engine starts no effect,
  places no run and steps no procedure, answers busy a run's call that
  would start an effect, and commits the outcomes of the effects in
  flight within their deadlines. Then it stops as today. Past a bound, it
  stops unsettled, naming what remains. A second signal aborts the wait,
  as jig's service does (jig's `protocol/service.md`, section 5).
- **Every stop, not only an upgrade's,** since an operator does not
  always know beforehand, and the box's person never does. Its cost is a
  stop that waits up to an effect's deadline.
- **A start patches an unsettled store,** unless a pending patch changes
  a family that holds an effect in flight: then it refuses, naming each
  effect and its task, and saying to start the previous version and stop
  it again. An entry no patch touches is recovered by the new version as
  after any stop (jig's `domain/engine.md`, section 6, step 5), by a key
  whose version every later version reads (section 11). So no patch has
  to keep a reshaped effect recoverable, and most starts never refuse.

## 9. The engine alone

- A pending patch marked fresh makes the upgrade a full one. Stop the
  workers and end every live attempt before the switch, even when their
  wires match. Only a mark beyond the store's applied level counts.
- **The workers may stay** when the new engine speaks their channel's
  version (jig's `protocol/channel.md`, 4.1) and temper's payload version
  (`../protocol/README.md`, section 9), and hands out smith's charters
  and turns in the versions their agents read, and there is no such
  pending fresh mark. Patching keeps live claims and attempts, since it
  keeps every record.
- **The tool says so.** The store keeps the wire versions of the engine
  that last opened it, beside its format row; `temper upgrade --check`
  compares them with the new binary's and says whether the workers may
  stay. It reports the full-upgrade requirement when a pending fresh
  mark requires the workers to stop. `temper versions` prints a binary's
  levels and wires.
- **The window must fit the workers' grace:** the settled stop, the
  patching and the start, within the grace the workers' configuration
  sets (jig's `protocol/channel.md`, 10.1). The rehearsal (3.1) measures
  the patching.
- **Meanwhile, the workers carry on:** they keep the turns and answers
  not acknowledged and redial, and the new engine's restart adopts each
  run as its worker says hello (jig's `domain/engine.md`, section 6,
  step 3). A run's call that would start an effect waits, answered busy.
- **The workers follow later,** one at a time if the wire allows, each
  stop ending its runs, which retry elsewhere.
- **Built second,** after the full upgrade. At today's churn most
  upgrades would change the workers' wire; the share falls as it
  settles.

## 10. Configuration and small files

- **The configuration is never rewritten.** It is the operator's, which
  they may keep in their own tools. Each release reads its own members
  and the previous release's: a member added is optional, with a
  default; a member removed or renamed is accepted for one release with a
  warning naming its replacement, then refused as unknown, as today
  (`../protocol/configuration.md`, section 2). A deployment that skips a
  release may find a removed member refused, by name.
- **Settings in the store shrink it** (`configuring.md`, section 3):
  charters, templates, policies and rules in the store, where patches
  carry them; the configuration keeps what a start fixes.
- **Small files outside the generation,** a forge's setup `state`, the
  box's records, the demo's, a worker's secret and digest: each carries
  its version, is read in any version from the baseline on, and is
  written in the current at its next write. None is written before the
  new generation's first commit, so going back (3.2) finds them as they
  were.

## 11. The forge

- **An owned Forgejo upgrades at its preparation,** as today
  (`../protocol/setup.md`, 6.3): a dump before the migration, pins that
  refuse a downgrade. A temper release may move its pins; the box
  prepares at its start, a service deployment by `temper forge prepare`
  before the engine's start. Going back past a migration needs its dump.
- **What temper wrote on the forge outlives every version:** its keys'
  markers in what it creates, the key's first byte its version
  (`../protocol/forge.md`, section 4), and the branches in the
  deployment's namespace. From the baseline the connector reads every
  version of a marker, and the deployment's id is kept, so the
  namespace and the markers stay its own.
- **The forge is no backup.** Its issues and pull requests cannot rebuild
  the engine's records; the generations are the record.

## 12. The policy and the gate

From the baseline:

- **A stored change brings its patch,** in the landing that makes it
  (4.5).
- **The gate checks:**
  - each family's schema against its latest kept version's layout: a
    change with no patch fails, naming the command;
  - the kept schemas' and the landed patches' texts: an edit fails;
  - each patch's golden: the old goldens through it equal their kept
    results;
  - the golden stores of the baseline and of the last release, patched
    into the candidate and asserted (section 13), their expectations
    never regenerated from today's encoders;
  - the formats on disk (6.5): a changed fingerprint fails;
  - mutations that must fail it: a patch deleted, a kept schema edited.
- **Meaning is reviewed.** A change of what a value means moves no
  layout: its review and a scenario hold it.
- **Its cost:** most changes cost minutes (4.5). The investigation
  guessed 1 to 2 milestones for an additive change and 2 to 4 for one of
  meaning, for translators written against frozen code; generic values
  leave a change of meaning its written patch and its scenario. The
  churn it would meet: 26 moves of the engine's stored families in the
  four weeks to 2026-10-23 (20 jig's, 6 temper's; 21 mostly additive, 2
  repackings, 3 that reshape concepts), some of the 21 needing no patch
  at all under 4.3.

## 13. Proving it

On the existing tiers (skein's `testing-strategy.md`, section 2;
`../domain/testing.md`, section 2), storage mechanics tested once, below
the domains:

- **Step tests:** each patch over golden records, old in, expected new
  out. Generic values against the generated code: any value of a current
  schema decodes to the same tree by both, and encodes back to the same
  bytes.
- **A golden store at the baseline and at each release,** whole:
  records, secrets, transcripts, the configuration and the small files,
  made by that binary from a story and kept for good. Patched into the
  candidate, then asserted: identities and counters, policies, outcomes,
  old turns read in the current version.
- **A story through an upgrade,** in the one-process world
  (`../domain/testing.md`, 2.5): a project, a person and an account; a
  goal and a change through a landing; a chat paused and a task
  mid-run; settle, stop, start the new binary, and go on. The referee:
  the promised history equal before and after, no effect made twice,
  kept requests answered as before, the next numbers colliding with
  none, and the task going on, resumed or fresh as its patches say.
- **Power loss** on skein's crashing disk at every step of the copy,
  every commit of the patching, the manifest and the switch: the active
  generation is wholly the old or wholly the checked new, and the next
  start begins again from a fresh copy.
- **Faults:** a full disk, a short write, a failed sync, a malformed
  record, a patch refusing a record, a limit too small, an effect in
  flight in a patched family.
- **The engine alone:** the engine upgraded while a worker's run goes on,
  adopted at its hello; a window past the grace ending the run.
- **The process tier** (`../domain/testing.md`, 2.6): stop, start the new
  binary and go back, on fakes, with an owned Forgejo's migration as its
  tests have it.

## 14. Who builds what

| Owner | What |
|---|---|
| skein | skein-codec: schemas read at run time into generic values and encoded back; layouts' fingerprints; the diff and its derived operations; what a `bytes` field holds; the patch command and the gate's checks. skein-oauth's patches. Nothing of skein-kv |
| smith | its turns' and results' kept schemas and patches; a patch's mark that windows across it start fresh |
| jig | the levels in the format row; patching at open: the plan, the copy, the patches, the checks, the switch; generations and `active`; the order across owners; transcripts patched as read; closing windows where a patch says; the settled stop's core; the engine's wire versions kept; its families' patches |
| temper | its families' patches; `temper upgrade --check` and `--back`, `temper versions`; the configuration's previous release; small files; every marker's version read; the settled stop's connector |
| all | the golden stores; the story through an upgrade; power loss; the process drill; the gate's mutation checks |

A guess, not measured: about the investigation's 16 milestones of 2 to 3
hours, 32 to 48 hours. skein-codec's run-time schemas and the tooling,
about 4, take the place of its two milestones of skein-kv, its frozen
codecs' registries and its rewrite of the transcripts. Then about two
for the engine alone. Built as the baseline approaches: the full
upgrade, then the engine alone.

## 15. Open questions

- **When the first release is.** Its date is open; it is the baseline
  (skein's `codec.md`, section 5). Until then, section 4.1.
- **A bug in a landed patch,** found after deployments ran it. Default:
  fixed in place for those yet to run it, its fix declared to the gate;
  a later patch repairs those that ran it, where the old evidence allows.
- **A long chain,** squashed as Django squashes migrations. Default:
  kept whole from the baseline; squashed once patching from the baseline
  grows slow, with stores older than the squash upgraded through a
  release before it.
- **Old turns patched on every read.** Default: as read; a task's file
  rewritten once if reading old turns proves slow.
- **The client's kept requests,** patched by the same patches, which
  the client's binary holds. Default: dropped with a notice (7.4).
- **Effects in flight in a patched family.** Default: refused (section
  8).
- **Online backups.** Default: a copy with the deployment stopped (3.3).
- **Stores larger than twice the map's budget.** Default: one map,
  patched in the copy; a streaming patch if a deployment outgrows it.
- **The formats on disk** (6.5). Default: unchanged; a tool of their
  owner's when one first changes, run before the records' patches, as
  `pg_upgrade` runs before an application migrates.
- **Rolling upgrades,** an engine and workers of different wires, by
  negotiated ranges. Out.
