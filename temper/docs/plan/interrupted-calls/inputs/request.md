# Request: every interrupted readiness operation ends as the ring's does

From readiness-eintr (its question `other-interrupted`), answered unattended on 11-03 by
agents/loop at the director's word (the user asleep; the recommended option taken). The design is
`skein/docs/design/shell.md` 7.4 and its open question in 10.

## The question, as asked
skein's shell.md 7.4 now has the readiness backend complete an interrupted Socket, Accept, Connect or Open Other(EINTR), as the ring does (your answer to readiness-eintr), and reissue the eight transfers. Every other operation it still makes again on EINTR: Bind, Listen, Shutdown, Truncate, Sync, Stat, Rename, Remove, MakeDirectory, List, Lock, Wait, Spawn, Signal, Usage and Window (attempt makes each again, and each system's call wraps its syscall in retry), where the ring completes those it makes as ring entries (Bind, Listen, Shutdown, Truncate, Sync, Stat, Rename, Remove, MakeDirectory, Wait) Other(EINTR). shell.md 10 now leaves this open. Which rule does 7.4 state?
(a) Recommended: the same rule: each completes Other(EINTR), as the ring's do, so one error number means one thing on both backends. With the signals the shell watches blocked (Linux) or ignored (Darwin), these calls are seldom interrupted. The change: the retry goes from attempt and from each system's call on both systems, the callers outside the readiness backend that share those calls (open_listener's bind and listen among them) keeping theirs. One skein task, wider than this one: about sixteen calls on each system.
(b) The readiness backend absorbs them: each is made again, as now, a difference 7.4 then states beside the four. A design-only change.
No task waits for it: this plan's task changes the four alone.

## The answer: (a)

The same rule for every operation: under the readiness backend, each operation the ring makes as a
ring entry completes `Other(EINTR)` when interrupted, never made again, so one error number means
one thing on both backends; the eight transfers are still reissued as 7.4 says. One skein task:

- the retry goes from `attempt` and from each system's call (Linux and Darwin) for the operations
  the question names; callers outside the readiness backend that share those calls
  (`open_listener`'s bind and listen among them) keep theirs;
- shell.md 7.4 states the rule for all of them; 10's question goes, and the status entry with it;
- tests at the shared seam (as `readiness/interrupted.rs` does for the four), and below each
  system's syscall where a seam exists: an interrupted operation completes `Other(EINTR)` after
  one call.
