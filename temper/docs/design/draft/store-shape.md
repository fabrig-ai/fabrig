# store-shape: an upgrade is a check, and the store keeps what it needs

Draft, 2026-11-13, from a session with the user, who approved its shape, the horizons of section 6, dropping the frozen texts (section 3) and going back as today (section 5). The store stays skein-kv's ordered keys and values in numbered commits, but an upgrade rewrites nothing, and the store grows with live work rather than with everything that ever happened. The figures and the reasons it starts from are in `../../development/store-assessment-2026-11-13.md`, a census of dev.fabrig.ai's store. Section 10 is the way from today's store to this one, migrated in place, and `store-shape.execution.md` carries its milestones. This draft wins over `upgrades.md`, sections 4 to 6, 7.2 and 12, `staged-upgrades.md`, section 8, and `redeploys.md`, section 3.1 for the store's families (its wires keep their rule), where they differ, until its milestones fold it in.

## 1. In one page

**Today**

- **An upgrade checks the store's layouts.** Registered families change through defaults, former names or a written job, with no version move or patch. A store below the floor is refused by name, naming bridge build `3d1c717d379e` (`store-floor.md`, section 3).
- **The map holds about 430 rows per task, where `kv.md` planned for about 15.** 1.14 million rows for about 2,650 tasks, 3 days old, 87% of them milestones: one immutable row per phase change, kept forever.
- **No task or transcript ages out.** Settings' history, answered keys, landing history and temper's decisions for an away person are pruned, but the tasks' horizon, ended tasks shrunk to summaries and transcripts dropped are designed and not built: only a project's deletion erases tasks today. Transcripts reached 669 MB in 2.5 days, uncompressed.
- **The log is mostly whole records written again.** Over 90% of what the log writes is three kinds of hot record (a live task, a goal's projection, temper's change record), each written whole on every change.

**The ideal**

- **State in memory, history on disk** (section 2). The map keeps what changes and what a restart reads: live work, indexes, summaries. What is appended and never changed (milestones, activity, revisions, terminals, turns, an ended task's full record) goes to a log per task beside the store, read when asked and dropped whole.
- **Each value names its layout** (section 3), an id into a registry of layouts the store keeps, in place of its family's version. Defaults are in the schema. A value at an older layout is converted when read; writes are at the current layout, so a record converges when it is next written.
- **An upgrade is a check** (section 4): the store's layouts against the binary's, costing as many layouts as it holds, not records.
- **Going back is the previous generation, made of hard links** (section 5) in milliseconds, not copied.
- **What is kept, and for how long, is declared per keyspace** (section 6) by its owner, jig or temper.
- **Hot records stay small** (section 7), and **the store counts itself** (section 8).

**The way there** (section 10)

- Eight milestones, landed as goals and reaching each deployment as ordinary upgrades. The store migrates in place: nothing is exported, started afresh or lost beyond what section 6 lets go.
- The census measures the store, and generations use hard links. Layout ids name the layouts and texts held by the store's registry; a store without a registry is refused by name, naming bridge build `3d1c717d379e`. Then milestones become attempts, retention runs, history leaves the map, and hot records split.

## 2. State in memory, history on disk

Two places, each keyspace declaring which it uses:

- **The map holds state:** what changes, and what a restart or a step must read at once. That means live tasks, claims, holds, slots, people, accounts, settings, goals' projections, the indexes, each ended task's summary, and temper's live connector records. It is answered in the step that asks, as today, and restart reads it whole. On dev.fabrig.ai, a few MB.
- **A task's log holds its history:** what is appended and never changed. That means its attempts (section 6), activity lines, history revisions, terminals, decided proposals, turns, and its full record once it ends. It is skein-kv's payload log (skein's `data/kv.md`, section 6), one file per task, generalized beyond turns. Frames are synced before the commit that records the log's new extent, read by page when asked, compressed, and dropped whole as one file. A log lives in the store's generation, beside the snapshot and segments.

The split fixes growth where it starts: the map follows live work, and history costs disk, not memory, and goes in one unlink. It also makes every job over the map cheap. On dev.fabrig.ai's figures, the state map would be about 30,000 rows.

## 3. Values name their layout

- **A layout id in place of the family's version.** The u16 at a value's head names the exact layout of its value type, the alternative of its family's broad root that its key holds (the tasks family's `Row` holds a task, an attempt, an activity line and more). It is an id, within that value type, into the store's registry of layouts: a component keyspace of rows holding the value type, the id, its fingerprint (skein's `data/codec/fingerprints.md`, section 1, without the family's version it carries today) and its schema text. The key says which value type a value is, as today. A binary that writes a value at a layout the registry lacks adds it in the same commit. A change to one type moves only the value types that reach it: an attempt keeps its layout when `Tries` changes, and when `Class` gains a variant it gains a layout whose plan from the old one is the identity.
- **Defaults in the schema.** `infrastructure: u32 = 0`, and an `option` defaults to none. An added field, an appended variant, a widened bound, a dropped field and a widened integer are all derived from the two layouts. None needs a patch file, a version of its own or a frozen text: registered families keep their texts in the store's registry (jig's `protocol/store/layouts.md`, section 3).
- **Converted when read.** At open, the component builds one plan per stored layout, from its text to the binary's, out of the derived changes `diff.md` names today, run at read time rather than at patch time. A value at the binary's layout decodes by its generated code, as today. An older one is rewritten by its plan to the binary's bytes, then decoded. A derived plan is a pass over the value's bytes (copy, insert a default's bytes, skip, widen), so it needs no generic values; written changes use their owner's written job. Writes are always at the binary's layout, so a record converges the next time it's written. History is never rewritten: it is converted as it is read, as turns already are (jig's `protocol/store/transcripts.md`).
- **Written changes** (a change of meaning, a field computed from others) are their owner's written job at open, converting values at the stored layout to the binary's (jig's `protocol/store/jobs.md`, section 4).
- **Keys don't change in place.** A new key layout is a new keyspace, filled from the old one by a job at open. Since the map holds only state, a job's walk takes seconds.
- **Layouts retire** when no value holds them. The component counts values per layout as it applies commits (a value's first two bytes and its key's root type), and rebuilds the counts at recovery. A layout still held in logs stays until retention drops them.

## 4. An upgrade is a check

At open, before the root starts:

1. Read the registry.
2. For each stored layout, find its plan or its written job. If one is missing, refuse, naming the layout, its type and how many values hold it.
3. When the binary differs from the last one that opened the store, start a new generation by links (section 5), before anything is written.
4. Run the jobs of any keyspace whose keys changed, over the state map.

Steps 1 to 3 cost as many layouts and files as the store holds, not as many records. Nothing is copied or rewritten.

## 5. Going back

- **The previous generation, made of hard links.** An upgrade's new generation links the previous one's snapshot and sealed log segments, which skein-kv never writes again (it starts a new segment at every open), and copies only the newest segment, whose torn tail recovery may cut. The secrets' store, small and checked for single links, is still copied. It takes milliseconds and the two share the files, so it costs disk only as the live store moves on. The previous generation stays a whole store, at the commit it was left at.
- **Going back** is today's (`upgrades.md`, 3.2): before the new generation's first commit of its own, `temper upgrade --back` switches to the previous generation and the previous binary starts on it, losing nothing; after it, the previous generation is the backup to restore, losing what was committed since, with the effects the new version made on the forge to be made again.
- **Tasks' logs** are linked too. A log appended past the previous generation's recorded extent holds, for that generation, only garbage, by `kv.md`'s section 6 rule; a log dropped in the new generation stays in the previous one until that goes.
- skein-io gains a hard link, which jig's `protocol/store/README.md`, section 11 leaves open as a copy made by the file system.

## 6. What temper keeps, and for how long

The owner declares each keyspace's place, its retention and its trigger; the domain's timers apply them, as jig's `protocol/store/growth.md`, section 1.1 designs. The user set these horizons (11-13).

| What | Today | Ideal place | Kept |
|---|---|---|---|
| live tasks, claims, holds, slots, pools | map | map | while live |
| an ended task's summary and index rows (about 1 KB) | not built | map | 1 year |
| an ended task's full record, activity, revisions, terminals | map, forever | its log | 30 days after it ends |
| milestones | map, one per phase change, forever | its log, as attempts (below) | with its log |
| turns | a file per task, uncompressed, forever | its log, compressed | 14 days after it ends |
| parties' updates | map, forever | map | until read, then 30 days |
| goals' projections | map | map | until the goal ends |
| settings' history | map, bounded | map | its history limit |
| answered keys | map, expiring | map | the request retention |
| temper's connector records for a task | map, forever | map while live, its log once ended | with the task |

**Milestones as attempts.** The lifecycle people read is per attempt: claimed, started, ended, and how, and the holds between. A row per phase change for every wake of an agent (idle, due, preparing, claimed, running, backing off) is a debugging trace, not history. The ideal keeps one record per attempt, with a count of the phase changes within it, and one per hold, which escalations and the task record name today by its milestone's position; nothing per wake. What they hold is jig's tasks child's to design.

## 7. Writes carry what changed

A record written often holds only what changes often. A task's spec, words, contract and authority, written once, live under one key, and its phase, claim and counters under another. The same goes for a goal's projection and temper's change record. The log then carries what changed, not 7.8 KB per change. This is a rule for owners, which the census (section 8) checks, not a mechanism in the store. Partial puts in the store (a merge operator) would do the same at more cost to the store's simplicity, so they aren't the ideal.

## 8. The census

The store reports, per keyspace, its rows, keys' and values' bytes, layouts, and writes since the last snapshot, as `temper store census`, and the deployment's document shows it. The assessment's script is a stand-in for it.

## 9. Today against the ideal

| | Today | Ideal | Owner |
|---|---|---|---|
| what a value says about itself | its value type's layout id | its root type's layout id | skein-codec, jig's component |
| an added field with a default | the schema's default; nothing rewritten | the schema's default; nothing written | skein-codec |
| an appended variant or widened bound | a derived registry plan | nothing | skein-codec |
| conversion | at read, per layout | at read, per layout | jig's component |
| an upgrade's downtime | the open and pending jobs | grows with layouts (seconds) | jig's component |
| going back | a copied generation | a generation of hard links | jig's component, skein-io |
| history | in the map, in memory, forever | in a log per task, on disk, dropped whole | skein-kv (payload logs), jig |
| retention | designed, not built | declared per keyspace, run by the domain's timers | jig, temper |
| milestones | one row per phase change | one record per attempt | jig's tasks child |
| transcripts | uncompressed, forever | compressed, 14 days after the end | jig, temper |
| hot records | written whole (about 8 KB) | hot part apart from cold | jig, temper |
| seeing it | none | a census per keyspace | jig, temper |

## 10. The way there

Eight milestones, S1 to S8, take a deployment's store to the ideal in place: dev.fabrig.ai's first. A store below the floor is refused by name, naming bridge build `3d1c717d379e`. Each lands as goals through temper and reaches a deployment as an ordinary upgrade, by the mechanism the milestones before it leave. None exports the store, starts one afresh or drops what section 6 keeps. Going back stays as section 5 says throughout.

### 10.1 The milestones

- **S1, the census** (section 8). Counts per keyspace and per layout, kept as commits apply and rebuilt at recovery; `temper store census`; the deployment's document. At its upgrade nothing is written. It measures every milestone after it, on the deployment, before and after.
- **S2.** No head-only walk remains: stores behind the build are refused by name, naming bridge build `3d1c717d379e` (`store-floor.md`, sections 3 and 4.2).
- **S3, generations by hard links** (section 5). Opening's new generation links the previous one's snapshot and sealed segments rather than copying them, and skein-io gains the link, in its kernel, its ring, the simulator and the file layer, with crash cleanup. The previous generation stays a whole store, so going back is today's. At its upgrade nothing is written, and the upgrades after it link. The copy goes, and the two generations, about 400 MB on dev.fabrig.ai, share their files.
- **S4, layout ids** (sections 3 and 4). Defaults in the codec's schemas; the head as a layout id per value type, the families' broad roots resolved by key; fingerprints without the family's version; the registry; derived plans at read; written changes as written jobs; jobs at open; the open's check. An upgrade of derived changes writes registry rows alone. A store from before S4 is below the floor and refused by name, naming bridge build `3d1c717d379e` (`store-floor.md`, section 3).
- **S5, milestones as attempts** (section 6). jig's tasks child designs the attempt and hold records, moves the milestones' readers to them and retires the milestones' keyspace. The readers: escalations and the task record, which name a hold by its position; history and activity pages; goals' projections and temper's feed, which read holds and ends alone; the clients' wire and what renders it (the task page, the TUI, `temper ctl`). The milestones' keyspace holds nothing, and the growth per wake stops.
- **S6, retention** (section 6). Owners declare each keyspace's horizon; the domain's timers erase what has passed it, in pages within a step's budget, beside the prunings that run today (settings' history, answered keys, landing history, temper's decisions for an away person); an ended task's summary is written when it ends; turns' files are dropped 14 days after their task ends. Nothing is erased at once: on dev.fabrig.ai the first turns go about 11-24, 14 days after 11-10's tasks ended, and the first full records about 12-10. Until S7, 30 days of ended tasks stay whole in the map, about 380 MB at this week's pace.
- **S7, history in task logs** (section 2). skein-kv's payload logs, generalized; jig's history keyspaces moved to them, by page on read; new turns written to the task's log, compressed. Turns' files written before S7 stay where they are, read as today, until S6's horizon drops them; none is converted. The map then holds state and summaries alone.
- **S8, hot records apart from cold** (section 7). Live tasks, goals' projections and temper's change records keyed as a part written once and a part that changes. The log then carries what changed, from about 24 MB an hour to a few.

### 10.2 Order

- **S1 and S3 first,** in parallel: each is small and needs nothing, and S3 makes the upgrades that bring the rest cheaper. S2 leaves no machinery. S1 lands before the others' upgrades reach a deployment where it can, so each is measured.
- **S4 next:** from it on, a derived change rewrites nothing, and the jobs' runner exists.
- **S5 and S6 after S4,** in parallel: the rows, then the growth.
- **S7 after S5 and S6:** it moves what S5 shaped and S6 bounds.
- **S8 after S4,** at any point: it is last in gain.

### 10.3 dev.fabrig.ai's store through them

Estimates from the assessment's census, each to be measured by S1 as its milestone lands.

| After | Rows in the map | The map | An upgrade's downtime | The log |
|---|---|---|---|---|
| today (11-13 21:25Z) | 1.14 million | 121 MB | the open and pending jobs | 24 MB an hour |
| S3 | the same | the same | the open's | the same |
| S4 | the same, and a row per root type | the same | the open's, for derived changes | the same |
| S5 | about 160,000 | about 50 MB | under 9 s | the same |
| S6 | bounded by the horizons | about 380 MB at 30 days of this week's pace | grows with the map | the same |
| S7 | state and summaries: about 30,000, and one per ended task | a few MB, and about 1 KB per ended task | a few seconds | the same |
| S8 | the same | the same | the same | a few MB an hour |

### 10.4 Meanwhile

- **Registered families** change through defaults, former names (`was`) or a written job, with no version move or patch. No build keeps the patches or seeds a registry from code; an older store is refused by name, naming bridge build `3d1c717d379e` (`store-floor.md`, sections 3 and 4).
- **Going back** stays the previous generation's switch, losing what was committed after it.
- **S5 moves the clients' wire,** whose history and activity lines carry milestones today: a client's wire moves its version unbridged, and a client of another version reloads, as the rule for upgrades says. No worker reads milestones, and no other milestone changes a wire.

## 11. Open questions

- **Compression:** a pure-Rust `no_std` codec (LZ4 or DEFLATE), per frame or per block of frames, for logs and turns: S7's to pick.
- **Milestones' readers:** goals' lifecycle projections and the task page read them. What they read from attempts is S5's to design.
- **Summaries' weight:** at dev.fabrig.ai's pace, about 900 tasks a day, a year of 1 KB summaries is about 330 MB in the map. S6 sets what a summary holds; if they weigh too much, older ones move to a log by month.
- **The long engine passes** of the assessment's section 4 aren't traced: if they come from walks over history in the map, S5 and S7 remove them, and S1 can tell.
