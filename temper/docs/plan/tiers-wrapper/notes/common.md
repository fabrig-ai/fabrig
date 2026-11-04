# tiers-wrapper: standing rules

Both tasks name this note. The request is `temper/docs/plan/tiers-wrapper/inputs/request.md`; its
Bounds hold. `tools/tiers/design.md` is already written for this plan and is the contract:
`tools/tiers/design.status.md` lists what the code still lacks.

## What holds for both tasks

- **The design is not yours to edit:** what the code teaches goes in your report. The one
  edit on that side is deleting what your task closes from `tools/tiers/design.status.md`'s
  **Not yet** line; closing the last of it leaves `No gap.` (`DESIGN.md`, 2).
- **Section 10 is frozen:** `budgets`, `prepare`, `execute`, `summary` and `print_summary` keep
  their signatures, and `summary`'s keys theirs. `run.json` keeps every key. Only the output
  file's name changes (`tools/tiers/design.md`, 10.3).
- **Nothing outside these files changes:** `tools/tiers/` and `tests/boundary/outward.txt`. In
  particular `tools/tiers/cpu-wrapper.c`, `budgets.toml`, `packages.toml`, `.config/nextest.toml`
  and every tree stay as they are, as do the other documents that cite `tools/tiers/`.
- **No tool of the lab, anywhere under `tools/tiers/`:** not in code, comments, test fakes or
  fixture text. A test's fake wrapper is called `wrap`. A word the boundary check does not count
  still goes: when your task ends, `grep -il heavy` over the files it owns prints nothing.
- **A test sees none of the caller's settings.** The suites run under callers that set
  `TIERS_WRAPPER` and `TIERS_DISK_BYTES`: every environment a test builds from `os.environ`
  drops both first, and a test that wants one sets it.
- **Comments say what is true now**, in the design's words (`AGENTS.md`, 2): no history, no
  "formerly". In `tools/tiers/`, "wrapper" alone keeps meaning the CPU wrapper; the setting is
  `TIERS_WRAPPER`, and in code the prefix it gives.
- **The suites' limits hold:** `tools/tiers/design.md`, 5 gives `test_run.py` 60 seconds, and a
  test that waits does so on a mocked sleep or a shortened grace, never the real 15 seconds.

## Checks, in place of the implementer's text

The branch changes Python, Markdown and one text inventory, so a task departs from the
implementer's text here: no `cargo fmt`, no `cargo clippy`, no `old_forms.py` (the changed files
are outside every tree's counts). Run, each through `heavy`, one at a time:

- `heavy python3 -I -B tools/tiers/test_run.py` and `heavy python3 -I -B tools/tiers/test_cpu.py`
  (the second builds the CPU wrapper with `cc`);
- `heavy cargo nextest run -p fabrig-boundary --test outward`, since the inventory changes;

and directly, once committed: `python3 -I -B tools/citations/citations.py --all`, at 0.

Rust reads none of this code: no other package's tests are needed.

## The overlap beside this plan

`playground-browser` changes the same files on its own trunk: a `playground bundle` step in
`tools/tiers/run`'s `main` (a `step` call whose command starts with the lab's tool), its tests in
`test_run.py`, the bundle table and its paragraph in `design.md`. It may or may not be on your
base. Look with `grep -n 'playground bundle' tools/tiers/run`:

- **On your base:** that step is one more wrapped command, and it loses the literal tool name
  like every other; its tests take the fake `wrap`. Keep its `CARGO_TARGET_DIR` environment.
- **Not on your base:** do nothing about it; `plan.md` carries it, and you add no code for it.
