# ctl-waits: `temper ctl` says what it waited for, and a kept message gets its answer

temper/qa's walk on a real box met `temper ctl: the engine at ADDRESS did not answer within 30 s` five times on reads, and one read of 17 s, then 0.1 s again, with nothing in `serve.log` to say why. An investigation on fake boxes, 2026-11-06 at `3a034de1ca`, couldn't reproduce a stall of the engine, but found the message isn't evidence of one. Its scripts and evidence are outside this repository, so the words here are what you have. What's below marks what was run and what was read.

**Run:** across 1,543 cheap `temper ctl me` reads on running boxes, under a load of up to 27 on 8 CPUs, the longest took 1.95 s. Large settings proposals (82,798-byte reviews) were decided in about 0.1 s. A long chat reached a 1 MB transcript with reads at 1.4 s at most. **But `task message` itself:**

| Act, on a long chat whose resume the fake model refused (HTTP 413) | Elapsed | Other reads meanwhile, longest | Engine's main thread |
|---|---|---|---|
| `long-476`, succeeded | 23.2 s | 0.04 s | 0.45 CPU s, sleeping |
| `long-835`, timed out; its same-key retry answered in 0.22 s | 30.1 s | 0.28 s | 0.46 CPU s, sleeping |
| `long-933`, timed out | 30.1 s | 0.28 s | 0.19 CPU s, sleeping |
| same-key retry of `long-933`, timed out | 30.0 s | 0.15 s | 0.25 CPU s, sleeping |

During that last retry the client's state read: act; link live; engine heard; dispatched; recovery ready; one kept request; replay complete; no HTTP call held or queued; only the event stream active. So the wait is on a kept act's terminal answer, not on a CPU-bound step. **Whether the service never released the answer, or the client didn't hand it on, isn't known: find out.** An earlier walk may be the same family: a settings item naming an effort no model offered got no answer at all, a `change-settings` kept and unanswered.

**Read, not run:** `temper/crates/temper-client-shell-cli/src/host.rs`, near line 604, gives the one message whenever a deadline passes, whatever's outstanding. `temper/crates/temper-service/src/system.rs` runs IO, routing, the engine and the worker on one thread, and `serve.log` shows nothing of how long any of it took.

## What's wanted, ranked. Drop from the bottom if it passes four tasks, and say what you dropped

1. **A kept message reaches its answer.** Reproduce it in the lowest tier that shows it: a client protocol or temper service simulated world, a long chat, a fake model refusing a resume, concurrent reads, a same-key retry. Capture the terminal outcome at both sides of the client and service boundary, then reduce it to a client step test if the answer arrived and its handle stayed pending, or to the core or journal step if the service never released it. Then fix it there. **A retained act ends in its terminal answer or an explicit pending result, and no invocation is stranded with no request in flight.** Why a message takes 23 s even when it succeeds is part of this.

2. **The deadline's words say what it waited for.** Distinguish waiting for the link, for the stream's first answer, for a read's answer and for an act's outcome. An act's words say that it's kept and that the same key may be tried again, or whatever's true. Payload-free: no words, no secrets. The existing streams-full and busy messages stay. `temper/docs/help/` follows where it names this message.

3. **A long iteration can be seen.** When one pass of the service's loop, or one of its stages, takes longer than a named bound, one line in the service's log says which stage and how long, rate-limited, with no payload. A test holds the words and the bound with a fake clock. So the next walk's log can say whether the engine stalled. Say what bound you chose and why, and what the line costs when nothing's slow. Nothing else about the loop changes.

Failing tests first, in the lowest tier; no test reaches a real host. **A plan runs the focused and fuzzy suites alone (`AGENTS.md`, 5).** A generic mechanism goes in skein first, and skein names no temper; jig never names temper. temper/ui owns `temper-client-*` except `temper-client-tools` and `temper-client-shell-cli`, and a stored format or protocol change is a question first, so its owner can be told. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1): the 30 s stays. Formats change in place before v0.1.0. `walk = false`: temper/qa's next walk is the proof.
