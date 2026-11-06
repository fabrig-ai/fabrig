+++
id = "figures-before"
part = "before"
kind = "docs"
needs = []
files = ["hearth/docs/plan/hearth-memory-6/figures.md"]
effort = "medium"
minutes = 30
+++

# The gate's figures before this round

## Goal

Record, at your branch's base, what the request's section 5 asks to compare: hearth's `default` and `fuzzy` CPU, the peaks and the counts. Later tasks change all of them, and nothing else records them, so this runs on the code as it stands and changes no code.

Acceptance:

- `hearth/docs/plan/hearth-memory-6/figures.md` exists, with a "Before" section holding, each with the commit it was measured at and the date from `date -u`:
  - `default` and `fuzzy` CPU seconds for hearth, by `tools/tiers/cpu.py` with `NEXTEST_DOUBLE_SPAWN=0`, and the ten most costly tests of `default`, with their CPU seconds;
  - the peak of each test in `hearth/tests/builder/tests/suite/memory.rs` and the large render's in `hearth/tests/builder/tests/suite/fuzzy/memory.rs`, beside its `CEILING`;
  - `max_out` and `worst_case` from `hearth/scripts/old-forms.txt`.
- A figure you can't get is written `unknown`, with why. No figure is a guess.
- No file but `figures.md` changes.

## Inputs

- `hearth/docs/plan/hearth-memory-6/notes/common.md`
- `hearth/docs/plan/hearth-memory-6/inputs/request.md`, 5
- `tools/tiers/design.md`, 7, 7.3 and 10.2: `prepare`, `execute` and `summary`, and how a caller keeps its own package selection and build command
- `hearth/docs/design/domain/testing.status.md`, the **Departs** on 9.1: it says why earlier figures disagree. Say how yours compares, and don't resolve it.

## Owns and may change

- **Owns:** `hearth/docs/plan/hearth-memory-6/figures.md`.
- **Departs from your text:** this writes in a plan's directory, and adds no test.
- **A throwaway driver, pre-approved:** `tools/tiers/design.md`, 7 has a caller build the wrapper once and call `cpu.py` through `runpy`. A short untracked script in your slot does that. It's deleted before the push and named in the report.
- **Through heavy:** the build command around nextest is `heavy cargo nextest run --profile <default|fuzzy> -p <each package>`, with `prepare`'s arguments. Take the packages from `cargo metadata`: every workspace member whose manifest is under `hearth/`.
- **If the collector can't run through heavy,** say what stopped it, take the figures from a plain `heavy cargo nextest run` at the same profile (`--message-format libtest-json` or JUnit), and label them as that, not as `cpu.py`'s.
- **Peaks:** `Outcome::peaks()` holds them, and a ceiling test only checks them. A temporary print in those tests, in your slot, is fine. It's never committed. If a peak still can't be read, lower the test's `CEILING` by hand in your slot until it fails: the failure gives the figure.

## Steps

1. Note the base commit, `git rev-parse HEAD`.
2. Measure `default`, then `fuzzy`, one command at a time.
3. Read the peaks, and restore every test file.
4. Write `figures.md`:

   ```md
   # hearth-memory-6: figures

   ## Before

   Measured at `<commit>` on <date>, on <the machine>.

   | Figure | Value | How |
   ...
   ```

5. `git status` shows only `figures.md`. Commit and push.

## Report

The figures, what ran, anything `unknown` and why.
