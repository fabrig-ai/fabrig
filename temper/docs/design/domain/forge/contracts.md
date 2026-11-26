# Contracts of the goal for temper's own writes

The shapes the goal's changes share while it runs (`../../draft/design-economy.md`, 4.3), for the record of temper's writes (`writes.md`). They bind as any delivered section does; the goal's last change deletes this document once the code holds them.

## 1. The record's types

In `temper-engine-domain-forge-vocabulary` (`boundary.rs`), beside `Hold` and `BranchHead`:

```rust
/// One branch's record of temper's writes (domain/forge/writes.md, 1).
pub struct Writes {
    pub name: Name,               // What::Branch
    pub head: Option<Commit>,     // writes.md, 1: the head; none while absent or unknown
    pub next: u64,                // the next entry's order, from 1
    pub entries: Box<[Written]>,  // ascending order, at most WRITES
}
pub const WRITES: usize = 32;     // writes.md, 6
pub struct Written { pub order: u64, pub operation: Operation, pub sender: Sender, pub state: WriteState }
pub enum Operation {
    Push { before: Option<Commit>, after: Commit },                     // a worker's, leased on `before`
    Create { after: Commit },
    Delete { before: Option<Commit> },
    Update { pull: u64, before: Commit, base: Commit, after: Option<Commit> },
    MergedInto { pull: u64, before: Option<Commit>, after: Option<Commit> }, // on the base branch
    Merge { pull: u64, head: Commit },                                  // on the head branch
    Open { pull: Option<u64>, base: Box<[u8]>, head: Commit },
    Close { pull: u64 },
    Reopen { pull: u64 },
    Retarget { pull: u64, from: Box<[u8]>, to: Box<[u8]> },
    Writer { before: Option<Commit> },                                  // converted (writes.md, 7)
    Compare { after: Commit },                                          // converted (writes.md, 7)
}
pub enum Sender { Attempt { task: u64, attempt: u64 }, Entry { task: u64, entry: u64 }, Previous }
pub enum WriteState { Admitted, Sent, Confirmed, Refused, Uncertain }
```

`Stored` gains `Writes(Writes)`, kept in temper's store as a family member of its own, one row per branch keyed by its `Name` (`../../protocol/store.md`, 2), its entries' list bound `WRITES`.

## 2. Judging and settling

In `temper-engine-domain-forge-vocabulary`:

```rust
/// A hint's pusher for the read's commit, where a hint named it (skein's forge/client.md, 4).
pub enum Actor { Unknown, Writer, Other }
/// A read against its branch's record (writes.md, 4).
pub enum Seen {
    Current,                          // the head
    Explained { order: u64 },         // unsettled entries up to `order` made it; now the head
    Behind { order: u64 },            // an older state of temper's; `order` the entry it is older than
    Owed,                             // a readback or comparison read is out; judged again at its answer
    Drift { nearest: Option<u64> },   // no entry explains it; the nearest entry by order
}
```

In `temper-engine-domain-forge`, on its `Domain`, each in the caller's decision:

- `admit(&mut self, name: &Name, operation: Operation, sender: Sender, out) -> Result<u64, Full>`: one entry, *admitted*, its order returned; `Full` names the record's unsettled entries' senders (writes.md, 6).
- `observe(&mut self, name: &Name, observed: Option<Commit>, actor: Actor, out) -> Seen`: judges a branch read and settles what it explains; asks the readback or comparison read itself and answers `Owed` meanwhile.
- `observe_pull(&mut self, head_branch: &Name, pull: u64, read: &skein_forge::api::Pull, out) -> Seen`: a pull read, its state and base by the pull's entries, its head and base commits by their branches' records (writes.md, 4).
- `finished(&mut self, task: u64, attempt: u64, lost: bool, out)`: settles an attempt's unsettled entries by readback once it answers, or makes them uncertain when it is lost (writes.md, 3).

Each change that needs one of these and finds it missing adds it with that name and shape.

## 3. The admission ask

`temper-worker-wire` (`temper-protocol-worker-wire`'s schema), each added last, its default as given, with no version move:

```
record Repository { … admit: bool = false }      # the engine asks this repository's pushes be admitted
record Admit      { tag: u64  branch: bytes 1024  expected: option fixed 32  commit: fixed 32 }
record Unadmitted { words: bytes 512 }
enum   Ask        { reds, results: CheckResults, landed: Head, admit: Admit }
enum   Answer     { reds: Reds, kept, landed, unlanded, admitted, unadmitted: Unadmitted }
```

The worker asks `admit` before each push in a repository whose `admit` is true and pushes on `admitted` alone (`../worker/checkouts-push.md`, 1); the engine sets `admit` on every repository an assignment or a job gives to write, and answers (writes.md, 2).
