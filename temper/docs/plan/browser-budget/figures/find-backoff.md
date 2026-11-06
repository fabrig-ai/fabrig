# find-backoff: comparison blocked by the long story's script admission

2026-11-06 20:05:48 UTC, base commit `1ba0412483`, incorporating
`window-story` commit `7485300d7b`; 16 CPUs, AMD EPYC-Genoa Processor.

A is the base's semantic-find polling. B would apply the request's fixed
50, 100, 200, 200 ms backoff, reset on success and a person's act, while
press re-finds keep the 50 ms poll. A temporary environment switch selected
A for this run; a query marker in `BrowserFace::ask` counted emitted requests.
Both were removed after the baseline failed.

The exact selection was
`test(=browser_pages::a_long_chat_opens_at_its_end_and_reads_back_from_the_composer)`.
The run used `heavy env FIND_BACKOFF=A python3 -I -B tools/tiers/run --cpu DIR temper browser -- -E FILTER --success-output immediate`.
The CPU summary selected exactly one test, one attempt, no starvation,
no leak and complete accounting. CPU excludes prerequisite builds.
Load columns are the one-, five- and fifteen-minute averages.

| Story | Arm | Run | CPU s (user + system) | Wall s | Load before | Load after | Result |
|---|---|---:|---:|---:|---|---|---|
| long | A | 1 | 1.022407 | 0.761347 | 36.49 / 19.73 / 16.39 | 55.91 / 34.64 / 22.67 | failed at person 0, step 0: scripts past their joint byte bound; `A-long-1.log` |
| long | A | 2–3 | — | — | — | — | not run: no unchanged rerun after failure |
| long | B | 1–3 | — | — | — | — | unobserved: long-story baseline blocked |
| latest | A / B | 1–3 each | — | — | — | — | unobserved: comparison paused at prerequisite failure |
| earlier | A / B | 1–3 each | — | — | — | — | unobserved: comparison paused at prerequisite failure |
| short | A / B | 1–3 each | — | — | — | — | unobserved: comparison paused at prerequisite failure |

**Medians:** unavailable in both arms for every story. No alternated pair
completed. The failed startup's CPU is not a measure of the long story.

**Accessibility requests:** A long run 1 emitted **0**. All remaining A
and B counts are unobserved. The role-by-role search cost cannot be
assessed because the server refused startup before a semantic find.

**Decision:** blocked, not a finding that the patch does or does not pay.
The two required conditions cannot be evaluated: long-story B median CPU
at most 90% of A and lower in two pairs; each story's B median wall at
most 105% of A. No browser code, tests or design sentence change lands.

The inherited long fixture scripts 408,112 reply bytes before wrappers,
above `temper-fake`'s 262,144-byte joint script bound. Server stderr says
`temper box: application.demo.llm.otherwise: scripts past their joint byte bound`.
The assertion at `temper/tests/client-browser/tests/browser_all/browser_pages.rs:464`
observed a child exit with code 1 instead of `Outcome::Done`; kept boxes
were empty. This matches the dependency's recorded blocker in
[window-story.md](window-story.md). Repairing the fixture or fake's
admission is outside this task's file list. Neither the story nor the
bound was changed to obtain a measurement.

The full log, CPU collector files and load samples are retained beside
the task report as `A-long-1.log`, `A-long-1-cpu/`,
`A-long-1.load-before` and `A-long-1.load-after`.
`target/browser-budget/` is removed before push.
