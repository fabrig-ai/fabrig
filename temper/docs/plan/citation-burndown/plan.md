# citation-burndown: the plan

From `inputs/request.md`: bring `python3 -I -B tools/citations/citations.py --all` to zero, after
which the queue's citation rule becomes absolute. At the plan's base (17273de9e2, 2026-11-02) it
prints 67 findings. Each has one owner task, listed in `notes/findings.md`; `notes/common.md`
holds the rules every task names.

## Parts and tasks

| Part | Task | What | Needs | Effort, minutes |
|---|---|---|---|---|
| jig | `jig-store-items` | `store.md`'s 10.1 steps and 15 items as numbered headings (6 findings) | | medium, 30 |
| skein | `skein-forge-readme` | `forge-github.md` 16 names the README from the root (1) | | medium, 10 |
| temper | `temper-citations` | six sections given their file; `(README, N)` names `README.md`; me-retention's moved backlog path (6) | | medium, 20 |
| checker | `checker-readings` | nine misreadings, R1 to R9, each with its test (37) | | high, 60 |
| zero | `allow-and-zero` | 17 `allow.txt` lines; `--all` exits 0 on the trunk | the four | medium, 15 |

One slice: `--all` exits 0. The first four start at once; `allow-and-zero` starts when they have
merged. Critical path: `checker-readings` (60), then `allow-and-zero` (15), about 75
implementer-minutes.

## What was decided, and why

- **The request's classes, checked one by one.** Every finding was read at its line and its
  target checked. Where the survey disagreed with the request, the survey won:
  - `client/mockups/STATUS.md` 84 and 119 are no break: `first-run.html` and `connect-model.html`
    are tracked. The checker extracts only `.md` and code, so a link to any other tracked file
    reads as dead, and a link's `?query` is kept in its path. They go to the checker (R9).
  - Four findings the request does not name: `ci-on-forgejo-17.md` 140, 142, 145 and jig
    `client/domain/README.md` 38 are a list item headed by a path whose sections are the path's
    (R8, a misreading); `draft/archive/first-run.md` 393's "section 7.2's" means `setup.md`'s,
    a real break (temper task); `key.rs` 101's `crates.md` is test data (allow.txt).
  - The jig README, `contract.md` and `testing.md` findings need R1 too: `MD_TOKEN` reads one
    leading `../`, so `../../x.md` is never a citation today. Measured on a throwaway copy of the
    checker: reading any number of `../` with R2's narrow possessive drops exactly the four and
    reveals nothing; a loose possessive would flag "forge.md's 32 bytes" (forge-github.md 131).
    R4 likewise drops its one and reveals nothing. The other rules only skip or add targets, so
    they cannot reveal a finding.
- **reference-case.status.md's step labels:** read as steps (R7: a status file that says "By
  step" before its labels), not 21 allow lines or a `*` that would excuse a real break later.
- **store.md:** the director's pick, numbered headings. 10.1's steps become 10.1.1 to 10.1.7 and
  the text after them 10.1.8; 15's nine items 15.1 to 15.9, skein's and the application's 15.10
  and 15.11. Item N keeps number N, so the six citations hold as written. `store.md` is read by
  `jig-protocol-store`'s `tests::words`: the task runs that package's lib tests.
- **forge-github.md** is read by `github-conformance`'s test of section 13; the fix is in 16, and
  the task runs that test.
- **The `(README, N)` citations:** the request names two; the task names the file in all eight
  of temper's `protocol/` (configuration, credentials, channel), the same defect, each target
  checked.
- **allow.txt** goes with the last task, `allow-and-zero`, which needs the other four, so it
  always has a commit and checks `--all` at zero on the trunk. Its 17 lines were checked against
  the checker's output: they excuse exactly the 17 findings, and every line excuses one.
- **`temper/docs/design/qa.md` stays.** The director's pick is to remove it once nothing cites
  it; the v0.1.0 ladder's frozen kit (`temper/docs/plan/v0.1.0-ladder/kit.md`, 10) still cites
  it, and that kit is the ladder's, not this plan's. It goes with the kit's next edition.
- **csv_repair.rs's `qa-focused/...`:** allowed, as the request says. AGENTS.md 2 bars a `qa`
  path in public code; the comment sits in a test support file. Rewording it is the request's
  other option, left to whoever owns that file.

## Overlaps with the drafts beside this one

- **sign-in-wait:** smith's shell and `shell.md`; no overlap.
- **v0.1.0-ladder:** its repairs change temper's code and docs as findings come, so no file is
  safe for sure. The edits here are one-line text fixes; the likeliest meeting points are
  `temper/docs/design/protocol/configuration.md`, `credentials.md` and
  `jig/docs/design/protocol/store.md`, which earlier ladder repairs touched. The ladder's kit and
  plan are left alone (`qa.md`, above).

## After the plan

When `allow-and-zero` lands, `--all` is zero on main: tell the session "fabrig citation
consistency gate", whose queue and after-landing switch to the absolute rule (the request).
