# real-loop-kill-cause: status

2026-11-16. Where the design lags or departs from [real-loop-kill-cause.md](real-loop-kill-cause.md).

The design carries this draft. Section 1's reason is skein-io's
`kernel::Reason`, carried by io's `Request::Signal` and the kernel's
`Op::Signal`, io's own close naming its kill `Closed`
(`../../../../skein/docs/design/runtime/io/processes.md`, 5). The real
world's record of a kill, with its time, target, requester and reason, is
`../../../../skein/docs/design/testing/README.md`, 5. Smith's kill after a
terminate's grace is named `Grace` (`../../../../smith/docs/design/domain/host.md`, 4);
a git operation's and a job's kills name their deadline, cancel or abort
(`../protocol/git.md`, 7; `../domain/worker/jobs.md`). The check's finding,
its records and the service's notes from the run's last 5 seconds are
`../testing/real-loop.md`. No wire or stored format changes: the reason
crosses in-process records alone.

- **Design lags:** none.
- **Design departs:** 1: a stop bound's kill is named `Closed`, a close or
  abort while running, not a stop bound: the worker ends what runs at its
  stop bound by aborting it (`Request::Abandon`), and an abort carries no
  reason down to io's close. The worker's `StopBoundPassed` note, printed
  with the kills from the run's last seconds, names the bound. The draft
  asks for a reason where the sender knows it, and the one sender, io's
  close, knows only the abort (the goal's designer, 2026-11-16). A story's
  kill is named `Asked`, the reason a person's kill takes too, since skein
  names no test's story apart from its driver.
- **Open:** none.
