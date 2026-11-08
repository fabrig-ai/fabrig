+++
id = "raise-half"
part = "raise-half"
kind = "feature"
needs = []
files = [
  "tools/tiers/budgets.toml",
  "tools/tiers/design.md",
  "tools/tiers/design.status.md",
  "temper/docs/design/domain/testing.md",
  "temper/docs/design/domain/testing.status.md",
  "temper/docs/design/client/browser.md",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/STATUS.md",
]
effort = "medium"
minutes = 20
+++

# Four budgets rise by half, for now, and the documents with them

## Goal

The user, on 11-08: "if any limit is close to causing reds, just raise it 50% and will revisit". Four `cpu_suite_s` figures in `tools/tiers/budgets.toml` rise by half: `[gate.temper.fuzzy]` 120 to 180, `[browser-skein]` 145 to 217.5, `[browser-temper]` 85 to 127.5, `[process-temper]` 360 to 540. The file's comment lines that give their reasons say so, and every design and status line that gives one of those four figures as the current one gives the new one, in the same words: raised by half by the user's word of 11-08, for now, to be revisited. One commit holds the figures and their statements together.

Acceptance:

1. **The file:** `git diff` of `tools/tiers/budgets.toml` against your branch's base is exactly six changed lines, E1 to E3 below: line 1's sentence appended, line 8 replaced, and the four `cpu_suite_s` values. No table, key or comment line is added or removed; every other figure and field is byte for byte as it was.
2. **The reader takes it:** the check of step 2 prints the four new figures and exits 0.
3. **The documents:** E4 to E19 are applied, each replacing its old lines with its new ones exactly; no other line of those files changes, but for a status file's date line under Options.
4. **No test pins an old figure:** `git grep -n -E '(^|[^.0-9])(85|360|145|120)([^.0-9]|$)' -- 'tools/tiers/test_*.py'` prints nothing (the cut's survey: `test_run.py` and `test_cpu.py` write their own budget files in temporary roots and none reads the tracked one). If it prints a line that reads the tracked file's figure, that line reads the file instead, and the report names it.
5. **No current statement left behind:** step 5's grep prints nothing.
6. **Checks:** `python3 -I -B temper/scripts/old_forms.py` reports no raised count; `python3 -I -B tools/citations/citations.py --base origin/main`, after the commit, reports nothing broken in a line this task changed; `heavy cargo nextest run -p fabrig-boundary --test outward` passes, after the commit, with its count in the report.

## Inputs

- `temper/docs/plan/limits-raise/notes/common.md`: its departure (the design and status lines below are this task's) and its checks.
- `temper/docs/plan/limits-raise/inputs/request.md`, whole.
- `tools/tiers/design.md`, 7.3 (*Summaries, budgets and starvation*), the paragraph on `budgets.toml`'s tables and gate entries, which E4 rewrites; `tools/tiers/design.status.md`, its 7.3 paragraph (E5).
- `temper/docs/design/domain/testing.md`: 1 (*Two suites, and two tiers outside them*), 2.6 (*Outside the gate suites*), 6.1 (*Budgets*) and 11 (*The running box's lifetime stories after landing*, *The process tier's concurrency proof*, *The fake box's two walks, after landing*); its status's **Not yet** 2.6 and **Departs** 6.1.
- `temper/docs/design/client/browser.md`, 5 (*Merged stories after landing*); its status's **Not yet** 4.3; `temper/docs/design/STATUS.md`'s browser and fuzzy paragraphs.
- The cut found these and left them as they are, on purpose: the strategy's 30 and 120 (`skein/docs/foundation/testing-strategy.md`, 8, the rule this is an exception to); `testing.md`, 2.6's *6-minute aim* (the design's aim, from which the budget is raised); `.config/nextest.toml`'s 10-22 comment; the dated measurement in `temper/docs/design/draft/bundle-loading.status.md` (*83.069911 of its 85 CPU s*, at `668f7cd3e5`); every `docs/development/` account (`browser-budget.md`, `playground-figures.md`, `limits-heavy.md`, `limits-fuzzy.md`, `limits-worlds.md`), debt review and plan; hearth's, smith's, skein's and jig's own 30, 40 and 120.

## Owns and may change

Owns `tools/tiers/budgets.toml`, and in the other seven files of `files` exactly the lines E4 to E19 name. Nothing else: no other design, draft, status or `docs/development/` line, no test, no code.

Options, each decided:

- **An old block that no longer matches** (a merge beside this plan reworded a line of it): apply the same figure change to the lines as they now read, with this spec's new words, and name the block and what you found in the report.
- **A status file's date line** (`2026-11-08. Where ...`) older than the day you commit (`date -u +%F`): set it to that day. Otherwise leave it.
- **Step 5's grep prints a line E4 to E19 do not name:** if it is in `docs/design/` (not under `draft/`) and gives one of the four old figures as the current budget, change it in this spec's words (the new figure, then "raised by half by the user's word of 11-08, for now, to be revisited", or the shorter form the nearest E block uses) and name it in the report; anything else (a dated measurement, a history, another tree's or suite's figure) stays, and the report lists it.
- **The citations check flags a line this task changed:** fix the citation in that line. One it flags elsewhere goes under `## Left`.

## Steps

1. `date -u`. Apply E1 to E3 (*The edits*, below) to `tools/tiers/budgets.toml`.

2. Check the file with its own reader, from the checkout root (it builds nothing):

   ```sh
   python3 -I -B -c 'import runpy; b = runpy.run_path("tools/tiers/cpu.py")["budgets"]("tools/tiers/budgets.toml", None); print(b["gate"]["temper"]["fuzzy"]["cpu_suite_s"], b["browser-skein"]["cpu_suite_s"], b["browser-temper"]["cpu_suite_s"], b["process-temper"]["cpu_suite_s"])'
   ```

   It prints `180 217.5 127.5 540`. Then `git diff -U0 tools/tiers/budgets.toml` (acceptance 1).

3. Apply E4 to E19 (*The edits*), each replacing its old lines with its new lines exactly.

4. `git diff --stat` shows the eight files of `files` and no other; read each file's diff against E1 to E19 (acceptance 1 and 3). Run acceptance 4's grep.
5. The statements left behind, which must print nothing:

   ```sh
   git grep -n -E '(^|[^.0-9])(85|360|145) CPU|its (85|360) |against (its |the same )?(85|360|120)([^.0-9]|$)|alone at 120, after|unchanged 85|40\.150100|within its 6 minutes|30 s and 120 s' -- tools/tiers temper/docs/design ':!temper/docs/design/draft/*'
   ```

   A line it prints is handled under Options.
6. `python3 -I -B temper/scripts/old_forms.py`. Commit, saying why: the user raised four budgets near their limits by half on 11-08, for now, to be revisited, and the documents that state them change with them. Then `python3 -I -B tools/citations/citations.py --base origin/main` and `heavy cargo nextest run -p fabrig-boundary --test outward`; push with `git push origin HEAD:refs/heads/<your branch>`.

### The edits

Each block is the file's text as it stands, unindented here: copy its lines as they are, leading spaces included.

**E1. `tools/tiers/budgets.toml`, line 1.** Append one sentence to the line itself; add no line. The line becomes, whole:

```text
# The user's budgets, director/decisions/0028 (10-30): x1.5 of the cold measurement of 10-30 (main 44230b8fc), process-temper at its design aim (temper/docs/design/domain/testing.md 2.6), red until its repair; re-measure after a week. Changing a figure is the user's. browser-skein's 217.5, browser-temper's 127.5 and process-temper's 540 CPU seconds are 145, 85 and 360 raised by half by the user's word of 11-08, for now, to be revisited.
```

**E2. `tools/tiers/budgets.toml`, line 8,** the request's own example, word for word. Replace:

```text
# Temper's fuzzy suite: 120 CPU seconds, the strategy's section 8, by the user's word of 11-07.
```

with:

```text
# Temper's fuzzy suite: 180 CPU seconds, raised by half from the strategy's 120 by the user's word of 11-08, to be revisited.
```

**E3. `tools/tiers/budgets.toml`, the four figures.** In each of these tables only the `cpu_suite_s` line changes:

```text
[gate.temper.fuzzy]
cpu_suite_s = 180
```

```text
[browser-skein]
cpu_suite_s = 217.5
```

```text
[browser-temper]
cpu_suite_s = 127.5
```

```text
[process-temper]
cpu_suite_s = 540
```

The budget reader accepts finite positive floats (`tools/tiers/cpu.py`, `budgets`); no reader of the file needs an integer.

**E4. `tools/tiers/design.md`, 7.3.** Replace:

```text
Smith's and Skein's `default` and `fuzzy` entries use `cpu_suite_s` alone,
Smith's at 30 and 120 CPU seconds and Skein's at 40 and 120. Jig's and
Temper's `fuzzy` entries use `cpu_suite_s` alone at 120 CPU seconds;
neither has a `default` entry. These figures are the strategy's section 8
figures (`skein/docs/foundation/testing-strategy.md`), by the user's word
of 11-07, but for Skein's `default`, 40 by the user's word of 11-08; each
is recorded beside its entry. Temper's entry follows Jig's, as does its
comment, and sets no `wall_hard_s`.
```

with:

```text
Smith's and Skein's `default` and `fuzzy` entries use `cpu_suite_s` alone,
Smith's at 30 and 120 CPU seconds and Skein's at 40 and 120. Jig's and
Temper's `fuzzy` entries use `cpu_suite_s` alone, Jig's at 120 CPU seconds
and Temper's at 180; neither has a `default` entry. These figures are the
strategy's section 8 figures (`skein/docs/foundation/testing-strategy.md`),
by the user's word of 11-07, but for Skein's `default`, 40 by the user's
word of 11-08, and Temper's `fuzzy`, the strategy's 120 raised by half by
the user's word of 11-08, for now, to be revisited; each is recorded beside
its entry. Temper's entry follows Jig's, as does its comment, and sets no
`wall_hard_s`.
```

**E5. `tools/tiers/design.status.md`, 7.3.** Replace:

```text
word of 11-08 for it and of 11-07 for the fuzzy 120; Temper's fuzzy entry
is `cpu_suite_s` alone at 120, after Jig's entry, with the requested
comment after Jig's and no `wall_hard_s`. The other gate figures agree
```

with:

```text
word of 11-08 for it and of 11-07 for the fuzzy 120; Temper's fuzzy entry
is `cpu_suite_s` alone at 180, by the user's word of 11-08, after Jig's
entry, with its comment after Jig's and no `wall_hard_s`. The other gate figures agree
```

**E6. `temper/docs/design/domain/testing.md`, 1.** Replace:

```text
- **Two suites, and two tiers outside them.** The focused and the fuzzy
  suites keep their 30 s and 120 s, which nextest enforces; the process
  and live tiers run by their scripts, never at the gate. The process
  tier holds only what processes show, within its 6 minutes (2.6).
```

with:

```text
- **Two suites, and two tiers outside them.** The focused and the fuzzy
  suites keep their 30 s and 180 s, which nextest enforces; the process
  and live tiers run by their scripts, never at the gate. The process
  tier holds only what processes show, within its 540 CPU s (2.6). The
  180 and the 540 are the strategy's 120 and the tier's 6-minute aim,
  raised by half by the user's word of 11-08, for now, to be revisited.
```

**E7. `temper/docs/design/domain/testing.md`, 2.6.** Replace:

```text
  The suite CPU budget remains 360 CPU seconds; its wall hard limit
  belongs to `tools/tiers/budgets.toml`. Harness waits use `Patience` or
  skein-world's `Deadline`; product `ctl --timeout` is not their bound.
  Starved waits pass with their figures after cleanup (testing-strategy.md,
  2.10). The owned preparation command watches its child with a deadline;
  the ctl referee reports overdue work to the world's starvation judge.
```

with:

```text
  The suite CPU budget is 540 CPU seconds, the 6-minute aim raised by
  half by the user's word of 11-08, for now, to be revisited; its wall
  hard limit belongs to `tools/tiers/budgets.toml`. Harness waits use
  `Patience` or skein-world's `Deadline`; product `ctl --timeout` is not
  their bound. Starved waits pass with their figures after cleanup
  (testing-strategy.md, 2.10). The owned preparation command watches its
  child with a deadline; the ctl referee reports overdue work to the
  world's starvation judge.
```

**E8. `temper/docs/design/domain/testing.md`, 6.1.** Replace:

```text
- **Budgets.** The suites' 30 s and 120 s stand until the user sets
  their CPU budgets from measured numbers; they are separate from the
  profiles' wall hard limits. Within them, guides:
```

with:

```text
- **Budgets.** The focused suite's 30 s stands until the user sets its
  CPU budget from measured numbers; the fuzzy suite's is 180 s, the
  strategy's 120 raised by half by the user's word of 11-08, for now, to
  be revisited (`tools/tiers/budgets.toml`). Both are separate from the
  profiles' wall hard limits. Within them, guides:
```

**E9. `temper/docs/design/domain/testing.md`, 11, the running box's lifetime stories.** Replace:

```text
  for the hangup/rebind and killed-box stories within the process tier's
  existing 360 CPU s suite budget, 60 CPU s per test and wall hard limit.
```

with:

```text
  for the hangup/rebind and killed-box stories within the process tier's
  540 CPU s suite budget (360 raised by half by the user's word of 11-08,
  for now, to be revisited), 60 CPU s per test and wall hard limit.
```

**E10. `temper/docs/design/domain/testing.md`, 11, the process tier's concurrency proof.** Replace:

```text
  The 360 CPU s budget and the profile's 25-minute timeout stand; a red
  after landing is repaired under 2.6's rule, without raising either.
```

with:

```text
  The 540 CPU s budget (360 raised by half by the user's word of 11-08,
  for now, to be revisited) and the profile's 25-minute timeout stand; a
  red after landing is repaired under 2.6's rule, without raising either.
```

**E11. `temper/docs/design/domain/testing.md`, 11, the fake box's two walks.** Replace:

```text
  360 CPU s. A plan compiles the walks and runs none; a red after landing
```

with:

```text
  540 CPU s. A plan compiles the walks and runs none; a red after landing
```

**E12. `temper/docs/design/domain/testing.status.md`, Not yet 2.6, the lifetime stories.** Replace:

```text
  within 360 CPU s is established by the instruction account.
```

with:

```text
  within 540 CPU s is established by the instruction account.
```

**E13. `temper/docs/design/domain/testing.status.md`, Departs 6.1.** The suite stays over the strategy's 120, the rule the raise is an exception to, and within the 180. Replace:

```text
  fuzzy median is 125.579032 CPU s against 120, in three warm serial runs
  including the driver (119.288299–127.664356 CPU s).
```

with:

```text
  fuzzy median is 125.579032 CPU s against the strategy's 120, in three
  warm serial runs including the driver (119.288299–127.664356 CPU s),
  within the 180 to which the user raised the budget by half on 11-08,
  for now, to be revisited.
```

**E14. `temper/docs/design/domain/testing.status.md`, Not yet 2.6, the tier's CPU.** Replace:

```text
  2.6: the tier's CPU against its 360 s: of the seven whole runs in
  `test-speed`'s figures at `b23901eab7`, at differing loads, four read
  over it (367.8 to 433.7 s) and three within (287.1 to 349.7 s), and
  they hold no run with that plan's parts together. The request
```

with:

```text
  2.6: the tier's CPU against its 540 s, the 360 raised by half by the
  user's word of 11-08, for now, to be revisited: the seven whole runs in
  `test-speed`'s figures at `b23901eab7`, at differing loads, read 287.1
  to 433.7 s, four of them over the 360 (367.8 to 433.7 s), and they hold
  no run with that plan's parts together. The request
```

**E15. `temper/docs/design/client/browser.md`, 5, merged stories after landing.** Replace:

```text
  run deadline and the whole tier's 85 CPU-second budget. A whole-tier
  run supplies the cost proof; separate story measurements do not.
```

with:

```text
  run deadline and the whole tier's 127.5 CPU-second budget, 85 raised by
  half by the user's word of 11-08, for now, to be revisited. A whole-tier
  run supplies the cost proof; separate story measurements do not.
```

**E16. `temper/docs/design/client/browser.status.md`, Not yet 4.3, the budget.** Replace:

```text
  tier fits its unchanged 85 CPU-second budget. The flagged run, of 17 tests, was
```

with:

```text
  tier fits its 127.5 CPU-second budget, 85 raised by half by the user's word
  of 11-08, for now, to be revisited. The flagged run, of 17 tests, was
```

**E17. `temper/docs/design/client/browser.status.md`, Not yet 4.3, the remainder** (127.5 less 44.849900). Replace:

```text
  40.150100 s for the rest of the tier; the final cost awaits after-landing.
```

with:

```text
  82.650100 s for the rest of the tier; the final cost awaits after-landing.
```

**E18. `temper/docs/design/STATUS.md`, the browser tier.** Replace:

```text
repair within 85 CPU s. The listener-full ctl failure is repaired; the public
```

with:

```text
repair within its 127.5 CPU s, 85 raised by half by the user's word of 11-08,
for now, to be revisited. The listener-full ctl failure is repaired; the public
```

**E19. `temper/docs/design/STATUS.md`, the fuzzy suite.** Replace:

```text
pass all 81 tests each, but their median is 125.579032 CPU s against 120
(119.288299–127.664356, including the driver).
```

with:

```text
pass all 81 tests each, but their median is 125.579032 CPU s against the
strategy's 120 (119.288299–127.664356, including the driver), within the 180
to which the user raised the budget by half on 11-08, for now, to be revisited.
```

## Report

fab's implementer report, with: step 2's output; `git diff --stat` against the branch's base and `git diff -U0 tools/tiers/budgets.toml`; acceptance 4's and step 5's grep output (empty, or each line and what you did with it); each E block applied as given or under an option, named; the citations check's and the outward test's results with its count; the date lines you changed, if any. No slow tier ran, and none is to: the raised budgets are read after landing.
