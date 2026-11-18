# A journey's clock

Provisional, 2026-11-18. `again` walks a journey on a fixed start: the
fake box's engine, worker and fake peers, and the browser or the TUI it
drives, start from one instant and one seed, the same for every walk,
and each one's clock then moves only to the deadlines its own timers
reach. The same steps then make the same turns at the same times, so two
walks of one journey make the same recording and the same prints, a
time a page draws included, but where section 5 says. Both clients'
journeys share it (`web/journeys.md`, 8.1; `tui/journeys.md`, 8.2),
written from `../draft/journey-clock.md`. Section 2 is the start, 3 the
clock, 4 each part, 5 what still differs, 6 the code, 7 its tests, 8
what is open.

## 1. In one page

- **Why:** a walk's times and seeds were the host's, so each walk
  rewrote its recording almost whole, every print that draws a time with
  it, and two branches that recorded the same journey again conflicted
  on it (`journey-placeholders.md`, 2.4).
- **One start:** `again`'s walks start at 2026-01-06T09:00:00Z with seed
  7, written `1767690000,7`, and each part it starts takes that text
  (section 2).
- **A clock that stands still between deadlines:** a part on a fixed
  start reads the start's instant, and its clock moves only once the
  host's time for its next deadline has come, and then to that deadline
  exactly. It still waits for each deadline as long as the host would
  (section 3).
- **Who takes it:** the box's engine and worker and its four fake peers,
  by `--fixed-start`; the page, by `internal-fixed-start`; the TUI, by
  `TEMPER_INTERNAL_FIXED_START` (section 4). A box takes it only with
  `--fake-backends`, a client only at a loopback address.
- **Who keeps the host's:** the box's own loop, which fetches from real
  hosts over TLS; the agents and their programs, none of whose times or
  draws reach a client; an owned Forgejo, a real program (4.1).
- **What still differs:** what real processes decide, the order of
  exchanges and whether a deadline comes before or after an answer, and
  the times of the parts that keep the host's, where a page draws one
  (section 5).
- **A person's walk keeps its times:** `add` of a recording walked by
  hand is unchanged (`web/journeys.md`, 3.5).

## 2. The start

- **The instant and the seed:** a wall time in whole seconds since the
  Unix epoch, UTC, and a 64-bit seed, written as the seconds in decimal,
  a comma and the seed in decimal: `1767690000,7`. No sign, no space and
  no other byte; no leading zero but in `0` itself; the seconds within
  `Wall` once made nanoseconds, the seed within `u64`. skein's
  `FixedStart` reads it (section 6).
- **The monotonic start:** a part on a fixed start reads its monotonic
  clock from one hour, 3,600,000,000,000 nanoseconds, whatever the
  host's: `FixedStart::NOW`. With the wall, that is the start's `Now`.
- **`again`'s:** `1767690000,7`, 2026-01-06T09:00:00Z, a Tuesday
  morning, so that a page that draws a day and an hour draws "Tuesday,
  09:00" on every walk; seed 7, the fake peers' (`../protocol/setup/box.md`,
  5). One constant of the journeys tool's, which every part it starts
  takes (4.4).
- **The seed's draws:** a part that draws a 32-byte value at its start (a
  cryptographic seed, an identifier) takes it from the start instead of
  the kernel: `FixedStart::draw(stream, out)`, ChaCha20 keyed by the
  seed's eight little-endian bytes and 24 zero bytes, on a stream number
  of its own for each use, so that two uses never yield the same bytes
  (skein's `runtime/lib.md`, 8). Its 64-bit seed is the start's seed.
- **Fixed parts only:** a start is for a walk that a tool drives on fake
  backends. Its draws are known to whoever reads this page, so nothing
  that guards a real account or a real person's data starts from one
  (4.1 to 4.3 say where each part refuses or ignores it).

## 3. The clock

A part on a fixed start reads its clocks from skein's `FixedClock`
(section 6), not the host's:

- **It starts** at the start's `Now`, and stands still.
- **It is asked** for the host's next deadline at every iteration,
  before the loop waits, and gives the host's monotonic instant to wait
  until: the host's reading when that deadline was first asked, plus the
  deadline less the clock's reading. A deadline asked again, unchanged,
  keeps the instant it was first given; another deadline, or none,
  replaces it.
- **It moves** once the host's monotonic reading has reached the instant
  of the deadline asked: the clock reads that deadline, exactly, from
  then on, and its wall is the start's wall plus the same span. It never
  moves back: a deadline at or before its reading falls due at once, and
  the clock stays where it is.
- **So a part's time is its own timers':** each turn an answer, a key or
  an act brings reads the last deadline reached, and each turn a deadline
  brings reads that deadline. The same steps, in the same order, make the
  same turns at the same times.
- **It waits as the host would:** a deadline falls due no sooner than its
  span after it was first asked. One the host armed behind an earlier
  deadline is first asked once the earlier one is reached, so it may fall
  due later than its span; nothing in a walk needs it sooner.
- **Spans between answers read zero:** a step that times an answer by
  two readings of its clock reads no time unless a deadline fell between
  them. A layer's timeouts, retries and leases are deadlines, and keep
  their meaning. A measure for the operator, a pass's length, keeps the
  host's clock in the shell, as it does today.

## 4. Each part

### 4.1 The fake box

- **`--fixed-start SECONDS,SEED`,** on `temper-box` with
  `--fake-backends` alone, and so on `temper box`, which passes it on:
  without `--fake-backends` it is refused as `--bundle`, `--scripts` and
  `--strict` are, and a value `FixedStart` doesn't read is refused,
  naming the flag (`../protocol/setup/box.md`, 5).
- **The box keeps the host's clock and draws:** its own loop, its
  set-up's steps and its supervisor. Its fetch reaches real hosts over
  TLS, whose certificates skein's TLS checks against the loop's wall
  (skein's `net/tls.md`), and a wall in the past would refuse them. What
  it draws, the printed link's secret and the worker's, reaches a
  recording redacted, of its fixed length, or not at all.
- **It gives the start to its children,** the same text at each of their
  starts: `internal serve` and the four fake peers, `fake forge` for
  GitHub and for Forgejo, `fake oauth` and `fake llm`, each with
  `--fixed-start SECONDS,SEED` after its other arguments.
- **`internal serve`** on a fixed start runs its engine and worker on
  `Clock::fixed`, and takes its seeds from the start: the seed itself,
  and the cryptographic seed, the deployment's identifier and the
  installation's from `draw` on streams 1, 2 and 3. So the engine's times
  in its replies (a chat's, a task's, a sign-in's expiry) and what it
  draws are the same on every walk.
- **The fake peers** on a fixed start run on `Clock::fixed`; their seed
  stays `--seed`'s, 7.
- **Its agents keep the host's clock and randomness,** `internal agent`
  and the program it runs, `fake-claude` among them: an agent's trace
  times stay its own, an elapsed figure it reports reaches no client
  (jig's fleet progress keeps the engine's wall for it), and a run's and
  a session's identity are the engine's or derived from what was asked,
  never drawn. Nothing they draw reaches a client.
- **An owned Forgejo is real:** one the walk runs on this machine reads
  the host's clock, which nothing here fixes (section 5).
- **A step started again** starts from the fixed start again, as its
  first start did: a restart is no walk's.
- **No other box takes it:** only a fake box passes it on, and `internal
  serve` and `fake ...` are no person's commands.

### 4.2 The browser

- **`internal-fixed-start=SECONDS,SEED`,** a page's internal setting
  (`web/README.md`, 2.1), honoured on a loopback origin alone and
  ignored elsewhere, as the browser tier's are; a value that doesn't read
  leaves it off.
- **The start's facts:** the seed is the start's, the clocks its `Now`,
  and the zone UTC's, 0, whatever the browser's; the recording keeps
  them, and its fact `fixed`, after `clocks`, says the page started on a
  fixed start (`web/journeys.md`, 3.2).
- **Each turn's clocks** are a `FixedClock`'s, read with the platform's
  monotonic reading, `performance.now()`; at the turn's end the host's
  deadline is asked of it with that reading, and the turn's timer is
  armed for the instant it gives, from that reading.
- **The shell's clock, not Chromium's virtual time:** the harness could
  set Chromium's virtual time through the DevTools protocol, but it stands
  still while a fetch is pending, which a stream's always is, or skips
  over the wait for an answer; the shell's own clock is the native host's
  and its tests', with no browser.
- **Its key marks go unjudged:** a recording holding `fixed` was walked
  by `again`, which marks a still page alone (`web/journeys.md`, 8.1),
  and its turns stand still between deadlines, so its clocks can't tell a
  moving page; `walk` judges a key mark moving only in a recording
  without `fixed`.

### 4.3 The TUI

- **`TEMPER_INTERNAL_FIXED_START=SECONDS,SEED`,** internal, read where
  bare `temper` starts its TUI, from the binary's captured environment
  (`../cli/README.md`, 8): honoured when the engine's address is
  loopback, its host `127.0.0.1` or `[::1]`, and ignored elsewhere; a
  value that doesn't read refuses the start, naming the variable.
- **On it,** the host's seed is drawn from the start, the first eight
  bytes, little-endian, of `draw` on stream 4, and the run's loop reads
  `Clock::fixed`.
- **Why a drawn seed:** a TUI journey's setup signs in with the browser,
  whose seed is the start's (4.2), and the host draws its request keys
  from its seed: on the same seed the TUI's first request key is the
  page's, and the engine refuses it as naming another act. Drawn on a
  stream of its own, the TUI's keys are its own, as a real client's
  random ones are. The host is still made at zero, so the recording's
  `clocks` stay `0 0` (`tui/journeys.md`, 3.2), and its first turn is at
  the start's `Now`; the run's file is named by the fixed wall,
  `recording-20260106T090000Z` (`tui/journeys.md`, 3.1).
- **Its walk needs no fact:** the TUI's marks are its `walk`'s
  (`tui/journeys.md`, 8.1), never judged from its clocks, so its
  recording holds no `fixed`.

### 4.4 `again`

- **One start for every walk,** `1767690000,7`, the journeys tool's
  constant, given:
  - to the box, as `--fixed-start` (`web/journeys.md`, 8.1;
    `tui/journeys.md`, 8.2);
  - to the page, as `internal-fixed-start`, beside its recording
    settings, at its first load and at a setup's `reload` alike, and to
    the browser of a TUI journey's approval;
  - to the TUI, as `TEMPER_INTERNAL_FIXED_START`.
- **Its own bounds keep the host's clock:** a step's seconds, a still
  page's half second and the harness's horizon are `again`'s, read from
  the harness, as they were.
- **Then `add`,** unchanged: redaction and placeholders leave the facts,
  the times and `fixed` as they are.

## 5. What still differs

Two walks of one journey make the same bytes but where real processes
decide, each named here:

- **The order of exchanges:** whether two answers come in one turn or
  two, a stream's record before or after an answer. Walks that differ
  there differ in their turns, and in their prints only where a page drew
  the difference.
- **A deadline against an answer:** whether a timer's deadline is reached
  before or after an answer is the host's time's. A periodic one, a
  stream's heartbeat or its silence, moves a part's clock by its period
  each time a step outlasts it, so in a long step (a fake model's turn, a
  set-up's pause) the times after it may differ by such periods.
- **The parts that keep the host's time:** an owned Forgejo's times, and
  the box's own, its set-up's steps and the fetch whose bytes come from
  the network (`welcome-set-up`'s and the setups' Forgejo, when the
  cache doesn't hold it), wherever a page draws one of them.
- **The kernel's ports and the build** are placeholders already
  (`journey-placeholders.md`, 2.1 and 2.2).

A journey that meets none of these is recorded again as the same bytes:
`again` leaves its files as they were.

## 6. The code

- **skein-lib** (skein's `runtime/lib.md`, 8):
  - `pub struct FixedStart { pub wall: Wall, pub seed: u64 }`, with
    `pub const NOW: Time`, `pub fn read(text: &[u8]) ->
    Option<FixedStart>` and `pub fn draw(&self, stream: u64, out: &mut
    [u8])`;
  - `pub struct FixedClock`, with `pub fn new(start: &FixedStart,
    native: Time) -> FixedClock`, `pub fn read(&mut self, native: Time)
    -> (Time, Wall)` and `pub fn until(&mut self, deadline:
    Option<Time>, native: Time) -> Option<Time>`.
- **skein-shell** (skein's `runtime/shell.md`, 5 and 11): `Clock::fixed(start:
  &FixedStart) -> Clock` and `Clock::until(&self, deadline: Option<Time>)
  -> Option<Time>`; `drive` asks `until` at every iteration.
- **temper:**
  - `temper-box-shell`: `--fixed-start`, its refusal and the children's
    arguments, through `temper-forge-setup`'s `serve` step and
    `temper-fake`'s `backends::steps`; `internal serve`'s clock and
    seeds; the fake peers' clock;
  - `temper-client-shell-tui`: the variable,
    `temper_shell::environment::INTERNAL_FIXED_START`, the seed and the
    clock, `drive_front` asking `until`;
  - `temper-client-shell-web`: the setting in `Internal`, the facts, each
    turn's clocks and timer, `Line::Fixed`, and `walk` leaving a fixed
    recording's key marks unjudged;
  - `temper-client-journeys-tool`: `again`'s start, given to the box, the
    page and the TUI.

## 7. Its tests

Focused and native, within the journeys' cost (`web/journeys.md`, 6): no
browser, terminal or box, and no fuzzy test.

- **skein-lib's:** `FixedStart::read`'s form and each refusal; `draw`
  the same bytes for one seed and stream, and others on another stream
  or seed; `FixedClock` standing still across readings, reaching an
  asked deadline once the native reading passes its instant and not
  before, keeping a deadline's instant when it is asked again, taking a
  new one's from the reading it is asked at, never moving back, and its
  wall following its span.
- **skein-shell's:** `Clock::fixed`'s first reading the start's `Now`;
  the host's clock's `until` the deadline itself.
- **The fake box's** (`temper-box-shell`'s `src/fake_backends/tests.rs`
  and `src/fake/tests.rs`): `--fixed-start` taken with `--fake-backends`
  and given to `internal serve`'s and each peer's arguments; refused
  without it, and for a value that doesn't read; each peer and `internal
  serve` reading it; `internal serve`'s seeds from a start, the same for
  one start and distinct by use.
- **The TUI's:** the variable honoured at a loopback address and ignored
  at another, a bad value refused by name, a run's seed stream 4's draw
  and its first turn the start's, and its first request key other than
  a host's on the start's own seed.
- **The browser's:** the setting honoured on loopback alone and taken out
  of the address everywhere; a host started on a fixed start recording
  `seed`, `clocks` and `fixed` as given, its turns at the clock's
  readings, and replaying; `fixed` read back, and refused out of its
  place; `walk` leaving a fixed recording's key mark unjudged, and judging
  one without `fixed` as before.
- **The journeys tool's** (`src/tests.rs`, `src/again/normalised_tests.rs`):
  `again`'s walk giving the box, the page, at its first load and a
  setup's `reload`, and the TUI the start; a recording made on the fixed
  start keeping its facts through `add`; two runs of the walk's logic over
  the same scripted saves making the same recording, byte for byte.
- **The committed journeys** still replay and print unchanged, by name.

## 8. Open questions

- **Every deadline's own instant:** the clock could keep the first
  instant of every deadline a part arms, not of the next one alone, so
  that one armed behind an earlier deadline falls due at its own span.
  Default: the next one's alone (section 3).
- **An owned Forgejo on the start:** the box could start a Forgejo it
  runs on a fixed time too. Default: it keeps the host's (section 5).
- **A periodic deadline in a long step:** the fake model's turns, or the
  heartbeats, could be shortened under `again` so that no step outlasts a
  period. Default: none; a walk that differs there is added as it walked.
- **The TUI in front of a box** (`tui/journeys.md`, 8.3): `temper box`
  in the pseudo-terminal would take `--fixed-start`, and its TUI the
  variable. Default: with 8.3's build.
