## skein review: sim, shell, kv, channel and their tests

The weakest code in this area is in two step crates. `skein-kv` and `skein-channel` break the step-code rules they claim to follow, in ways that cost clarity. The second big cost is duplication: the POSIX code in shell's two backends, the test harnesses, and the types that join sim, fake-machine and sim-world.

### 1. Scope read

**Read closely:**
- **shell:** `lib.rs`, `kernel.rs`, and the facade/backend pairs, diffed side by side: `terminal`, `group`, the `process` helpers, private-file reads, `make_root`, `linux.rs` against `darwin.rs`.
- **sim:** `lib.rs`, the `sim.rs` types and `check`, `fds_full` and `assert_no_open_fds`; the `processes.rs` types.
- **fake-machine:** `lib`, `face`, and the types in `fs`.
- **sim-world:** `lib`, `program`, `meter`, and in `world.rs` the struct, builders, `turn` and `drop_killed`.
- **kv:** `store.rs` (`up`, `fire`, `snapshot_up`), `frame`, `key`.
- **channel:** `machine.rs` (state, close paths, read path).
- **conformance:** `lib.rs`, samples of `scenarios.rs` and `files.rs`, both backends, the case lists.
- **test harnesses:** `tests/sim/src/support.rs`; `tests/kv` world and store; the `tests/ring` harness against `tests/readiness`; the channel test harnesses.

**Skimmed:** `linux/ring.rs`, `readiness.rs`, shell settings and drive, sim trace and config, fake-machine `fs` internals, `skein-fake-channel`, the real-kernel suite files, and the `tests/sim` suites other than `files.rs`.

### 2. Scores

| Criterion | Score | Why |
|---|---|---|
| Cleanliness | 4 | Comments earn their place; little dead code. Noise comes from kv's module-wide `#![expect]`s and sim-world's 20 `.expect("a status per process")` calls. |
| Organization | 3 | The linux/darwin trees don't match file for file. Large files mix concerns (`processes.rs` 1,520 lines with 430 lines of tests inside it; kv `store.rs` 1,092). |
| Taste | 3 | sim and conformance are careful. kv and channel lapse into flag soup and copied state-machine cells. |
| Naming | 3 | `Machine` is both a trait and a struct; `Handle`/`Opened`/an internal `Handle` name one thing; `fds` means sockets only; `sync.rs` holds every synchronous call. |
| Primitives | 3 | The conformance `Backend` trait is exactly right. Elsewhere: six descriptor tables, a duplicated `Item`, two handle types. |
| Elegance | 3 | kv's `up`/`fire` and channel's close paths don't follow the shape of their state machines; `World::turn` does five jobs. |
| Economy | 3 | Roughly 1,300 to 1,500 lines (the sum of the estimates below) could go, without touching rule-forced verbosity. |
| Complexity | 3 | Mostly in proportion. State spread across bools and options adds complexity the enums would remove. |

### 3. Strengths

- **The conformance suite is written once.** Each scenario is generic over `Backend`, returns a plain "seen" struct, and is judged by a `Check`. The sim and real crates only implement `Backend`. The Darwin differences are factored into `check_common` plus `check_darwin`, with XNU line citations (`scenarios.rs:176-205`).
- **`tests/sim/src/support.rs` is a model harness.** It is small and well named (`call`, `listener`, `pair`, `settled`), with `ROOM` named.
- **`tests/kv/tests/suite/golden.rs:16-29` carries a derivation beside every limit**, which is what section 10 asks for.
- **The shell already has a tool for sharing POSIX code.** `listener_calls.rs` is compiled into both unsafe boundaries via `#[path]` (`linux.rs:29`, `darwin.rs:20`), and SAFETY comments are kept per module.

### 4. Findings, ranked by payoff

**1. kv's store is not the 5.4 step machine its design describes** (elegance, primitives, complexity)
- **Evidence:**
  - kv.md section 1 says "ordinary `no_std` step machine". But `store.rs:3-10` opts out of `disallowed_types`/`methods`, `struct_excessive_bools`, `too_many_lines` and `too_many_arguments` for the whole module. There are 10 `loop`/`while` sites (`store.rs:259,301,402,436`, `key.rs:89`, `frame.rs:92`, `snapshot.rs:79`, …).
  - `up` matches on a tuple with a catch-all (`store.rs:637`, `:844 (phase, event) =>`), and `snapshot_up` ends in `(_, _) => unreachable!` (`:1086`).
  - Snapshot progress lives in five places: `Phase::SnapIo{action}`, two parallel enums `SnapAction`/`SnapStage` (`:59-86`), `Snapshot{file: Option<Token>, stage}`, `snapshot_start`, and `has_snapshot`/`has_temp`. As a result `snap.file.expect("snapshot file opened")` appears 6 times.
  - `Store` carries 6 bools (`:109-130`).
  - Copied cells: the read loops for `ReadingSnapshot` and `ReadingLog` (`:675-693` and `:711-733`); "start the next batch or close" (`:758-764` and `:784-790`); trailer encoding (`:942-948` and `:962-966`).
  - `b"snapshot.tmp"` is written as a literal 5 times. The bare sizes `12`/`8_usize` in chunk sizing (`:921-923`) restate `snapshot.rs`'s layout.
- **Why it matters:** this is a durable store. Its recovery logic is the hardest code in the area to review, and the subset's guarantees are switched off exactly there.
- **Fix:**
  - One enum per machine where each variant holds its own data, e.g. `Phase::Snapshot { log, offset, step: SnapStep }` with `SnapStep::Writing { file, at, cursor, rows, payload }` and so on. Drop `SnapStage`: the in-flight request *is* the stage.
  - One shared `Reading { file, len, at, bytes, then }` sub-state for log and snapshot reads.
  - One handler per cell, using the `mem::replace` idiom.
  - Constants for the file names, and `snapshot::ROW_OVERHEAD`.
- **Size:** about −200 to −250 of 1,092 lines, and most of the `#![expect]`s gone.

**2. kv re-implements lib's `Reader` and `Writer`** (economy, primitives)
- **Evidence:** `frame.rs:122-135` defines `u16_at`/`u32_at`/`u64_at`. `KeyReader::u64` (`key.rs:73-78`) duplicates `skein_lib::Reader::u64`. `frame::encode` works out the exact length first (`encoded_len`, `:20-29`) but then builds two `Vec`s with `extend_from_slice` (`:36-62`); `snapshot.rs:20-60` does the same. Channel's `frame.rs` uses lib's `Writer` and `Reader`.
- **Fix:** use `Writer::new(total)` and `put` for the encoders and `Reader` for the decoders. `KeyWriter` keeps only its escaping.
- **Size:** about −60 to −80 lines, and the `Vec` escape hatches in `frame.rs`, `snapshot.rs` and `key.rs` go.

**3. shell: POSIX code copied between `linux/` and `darwin/`** (economy, organization)
- **Evidence:**
  - `linux/terminal.rs` and `darwin/terminal.rs` are byte-identical apart from line 1 (90 lines).
  - In `group.rs`, `negative_group`, `signal_group`, `group_live`, `process_live` and the four tests are identical; only `child_id` and `group_has_other_members` differ (46 diff lines out of about 130 per side).
  - `read_private_file`, `check_private`, `read_file`, `read_opened` and `kernel_error` (`linux/startup.rs:20-159` against `darwin/files.rs:255-395`) differ only in the no-follow `open` and two casts.
  - `make_root` (`linux/roots.rs:40-69`, `darwin/files.rs:418-456`) differs only in `fsync` versus `F_FULLFSYNC`.
  - In `process.rs`, `cstring`, `action`, `block_sigpipe`, `usage`, `cpu` and `resources` are identical.
  - The same function also sits in differently named files per OS (`lock`: `linux/sync.rs` against `darwin/files.rs`; `last_errno`: `linux/sync.rs` against `darwin.rs`), so the trees can't be compared file by file.
- **Answer to your question:** sockets (`sin_len`, `SOCK_NONBLOCK` against `fcntl`), spawning (`pidfd_spawn` against fork), readiness (epoll against kqueue) and listing really are platform-specific. Terminal, group signalling, private reads and roots are POSIX.
- **Fix:** put the POSIX parts in shared files (for example `posix_terminal.rs`, `posix_group.rs`, `posix_startup.rs`) included the way `listener_calls.rs` is. Keep small per-OS functions: `open_nofollow`, `full_sync`, `pty_name`, `group_has_other_members`, `child_id`. Give matching files matching names.
- **Size:** about −350 to −400 of about 5,300 lines.

**4. channel's `Machine` keeps its output and terminal state in flags** (primitives, elegance)
- **Evidence:**
  - 11 bools plus several options, under `#[expect(struct_excessive_bools)]` (`machine.rs:116-144`).
  - The output right's lifecycle is spread over `output_right`, `output_granted`, `cancel_requested` and `cancel_sent` (`:129-132`). Combinations such as "granted with no right" are representable.
  - `cancel_requested = self.output_right.is_some()` appears 5 times (`:334,426,631,645,839`).
  - The "go closed" sequence (Closed, `read=Idle`, `clear_output`, cancel, `terminal_why`) is written about 5 times (`:321-325,420-426,627-632,641-646,835-840`).
  - `terminal_why` only means something in `Phase::Closed`.
  - `ping_queued` and `unsupported_queued` shadow the `QueueTag` of items already in the queue.
  - Bare numbers: `bytes.len() != 7` (`:666`), `reason == 3` / `auto_refuse(3)` (`:903,909`), `256` (`:327`).
- **Fix:** `enum Output { Idle, Asked{right}, Granted{right}, Cancelling{right, sent} }`, `Phase::Closed { why }`, one `fn shut(&mut self, why)`, and named constants.
- **Size:** about −60 to −80 lines; the larger gain is that impossible states go away.

**5. sim: six descriptor tables per process** (primitives, naming)
- **Evidence:** `Process` (`sim.rs:62-92`) holds `fds` (sockets only), `files`, `pipe_fds`, `inherited_pipes` (with a `process_open` flag), `pidfds` and `signal_fds`. Each cross-cutting question walks all six by hand: `check` (`:631-646`), `fds_full` (`:1507-1521`), `assert_no_open_fds` (`:507-527`).
- **Fix:** `descriptors: BTreeMap<Fd, Descriptor>` with `enum Descriptor { Socket(SocketId), File(File), Pipe(PipeEnd), Inherited(..), Pidfd(u64), Signal(SignalSource) }`, keeping inherited pipe objects outside the descriptor table. `fds_full` then becomes a `len()`, and `check` a single lookup.
- **Size:** about −60 lines, and `check` reads much more simply.

**6. sim / fake-machine / sim-world: two names for one handle, and a name clash** (primitives, naming)
- **Evidence:**
  - `skein_sim::Handle` (`machine.rs:28`) and `skein_fake_machine::Opened` (`fs.rs:50`) are the same `u64` newtype with the same `new`/`raw`. fake-machine also has an internal `struct Handle` (`fs.rs:246`). I counted 86 `Handle::new(x.raw())` / `Opened::new(x.raw())` conversions across skein and smith (for example `tests/kv/src/world.rs:42`, `tests/conformance/sim/src/lib.rs:160`, `face.rs`).
  - sim-world's `trait Machine` (`program.rs:91`) collides with fake-machine's `struct Machine`. So test crates alias it (`Machine as Files`) and each writes a wrapper `impl Machine for FileSystem { fn step(..) { skein_fake_machine::step(&mut self.files, ..) } }`: at least 12 of them (`tests/io` ×8, `tests/world`, `oauth-accounts`).
- **Answer to your question:** the dependency direction is clean (sim → fake-machine, sim → sim-world). The overlap is in these joining types. Program names are also known in two places: fake-machine's `program_of` (`face.rs:77-106`) and sim-world's `HostedProgram` registry.
- **Fix:** fake-machine uses `skein_sim::Handle` as its public type, and the internal struct is renamed `OpenFile`. Rename the trait (for example `Seam`) and ship one ready implementation for the fake machine.

**7. sim-world `World`: parallel vectors and builders that retype every field** (elegance, economy)
- **Evidence:**
  - `pids`, `hosted`, `finished`, `restart` and `submissions` sit parallel to `procs` (`world.rs:59-73`). They are pushed together (`:275-280`), removed together (`:528-533`), and read through 20 `.expect("a status per process")`-style calls. `programs` and `startup` are parallel too.
  - `with_machine` and `with_meter` (`:140-196`) each retype all 22 fields to change one type parameter.
  - `turn` (`:406-505`) mixes reaping, admission of hosted children, metering, digests, cut planning, submission and hosted exit.
- **Fix:** keep `procs: Vec<P>` (because `turned` takes `&[P]`) and put the rest in `Vec<HostState>`. Move the fields that don't depend on the type parameters into an inner `Core`, so each builder becomes 3 lines. Pull `admit_spawned` and `submit_or_cut` out of `turn`.
- **Size:** about −80 lines.

**8. Test harnesses and fixtures retyped** (economy, section 10)
- **Evidence:**
  - `tests/readiness/tests/suite/readiness.rs:29-163` is a near-copy of `tests/ring/src/lib.rs:24-240`. Only the `Backend` and the field names (`submits`/`submissions`) differ.
  - The given-listener scenario exists twice: `crates/skein-sim/src/tests/given_listener.rs` and `tests/conformance/real/tests/support/given_listener.rs`, with an identical `io::Limits` literal (`:26-38` against `:7-21`).
  - kv's `Limits` literal is written 5 times (`tests/kv` `store.rs:10`, `real.rs:12`, `memory.rs:7`, `fuzzy/map.rs:8`, `src/tests.rs:15`), and only `golden.rs` gives derivations. `tests/kv/src/world.rs:117-130` retypes the constructor (`FileIo::new(8,…,1024,…)`, six `Queue::with_capacity(128)`). The `Request::Open {owner: Token::new(1), …}` prologue appears 15 times in `store.rs`.
  - Channel: `schema()` and `limits()` are written 4 times; the `Peer::step` driver 3 times (`src/tests.rs:120-155`, `generic_protocol.rs:84-120`, and `memory.rs::feed` with nine parameters). There are 12 step loops with bare bounds from 4 to 80 (`src/tests.rs:169,237,…`). `tests/channel/src/lib.rs` is a one-line comment, so nothing is shared.
- **Fix:** give `tests/ring`'s `World` a `Backend` parameter and drop its cfg; add shared kv and channel fixtures in each test crate's `src`; add a `run_until_quiet` helper.
- **Size:** about −350 lines.

**9. Conformance call sites** (economy)
- **Evidence:** the 5-line block `let seen = …; #[cfg(linux)] seen.check(); #[cfg(macos)] seen.check_darwin();` appears 9 times in `real/tests/cases/sockets.rs`. `racing_recv` and `racing_accept` (`sim/src/lib.rs:229-244`) exist only because `each_seed` takes `fn` pointers.
- **Fix:** add a `HostCheck` trait and an `on_host(BACKEND, scenario)` helper; make `each_seed` take `impl Fn(&mut Simulated) -> S`.
- **Size:** about −65 lines.
- **Answer to your question:** scenarios are written once. The per-backend test lists are thin and diverge legitimately (seed sweeps, faults the ring can't show). The exceptions are the io-level listener scenarios in finding 8.

**10. Conformance `Item` duplicates fake-machine's `Item`** (primitives)
- **Evidence:** `skein_conformance::{Item, Made}` (`lib.rs:122-188`) has the same constructors and default modes as `skein_fake_machine::{Item, Made}` (`fs.rs:144-213`). They are translated variant by variant in `tests/conformance/sim/src/lib.rs:149-160`.
- **Fix:** fake-machine's `lay` accepts the conformance `Item` (both crates are test-only), keeping `owner` and `Device` as extensions of its own.
- **Size:** about −80 lines.

**11. Minor**
- `sim/processes.rs:6-10` opts out of `indexing_slicing` and `wildcard_enum_match_arm` for the whole module (12 `_ =>` arms), while `sim.rs` and `files.rs` comply.
- `Child` (`processes.rs:36-50`) is another set of flags: `exit`, `stopped`, `held_exit`, `wait_state`, `descendant_running`, `pidfd_open`.

### 5. Cross-area suspicions (not checked outside this area)

- The fake-machine wrapper and the `Handle`/`Opened` conversions recur in `tests/io`, `tests/world`, `tests/oauth-accounts` and smith's `tests/*` and `smith-local-service`.
- Module-level `#![expect(clippy::disallowed_types)]` in a step crate suggests the workspace clippy setup doesn't hold the subset uniformly. Grep `skein-io`, `skein-http` and `skein-oauth*` for the same.
- lib's `Queue` has no `clear`, so channel hand-writes `for _ in 0..capacity { pop }` (`machine.rs:435-441`). This probably recurs.
- (unsure) The ring's `Table` (`ring.rs:104-135`) and readiness's slots (`readiness.rs:14-31`) each hand-roll a generational slot table with a free list and a token map. lib's `Slab`/`Id` may cover this, if its storage addresses stay stable enough for the ring's `UnsafeCell`s.

### 6. Rule costs

- **Exhaustive matches for derived predicates.** Channel's `Phase::may_ping`, `may_end_input` and `refusing` (`machine.rs:43-80`) take 37 lines for three booleans, and the cost grows with each new phase. That is acceptable.
- **"One behaviour per test" against conformance scenarios.** Each scenario returns 9 to 12 observations (for example `Removes`, `files.rs:456-537`), so the first failing assert hides the rest. One real-kernel process per scenario justifies bundling them, but section 10 would be clearer with an explicit exception for conformance scenarios.