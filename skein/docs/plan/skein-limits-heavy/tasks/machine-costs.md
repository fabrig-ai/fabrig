+++
id = "machine-costs"
part = "forge-machine"
kind = "feature"
needs = []
files = [
  "skein/testing/skein-forge-kit/src/machine.rs",
  "skein/testing/skein-forge-kit/src/hooks.rs",
  "skein/tests/forge/machine/src/hand.rs",
  "skein/tests/forge/machine/src/hand/**",
  "skein/tests/forge/machine/tests/suite/machine.rs",
  "skein/tests/forge/machine/tests/suite/hand/**",
  "skein/docs/plan/skein-limits-heavy/figures/machine-costs.md",
]
effort = "medium"
minutes = 90
+++

# `skein-forge-machine-world`: stop settling once nothing moves

## Goal

`hand` is 387 tests and 1.92 CPU s, with a median of 4.9 ms against the binary's floor of 2.7. `machine` is 26 tests and 0.55, of which `owner_close_wins_or_follows_each_terminal` is 279 ms. Both drive their worlds in loops that go on long after anything moves. Find what the row pays for, and make each loop end when its work has.

Acceptance:

- Every test still exists and asserts what it asserts. `owner_close_wins_or_follows_each_terminal` still runs all 76 of its worlds.
- `Machine::settle` ends after a run of ticks in which nothing moved, and still ends on what it ends on today. 1,000 stays its bound.
- `Harness::drain` doesn't copy its peers on a round with nothing to do, and ends after a run of quiet rounds.
- Each new threshold is a named constant whose comment gives the reason in step 3.
- `heavy cargo nextest run -p skein-forge-machine-world` passes, and so does `--profile fuzzy` on that package, whose `fuzzy::machine` and `fuzzy::requests` read `Machine`.
- `heavy cargo nextest run -p temper-engine-protocol-forge-world` passes: its `protocol` suite reads `Machine`. This is a change of meaning for `Machine`'s readers.
- `skein/docs/plan/skein-limits-heavy/figures/machine-costs.md` has the row's costliest and median tests, before and after, with their causes and its *Found and not fixed*.
- The next cause's gain is under about 0.05 CPU s.

## Inputs

- `skein/docs/plan/skein-limits-heavy/inputs/request.md`, items 3 and 4.
- `skein/docs/plan/skein-limits-heavy/notes/common.md`: how to measure, the figures' form and the rules.
- `skein/docs/plan/skein-limits-heavy/plan.md`, *Belonging*.
- `skein/docs/foundation/testing-strategy.md`, 8, *A slow test is understood, never hidden*, and 8.1.

## Owns and may change

Owns the files in the header.

- **`skein/tests/forge/machine/src/hand.rs` meets `skein-world-costs`,** which changes `Wire::trace`'s form and its callers, this file's `peer.wire.trace.len()` and `.clear()` among them. Keep your hunk to the quiet count and `owners`. If a merge meets theirs, keep their form of the trace. `skein_world::stream::Wire` doesn't change here: its `pending` stays private, and no accessor is added.
- **`skein/testing/skein-forge-kit/**` has other readers:** `skein/tests/fake-forge`, `fake-peers`, `forge/client`, `forge/protocol`, `forge/setup`, `temper/tests/engine/protocol-forge` and `temper-service`. A change in `machine.rs` reaches only those that use `Machine`, named above. A change to a kit function outside it, `fixtures.rs` or `peer.rs` for instance, may be made if the profile asks for it and no meaning changes, and then `heavy cargo check --tests` covers each of those packages.

Doesn't change: `sign_rs256`, whose comment holds that a key is parsed at each call and never kept, and `Value::Key`'s check in `skein-forge-connection`'s `limits.rs`, which signs to judge a key. If RS256 is a large share, give its count under *Found and not fixed*, with what fixing it would take: a judge that parses without signing, and what `skein/docs/design/tls.md`, 4 says of keeping a key.

Options, each decided:

- **A test needs a pinned interleaving and the shorter loop shifts it:** a quiet tick or round doesn't draw from a `Wire`'s generator, so none should. If one does, find why. Change the world's seed only as a last resort, with a comment, and no assertion changes.
- **A test's fixture repeats itself** (the survey's readings, to check): `deadline_rows` rebuilding a three-response script to find one body offset, `locate_script` building responses a row truncates, `drive_listener_close` rendering one signed POST for five rows, and TLS trust and server settings rebuilt per row. Build it once per test, where the profile says it's worth it. Each row still starts from the state it starts from today.
- **`WAITING_CONNECT_REFUSALS` is 32 and two would do:** lower it if the profile puts its worlds above the stop rule, with a *Counts* line. So too `all_due_calls_finish_in_the_same_timer_turn`'s and `one_offer_drains_all_eligible_held_answers`' 20, which must stay above one to tell all from one. Each exact count asserted follows the new number.
- **Listener settlement** (`skein/testing/skein-forge-kit/src/hooks.rs`, 4,096 rounds and 32 quiet) takes the same constant and the same reason, if the profile asks.
- **Quiet needs more than 16 to pass:** read why, say so in the constant's comment, and raise it for that reason alone. A lower one needs a reason read from the code as well.

## Steps

1. **Profile,** with `-p skein-forge-machine-world --test suite` and the names `cargo nextest list` shows: `machine::owner_close_wins_or_follows_each_terminal`; `machine::the_start_reaches_ready_from_the_recorded_answers`; `hand::github::github_calls::repository_account_selects_its_accepted_installation`; and one median Forgejo test of `hand`. Starting points, to check and not findings:
   - `settle` runs 1,000 ticks unless the world closed or every peer awaits held settlement, and `advance_to` calls it twice. Plain quiet doesn't stop it: about 176,000 ticks of the close test's worlds come before their close.
   - `drain` runs until 32 quiet rounds, and each round copies its peers through `owners()`, as each `Stream` request does. GitHub's `ready` drains eight times.
   - `request_lines`, `request_bytes` and `all_request_bytes` scan and copy.
2. **`Machine::settle`.** Make a tick say whether anything moved: an event taken, a request served, an answer given, a deadline fired, or `Wire`'s trace growing, as `hand.rs` reads it. Then, in the shape of `drain`:

   ```rust
   fn settle(&mut self) {
       let mut quiet = 0;
       for _ in 0..1000 {
           let moved = self.tick();
           if self.closed || /* every peer awaits held settlement, as today */ {
               break;
           }
           quiet = if moved { 0 } else { quiet + 1 };
           if quiet == QUIET {
               break;
           }
       }
   }
   ```

3. **The threshold's reason.** With bytes pending and no demand, `Wire::answer` delivers a fragment only on `rng.chance(700)`, so a round can move nothing with bytes still to come, three times in ten. `const QUIET: u32 = 16` leaves about 4 in a billion that such a run ends early. Worlds are seeded, so a test either passes for its seed or doesn't, and an early end shows as an answer not delivered. Say that in the comment. `drain`'s 32 takes the same constant. Fix it once: one may read the other, or say so in its comment.
4. **`Harness::drain`:** iterate the peers in place on a round, gathering events into the local list, and return at once if there are no peers and no requests. A `Stream` request finds its peer without a copy of the keys.
5. Fix the next causes, largest first, re-measuring after each.
6. Write the figures.
7. The implementer's text's end of a run: `cargo fmt`, old forms, `heavy cargo clippy -p skein-forge-kit -p skein-forge-machine-world --all-targets -- -D warnings` with any other package you changed, then the runs in the acceptance. Commit and push.

## Report

What the row's tests paid for, the before and after, the causes found and not fixed, and which options you took. Whether `temper-engine-protocol-forge-world` read lower too. Your implementation and check minutes apart.
