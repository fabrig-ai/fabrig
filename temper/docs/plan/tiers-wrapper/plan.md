# tiers-wrapper: plan

Request (`inputs/request.md`, step 8 of 12 of a user-approved plan): `tools/tiers/` runs plain
commands by default, with the lab's choices as settings, and cancels on its own. The caller's
interface (`tools/tiers/design.md`, 10) keeps its signatures and results.

This cut wrote the design: `tools/tiers/design.md`, 1 to 3, 5 to 7.5, 8 and 10, and the status
file's **Not yet** line. `tests/boundary/outward.txt` loses `tools/tiers/design.md`.

## Parts and tasks

All in slice 1, released together in the first push.

- **settings:** `tasks/settings.md`, no needs, ~55 min, high. `TIERS_WRAPPER` and
  `TIERS_DISK_BYTES` in `run` and `cpu-run`; `qa-temper`'s handling out of `run`; `test_run.py`
  and `cpu-run`'s tests in `test_cpu.py`. Requests 1, 2, 5 (its `qa-temper`), 6.
- **cancellation:** `tasks/cancellation.md`, needs settings, ~110 min, high. `execute`'s own
  session, signals and grace; `command-output.txt`; a nextest global timeout and a cancelled
  attempt counted starved; the rest of `test_cpu.py`. Requests 3, 4, 6, 7.

**Critical path:** settings (~55) → cancellation (~110), about 2.75 hours of implementer time
plus fab's merges.

## Decisions

- **Two tasks, in order, not side by side.** Every file is in `tools/tiers/`.
  `test_cpu.py` holds both tasks' tests, and the inventory's lines to lower sit next to each
  other, so two tasks at once would conflict for a saving of under an hour.
- **Cancellation reaches the session, not only the group.** The request has SIGKILL go to the
  command's group, "descendants go with it". Nextest gives each test a process group of its own
  (`tools/tiers/design.md`, 7.2), so a group signal alone leaves tests running once nextest is dead.
  The command starts its own session, and after the grace `execute` kills what is left of that
  session, found in `/proc`. SIGTERM still goes to the group, as the request says.
- **The grace is 15 seconds:** past nextest's default 10 (no profile sets another) and the
  wrapper's one-second settlement, so that an orderly cancellation still writes JUnit.
- **A cancelled attempt with a record is starved, not failed: this closes a Later early.** The
  design left that for later, and the request's item 7 names only a pending attempt and a JUnit
  abort with no record. But the old cancellation was SIGKILL, with no JUnit, so that case
  rarely arose. SIGTERM first means nextest stops the test, the wrapper records the signal and
  JUnit reports an abort, on nearly every overrun. Left as a failure, `execute` would return
  100 for a run that only starved, against the request's bound (section 10's results) and
  `AGENTS.md`, 5 (starved is never red on that account). The rule is narrow: JUnit's abort by
  SIGTERM or SIGKILL with a record that is pending or ended by one of those; an exit code or
  another signal keeps its failure. Its cost: a test that died of SIGKILL by itself in a starved
  run reads as starved; 7.3 says so.
- **`run --cpu` takes SIGTERM and SIGHUP as interruptions.** A command in its own session no
  longer gets a signal sent to the runner's group, and Python's default would die leaving it
  running. Callers of `execute` hold their own signals; a user unit's stop reaches the whole
  control group regardless.
- **`cancel_code` stays in `run.json`, always null:** section 10 keeps its keys.
- **`command_prefix`, not `wrapper`, in the code:** "wrapper" already means the CPU wrapper,
  thirty times in the design. The design's sections 1 to 6 say "the wrapper", as the request has
  it; from section 7 on the setting is `TIERS_WRAPPER`, and section 3 says so.
- **Also corrected in the design, for what the request's edits touched:** 2.1's "fourteen pairs"
  (eight are configured) and `qa-temper`'s fake-claude line; section 1's "on a builder".

## Overlaps

**Not avoidable: `playground-browser`** (on its trunk, all merged) changes `tools/tiers/run`,
`test_run.py`, `design.md` and `design.status.md`, the only tree this plan may touch.

- It adds a `playground bundle` step whose command starts with the lab's tool, and tests that name
  that tool. If it lands after these tasks, the merge brings a literal name back into `run` and
  `test_run.py` with their inventory entries gone: `fabrig-boundary` goes red, and a repair must
  put that step behind `command_prefix`.
- If it lands first, `notes/common.md` tells each task what to do; both tasks look at their base.
- Either way `design.md`'s bundle table row conflicts: it splits `browser-temper, qa-temper` in two
  and this plan drops `qa-temper`. The result is its `browser-temper` row alone.

`sign-in-wait`, `v0.1.0-ladder`, `debt-4` and `bugs-skein` show nothing in these files on their
trunks. `fabrig-scale` touches none of them.

## Left as it stands

- `run`'s last `wait_for_disk` in `--cpu` mode lets a failing `df` escape as a traceback, not exit 2;
  it did before, and the request does not name it.
- `budgets.toml`'s first comment (`tools/tiers/design.md`, 8) stays for later.
