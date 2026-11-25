# restart: status

2026-11-25. Where jig's code lags or departs from [restart.md](restart.md).

- **Not yet:** 3: `Told` indexes the settings list by worker number
  minus one instead of resolving the authenticated name as `Opened`
  does. It can name the wrong worker or panic for sparse worker numbers.
- **Not yet:** 3: `Told` reports a wait as sent when the worker's layout
  has the kind, even if conversion or queuing the frame fails.
- **Not yet:** 4: a `restarting` record on a new link after the worker
  sends its hello but before it hears `sources` clears a running grace
  or restart clock. The link treats having sent the hello as sufficient
  on receipt of the word, and the seeded restart test models that clear.
- **Not yet:** 5: no focused wire test checks the conversion plan's
  per-value refusal for `restarting` to a worker layout without it. The
  component test checks that a missing kind is skipped.
- **Not yet:** 5: the test for silence before hello or during closing
  queues the hello but closes before delivering it to the engine. It
  does not check a channel that was open and is now closing.
