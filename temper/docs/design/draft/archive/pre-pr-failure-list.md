# A refused delivery's failures stay readable

Draft, 2026-11-15, on the user's word. It changes the reference project's check. On 11-14 and 11-15, four producers in four goals asked their goals for failure keys their delivery check hadn't shown. One check showed 5 of 26 failures, and others had 194 and 349. pre-pr's summary keeps failures whole, each with its log, up to 8,192 bytes, and then counts the rest. Its run directory is gone by then, so nothing else holds the missing keys (P52).

## 1. Every failure's key is kept where the agent can read it

- **Today:**
  - `one_process` reads each failure from the run's `results.json` and `junit.xml` in a temporary directory, which is removed before `summary` prints (`tools/pre-pr/pre_pr.py`).
  - `summary` stops at its own 8,192-byte cap with "[… N more failures]".
  - The host keeps a larger tail, by default 49,152 bytes (smith's `shell_tail`). A session may later fit the words to a shorter prefix, though, so the start of the summary is what survives.
  - `temper/check-results` is no substitute: it holds goal reds and flakes only, and the worker reads and removes it.
  - `tools/pre-pr/design.md`, 6, describes the summary and its cap.
- **Wanted:**
  - Before the summary prints, pre-pr writes every failure it found, new, flake and inherited alike, to one file in the git directory, never in the tree (`git rev-parse --git-path temper/pre-pr-failures.json`). Each entry holds its structured key (a test's binary and name; a lint's package, code, file and message), its display name and its class, and the file names the tree and the base the check ran against. Write it atomically.
  - The summary's header names that file's path and the counts by class, ahead of any log, so the pointer survives a prefix cut.
  - The file is replaced at the next completed check and stays until then. A run that reuses cached results says so in the file.
  - A failure to write the file is named in the summary and never changes the check's verdict.
  - The logs that fit stay as they are.
  - Revise `tools/pre-pr/design.md`, 6, and its overview and focused-test contract, to say so. Make clear that pre-pr's cap is its own, apart from the host's tail and a session's later fitting.

## 2. Out

- The cap itself, and grouping keys inline in the summary.
- Why those checks failed so often (P51, a separate draft).

## 3. Tests

Cheap: focused tests in `tools/pre-pr/test_pre_pr.py` only; walk = false.
- 26, 194, 349 and 450 failures: the file holds every key, and the summary stays within its bound, with the path and the counts first.
- Long keys and keys with non-ASCII text.
- The file is complete after the temporary directory is removed.
- In a linked worktree, the file lands in that worktree's git directory.
- A failed write is named in the summary and leaves the verdict unchanged.
- A later passing check replaces the file, and a cached run is marked as one.

Run only the new and affected tests, by name.
