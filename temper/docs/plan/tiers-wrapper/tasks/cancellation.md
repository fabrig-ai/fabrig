+++
id = "cancellation"
part = "cancellation"
kind = "feature"
needs = ["settings"]
files = ["tools/tiers/cpu.py", "tools/tiers/test_cpu.py", "tools/tiers/run", "tools/tiers/test_run.py", "tools/tiers/design.status.md", "tests/boundary/outward.txt"]
effort = "high"
minutes = 110
+++

# Cancellation is the tiers' own; a global timeout is starvation

## Goal

`execute` stops its command with signals of its own: it starts the command in its own session,
sends SIGTERM to the group at the hard wall, and SIGKILL to what is left of the session 15 seconds
later. It parses no output for a token and calls no cancel command. The command's output is
`command-output.txt`. `summary` counts a run nextest's `global-timeout` cancelled as starved, and,
in any starved run, an attempt cancellation aborted, so a cancelled run stays green.

Acceptance:

- A hard-wall overrun on a command that ignores nothing ends on SIGTERM; on a command and a
  descendant that ignore SIGTERM, the descendant in a process group of its own, as nextest gives a
  test, both are gone by SIGKILL when `execute` returns. Output forwarded before and during the
  grace reaches `command-output.txt`. `execute` returns 0, `run.json` has `starved` true and
  `cancel_code` null.
- An interruption (SIGINT to the process in `execute`; SIGTERM or SIGHUP to `run --cpu`) stops the
  command and its descendants the same way and is raised again. A collector fault does the same.
- In a starved run, an attempt with a JUnit abort by SIGTERM or SIGKILL and a record that is
  pending or ended by one of those signals is starved, not failed; `execute` returns 0 for it. A
  record with an exit code or another signal keeps its failure.
- With nextest's cancellation line in `command-output.txt` and `run.json` `starved` false: a
  pending attempt, a JUnit abort with no record and a recorded abort are as above, `starved_run`
  is true, the verdict `starved`, `execute` returns 0 for nextest's exit, and `run.json` stays as
  written. Without the line the same directory fails coverage (exit 2). Another `Cancelling due
  to ...` line does not count.
- The fixture line is a recorded one, from a real run (step 5).
- `grep -rIil heavy tools/tiers` prints nothing, `tests/boundary/outward.txt` has no
  `tools/tiers/` entry, `fabrig-boundary`'s `outward` passes, citations at 0.
- `tools/tiers/design.status.md` is `No gap.` (or, with a `playground-browser` entry on your base,
  holds only that).

## Inputs

- `temper/docs/plan/tiers-wrapper/notes/common.md`: the standing rules, the checks and the
  overlap.
- `tools/tiers/design.md`, 7.3 whole (the starved run, the hard wall, the interruption), 7.2's last
  paragraph (nextest's process groups), 7.4's list, 10.2 and 10.3: written for this plan.
- `temper/docs/plan/tiers-wrapper/inputs/request.md`, changes 3, 4, 6 and 7.

## Owns and may change

- **Owns:** `tools/tiers/cpu.py`, `tools/tiers/test_cpu.py`.
- **`tools/tiers/run`:** only the handler of step 3, in `main`. **`tools/tiers/test_run.py`:** only
  that handler's test.
- **`tests/boundary/outward.txt`:** the `tools/tiers/cpu.py` and `tools/tiers/test_cpu.py` entries
  go. **`tools/tiers/design.status.md`:** as in Goal.
- **`CANCELLED`, `junit`'s attempts and `records`' validation stay as they are.** The `interrupted`
  field stays read and stays zero.
- **`cancel_code`:** stays in `run.json`, always null; section 10 keeps the key.
- **A real nextest that behaves otherwise than the design says** (no JUnit after SIGTERM, an exit
  other than 100 from a cancelled run, another form of the line): make the code hold the design's
  results for what you recorded and say so in the report. If the design's result itself cannot
  hold, stop with `Question:`.
- **A grace that cannot be shortened in a test by anything else:** the test driver sets
  `cpu['execute'].__globals__['GRACE_S']`; `execute`'s signature does not grow an argument.

## Steps

1. **Signals, in `cpu.py`.** `GRACE_S = 15.0`, with a comment saying why it outlasts nextest's
   grace (`tools/tiers/design.md`, 7.3).

   ```python
   def session_processes(session):
       """The live processes of `session`; a zombie is nothing left to stop."""
       ...  # /proc/PID/stat: after the last b')', field 0 is the state, field 3 the session.
            # A process gone between listing and reading is skipped.


   def stop(process, deadline):
       """Wait until `deadline` for the command's session to end, then SIGKILL what is left."""
       ...  # poll session_processes(process.pid) every 50 ms; then SIGKILL each, repeating
            # up to one second, for what forks meanwhile; ProcessLookupError is no fault.
   ```

   `session_processes` is what makes SIGKILL reach tests: nextest gives each its own group, in
   the command's session, so `os.killpg` alone leaves them running once nextest is dead.
2. **`execute`.** Popen gains `start_new_session=True` and passes `env` as it stands. The token,
   its regex, `cancel_code`'s variable and `run_env` go.

   ```python
   kill_at = None
   process = subprocess.Popen(command, env=env, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                              stderr=subprocess.STDOUT, start_new_session=True)
   try:
       with process.stdout, selectors.DefaultSelector() as selector:
           selector.register(process.stdout, selectors.EVENT_READ)
           while selector.get_map():
               ...  # forwarding as it stands
               if <the overrun test as it stands>:
                   starved = True
                   save(directory / 'starved.json', {'wall_hard_s': hard})
                   print(...)
                   signal_group(process, signal.SIGTERM)
                   kill_at = time.monotonic() + GRACE_S
               if kill_at is not None and time.monotonic() >= kill_at:
                   break
       if kill_at is not None:
           stop(process, kill_at)
   except BaseException:
       ...  # SIGTERM to the group, then stop(process, now + GRACE_S); a second interruption
            # during it stops at once, with no grace. The exception is raised again.
       raise
   ```

   `signal_group` is `os.killpg(process.pid, sig)` with `ProcessLookupError` suppressed. The `break`
   matters: a process that left the session may hold stdout open for ever. After the loop the
   code is as it stands, but for `command-output.txt`, `cancel_code=None`, and the refused-cancel
   `raise`, which goes. The comment on the no-JUnit branch of `summary` says what holds now:
   SIGKILL may come before JUnit, and the pre-exec records survive it.
3. **`run --cpu`.** A command in its own session no longer gets a signal sent to the runner's
   group, and Python's default dies without cleaning up. In `main`, before `cpu["execute"]`, in the
   `--cpu` branch only, SIGTERM and SIGHUP raise `KeyboardInterrupt`.
4. **`summary`.**

   ```python
   # nextest's line when its global-timeout cancels a run (tools/tiers/design.md, 7.3).
   GLOBAL_TIMEOUT = re.compile(rb'^\s*Cancelling due to global timeout\b', re.MULTILINE)
   COLOUR = re.compile(rb'\x1b\[[0-9;]*m')


   def timed_out(directory):
       """Whether nextest's global-timeout cancelled the run in `directory`."""
       ...  # command-output.txt, absent is False; COLOUR removed before the search
   ```

   After `summary`'s validation of `run.json`: `if timed_out(directory): run = dict(run, starved=True)`.
   Every `run['starved']` below then holds, and the file on disk does not change. In the JUnit
   branch, under `if run["starved"]:`, beside `unobserved`:

   ```python
   cancelled = {key for key in aborts & actual
                if by_key[key]['pending'] or by_key[key]['signal'] in (signal.SIGTERM, signal.SIGKILL)}
   timeouts |= cancelled
   ```

   and `failures` drops `keys - unobserved - cancelled` (`cancelled` is empty when the run is not
   starved). A failed attempt 1 with a cancelled attempt 2 keeps its failure.

   `execute`'s last lines: after `if result['failures']: return 100`, return 0 when `starved`, or
   when the code is nextest's failure exit and `result['suite']['starved'] or
   result['suite']['starved_run']`: a global timeout may end a run with no attempt in flight.
5. **Record the line.** Once, on your slot, from a real run: copy `.config/nextest.toml` to an
   untracked file, set a `global-timeout` shorter than the tests of a small package of your choice
   in one profile, run `heavy cargo nextest run -p PACKAGE --profile PROFILE --config-file COPY`,
   and take the line (expected: `  Cancelling due to global timeout: 1 test still running`) and
   the exit code. The fixture is that line, verbatim. Then one real proof of the hard wall: a
   throwaway driver calling `prepare` and `execute` on `heavy cargo nextest run -p PACKAGE
   --profile PROFILE` with `{PROFILE: {'wall_hard_s': 1}}`, and report its exit, `summary`'s
   `starved_run`, `failures` and `unobserved`, whether JUnit exists, and that no test process is
   left. The copy and the driver stay untracked and go before the push.
6. **`test_cpu.py`,** replacing the `heavy` fakes with fixture commands that act on signals:
   - `test_hard_wall_terminates_the_group_and_stays_green` (from
     `test_hard_wall_uses_heavy_cancel_and_stays_green`).
   - `test_hard_wall_kills_a_command_and_descendants_after_the_grace`: the command and a
     descendant under `process_group=0` ignore SIGTERM; a shortened `GRACE_S`; neither pid lives
     after `execute`, and output printed during the grace is in `command-output.txt`.
   - `test_hard_wall_ends_though_a_descendant_holds_stdout`: one that left the session; `execute`
     returns and the test cleans it up.
   - `test_interruption_stops_the_command_and_descendants`: SIGINT to a driver in `execute`;
     the same two pids gone, `KeyboardInterrupt` raised, its exit not 0. A second SIGINT during
     the grace ends it at once.
   - `assert_cancelled_verdict`'s fake exits on SIGTERM instead of a cancel call; its users stand.
   - `test_recorded_cancellation_abort_is_starved`: `signal=15` and `signal=9`, and pending, with
     a JUnit abort in a starved run: `failures` empty, `tests`' `starved` 1,
     `assert_cancelled_verdict(0)`; `exit_code=7` or `signal=11` beside the same abort stays failed;
     not starved, the abort stays failed.
   - `test_global_timeout_is_starvation`: the recorded line, plain and coloured; the cases of
     Acceptance; `cli()` exits 0; `run.json` unchanged; `execute` on a command that prints the
     line and exits with nextest's code returns 0. No line, or `Cancelling due to test failure`:
     exit 2.
   - `execute`'s output file: `command-output.txt` exists, and `run.json` has the key
     `cancel_code`, null.
   In `test_run.py`: `run --cpu` killed by SIGTERM stops a fake command in its session.
7. Lower the inventory, edit the status file, run the checks of `notes/common.md`, commit and push.

## Report

As the implementer's text has it, and: the recorded line and exit code; step 5's real proof; what
happened with a nextest that differed from the design, if it did; which of the overlap's two cases
held.
