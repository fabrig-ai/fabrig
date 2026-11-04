# live-login-guard: the plan

From `inputs/live-login-guard.md`: from the moment a lent record moves into a box until it is
exported back, every exit path exports it. One part, one direct task, test support only. The
design is `temper/docs/design/domain/testing.md`, 2.7, "The held record's guard", written by this
cut with its status entry and a Later entry.

## Parts and tasks

1. **guard** (temper; `temper/docs/design/domain/testing.md, 2.7`), slice 1.
   - `tasks/guard.md`, high, ~75 minutes, needs nothing: `Lent` in `support/held_login.rs`, its
     two returns in `support/forge_land/rung.rs`, standing in `tests/live.rs`'s
     `owned_forge_landing`; one process test on the fake issuer, three suite tests on a stub.

## Critical path

`guard` alone. The trunk's end check follows it.

## What the draft's mechanism met in the code

The draft has the guard's `Drop` run "while the box is still up", created "right after the
import", and disarmed by an explicit export. The code changes all three, and the goal stands
above them: every exit path exports the record.

- **The panic that lost the login came with the box stopped.** `ctl.rs:38` is `landed_result`. In
  this story it is reached only through `landed_identity`, from `live.rs:114`: after the first
  `run_live` returned, after `outcome.exit` was asserted 0. A good first sitting does not return
  the record (`due_back(true, false)` is false), because the second sitting proves the restart on
  it. So the record sits in the store with no box up, and the launcher's `give-back` cannot run
  `temper ctl`. A `Drop` that needs a box up would have failed on this very path. The guard starts
  one: over the store, a new client signed in by the box's link, then `give-back`, then the box
  stopped. A box still up is tried first. Inside a sitting the referee already catches a failure
  and returns the record in its `Returning` phase; a panic through `world.run` kills the box
  (`Tree::force_settle`).
- **The guard stands before the import, in `owned_forge_landing`.** It must outlive `run_live` to
  span the gap, and the import is inside the first `run_live`'s referee. Standing earlier also
  covers an import whose reply was lost, which the hook's comment already counts on.
- **`stat` is the disarm.** A record at its lent path owes nothing, whether the import never ran
  or the story's return did. So there is no flag to forget, and it acts once by construction. The
  story's explicit export stays where it is: the second sitting's `Returning`.

## Every live story that imports

| Story | Lends? | Cover |
|---|---|---|
| `a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge` | a held record, across a stopped box | this plan's guard |
| `rung_b0_a_person_agrees_a_plan_before_the_change_lands` | a held record, one sitting | none needed: `run_walk`'s `catch_unwind` runs `give-back` with its `setsid` box up, and that box outlives a panic |
| `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge` | nothing | `account declare`, no import; the directory is used in place and the launcher checks its inode |
| `a_real_model_answers_a_chat_and_pushes_a_small_change`, `live_github`, `live_github_managed` | nothing | a person's sign-in, no import |
| `proof_live`, `account_move`, `process.rs`'s moves | synthetic records | not held |

## Left outside

A signal to the test process between the sittings runs no `Drop`, and the runner's `give-back`
finds no box up. The record stays in the store and the notice says so, as today. The draft's list
does not name a signal, and closing it is a return that starts its own box, in the runner's part:
`testing.md`, section 11. The fab side of the same incident is agents/loop's plan al-logins.

## Overlaps

None with the drafts beside this one. debt-7's temper tasks are in `temper-service` and
`support/owned.rs`, and kv-chunk-target is in skein. The queued drafts list no files yet. A ladder
draft may later touch `rung.rs` and `live.rs`, so this cut's one edit to `run_walk` is one argument.
