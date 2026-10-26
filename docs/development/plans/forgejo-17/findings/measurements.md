# Measurements at the base, and v17's facts probed while planning

2026-10-16, on x1 (8 threads, 15 GB).

## 1. The suites at the base

In a worktree of temper at `118f7fb7`, each
suite three times after one build (`export NEXTEST_DOUBLE_SPAWN=0`):

| Suite | Runs (s) | Median | Tests |
|---|---|---|---|
| default | 5.248, 5.221, 5.372 | 5.248 | 1248 run, 1 skipped |
| fuzzy | 26.226, 27.611, 26.614 | 26.614 | 46 run |

`facts.txt` at the base: 97 `holds` (39 of them `component-`), 4 `fails`
(`pull-for-names-the-newest-of-several`, `open-pull-no-changes-is-422`,
`delete-missing-branch-is-404`, `effective-protection-name-for-a-writer`),
1 `unchecked` (`webhook-deliveries`). `tools/git-transcripts/transcripts/`
holds 41 cases.

## 2. A throwaway spike on v17 (the design's)

`~/src/rust/tmp/temper-forgejo-17-spike.md` (not versioned): the pin
moved to 17.0.0 and 13.2.0 by 13 version strings, both conformance
modes recorded on v17 with every fact unchanged (97 / 4 / 1), five
codecs tests failing on re-recorded literals (`findings/tools.md`, 3),
the fuzzy suite passing. Observed there, among others:

- `GET R/actions/jobs/<job>` answers the job with `steps`, each
  `number`, `name`, `status`, `started`, `stopped`: 0 `Set up job`, the
  workflow's steps, the last `Complete job`; a failed workflow step and
  the completion step are both `failure`; a step with no YAML `name` is
  named by its command text; a job waiting for a runner has no `steps`.
- `logs?attempt=1&step=1` answers that step's lines alone; a step past
  the last answers `404`, `{"message":"step index out of range",…}`;
  `logs?step=` without `attempt` reads the latest attempt.

## 3. The facts probe (planning, through Codex)

One throwaway Forgejo 17.0.0 with runner 13.2.0, on loopback; a job of
three workflow steps (three lines and passing; `run: "true"`; one line
and `exit 1`). The report and its raw captures:
`~/src/rust/tmp/temper/forgejo-17-plan/facts-probe.md` and
`facts-probe/` (not versioned).

- **`files=false`** keeps the top-level `files`, empty (`[]`), and gives
  each commit `files: null`; `total_commits` and `commits[].sha` are as
  without it. The reverse comparison with `files=false&verification=false`
  answers `200`, `total_commits` 0, `commits` `[]`, `files` `[]`. So the
  comparison's decoder, which requires `files`
  (`crates/temper-forge-forgejo/src/client/files.rs:40-42`), reads
  these answers as they are.
- **`verification=false`** turns each `commits[].commit.verification`
  into `null`; `files[].filename`, `commits[].sha` and `total_commits`
  are equal with and without it.
- **The job:** steps 0 `Set up job` `success`, 1 `success`, 2
  `success`, 3 `Known failure` `failure`, 4 `Complete job` `failure`:
  the first failure is the workflow's step.
- **A suffix range on a step's log:** `Range: bytes=-10` over a
  112-byte slice answers `206`, `Content-Range: bytes 102-111/112`,
  the last 10 bytes; `bytes=-100000` answers `206`, `Content-Range:
  bytes 0-111/112`, the whole slice. On the whole log, `bytes=-10`
  answers `206`, `bytes 589-598/599`. Every answer has `Accept-Ranges:
  bytes`.
- **An empty log** (the `run: "true"` step's slice): no range, `200`,
  `Content-Length: 0`; `bytes=-10`, **`206` with `Content-Range: bytes
  0--1/0`** and an empty body; `bytes=0-10`, `200`, empty, no
  `Content-Range`. The component's `Content-Range` parser
  (`crates/temper-forge-forgejo/src/header.rs:19-38`) is to read
  `bytes 0--1/0` as an empty log (`forge.md`, 3.2: "a range over an
  empty log is an empty log").
