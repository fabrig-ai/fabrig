# reference-m7a: jobs in jig and temper's worker

Carry out the first part of milestone **M7**, M7a, of `temper/docs/design/draft/reference-case.execution.md`. The scope is M7's entry in its section 3, narrowed below. Its section 2 says how a milestone runs: its design and its code go through this plan as one pipeline, and its designers write any design it still lacks into the trunk with the specs that cite it (the repository's `DESIGN.md`).

## The scope: M7a only

- **Builds:** jobs, as already designed and not built:
  - in jig: `jig/docs/design/domain/core.md`, 3.2; `domain/tasks.md`, 5.3.1; `domain/engine.md`, 7.5; `domain/hosts.md`, 6.9; and the client contract's part for jobs;
  - in temper's worker: `temper/docs/design/domain/worker.md`, 4.4, the job capability and its delivery.
- **The jig documents' statuses** list jobs as Not yet. This plan builds them and brings those entries current.
- **Out of scope (M7b, later):** the merge job itself (`merge_job`, which the design does not have yet), its uses (a merge into a goal's branch refused for a conflict, a catch-up change, the goal's landing's update), and the project's `prepare` and `key` kinds, which only landing by turn uses. Build jobs so that M7b can add the merge job as one more kind, but do not design it here.
- Where the jobs design is silent on something M7b will clearly need, record it in the document's Later section rather than deciding it.

## Old code, for reference only

Nothing in the old repositories helps M7a (hel8's `~/src/fabrig/coord/old-repos-value/`).

## Walk

M7a has no walk of its own. M7's walk follows M7b. Its acceptance is its tests and worlds, as the design's world sections say.
