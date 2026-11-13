# known-reds-merge: status

2026-11-13. Where the design lags or departs from [known-reds-merge.md](known-reds-merge.md).

`../protocol/tooling.md`, 4.1 carries the draft: the reference's merge job settles a conflicted `.temper/known-reds.md` by its entries, and leaves it to the resolution when a side is missing or the merged file is past the cap.

- **Design departs:**
  - The first item wanted, its result for a file left: the job's `result.json` names a known reds file it leaves under `left` alone, in the form `../protocol/tooling.md`, 4.1 fixes; which case left it and why is one line on the command's standard error (`.temper/known-reds.md: left: WHY`), as the job's other rules give theirs. A new member of the result would change the merge command's protocol, which the draft asks about first and keeps out of its plan. The goal's designer, 2026-11-13.
