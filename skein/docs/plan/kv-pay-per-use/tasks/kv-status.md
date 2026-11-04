+++
id = "kv-status"
part = "status"
kind = "docs"
needs = ["kv-open-reads", "kv-snapshot-bound", "kv-recover-stream", "kv-recover-count"]
files = ["skein/docs/design/kv.status.md", "skein/docs/design/STATUS.md"]
effort = "medium"
minutes = 20
+++

# skein: kv's status, once the store pays for what it holds

## Goal

Once every task of this plan has merged, `skein/docs/design/kv.status.md` says where skein's
code still lags `skein/docs/design/kv.md`, and nothing more (`DESIGN.md`, 2). This task
departs from the implementer's text, which keeps `docs/design/` out of reach: the two status
files are its own.

Acceptance:

- Read the trunk's code and tests for each **Not yet** entry this plan's cut wrote in
  `kv.status.md` (3: recovery's reads, whole files and leftover segments; 5: the trigger and
  the replayed log; 7: recovery's buffers and the two exported constants; 9: "What it pays
  for"), and remove each one the code now meets. An entry the code still misses stays, said
  as it now stands. Keep every other entry as it is (9's model-map referee; the **Departs**
  entry on 2 and 4), and date the file today (`date -u`).
- A part of kv.md 3, 5, 7 or 9 built otherwise than it says is a **Departs** entry with its
  reason, taken from the commit that built it (`git log -- skein/crates/skein-kv
  skein/tests/kv`); one whose reason no commit gives is listed in the report as a question,
  not written.
- `skein/docs/design/STATUS.md`: unchanged while kv still lags (its "A few items lag in ...
  `kv.md`" stands); if no **Not yet** remains in `kv.status.md`, its sentences change only
  to say so, in their order.
- `python3 -I -B tools/citations/citations.py --base origin/main` reports nothing broken.

## Inputs

- `skein/docs/plan/kv-pay-per-use/notes/common.md`; `DESIGN.md`, 2 and 4.
- `skein/docs/design/kv.md`, 3, 5, 7 and 9; `kv.status.md` and `STATUS.md` as they stand.

## Owns and may change

- The two files in the header; nothing else. A gap a task should have closed is written as
  **Not yet** and listed in the report, never fixed here.

## Steps

1. Read the code, the tests and the commits; write the two files.
2. The citations check.
3. Commit (why: kv's status says what still lags once the store pays for what it holds); push
   your branch.

## Report

As the implementer's text says, and also: each entry removed, kept or written, and why.
