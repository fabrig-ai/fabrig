# interrupted-calls: the plan

From `inputs/request.md`: readiness-eintr's question `other-interrupted`, answered (a) on
2026-11-03 at the director's word. Under the readiness backend, every interrupted operation but
the eight transfers completes `Other(EINTR)`, made once, as the ring's does, so one error number
means one thing on both backends. One part, one task, in skein alone; every task names
`notes/common.md`.

## The design, written by this cut

- `skein/docs/design/shell.md`, 7.4: the bullet for the four (`Socket`, `Accept`, `Connect`,
  `Open`) becomes the rule for every operation but the transfers: made once by the shared part,
  by the system's call and by no call of its own; the calls that set up or inspect around an
  operation's own keep their retries; the ring's synchronous operations, which share Linux's
  calls, complete so too; the callers outside the kernel (`open_listener`'s `bind` and `listen`,
  the harness's `poll_child`) make their call again. Section 10: the question *Every other
  interrupted operation* goes.
- `skein/docs/design/shell.status.md`: a **Not yet** 7.4 item (the sixteen still made again,
  and where), which the task removes. `STATUS.md`'s summary says so until then.

## Parts and tasks

1. **interrupted-calls** (skein, `skein/docs/design/shell.md, 7.4`), slice 1.
   - `tasks/every-call-once.md`, high, ~60 minutes, needs nothing: `retry_interrupted` true
     for the transfers alone and `List`'s retry gone from `perform_call`; the retry gone from
     the eleven calls below `attempt` that hold one, on Linux and Darwin (`bind`, `listen`,
     `shutdown`, `truncate`, `sync`, `stat`, `rename`, `remove`, `mkdir`, `lock`,
     `poll_child`), Darwin's `raw_stat` split for its two other readers; `open_listener` and the
     public `poll_child` keep their retry; one seam test over the sixteen; the status entries
     gone.

Critical path: `every-call-once` alone.

## What was decided, and why

- **Where the retry is, by operation.** The survey of `skein-shell` found it in `attempt`'s
  loop for all sixteen, and below it in each system's call for `Bind`, `Listen`, `Shutdown`,
  `Truncate`, `Sync`, `Stat`, `Rename`, `Remove`, `MakeDirectory`, `Lock` and `Wait`, on both
  systems; `List`'s is in the shared part (`perform_call`); `Spawn`, `Signal`, `Usage` and
  `Window` have none below `attempt`. Dropping `attempt`'s alone would leave eleven absorbed
  underneath, as readiness-eintr found for the four.
- **The ring's `Lock` follows.** Linux's `lock` is shared by the ring's synchronous `Lock`
  (shell.md, section 3, 7.5). Its retry goes for both: the answer's point is one meaning for
  one error number on both backends, and the ring's other synchronous operations (`List`,
  `Spawn`, `Signal`, `Usage`, `Window`) already make their call once. The ring is not a caller
  "outside the readiness backend" in the answer's sense, which names startup's and the
  harness's callers; 7.4 says so, and the task runs `skein-ring-tests`.
- **What keeps its retry.** The public callers that share a call: `open_listener` (`bind`,
  `listen`, wrapped in `system::retry`) and `poll_child` (the harness's, `skein-world`, which
  loops on `Other(EINTR)` in `process.rs`, since the systems' `poll_child` answers an `Error`).
  And the calls around an operation's own, which make no operation again: `setsockopt`,
  `getsockname`, `fcntl`, an `Open`'s check of what it opened (Darwin's `opened`, through
  `raw_stat`, hence the split), Darwin's `remove`'s inspection of an `EPERM`, a `Spawn`'s steps.
- **The tests are at the shared seam only.** The request asks for tests below each system's
  syscall "where a seam exists"; none does (each system's call makes its libc call directly),
  and adding one for some twenty calls on two systems is beyond the task. The seam test runs
  the sixteen through `attempt_with` (`readiness/interrupted.rs`), `Wait` both at submit and
  on its readiness; the code itself, one call and no `retry`, and the Darwin clippy hold the
  systems' half.
- **One task, as the answer says.** All of it is in one crate, `skein-shell`; splitting by
  system would put two writers in one crate's shared face for an hour's work.

## Overlaps with other drafts

None in code: no branch of `reference-m1`, `forgejo-connect` or `withdrawn-forms` touches
`skein/crates/skein-shell/`, and the queued drafts (`v0.1.0-ladder`, `sign-in-wait`,
`reference-walks`) name no files yet. `skein/docs/design/STATUS.md`'s summary paragraph is
shared: `withdrawn-forms/skein-anthropic-bearer` and `withdrawn-forms/skein-codex-route` (not
merged) own it too. This cut and the task each edit one clause of it, a few lines from the
`llm.md` clause those branches may change; a merge conflict there is textual.
