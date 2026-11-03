# Request: the readiness backend's interrupted calls end as the ring's do

From status-gaps (its item 1, question `readiness-eintr`), answered unattended on 11-03 by
agents/loop at the director's word (the user asleep; the recommended option taken). The design is
`skein/docs/design/shell.md` 7.4 and its open question in 10; the gap is
`skein/docs/design/shell.status.md` 7.4.

## The question, as asked

skein's shell.md leaves open (section 10) what an interrupted Socket, Accept, Connect or Open does under the readiness backend; the code makes the first three again (readiness.rs, attempt) while the ring completes them Other(EINTR), and a Connect made again calls connect twice, which a socket mid-connection answers EALREADY (shell.status.md 7.4). Which rule does 7.4 state?
(a) Recommended: the readiness backend completes an interrupted Socket, Accept, Connect and Open Other(EINTR), as the ring does, so one error number means one thing on both backends (7.4: errors map as the ring's do). On the non-blocking sockets the backend makes, Linux and Darwin practically never interrupt these calls, so a caller seldom meets it. The change: drop the three from attempt's retry; 7.4 states the rule; 10's question and the status entry go. One small skein task.
(b) The readiness backend absorbs it: Socket and Accept made again (an interrupted socket or accept4 made or took nothing), Connect taken as under way as EINPROGRESS is (wait for writability, read SO_ERROR, never connect again), Open as now; the ring keeps Other(EINTR), a difference 7.4 then states. One skein task, a little larger.


## The answer: (a)

The readiness backend completes an interrupted Socket, Accept, Connect and Open `Other(EINTR)`, as
the ring does, so one error number means one thing on both backends. One small skein task:

- drop Socket, Accept and Connect from `attempt`'s retry in the readiness backend (`readiness.rs`),
  so none is made again and a Connect never calls `connect` twice;
- shell.md 7.4 states the rule (errors map as the ring's do, an interrupted call included); 10's
  question goes, and the shell.status.md entry with it;
- a test in skein's shell tests: an interrupted Socket, Accept, Connect and Open under the
  readiness backend each complete `Other(EINTR)` and are not made again (the backend's fake
  syscall seam, as its other errno tests do).
