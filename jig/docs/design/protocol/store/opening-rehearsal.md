# Opening the store: rehearsing and going back

## 1. Rehearsing and going back

Two operations the application's tools run with the engine stopped
(temper's `draft/upgrades.md`, 3.1 and 3.2), each taking the
deployment's lock as a start does (1, step 1).

- **A rehearsal** is a start that runs on to its opening with no
  network (*On to the opening*, below), never past it. It
  refuses a directory holding `restore`, `Generations::Unfinished`, before
  copying anything (`restoring.md`, 5). It never writes the directory it is given, a copy of a stopped
  deployment's such as a backup: it reads `active` and that
  generation's maps, copies them into a scratch deployment as that
  generation, file by file and never by links (the directory it is
  given may be a backup on another file system, and a link would
  change its files' links; `Openings` links what it can, below), its
  task logs among them (`logs.md`, 7),
  and opens the
  scratch as any start does (1 and 2), with its owner sets, the
  transcripts read where they are and never cut, and no generation
  dropped. The plan, the copy, the registry's commit and
  the jobs (`layouts.md`, 5.5; `jobs.md`, 2), and the restores (loaded and
  linked, `../../domain/engine/restart.md`)
  run (over `A + 1` where the open writes, over the scratch's `A`
  otherwise). It answers what it ran:
  the jobs with their counts (`jobs.md`, 3), the
  refusal a start would meet, or the restores whole; the wires `A`'s
  last start spoke (`records.md`, 3), and the workers' openings `A` keeps since that start
  (`records.md`, 3, *The workers' openings*), read before its own wires
  commit, and the worker settings `A`'s settings family keeps (*The
  workers kept*, below), from which the application
  says whether each of its workers may
  stay; how many files it linked into the scratch and how many it
  copied, a link the file system refused counted as a copy, with
  `Rehearsing`; and the opening (below). The scratch is
  emptied as it ends.
- **On to the opening.** After the restores the rehearsing root takes
  the switch step inside the scratch: the store marks the scratch's
  generation active there, the directory it was given untouched, and
  commits the start's wires into the copy (`records.md`, 3). The runs
  are adopted with no worker listening: claims in the engine and on
  in-process workers settle at once as at any start, and claims on
  other workers wait in the fleet, unawaited. Decisions open, and each
  connector begins catching up (`../../domain/engine/restart.md`, steps 4
  and 5) with every call to its system refused at once, so nothing
  leaves the process: no effect is made and no secret served beyond
  it. The rehearsal ends there, at the opening, and answers how long
  the start took to open and what each connector would still have to
  catch up, as the application reports it. A transcript is never
  written or dropped in a rehearsal, since its transcripts are the
  given directory's (above).
- **Its reach,** `Reach`, which the request names. `Whole` is all of the
  above. `Openings` reads only the rows from which the application
  judges its workers, and ends there, for a redeploy that judges them
  again at its stop (temper's `protocol/binaries.md`, section 2):
  - it locks the directory, refuses an outstanding intent and reads
    `active` as `Whole` does;
  - it makes that generation in the scratch as an upgrade makes `A + 1`
    (2, step 2), but for `store/logs/` and every task log in it, which
    no row it reads names, so that a redeploy's stop, where it runs
    over the stop's checkpoint, makes links and small copies alone
    (temper's `draft/checkpoints.md`, 6, step 3):
    - **linked:** `store/`'s `snapshot` and each of its segments but
      the newest, the newest found over every page first, by io's
      `HardLink`;
    - **copied,** file by file as `Whole` copies: the newest segment,
      any other file of `store/`, and the whole of `secrets/`;
    - **a link the file system refuses** (the scratch on another file
      system than the directory, as a backup's may be) is that file
      copied instead.

    Nothing writes a linked file again (2, step 2), so the directory's
    files keep their bytes; only their link counts move, while the
    scratch holds them;
  - it opens the scratch through the stores' recovery, and answers
    `Rehearsing` with the wires, the openings and the workers kept, as
    `Whole` does, before any check of the levels, the floor or the
    format;
  - then it ends as `Close` ends a rehearsal: the scratch emptied, and
    `Closed`. It answers no `Opened` or `OpenRefused`, decodes no record
    and runs no plan, job or restore, and the application asks no
    `Close` of it.

  A refusal before the rows are read (the lock `Busy`, `Unfinished`,
  `Unreadable`, a copy or a recovery failing) is answered as `Whole`'s
  is, `OpenRefused` after `Rehearsing`, and the application closes it
  as it closes any rehearsal.
- **The workers kept,** `Rehearsing`'s `workers`, in both reaches: the
  stored changes of the `worker` kind (`../../domain/engine/workers.md`,
  1) in the copy's settings family (`keys.md`, 1.4), the deployment's
  scope, read as the settings' changes are kept
  (`../../domain/engine/settings-store.md`, 1) and never restored: each
  change's value decoded as a generic value of its layout's text
  (`layouts.md`, 4), its number, its name and whether it is withdrawn
  taken from it, and a removal withdrawn; in number order. The store
  knows no defaults: the application lays them over its own
  (`Defaults.workers`), as a start's current workers are the defaults
  with the stored changes over them (`../../domain/engine/workers.md`,
  1), and leaves the withdrawn out (temper's `protocol/wires.md`, 10.2).
  `None` where a change does not decode, so that the application says
  it cannot tell, and the rehearsal goes on as it would.
- **Every record decoded** in a rehearsal, after the copy and before
  the plan, whether or not a job changes its family (the
  user's words of 2026-11-11): each family of each set is walked over its
  ranges of `A`'s maps (3, `Keys`), each record of a registered family
  decoded as a generic value of its layout's text from its value type's
  root (`layouts.md`, 4), and each other record as a generic value of
  its family's `current` from its range's root; each frame of each
  transcript and turns log, its body and its turn each as a generic
  value of its layout's text, from its frames' root
  (`transcripts.md`, 6). A start
  never does it: its codecs decode each record as read. The first
  record that does not decode ends the
  rehearsal there, with `Event::Undecodable` naming its family, version
  and place, and the plan does not run; all decoding, `Event::Decoded`
  with the count, and the plan runs. A store 1 refuses (ahead, behind,
  unleveled, below the floor, unregistered: `records.md`, 3; `layouts.md`, 5.1) is not
  walked: the refusal, the same as a start's, says why.
- **Going back** (`Store::back`, over the deployment's directory): once
  an upgraded generation is active, its previous one, `A`, stays on disk
  (`README.md`, section 2), and `active` may name it again until `A + 1`'s first
  commit of its own, the first past the position its manifest names
  (2, steps 6 and 8). It takes the lock, rolls an outstanding restore
  intent forward first (`restoring.md`, 5), then reads `active` and `A + 1`'s
  manifest, opens `A + 1`'s store as 1 opens one (skein-kv's recovery
  alone writes), and compares its last commit with the manifest's
  position. Equal, it replaces `active` by io's durable replace,
  expecting the digest it read, to name `A`, and answers `Back`; `A + 1`
  is then a stray the next start removes, and that start opens `A` as it
  was, cutting the transcripts to `A`'s lengths (`transcripts.md`, 4):
  `A`'s files are as `A` left them, those it shares with `A + 1`
  included, since nothing `A + 1` did (upgrading, snapshotting, this
  recovery, the stray's removal) writes a shared file (2, step 2). Past it, or with
  no previous generation or no manifest, it refuses, `Back { why }`,
  changing nothing: the upgraded generation has made commits of its own,
  which may have acted beyond the store, and going back is restoring a
  backup. A restored generation names no predecessor in its manifest, so
  going back refuses it `NoPrevious` (`restoring.md`, 4.4).
- **Going back past the first commit:** the previous build is started
  over the store as it stands, from the first build that reads the
  layouts' order (`layouts.md`, 5.6). Its open plans the newer build's
  layouts to keep their values and checks every value at them before
  anything is written: it serves, keeping what the newer build
  committed, or refuses by value type and count (`opening.md`, 1.8,
  `Layout { why: Unkept }`), changing nothing, and restoring a
  checkpoint taken before the newer build started is then the way back
  (temper's `protocol/checkpoints.md`, 2.5). A rehearsal of the previous
  build over a copy of the stopped deployment says which first.

```rust
Request::Rehearse { owner: Token, directory: Root, scratch: Root, installation: [u8; 16], reach: Reach } // `directory` never written; `scratch` emptied, never removed
pub enum Reach { Whole, Openings }                     // `Openings`: `Rehearsing`, then `Closed` unasked
Adoption::Rehearse { scratch: Fd, reach: Reach }      // the host's adoption (`host.rs`), beside `Back` and `Checkpoint`
Event::Rehearsing { owner: Token, wires: Option<Box<[KeptWire]>>, openings: Box<[WorkerOpening]>, workers: Option<Box<[KeptWorker]>>, linked: u32, copied: u32 } // before `Opened` or `OpenRefused`; files linked into the scratch and copied (`Whole` links none)
pub struct KeptWire { pub name: Box<str>, pub version: u32 }                               // `None`: A's last start kept none
pub struct KeptWorker { pub number: u32, pub name: Box<[u8]>, pub withdrawn: bool }        // a stored worker change, in number order; `None`: one does not decode
pub struct WorkerOpening { pub worker: Box<str>, pub wires: Box<[OpenedWire]>, pub output: u32, pub terms: Box<[OpenedTerm]>, pub at: Wall } // in the workers' names' order
pub struct OpenedWire { pub name: Box<str>, pub layout: [u8; 32], pub text: Box<[u8]> }   // `records.md`, 3
pub struct OpenedTerm { pub kind: u16, pub largest: u32 }
Write::Opening(WorkerOpening)                         // replaces that worker's row; `Write::Wires` clears them all
Event::Decoded { owner: Token, records: u64 }                                              // every record of A decoded; the plan runs
Event::Undecodable { owner: Token, family: Box<str>, version: u32, at: Undecoded }          // the rehearsal's last event but `Closed`
pub enum Undecoded { Record { map: KeptMap, key: Box<[u8]> }, Frame { transcript: Box<str>, offset: u64 } }
Request::Back { owner: Token, root: Root }
pub enum BackRefusal { Busy, NoPrevious, Committed { manifest: u64, last: u64 }, Unreadable }
Event::Back { owner: Token, to: u64 }                 // `active` names generation `to`
Event::BackRefused { owner: Token, why: BackRefusal }
pub fn levels(sets: &OwnerSets) -> Levels             // the binary's levels, for its versions' lines
Request::Close { owner: Token }                       // a rehearsal's end, in any state; refused for any other store
Event::Closed { owner: Token, removed: bool }         // `scratch` emptied; false on a file's fault: the next rehearsal removes the rest
```

The scratch is a `Root` like the directory: a directory the
application makes under its cache directory and adopts before it
asks, as it adopts the directory. The store never removes that directory, only what is under it, deepest
first as 1, step 5 removes a stray; making and removing the directory
itself are the application's. A rehearsal's store refuses
`Request::Switch`, though its root never asks, and its close
(`Request::Close`; an engine's store ends with its process and has
none) empties the scratch, `removed` saying it is empty; a rehearsal
that crashed leaves entries there, which the next rehearsal given that
scratch removes before it copies. An application that makes a new
scratch for each rehearsal removes those left behind itself, as
temper's does (temper's `protocol/binaries.md`, section 2). Its jobs
answer their counts as a start's do.
