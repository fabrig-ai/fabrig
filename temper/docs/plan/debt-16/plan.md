# debt-16: the sixteenth debt round

Three parts (`inputs/request.md`), from the spot review of debt-13 (`inputs/debt13-review.md`), read and not run. This
file is brought up to date with each release; the parts still to come are listed at its end.

## Parts and tasks so far

| Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---:|---|
| `wall-due` | `wall-due` | | medium | 15 | finding 5: `expiry <= env.wall` at the call sites |
| `core-small` | `core-small` | | medium | 20 | finding 10, jig's half |
| `side-out` | `side-out` | | high | 75 | finding 2: `Side` out of `Waiting` in `drive_calls` |
| `own-exchange` | `own-exchange` | `side-out` | high | 50 | finding 4 |
| `test-layout` | `test-layout` | `own-exchange` | medium | 50 | finding 8, and finding 10's `remove` |

## Findings checked against the code so far

- **5, right.** `Wall` is `Ord`; `wall_due` differs from the comparison only at a saturated `now`, the wrong answer its
  test pinned. Three call sites, two in `jig-test-domain` and one in `ops`, beyond the review's "call sites".
- **10, jig's half, right.** `Core::cold_header` has only its own test; `Core::commit_header` has only tests and
  `cold_header`. `Counters::commit_header` stays: temper's `decision.rs` calls it.

- **2, right, but for its last sentence.** All 23 sites hold. `a_call_of_the_other_dialect_is_unavailable` stays:
  `skein/docs/design/forge-connection.md`, 7.1 says a call whose variant isn't its forge's is answered as a call to a
  forge that has gone, and that arm is the one place the pair is met, with no `expect` and no `unreachable!`. Changing it
  would change a design sentence for a test's 40 lines, in a round where behaviour stays.
- **4, right, and one site more.** `own` clears in `connect`, `own_closed` and `assign`; the review's `shed` doesn't clear
  it but does `state.forget()`, so it resets too. `assign` is covered once in `drive_calls`. Plain removal of `h.pump()`
  fails three of the test's four terminals, because a bare `Connection` has no hold on its exchange: those two terminals
  move up to a component test.
- **8, right.** The three `#[path]` moves make `connection/tests.rs` the connection's own tests.
- **9, skipped: the saving isn't there.** The two blocks do match as text, but over two `Body` types and two `Heard`
  types, and step code bans generics and `macro_rules!`. `Payload::closes` can't be written without a `Body`.
  `Exchange::release` replaces three two-line resets with one four-line function on each dialect: +1 line each, not -12.
- **10, skein's half, right.** It goes in `test-layout`, the crate's last task.

## Order in `skein-forge-connection`

One writer per crate, so one chain: `side-out`, `own-exchange`, `test-layout`.

## Still to come

Part 2 and part 3.
