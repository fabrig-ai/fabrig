# merge-job-runs-2: what every task keeps

Every task of this plan reads this note first.

## What the plan is for

On a real deployment, every merge job a change made (kind 3,
`temper/docs/design/domain/worker.md`, 4.4) ended held `out of tries:
transient failures`, with no line in either the engine's log or the
worker's. No test runs a merge job from the engine through the worker to
the forge. The cut found by reading one fault that fits every symptom: a
merge with more files in conflict than the checkout's `conflicts` (4 in
both profiles) lists its unmerged paths past the bound, which the git
layer reads as `Broken`, so the job's preparation fails as transient on
every try (`temper/docs/design/protocol/git.md`, 4.6). Its evidence is
outside the repository; the story of task `story` and `story-rows` is
where it is shown or not.

## Reading

- fabrig's `AGENTS.md` and `temper/AGENTS.md`.
- The design sections your spec cites, never a whole document. This
  plan's sections: `temper/docs/design/domain/worker.md`, 4.4 (its last
  three paragraphs and the list after them, dated 2026-11-09) and
  section 5's *A merge in progress*; `temper/docs/design/protocol/git.md`,
  4.6, section 6's *What is kept* and section 10's *A merge's commit
  past the checkout's `conflicts`*; `temper/docs/design/domain/forge.md`,
  8.4 (*The merge job*); `temper/docs/design/domain/testing.md`, 2.1 (the
  merge job's story); `temper/docs/design/protocol/tooling.md`, 2 and 4.1.

## Rules

- **Tests:** focused and fuzzy only, through `heavy`, narrowest first
  (`AGENTS.md`, 5). No slow tier, no walk, no throwaway driver. No test
  reaches a real host.
- **A new case of one behaviour is a row** of an existing table test.
- **Failing tests first:** a task that fixes a fault commits the row or
  test that shows it failing before the commit that fixes it, and its
  report names the failure seen.
- **Fakes stand for their layer:** a fake that plays the git layer
  follows `temper/docs/design/protocol/git.md` as the layer does; a fake
  that plays a command plays tooling.md's contract (2 and 4.1). Where a
  fake departs from its layer, say so in the report.
- **Not yours:** any `docs/design/`, drafts, `docs/help/` and this plan's
  files. `temper/docs/help/hold.md` (read by `temper-docs`' tests) names
  the engine's line as `temper serve: task TASK run RUN attempt ATTEMPT
  failed CLASS: DETAIL`; keep that form so the page stays true.
- **No figure:** no limit, bound or default value changes in this plan
  (the request's *Not in this plan*). A fix changes what happens past a
  bound, never the bound.
- **No stored format or protocol change:** no store schema, wire schema,
  golden or codec changes. Jig's types are not changed; where a task
  would need one, it stops with a `Question:`.
- **Old forms:** `python3 -I -B temper/scripts/old_forms.py` before the
  push; `--write` only to lower a count for what you removed.

## Packages

| Package | Path | Its tests here |
|---|---|---|
| `temper-system-world` | `temper/tests/system` | `--test suite` |
| `temper-worker-checkout-world` | `temper/tests/worker/checkout` | `--test suite` |
| `temper-worker-domain-world` | `temper/tests/worker/domain` | `--test suite` |
| `temper-worker-protocol-git` | `temper/crates/temper-worker-protocol-git` | `--lib` |
| `temper-worker-domain-checkout` | `temper/crates/temper-worker-domain-checkout` | `--lib` |
| `temper-worker-domain-tooling` | `temper/crates/temper-worker-domain-tooling` | `--lib` |
| `temper-worker-domain` | `temper/crates/temper-worker-domain` | `--lib` |
| `temper-engine-domain` | `temper/crates/temper-engine-domain` | `--lib` |
| `temper-service` | `temper/crates/temper-service` | `--lib` (feature `worker` is on in the workspace's tests) |
| `temper-engine-shell` | `temper/crates/temper-engine-shell` | `--lib` |

## Who owns what

| Task | Owns |
|---|---|
| `story` | `temper/tests/system/**`; in `temper-service`: `limits.rs`'s merge items, `worker.rs`'s `worker_root` alone, `assignment.rs`, `route.rs`'s `job_assignment`, `worker_route.rs`'s `job_items` and `job_spec` |
| `conflicts` | `temper-worker-protocol-git`; `temper/tests/worker/checkout/src/translate.rs` and the checkout world's merge stories |
| `lines` | `temper-engine-domain`'s diagnostics; `temper-service`'s `lib.rs` notes, `engine.rs`, `worker.rs` but `worker_root`; `temper-engine-shell`'s lines; `temper-worker-domain`; `temper-worker-domain-tooling`; `temper-worker-domain-checkout` |
| `story-rows` | `temper/tests/system/**` after `story` |

A file another task owns is changed only as its spec's *Owns and may
change* allows, and the report names each such edit.
