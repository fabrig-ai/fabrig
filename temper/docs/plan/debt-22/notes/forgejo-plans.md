# debt-22: Forgejo's plans onto data-carrying states

Binding for every task of the part `forgejo-states`. `P` is `skein/crates/skein-forge/src/forgejo/plans`.

## The searches

From the repository root, with `G=(-g '!**/tests/**' -g '!**/tests.rs' --type rust)` and `P` set. Each task's acceptance is its own file's rows of A and B reading 0; `plan-states-close` runs them whole.

```
A: rg -n --pcre2 "${G[@]}" '\blet\s+(Some\(|Ok\(|Err\()?(?:[a-z_]+::)*[A-Z]\w*(::\w+)+|\bstage\s*(==|!=)|^\s*(stage|mode): (Stage|Mode),|\benum Stage\b' "$P"
B: rg -n "${G[@]}" 'Option<(super::|crate::forgejo::plans::)\w+::Plan>' "$P"
C: rg -n 'length\(|_BYTES|checked_add' "$P/reads/size.rs"
```

At this plan's base: 18 own-enum `let` forms, 56 stage comparisons, 5 `Option<Plan>`s, 46 in C. Tuple scrutinees read 0 in the plans already.

**Exempt, by name:** `failure.rs`'s `_` on a `u16` status; `turn::Mode` as `new`'s argument only; a `let … else` or `if let` on a plain `Option` or `Result` that isn't a lifecycle; tests. `crate::measure`'s constants used for running per-row charges in `commits.rs`, `turn.rs`, `contents.rs` and `reads.rs` agree with the measure and aren't a second one.

**The dialect is asked of nothing here.** No plan reads a dialect. What these plans re-ask is the call: `self.read`, `self.write` and `self.mode` are read 51 times, and the repository row is re-asserted with `row.as_ref().expect(..)` 19 times. After this part, a state holds what it needs: the call is matched once on entry, and no `row.as_ref().expect(..)` is left.

## The form

- **One `State` enum per plan,** each variant holding what that state holds. What every state reads stays in the struct beside it (below). `Stage`, `Mode` as a field, and the bags of fields go. `skein/docs/foundation/programming-model.md`, 5.4.
- **The idiom:** move out, leaving `State::Ended`, and assign the match's result back. Handlers take the source's data by value and return the target. A plan's owner hears nothing after its `Done`, so `Ended`'s cell is `unreachable!("...")`, unless a test feeds it or the plan says it ends at most once (`start::Plan`).
- **A document of the wrong kind** is spelled as `writes.rs` already does: `match document { Some(d::Document::Compare(compare)) => .., Some(_) | None => return Err(..) }`. `Body::Document(_)` the same. The answer to an exchange isn't a state × event cell a new `Document` variant should break.
- **`error()`** reads the state: see `writes-states`.
- **A row a state's exchange can't decode** answers `Err(api::Error::InvalidAnswer)`, as `artifact.rs` does. `reads.rs`'s `Rows::None` answers `TooLarge` and a wrong kind on full `Rows::Comments` a cut today; no test feeds those by hand and the wire can't reach them, since the exchange's decoder picks the row kind. If a test does see a difference, stop and ask.
- **Allocation:** `List::with_capacity`'s sizes and their timing decide `TooLarge` and the heap's peak, so each list keeps its capacity and the moment it's made. `skein-forge-machine-world`'s `full_forgejo_holders_stay_under_the_ceiling` holds it: boxing a large variant is yours; over budget, stop and ask with the measurement.
- **A `#[expect(clippy::too_many_lines)]` that stops firing goes,** or the build fails.
- **The sketches below bind the variants' meaning.** A variant may split or merge where a field's life says so; no field meaningful in only some states stays beside the state.

## Public

`plan::Plan`'s and each plan's public methods stay, so `reads.rs` doesn't change when a leaf plan does, and no reader outside `P` changes (`skein-forge-connection`, `tests/forge/machine`). `Stage`, `Mode` and `Rows` are private. Nothing here is stored or on the wire, and plans hold no credential: `#[derive(Debug)]` stays.

## Tests

Each task runs `heavy cargo nextest run -p skein-forge --lib` and `heavy cargo nextest run -p skein-forge-connection --lib`; the last before pushing, also `heavy cargo nextest run -p skein-forge-machine-world -p skein-forge-protocol-world -p skein-forge-client-world --test suite` and `heavy cargo nextest run -p skein-fake-peers-tests`. The conformance suites stay out. No existing test's assertion changes. Pins go first, on the base's code, in their own commit, and pass unchanged after. Search before pinning: skip a row an existing test holds, and say so. A pin that turns out only to copy a production arm is cut before the task ends.

## The shapes

```rust
// artifact.rs. Beside: row, run, name, max
enum State { Listing { id: Option<u64> }, Run { id: u64 }, Zip { head: api::Commit }, Ended }

// start.rs
enum State { Asked(Read), Ended }

// turn.rs. Beside: row, size. turn::Mode stays new's argument
enum State {
    Runs { workflow: Box<[u8]>, commit: api::Commit, runs: List<api::Run>, count: u32 },
    Tree { entries: List<api::TreeEntry>, count: u32 },
    Run { expected: u64 },
    Jobs { run: api::Run, jobs: List<api::RunJob> },
    Steps { run: api::Run, jobs: List<api::RunJob>, position: u32 },
    Ended,
}

// actions.rs. Beside: row, limit, count. Mode goes
struct Checks { head: api::Commit, shape: Shape, statuses: List<api::Status>, position: u32, size: u64, resolve_jobs: bool }
enum Shape { Checks, Statuses { ci: api::Ci, more: bool } }
enum State {
    Listing { checks: Checks, page: u32 },
    RunLookup { checks: Checks, run_number: u64, job_index: u32, run: Option<u64>, missing: bool },
    JobsLookup { checks: Checks, job_index: u32, run: u64, missing: bool },
    Run { attempt: u32, max: u32 },
    Job { attempt: u32, max: u32, step: Option<api::Step> },
    Log { attempt: u32, step: Option<api::Step>, truncated: bool },
    Ended,
}

// contents.rs. Beside: row, limit, size. Mode goes
struct Sides { before: .., after: .., changed: List<d::ChangedFile>, files: List<api::File>, position: u32, ask: Ask }
enum Ask { Pull { head: .., more: Option<bool> }, Compare { commits: List<api::Commit>, contains: bool } }
enum State {
    Pull { number: u64, head: .., page: u32, changed: .., files: .. },
    Files(Sides), Compare(Sides), Reverse(Sides),
    Before { sides: Sides, missing: bool }, After { sides: Sides, missing: bool },
    File { head: .., path: .., max: u32, missing: bool },
    Directory { head: .., path: .., listing: Box<Listing> },
    Ended,
}

// writes.rs. Beside: repository, request, status, retry
struct Open { title: .., body: .., head: .., base: .. }
struct Looking { missing: List<u64>, position: u32, add: List<Box<[u8]>>, remove: List<Box<[u8]>> }
enum State {
    New { write: api::Write }, Resolving { write: api::Write }, Writing { row: Repository, write: api::Write },
    Merged { row: Repository },
    OpenHead { row: Repository, open: Open }, OpenBase { row: Repository, open: Open, head: ObjectId }, OpenCompare { row: Repository, open: Open },
    Reviewers { row: Repository, number: u64, reviewers: Box<[u64]> },
    Searching { row: Repository, number: u64, looking: Looking },
    Adding { row: Repository, number: u64, remove: List<Box<[u8]>> },
    Removing { row: Repository },
    Ended,
}

// reads.rs, reads/done.rs. Beside: repository, request, cut, count, bound. Rows, stage, read go
enum Delegate { Artifact(artifact::Plan), Commits(commits::Plan), Turn(turn::Plan), Contents(contents::Plan), Actions(actions::Plan) }
struct Roster { rows: List<api::Collaborator>, position: u32, more: bool, size: u64 }
enum Whose { One, Each(Roster) }
enum State {
    New { read: api::Read }, Resolving { read: api::Read }, Locating, Settings, Delegated(Delegate),
    Items { rows: .., now: Time, size: u64, limit: u32 }, Reviews { format: .., rows: .., size: u64, limit: u32 },
    Remarks { rows: .., more: bool, page: u32, limit: u32, size: u64 }, Issue { row: Repository, after: u64 },
    Comments { item: api::Summary, rows: .., more: bool, size: u64, after: u64 },
    Pull { row: Repository }, PullFor { row: Repository, head: .., base: .. },
    Pulls { row: Repository, head: .., base: .., pull: d::PullRequest, found: bool, page: u32, limit: u32 },
    Status { row: Repository, pull: d::PullRequest, actions: actions::Plan }, Branch { format: .. },
    ProtectionBranch { row: Repository }, Protection, Branches { prefix: .., rows: .., size: u64, missing: bool },
    Collaborators { row: Repository, rows: .., size: u64, limit: u32 }, Permission { row: Repository, whose: Whose },
    Searching { row: Repository, user: u64, whose: Whose }, Ended,
}
```

No plan's `commits.rs` changes: it has no stages.
