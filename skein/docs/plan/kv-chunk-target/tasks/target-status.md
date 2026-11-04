+++
id = "target-status"
part = "status"
kind = "docs"
needs = ["target-pack", "target-read"]
files = ["skein/docs/design/kv.status.md"]
effort = "medium"
minutes = 20
+++

# skein: kv's status, once a chunk packs to the target

## Goal

Once both code tasks have merged, `skein/docs/design/kv.status.md` says where skein's code
still lags `skein/docs/design/kv.md`, and nothing more (`DESIGN.md`, 2). This task departs from
the implementer's text, which keeps `docs/design/` out of reach: this status file is its own.

Acceptance:

- Read the trunk's code and tests for each **Not yet** entry this plan's cut wrote (5, 7: the
  target and the one encoding; 3, 7: reads of the target and the frame's carry; 9: the checks
  under a large `Limits::chunk`), and remove each one the code now meets. An entry the code
  still misses stays, said as it now stands. With only the entry on 9 left, `**Not yet:**`
  takes it in the form of its first line again, unnested. Date the file today (`date -u`).
- The **Departs** entry on 3, 7, 9 keeps its account of what a restart holds, and its last
  sentence's `3 × chunk + commit` and `4 × chunk` say the bounds as the tests now have them:
  `skein/tests/kv/tests/suite/recovery.rs`'s, whose `chunk` is below the target, so that it
  stands for the read; and `target.rs`'s under a large `Limits::chunk`, in the target and the
  largest chunk or frame held (`4 × target`, `2 × target + 2 × chunk` and
  `3 × target + 2 × frame`, each plus its slack, as `target-read` measured and left them). Say
  that `Limits::chunk` sets none of them.
- The **Departs** entry on 11: its figures say the two files as they now read
  (`wc -l skein/crates/skein-kv/src/store.rs skein/crates/skein-kv/src/store/compaction.rs`):
  the total, and how far it is past 1,250 and past 1,490. The reason it gives stands, with the
  lines this plan added named as what moved them.
- A part of kv.md 3, 5, 7 or 9 built otherwise than it says is a **Departs** entry with its
  reason, taken from the commit that built it (`git log -- skein/crates/skein-kv
  skein/tests/kv`); one whose reason no commit gives is listed in the report as a question, not
  written.
- `skein/docs/design/STATUS.md` stays: kv still lags on 9.
- `python3 -I -B tools/citations/citations.py --base origin/main` reports nothing broken.

## Inputs

- `skein/docs/plan/kv-chunk-target/notes/common.md`; `DESIGN.md`, 2 and 4.
- `skein/docs/design/kv.md`, 3, 5, 7, 9 and 12; `kv.status.md` as it stands.
- `skein/tests/kv/tests/suite/target.rs` and `recovery.rs`, for the bounds as they are.

## Owns and may change

- `skein/docs/design/kv.status.md`; nothing else. A gap a task should have closed is written as
  **Not yet** and listed in the report, never fixed here. Keep every other entry as it is.

## Steps

1. Read the code, the tests and the commits; write the file.
2. The citations check.
3. Commit (why: kv's status says what still lags once a chunk packs to the target); push your
   branch.

## Report

As the implementer's text says, and also: each entry removed, kept or written, and why.
