# The store on 2026-11-13: what it holds, why an upgrade rewrites it, why it grows

An assessment of temper's store on dev.fabrig.ai, read on 2026-11-13 against main at `e40ee6b2d9`, with the newest figures added at `6c487d6e15`. It is the evidence for the draft `../design/draft/store-shape.md`, which sets out the store's ideal shape and the way there.

## 1. What the map holds

The active generation's snapshot at commit 620,673 (07:29Z): 117 MB on disk, 1,055,512 rows, 20.5 MB of keys and 88.4 MB of values. The store was about 2.5 days old (its oldest transcript is from 11-10), and held about 2,300 tasks, 865 of them agents'.

| What | Key | Rows | Values | Note |
|---|---|---|---|---|
| tasks' milestones | `01 01 0a` | 922,619 | 50.6 MB | 54 B each; one per phase change; per task: median 19, p90 381, p99 7,889, most 34,543 |
| tasks' activity | `01 01 0b` | 96,066 | 11.9 MB | |
| ended tasks | `01 01 06` | 2,284 | 15.7 MB | 6.9 KB each, kept whole |
| parties' updates | `01 01 0c` | 18,771 | 0.9 MB | the inbox by time |
| core's terminals | `01 00 05` | 3,937 | 2.0 MB | |
| temper's change records | `02 0c` | 379 | 3.1 MB | 8.1 KB each |
| settings' history | `01 00 09` | 11 | 0.8 MB | 69 KB each |
| live tasks | `01 01 00` | 20 | 0.2 MB | 9.2 KB each |
| everything else | | about 11,400 | 3.4 MB | |

What live work needs is the live tasks, their claims, holds and slots, people, accounts, settings, goals' projections and the indexes: a few MB. What history costs is the other 95%.

The rows grew from 707,728 (11-12 12:55Z) to 891,036 (19:42Z), 1,024,759 (11-13 00:32Z), 1,055,512 (07:29Z) and 1,142,238 (21:25Z, commit 680,058: 130 MB on disk, 993,441 milestones, 2,616 ended tasks): about 27,000 an hour through the 12th, about 6,000 an hour through the 13th. A few tasks hold most of them: four change tasks held about 34,500 milestones each, and one (2317) wrote 1,166 in 5 hours.

## 2. What the log writes

The live segment from 07:29Z to 12:29Z: 10,118 commits, 121 MB, about 24 MB an hour.

| What | Puts | Keys put | Values written |
|---|---|---|---|
| live tasks (`01 01 00`) | 7,139 | 73 | 55.8 MB |
| goals' projections (`01 00 01`) | 12,692 (and 6,151 erases) | 76 | 41.6 MB |
| temper's change records (`02 0c`) | 2,628 | 27 | 15.4 MB |
| milestones and activity | 9,239 | 9,239 | 1.4 MB |
| everything else | | | about 5 MB |

The first three are over 90% of the log, as a handful of records written whole again and again. A snapshot follows each twice the map's size of log, so about every 10 hours, each writing the whole map.

## 3. Transcripts

865 files, one per agent task, 669 MB in all, the largest 6.5 MB: about 250 MB a day, uncompressed, never dropped, since no retention runs.

## 4. Memory, disk and time

- **The engine:** a peak of 399 MB resident since its start at 01:45Z, the upgrade's patching included; at noon, 97 MB resident and 218 MB in swap, on an 8 GB host whose swap was 1.3 GB used.
- **On disk:** the active generation's 228 MB (snapshot and log), the previous generation kept until the next switch (179 MB), and the transcripts.
- **An open with no patch:** 9 s from the service's start to listening, and 14 s to the core answering (22:22:51Z to 22:23:05Z, the map at 1.14 million rows).
- **An open with a patch:** jig's patch 3 (`jig/crates/jig-protocol-store/patches/0003.patch`) added two enum variants and one `u32` defaulting to 0, and rewrote 1,020,992 records of the tasks family, 922,619 of them milestones, whose bytes none of its changes touched: a milestone holds no `Tries`, and can hold a `Class`, which only gained an appended variant. The engine started at 01:45:26Z and served from 01:56:15Z: about 0.64 ms a record. The rehearsals of the same upgrade took 480 to 693 s.
- **Long engine passes:** the rehearsal's serve logged passes of 7 to 12 s against a 1 s bound. They aren't traced to the store.

## 5. Why an upgrade rewrites the store

Five causes compound:

1. **A positional codec with no defaults.** A record is its fields in order, every one present (skein's `data/codec/README.md`, section 3). An added field changes every old value's bytes, so any addition is a derived change that needs a patch (skein's `data/codec/diff.md`, section 3).
2. **Frozen texts.** Once a landed patch names a version, even a change every old value survives (an appended variant, a widened bound) needs a patch and a new version (skein's `data/codec/command.md`, section 2, step 3). jig's patch 3's appended `Class.infrastructure` is one.
3. **A version per family, carried by every record.** The u16 at the head of each versioned record is the family's version, so moving it makes every record of the family old. When jig's patch 3 changed `Tries`, it rewrote every milestone, which holds no `Tries` and whose `Class` only gained a variant. Held fields add walks too: each of smith's patches runs an identity plan over every core record (jig's `protocol/store/README.md`, section 11).
4. **Eager, offline, at open.** Patching runs before the engine serves: it copies the store and the secrets' store into a new generation file by file, walks every record of each family the patch changes (decode by the kept text, run the machine, encode, put), checks the copy, then switches (jig's `protocol/store/opening.md`, section 2). Downtime grows with the records.
5. **Frequent.** Fifteen patches to stored families landed in the two days after the baseline (`bc23255eb7`, 11-11 20:25Z): jig's 1 to 5 and temper's 1 to 10. jig's patch 5 moves the tasks family again (11 to 12) for one appended variant with its payload, so the upgrade that brings it rewrites every tasks row again.

The copy also costs twice the maps on disk. The patch machine decodes every walked value into generic values, the identity's included, in a workspace that grows on demand to at most about 249 MB for smith's transcripts' machine (jig's `protocol/store/README.md`, section 11).

## 6. Why it grows

- **Milestones are history kept as state.** Every phase change of every task saves an immutable `Milestone { task, position, phase, at }`. An agent's wakes move its task through idle, due, preparing, claimed and running many times, so a change task writes hundreds or thousands of rows, and a stuck one tens of thousands. They are in the map, in memory, forever.
- **No task or transcript ages out.** `jig-domain-core`'s erasure selects by project alone (`routing/task/erase.rs`: "retention adds its horizon later"). Ended tasks stay whole (6.9 KB each), with their activity, milestones, terminals and transcript. Only `ForgottenOnceRead` tasks, which leave remains, and deleted projects go. What is pruned today: settings' history, answered keys, temper's decisions for an away person after 7 days, and landing history.
- **The map holds what the restart mostly never reads.** The restart point-loads open questions' activity and temper's change records; the rest of history is read on demand, but on demand still means in memory, in the snapshot every 10 hours, and in every upgrade's walk.
- **Hot records are written whole.** A live task's 7.8 KB is written for each change to it, so the log and its snapshots are mostly the same few records again.
- **Transcripts are uncompressed** and dropped by nothing but a project's deletion.

So the plan in skein's `data/kv.md`, section 10.1 (15 entries and 5 to 15 KB per task, and a 30-day horizon keeping the map at 0.1 to 0.2 GB) is off by 30 times on rows and 3 to 9 on bytes, and its horizon isn't built. At the average rate so far, about 420,000 rows a day, an upgrade's walk would take an hour within weeks, and the map would pass `kv.md`'s 10 GB within a year.

## 7. How this was read

A script that walks skein-kv's snapshot and log formats and counts rows, keys and value bytes by key prefix, never printing a value, run as the service's user over the live files, read only. The upgrades' figures are the rehearsal logs, the generations' file times and the service's journal. Not checked: what each milestone's phase is (its codec wasn't run), and what is in the secrets' store (never read).
