# landing-behind-main: a landing far behind main updates or resolves, and a resolution starts

A real deployment at fabrig `2c8b9a3f7d`, on Forgejo with one remote worker, met two faults that stop a goal landing. Evidence is outside this repository, so the words here are what you have. **Run, unless marked read.**

**A. A change far behind main stalls at its turn.**
- **What happened:** a goal's landing change was first in its branch's queue, with CI passed at its head. Its head was 409 commits behind main and conflicted with it. The branch's rules ask up to date, and the project names a `merge_job`. The change waited at First until it was held as stalled, twice, with no update and no resolution.
- **Where it stops:** `first()` reads whether the head contains the base before anything else (`temper-engine-domain-forge-change/src/domain.rs`, about 917-921). That read is `Read::Commits{before: head, after: base_tip, page: 1}` (`temper-engine-domain-forge/src/domain.rs`, about 3518). `ancestry_status` counts only a nonempty answer with `more: false` as "lacks the base". A nonempty page with `more: true`, or a `TooLarge` refusal from Forgejo's commits plan (`skein-forge/src/forgejo/plans/commits.rs`, which loads the whole range), stays Unknown. Unknown waits, and the stall holds it.
- **The design's choice, made:** any commit listed in that range proves the head lacks the base's tip. A nonempty page counts as "lacks", whether or not more follow. A range too large to list on a nonempty compare counts as "lacks" too, since it only arises when commits exist. Forgejo's commits plan stops at its capacity and answers `more: true` rather than refusing the whole range.

**B. A resolution can't start.**
- **What happened:** in another goal, a resolver task's every start was refused on the worker as `Invalid(Start(Other)): charter`. Its conflict was in a large generated codec, and its brief carries both sides of every conflicting file whole (`brief.rs`, about 295-307).
- **Where it stops:** the engine trims a charter to its own `hub.charter_bytes` (`temper-service/src/assignment.rs`, about 130-160). The worker refuses a charter past its own limits (`jig-protocol-worker-agents/src/route.rs`, about 145-149; `smith-host-domain`'s admission). A brief the engine accepts but the worker refuses is refused on every attempt.
- **The design's choice, made:** the engine never sends a run a charter its worker would refuse. It sizes the brief by the worker's charter bound, as `worker-reply-bound` sizes reads by the worker's reply bound. Where that's not known, it uses the smaller of the two defaults. A conflicting file too large for the brief is named, with its path and sizes, for the resolver to read from its checkout, not carried whole.

## What's wanted, ranked. Drop from the bottom if it passes four tasks, and say what you dropped

1. **A's failing rows first,** in the change domain's First table: a head behind main by one page, by more than a page (`more: true`), and by a range Forgejo refuses as too large. Each is updated, or, in conflict, leaves the queue for the merge job and then a resolution. Then the fix, in `ancestry_status` and Forgejo's commits plan.
2. **B's failing story first:** a resolution whose brief would pass the worker's charter bound starts, its large files named rather than carried. Then the fix. Say which limit refused task 199's charter, by the defaults the deployment runs, since its config sets neither.
3. **The design says so:** `forge.md` 5.3 (containment, about lines 964-969: a listed commit proves "lacks"), 8.4 and the brief's section for a resolution's large files.

## Not in this plan

Any figure: limits stay as configured. A raise is a question first.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test, on the fake forge and a scripted worker; no test reaches a real host. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. `walk = false`.
