+++
id = "settings"
part = "settings"
kind = "feature"
needs = []
files = ["tools/tiers/run", "tools/tiers/cpu-run", "tools/tiers/test_run.py", "tools/tiers/test_cpu.py", "tools/tiers/design.status.md", "tests/boundary/outward.txt"]
effort = "high"
minutes = 55
+++

# The wrapper and the disk wait are settings

## Goal

`tools/tiers/run` and `cpu-run` run plain commands unless the caller's environment says otherwise.
`TIERS_WRAPPER` is a command prefix; `TIERS_DISK_BYTES` is a minimum of free bytes; neither has
a default that names a tool. The retired `qa-temper` profile's handling leaves `run`. `execute`
in `cpu.py` is the next task's, and keeps its behaviour here.

Acceptance:

- With neither setting, no command carries a prefix and `df` is never called, for a tier, a
  `--verify`, a `--verify-all`, a `--cpu` run (its `cc` build included) and `cpu-run`'s build.
- With `TIERS_WRAPPER`, one word or several, every Cargo build, nextest run, nextest listing and
  CPU wrapper `cc` build carries the prefix. `cargo metadata` and the built bundle tool never do.
  An empty value is unset.
- With `TIERS_DISK_BYTES`, the disk wait runs before each of those commands, against that figure.
  Unset or empty, nothing waits. A value that is not a non-negative integer returns 2, naming
  `TIERS_DISK_BYTES`, and starts no later command.
- `DISK_BYTES`, the listing's tail variable and the `profile == "qa-temper"` test are gone from
  `run`, and `step`'s `heavy=` keyword with them.
- `python3 -I -B tools/tiers/test_run.py` and `test_cpu.py` pass, with no tool of the lab on PATH
  and with `TIERS_WRAPPER` and `TIERS_DISK_BYTES` set to junk in the calling environment.
- `grep -il heavy tools/tiers/run tools/tiers/cpu-run tools/tiers/test_run.py` prints nothing.
- `tests/boundary/outward.txt`: the entries of those three files are gone, `tools/tiers/test_cpu.py`
  is at 4. `fabrig-boundary`'s `outward` passes; citations at 0.
- `tools/tiers/design.status.md`'s **Not yet** line no longer holds 1, 2, 2.1, 3, 5, 6, 7 or 7.5;
  7.3, 7.4, 10.2 and 10.3 stay, for the next task.

## Inputs

- `temper/docs/plan/tiers-wrapper/notes/common.md`: the standing rules, the checks and the
  `playground-browser` overlap.
- `tools/tiers/design.md`, sections 1, 2, 2.1, 3, 5, 6 and 7 (its opening, for the `cc` build)
  and 7.5 (its first paragraph): written for this plan.
- `temper/docs/plan/tiers-wrapper/inputs/request.md`, changes 1, 2, 5 (`qa-temper`) and 6.

## Owns and may change

- **Owns:** `tools/tiers/run`, `tools/tiers/cpu-run`, `tools/tiers/test_run.py`.
- **`tools/tiers/test_cpu.py`, only `cpu-run`'s fixture and tests** (the `setUp` beside
  `self.fake('heavy', ...)`, about l.687, and the tests that read it): the fake build command
  becomes `wrap`, the settings are set or dropped per test. The rest of that file is the next
  task's: its two `heavy` fakes stay, so `test_cpu.py`'s `execute` tests stay as they are.
- **`tools/tiers/cpu.py`:** nothing. `run` still calls `cpu["prepare"]` and `cpu["execute"]`.
- **`tests/boundary/outward.txt`:** the entries above. **`tools/tiers/design.status.md`:** as in
  Goal.
- **Fixture profile names:** `qa-temper`, `qa-smith` and `qa-jig` in the test fixtures are arbitrary
  pair names. Default: keep them, so `temper qa` is one more pair with no bundle; only
  `test_all_bundle_prerequisites`'s `temper qa` case and its `if tier == "qa":` branch go. If
  keeping a name would make a test read wrongly, rename it and say so in the report.
- **A wrapper value with shell quoting:** not supported; the split is on spaces, as the design
  says. Add no parsing.

## Steps

1. **`run`.** One function reads the prefix and the commands stop naming a tool; `step` adds the
   prefix and the wait, so its callers pass bare commands:

   ```python
   def command_prefix(env):
       """TIERS_WRAPPER, split on spaces (tools/tiers/design.md, 3); empty when unset."""
       return env.get("TIERS_WRAPPER", "").split()


   def wait_for_disk(env):
       minimum = env.get("TIERS_DISK_BYTES")
       if not minimum:
           return
       if not (minimum.isascii() and minimum.isdigit()):
           raise ValueError(f"TIERS_DISK_BYTES is no number of bytes: {minimum}")
       minimum = int(minimum)
       while True:
           ...  # the df loop as it stands, against `minimum`


   def step(name, command, env, *, wrapped=True):
       try:
           if wrapped:
               wait_for_disk(env)
               command = [*command_prefix(env), *command]
           print(f"tiers: {name}: {shlex.join(command)}", flush=True)
           return subprocess.run(command, env=env, stdin=subprocess.DEVNULL).returncode
       except (OSError, ValueError, subprocess.SubprocessError) as error:
           raise FailedStep(f"{name}: {error}") from error
   ```

   - `build` and the CPU wrapper's `cc` pass `["cargo", "build", ...]` and `["cc", ...]`; the
     bundle tool passes `wrapped=False`.
   - `list_tests` builds `[*command_prefix(env), "cargo", "nextest", "list", ...]`, keeps its
     `wait_for_disk` and its capture, and passes `env` unchanged. Its comment says only what
     holds: a wrapper may print around the JSON, which may be pretty-printed.
   - **The trap in `main`:** `command[:4]` counts four words before the overlay. Build the two
     halves instead, so that the overlay follows `run` at any prefix length:

     ```python
     selection = [*package_arguments(package_map[profile]), "--profile", profile,
                  "--no-fail-fast", *extra]
     if args.cpu:
         overlay = cpu["prepare"](args.cpu, profile, wrapper)
         command = [*command_prefix(env), "cargo", "nextest", "run", *overlay, *selection]
         wait_for_disk(env)
         print(f"tiers: nextest: {shlex.join(command)}", flush=True)
         return cpu["execute"](command, env, args.cpu, profile, limits)
     return step("nextest", ["cargo", "nextest", "run", *selection], env)
     ```

   - The bundle branch's condition loses `or profile == "qa-temper"`. A comment on "admission"
     says the disk wait instead.
2. **`cpu-run`.** Its build becomes `[*runner['command_prefix'](env), 'cc', ...]` after the same
   `runner['wait_for_disk'](env)`. The local `wrapper` stays: it is the CPU wrapper.
3. **`test_run.py`.** Keep the existing assertions' shape: `setUp` drops both settings from the
   inherited environment and sets `TIERS_WRAPPER="wrap"` and `TIERS_DISK_BYTES="80000000000"`, the
   fake called `heavy` becomes `wrap`, and the lists of call names read `["df", "wrap"]`. The fake
   behaves the same whether it is called as `wrap cargo ...` or plainly as `cargo ...`, `cc ...`.
   New tests in that world:
   - `test_no_settings_run_plain_commands`: with both dropped, a tier, a browser tier, `--verify`
     and `--cpu`: no `wrap`, no `df`; the nextest command is `cargo nextest run ...`.
   - `test_wrapper_prefixes_every_command`: `TIERS_WRAPPER="wrap --flag one"`: each command's
     arguments start `--flag one`; `--cpu`'s overlay stands right after `run` at no, one and three
     prefix words; `TIERS_WRAPPER=""` is plain; the bundle tool and metadata carry no prefix.
   - `test_disk_wait_on_and_off` (from `test_disk_wait_before_heavy`): the figure comes from
     `TIERS_DISK_BYTES`; dropped or empty, no `df`; `plenty` and `-1` return 2, naming the step,
     with no command run.
   `cc` joins `cargo` and `df` as a fake on PATH.
4. **`test_cpu.py`'s `cpu-run` tests:** with `TIERS_WRAPPER` set, the build goes through the fake
   `wrap`; dropped, `cc` runs as a fake on PATH; the disk fake is reached only with
   `TIERS_DISK_BYTES` set.
5. Lower the inventory, edit the status file, run the checks of `notes/common.md`, commit and push.

## Report

As the implementer's text has it, and: which of the two cases of `notes/common.md`'s overlap held,
and the pair names you kept or renamed.
