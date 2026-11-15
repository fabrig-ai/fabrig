# Jobs at open

How an owner rewrites what its store holds when a binary needs it in
another shape than the store keeps: a key layout changed, a record split
or folded, a summary to write for what already ended, a written change
of a value's layout. A job is an owner's walk over ranges of the map,
run once per store by the open, after the patches and the registry's
commit and before the root starts, in pages: the store reads each page,
hands it to the job's owner, and commits what the owner answers (temper's
`draft/store-shape.md`, sections 3 and 4, step 4). Milestones S5 to S8
of temper's `draft/store-shape.execution.md` bring the jobs; this
document designs what runs them.

- **Its identity** (1): its owner and its name, stable and never reused.
- **When it runs** (2): when the binary declares it and the store has
  no row saying it is done, in the new generation the open writes in,
  once.
- **Its pages** (3): the store reads its ranges at the binary's layouts,
  as any read, and hands each page to its owner through the shell, as a
  written site is handed (`opening.md`, 3); the owner answers the writes
  for the rows it consumed; the store commits them and goes on after
  them. Its end is a commit of the owner's last writes and the job's row.
- **Written jobs** (4) convert a registered value type's values from a
  layout no plan reaches (`layouts.md`, 6).

## 1. A job

Each owner's patch set names its jobs, in the order they run:

```rust
pub struct PatchSet {
    // ... as `opening.md`, 3
    pub jobs: &'static [Job],           // in the order they run
}
pub struct Job {
    pub name: &'static str,             // unique within its owner, never reused: its identity, at most 64 bytes
    pub keys: &'static [KeyRange],      // the ranges its pages walk, in order (`layouts.md`, 2); the application's relative to its tag
    pub converts: Option<Converts>,     // a written job's layout (4)
    pub counts: &'static [&'static str], // the names of what its answers count, in order (3); empty for a job that counts nothing
}
pub struct Converts {
    pub family: &'static str,           // a registered family of its set
    pub value_type: u16,                // its value type's number (`layouts.md`, 2)
    pub from: [u8; 32],                 // the layout it converts from, by fingerprint (skein's `data/codec/README.md`, 9.4)
}
```

- **Its identity** is its owner and its name. The store keeps a row per
  job done, `00 08 <owner> <name>` (`layouts.md`, 3), so a binary that
  still declares a job a store has run does not run it again, and a
  name once used is never given to another job.
- **Checked where the set is admitted,** beside `opening.md`, 3's
  checks, a failure refused `Set` with patch 0: names given once in a
  set and within their bound; ranges non-empty, ordered and within the
  owner's tag (the application's relative to it), a job's ranges in
  the map, never the secrets' store; a written job's family a registered
  family of the set and its value type one of that family's, its ranges
  that value type's; a job's count names at most 16, each given once
  within the job, 1 to 32 bytes of ASCII letters, digits and `-`.
- **jig's jobs:** `charter-versions`, a written job (4);
  `milestones`, which folds the tasks' milestones into their attempts
  and holds and erases them (`../../domain/tasks/lifecycle.md`, 12.4);
  and `live-task-parts` and `projection-parts`, which put a store's whole
  live tasks and projection rows into their parts (`parts.md`, 4).
- **temper's jobs** (temper's `protocol/store.md`, 4): `change-parts`,
  then `deleted-repository-forge-rows`, the first job that counts: it
  drops the forge rows naming a repository whose record is gone, its
  answers counting them per family, and reads the repositories and
  forges stored while each page is out (3).
- **Order:** the sets' order, `Owner`'s, upstream first (skein, smith,
  jig, the application), and within a set its own; each job runs to its
  end before the next starts, so a later job reads what an earlier one
  wrote.

## 2. When a job runs

- **Pending:** a job of the binary's sets with no row in the store.
  A pending job makes the open write before serving (`layouts.md`, 5.5):
  in a new generation, made by links, after the patches and the
  registry's commit, and before the seal and the check (`opening.md`,
  2, step 4).
- **Done:** its end's commit writes its row, `Job { ended }`, the number
  of that commit. A job is done once per store, whatever binaries come
  after.
- **An empty store** has nothing to convert: its first commit
  (`README.md`, section 5) writes a row for every job its binary
  declares, so none ever runs over it.
- **Within one start:** a job's progress is the open's, in memory, never
  a row. A start that fails or is refused before its switch leaves the
  active generation as it was and its new one a stray (`opening.md`, 2),
  and the next start begins again from the active one, its jobs from
  their first page. A job's owner therefore makes no effect beyond the
  store, and reads what it needs through the store alone.
- **A rehearsal** runs the pending jobs as a start does, over its
  scratch (`opening.md`, 5).

## 3. Pages

- **A page** is the next rows of the job's ranges, in key order, from
  the first key past the last one consumed: at most `limits.store.ops`
  rows and the store's page bytes, as a patch's page is
  (`opening.md`, 2, step 4), read through the read edge
  (`layouts.md`, 4), so each value is at the binary's layout with its
  family's version at its head. A written job's page holds only its
  value type's values at its layout, as stored (4).
- **Handed to its owner:** `Event::Job` carries the job's owner and
  name and the page; the shell routes it to that owner's jobs
  (`jig_protocol_store::jobs::page` for jig's, the application's own
  dispatch for its), as it routes a written site (`opening.md`, 3), and
  answers `Request::Job`. While a page is out, the owner may read other
  ranges of the map through `Store::patch_page`, as a written site's
  owner may.
- **The answer:** the writes, puts and erases of whole keys (the
  application's relative to its tag), each within skein-kv's bounds,
  together within the store's commit edge (`records.md`, 4) less the
  job's row; the last key of the page it consumed, `through`, none
  meaning the whole page; and `counts`, one number per name the job
  declares, in its order: what this answer counts, such as the rows its
  writes drop. A job that declares none answers none.
- **Its room:** each page says what its answer may hold, `room`: how
  many writes and how many bytes, the commit edge's less the job's row
  and the commit's frame, each write's bytes counted as the store counts
  them (`JobWrite::bytes`). A job whose writes are not one per row
  consumed (a fold erasing a row and putting another) fits its answer
  to it by `through`. The store commits the writes, one commit per
  answer through the component's path (`commits::submit`, so each put
  takes the binary's layout id, `layouts.md`, 4), then reads its next
  page after `through`. An answer that consumes no row of a non-empty
  page is refused, since the job would not end.
- **Its end:** when its ranges hold no row past the last consumed, the
  store hands one last page, empty and marked `end`, and commits that
  answer's writes with the job's row.
- **Refused:** the owner may refuse a page, naming a key and why; the
  start is refused, `OpenRefusal::Job { owner, job, key, why }`, with the
  active generation unchanged (`opening.md`, 1.8). So is an answer past
  its bounds, an empty page's `through`, a `through` that is not a
  key of its page, or `counts` that are not one per declared name.
- **Counted:** as each job ends, the store answers `Event::Ran` with
  its owner, name, the rows it was handed, the commits it made and its
  counts, each declared name with the sum of its answers' numbers, the
  last page's included, a sum saturating at `u64::MAX`, in the job's
  order, for the shell's line, as `Patched` is a patch's. The store
  sums what the owner counts and judges none of it: what a count means
  is its job's.

```rust
Event::Job { owner: Token, of: Owner, job: Box<str>, page: JobPage }
Request::Job { owner: Token, answer: Result<JobAnswer, JobRefusal> }
Event::Ran { owner: Token, of: Owner, job: Box<str>, rows: u64, commits: u64, counts: Box<[JobCount]> }
pub struct JobCount { pub name: Box<str>, pub count: u64 } // a declared name and its answers' sum
pub struct JobPage { pub rows: Box<[JobRow]>, pub end: bool, pub layout: Option<JobLayout>, pub room: JobRoom } // `layout`: a written job's, on its first page
pub struct JobRoom { pub writes: u32, pub bytes: u64 } // what the answer's writes may hold together
impl JobWrite { pub fn bytes(&self) -> u64; } // a write's bytes as the store counts them against `room.bytes`
pub struct JobRow { pub key: Box<[u8]>, pub value: Box<[u8]> }
pub struct JobLayout { pub root: Box<str>, pub variant: Option<Box<str>>, pub text: Box<[u8]>, pub held: Box<[Box<[u8]>]> }
pub struct JobAnswer { pub writes: Box<[JobWrite]>, pub through: Option<Box<[u8]>>, pub counts: Box<[u64]> } // `counts`: one per the job's names
pub enum JobWrite { Put { key: Box<[u8]>, value: Box<[u8]> }, Erase { key: Box<[u8]> } }
pub struct JobRefusal { pub key: Option<Box<[u8]>>, pub why: Box<str> }
// each owner's dispatch, jig's in jig_protocol_store::jobs, the application's in its store crate:
pub fn page(job: &str, page: &JobPage) -> Result<JobAnswer, JobRefusal>;
```

`OpenRefusal::Job { owner: Owner, job: Box<str>, key: Option<Box<[u8]>>, why: Box<str> }`.
jig's and the application's dispatches name no job at first: each
milestone that brings one adds its arm, and a name a dispatch does not
know is its refusal, `why` saying so.

## 4. Written jobs

A written job converts a registered value type's values to the
binary's layout from a layout that no derived plan reaches, or from one
whose values are not all the root it names (`layouts.md`, 6; skein's
`data/codec/layouts.md`, 2). jig's first is `charter-versions`
(`layouts.md`, 2).

- **Its pages** hold only the values of its value type whose head is the
  id of its `from` layout in the registry, as they are stored: the read
  edge leaves them unconverted, and its first page carries that layout's
  root, variant and texts (`JobLayout`), from the registry's rows. Values
  at other layouts are not handed to it.
- **Its owner's function** reads each old value as a generic value of
  that text (skein's `data/codec/README.md`, section 8; the head is the
  layout id, which the function passes over) and writes the new one by
  its generated code, a put at the same key, whose head the write edge
  maps.
- **The open's check** takes its layout as converted (`layouts.md`,
  5.3), pending or done; once done, no value holds the layout, and the
  next open that writes retires it (`layouts.md`, 5.4).
- **Its cost** is a walk of its value type's ranges, a put per value at
  that layout: a written change is the one upgrade that costs per
  record, and its owner weighs it so.

## 5. Tests

- **Step tests,** in `jig-protocol-store`, the tests the shell: the
  test owners' sets (`opening.md`, 3) each declare a job over their
  fixture ranges, answered by the test with puts and erases: a job runs
  in pages within the commit edge and marks itself done; a later start
  runs it no more; jobs run in the sets' order; an empty store's first
  commit marks every job done; a refusal, an answer past its bounds and
  an answer that consumes nothing each refuse the start with the active
  generation unchanged, and the next start runs the job from its first
  page; a written job is handed only its layout's values, with its
  texts, and its layout passes the check; a job that declares counts
  has its answers' numbers summed across its pages into `Ran`, by name
  and in order, an answer with the wrong number of counts refusing the
  start, and a set naming a count badly refused at admission.
- **The shell's routing,** in temper's store route: `Event::Job` of each
  owner reaches that owner's dispatch and comes back as `Request::Job`.

## 6. Open questions

- **Progress across starts:** a job's progress lives and dies with its
  start's new generation. A job too long for one start would need its
  cursor kept in its row and the generation kept across starts.
- **Files:** a job that writes files beside the store (S7's task logs)
  needs them made in the new generation and synced before the commit
  that names them; S7 designs it.
