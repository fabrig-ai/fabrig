# Bounded waits: the changes' contracts

The shapes goal 15518's changes share and no other area reads
(`../../draft/design-economy.md`, 4.3), while the goal runs: what the
first change, *Report procedure waits*, makes for the others to build on,
and what each later one makes for the next. The lasting design is
`waits.md`; this document goes with the goal's last change.

## 1. The figures

The forge's `Limits` (`temper-engine-domain-forge-vocabulary`'s
`limits.rs`) gains `pub waits: jig_domain_core_vocabulary::Figures`,
which the root fills from its own `Limits.waits`
(`temper-engine-domain-limits`), the value `core_limits` gives the core.
A step reads `env.limits.waits.step`, `.room` and `.sign_off`. Every
world that builds forge limits gives it a value.

## 2. The codes

`temper-client-protocol-wire`'s module `waits` (`src/waits.rs`) holds
`waits.md`, 3's table as constants and two names:

```rust
pub const CHANGE_TAKING: u32 = 101; // one per change state, CHANGE_<STATE>, through CHANGE_JUDGING = 122
pub const LANDING_PREFIXES: u32 = 201; // LANDING_<WAIT>, through LANDING_ROOM = 214
pub const CLOSER_CLOSING: u32 = 301; // CLOSER_CLOSING, CLOSER_LANDING, CLOSER_REPORTING, CLOSER_MAKING
pub const THING_ENTRY: u32 = 1; // THING_ENTRY, THING_TASK, THING_PULL, THING_RUN, THING_GATE, THING_ROLE, THING_RULE, THING_FORGE
pub fn step_name(code: u32) -> Option<&'static [u8]>;
pub fn thing_name(code: u32) -> Option<&'static [u8]>;
```

Where the crate graph keeps them out of that crate, the first change puts
them where both the forge's vocabulary and the client's tools reach, and
says so; the others use what it made.

## 3. The report

`temper-engine-domain-forge`'s module `waits` (`src/waits.rs`):

```rust
/// Computes the task's waits by its procedure and pushes `Request::Waits` where they differ
/// from the last it pushed for the task, or always, an empty list included, while catching up.
pub(crate) fn report(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>);
```

It reads one function per procedure, each in its own file, `waits/change.rs`,
`waits/landing.rs` and `waits/closer.rs`:

```rust
pub(super) fn waits(d: &Domain, env: &Env<Limits>, task: u64) -> List<Reported>;
```

The first change writes each as the task's room record (4) where it holds
a place in the line and none otherwise, and calls `report` after each
step of a change, a landing and a closer, as a waiter joins or leaves
the line, and for every live change, landing and closer at the end of
`Event::CatchUp`. *Report a change's waits*, *Report a landing's waits*
and *Bound closers, makings, releases* each fill their own file, and
call `report` after any other event of theirs that moves a wait. The top
keeps the last list it pushed per task, unstored, and drops it at the
task's end.

## 4. Room

`temper-engine-domain-forge`'s `room.rs`:

```rust
/// The task's `room` record where it holds a place in the line (waits.md, 7), else none.
pub(crate) fn waited(d: &Domain, env: &Env<Limits>, task: u64, step: u32) -> Option<Reported>;
```

The first change makes its object (the waiter's step; the entries holding
the room, oldest first, at most four, `THING_ENTRY` with the writer as
owner), its *since* (the waiter's join, kept with its place, unstored;
the start for a place restored) and its bound behind the oldest holder's
writer. *Bound a wait for room* makes the bound `waits.md`, 7 gives,
keeping each entry's writer's end, and names, in its report, the function
the holds read for it.

## 5. A landing's last advance

*Bound a landing from its advance* keeps, for each live landing, its last
advance's wall time and the reads since it (`../landing-waits.md`, 3),
unstored, and names, in its report, the function that reads them;
*Report a landing's waits* reads them for its records' *since*.
