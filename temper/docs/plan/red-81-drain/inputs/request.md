# red-81-drain: the service drains when a client leaves with a request unanswered

After landing, `process-temper`'s `temper::process temper_open_adopts_a_repository_by_commands` fails now and then. It failed 4 times in about 92 runs on 11-08, and once on both its run and its rerun (red-81). Each time the terminal prints "temper: closed" and exits. Then temper's service never finishes draining, and the story's 60 s deadline fires.

It was reproduced on the after-landing machine: 2 hangs in 61 runs. The service was captured while it hung.

## What was found

- `temper serve` (`temper/crates/temper/src/lib.rs`, about 784) never returns from `skein_shell::drive`. Its main thread waits in `io_uring_enter` with one read outstanding. It holds no sockets. Nothing else waits: no channel receive and no child reap.
- `temper/crates/temper-service/src/stop.rs`, about 82: `stop::advance` can't pass `engine.clients.closed`, so the store never closes and `End::Drained` never comes. Captured state:
  - `drain.closing`, `requested` and `agents_ended` are true;
  - `setup.closed` and `forge.closed` are true;
  - `clients.closed` and `clients.component_closed` are false.
- The embedded jig clients component is Closing, with one logical connection and one pending exchange. That exchange is `Waiting::Ask` for sign-in 1 and family 1, and has no response.
  - `jig/crates/jig-protocol-engine-clients/src/component.rs`, about 414, has no ready connection to remove.
  - `closing.rs`, about 212, refuses settlement while that logical connection exists.
  - So `temper-service/src/clients.rs`, about 639, never receives `Request::Closed`.
- **The cause:** a family request admitted before the terminal's transport went away never receives a terminal answer. Shutdown then waits forever on the logical connection that request holds.
  - The gates are old: `clients.closed` since 10-14, the settlement rule since 10-20, HTTP `Call::finish`'s stream-only disconnect since 10-18.
  - It is a race, not one change's regression.

## What's wanted

1. **Shutdown answers or cancels every admitted request whose caller has gone, then waits for `clients.closed`.** Durable keyed effects already committed are kept. Only the caller's correlation is released. Make the fix where the design puts it:
   - temper-service's stop sequence, if jig already offers a way to end an exchange;
   - otherwise jig's engine-clients component, through a disconnect that settles a logical connection's pending exchanges as cancelled. jig's design gains the sentence.
2. **A deterministic test that fails before the fix.** It goes in the lowest world that can interleave a peer's exit and the service's shutdown with an admitted family request (`Waiting::Ask`), with no sleeps or timing luck. Put it in a focused suite.
3. **The story's deadline doesn't change.** A longer patience can't supply the missing answer.

## Limits

- No budget, timeout or tier changes.
- This repairs a red, so it may run `temper_open_adopts_a_repository_by_commands` by name and alone to see it pass, a few times. It reads no CPU figure.
- `walk = false`. Effort medium. One or two tasks.

## Sources

- `temper/crates/temper-service/src/stop.rs`
- `temper/crates/temper-service/src/clients.rs`
- `jig/crates/jig-protocol-engine-clients/src/component.rs`
- `jig/crates/jig-protocol-engine-clients/src/closing.rs`
- `temper/crates/temper/tests/support/terminal.rs`

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands.** The exception is the story named above, by name and alone. Never `tools/tiers/run`, nor a browser or live tier. No walk, render check or throwaway driver.
- **Decide, don't ask.** A small edit just outside a task's file list is made, and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
