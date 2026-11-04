# Request: the slow tiers run plain commands unless told otherwise

From survey/agents-refs, 2026-11-04; the user approved the plan (step 8 of 12). `tools/tiers/run`,
`cpu.py` and `cpu-run` hard-code one lab's build tool as the prefix of every command, wait for
80 GB of disk before each one, parse that tool's output and call its cancel command. fabrig's tiers
should run anywhere: plain commands by default, with the lab's choices as settings. The caller's
interface (`tools/tiers/design.md` section 10) keeps its signatures and results; its known caller
already sets the settings below and reads the new output name. One plan; one or two direct tasks.

## Changes
1. **The wrapper is a setting.** Environment `TIERS_WRAPPER`: a command prefix split on spaces;
   unset or empty means plain commands. Every build, nextest, listing and the CPU wrapper's `cc`
   build go through it. No default names any tool.
2. **The disk wait is a setting.** `TIERS_DISK_BYTES`: minimum free bytes before each command;
   unset means no wait. The 80,000,000,000 figure leaves the design (it is a caller's value).
3. **Cancellation is the tiers' own.** A hard-wall overrun sends SIGTERM to the command's process
   group (the command starts its own group; signal the group, never the PID alone: a wrapper may
   forward nothing), then SIGKILL after a stated grace; descendants go with it. Remove the parsing of a
   wrapper's "lane token" output and the wrapper `--cancel` call (`cpu.py` ~l.390, 405, 415).
4. **Output names.** `heavy-output.txt` becomes `command-output.txt`; `HEAVY_TAIL` is no longer set
   (a wrapper that truncates output reads its own setting). Update 10.3.
5. **Design.** `tools/tiers/design.md` sections 1-3, 5-7 and 10 say "the wrapper" (the setting) in
   place of the tool's name; describe the two settings in section 3 and the cancellation in 7.
   Drop the retired `qa-temper` profile's handling (`run` ~l.256, design ~l.36, 63).
6. **Tests.** `tools/tiers/test_run.py` and `test_cpu.py` cover: no wrapper (plain commands), a
   wrapper set (prefix applied, CPU wrapper built through it), the disk wait on and off, a
   hard-wall timeout killing a command and its descendants, an interruption. No test needs the
   lab's tool on PATH.

7. **A nextest global timeout is starvation.** When nextest's own `global-timeout` cancels a run
   (its cancellation line in `command-output.txt`), `summary` counts it as a known timeout
   termination (7.3): a still-pending attempt or a JUnit abort with no record is then starved,
   not an error, as it is today only when `execute`'s own hard wall set `starved` in `run.json`.
   Say it in 7.3 and 10.2. A caller then never writes `run.json` itself. Test it with a recorded
   cancellation line.

## Bounds
- Section 10's `cpu.py` signatures and results stay as they are (only the output file's name changes).
- `tools/citations/citations.py --all` at 0; `tests/boundary/outward.txt` entries of the files
  cleaned are lowered (tools/tiers should reach 0).
- Only `tools/tiers/` and its design change.
