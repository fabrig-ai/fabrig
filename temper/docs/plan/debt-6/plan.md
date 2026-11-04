# debt-6: the plan

From `inputs/request.md`: two independent parts, one per tree, from the spot review of debt-4
(`inputs/kv-scripted-review.md`). `notes/common.md` holds the standing rules every task names.
This file grows with each release; what is here so far is what has been written.

## Parts and tasks

1. **kv-review** (`skein/docs/design/kv.md, 7`), slice 1. `notes/skein-kv.md` holds what stays and what each
   task runs. One writer for `skein-kv`, in the review's order: the request asks for that order, and
   every task touches `store.rs`.
   - `tasks/kv-charges.md`, high, ~60 minutes, needs nothing: finding 1.
   - `tasks/kv-snapshot-file.md`, high, ~60 minutes, needs the first: finding 2.
   - `tasks/kv-phase-ready.md`, medium, ~25 minutes: findings 3 and 4.
   - `tasks/kv-admission.md`, high, ~40 minutes: finding 5.
   - `tasks/kv-forwarding.md`, medium, ~30 minutes: findings 6 and 7.
   - `tasks/kv-recovery.md`, high, ~40 minutes: finding 8, and the part's line count.

2. **jig-review** (`jig/docs/design/client/domain/testing.md, 4`), slice 2. `jig/tests/client` has one
   writer at a time; the fake's two small findings run beside it.
   - `tasks/jig-changed-story.md`, high, ~40 minutes, needs nothing: finding 1.
   - `tasks/jig-delegate-story.md`, high, ~90 minutes, needs the first: finding 2.
   - `tasks/jig-fake-tidy.md`, medium, ~20 minutes, needs `jig-delegate-story`: findings 6 and 7.
   - `tasks/jig-story-fixtures.md`, medium, ~45 minutes, needs `jig-delegate-story`: findings 3 and 4.
   - `tasks/jig-story-tidy.md`, medium, ~35 minutes, needs the last two: findings 5 and 8, and the
     part's net lines.

## What the cut decided

Checked against the code at the plan's base. No finding was wrong or had moved on; the review's line
numbers still hold. Each task re-checks its own.

- **kv finding 2 reaches further than the review says.** A failed rollover create or directory sync
  reaches `open_failed`, which drops a held snapshot file just as `recover` does. `kv-snapshot-file`
  closes it on both. The rollover's unanswered commits stay: `kv.status.md`, 4 records them, and
  answering them is not this request's.
- **`Phase::RolloverClosing`'s failure has no test:** the sim never fails a `Close`.
- **kv finding 4 takes the expect onto `enum Phase`,** not a boxed `bytes`, which would hold the old
  suffix beside the joined one, past recovery's heap bounds. Tried with rustfmt: 43 lines.
- **kv finding 6 removes three wrappers, not four.** `recovery_closed` gains work in `kv-snapshot-file`.
  The relabelling handlers the request keeps are ten, where the review says eight: all stay.
- **The kv line aim does not add up.** The request says near 1,250, and about 200 down. `store.rs` and
  `compaction.rs` read 1,670, and the review's savings in them bring that to about 1,490 with the fixes
  added back. `kv-recovery` reports the number and cuts nothing to reach it.
- **jig finding 2 goes further than the review's fix.** The review separates the parent's number from
  the person, and leaves the `by` gap; the request wants the story red on `requester: by`. That needs a
  decider other than the person who asked, which the default profile's one role cannot give, so the story
  moves to the role fixture: a member's chat, held to the owner. The request's deletion path is
  pre-approved, with a bound the task states.
- **The fake's findings 6 and 7 wait for jig finding 2,** though they share no code with it, because the
  request puts findings 1 and 2 first. The chain behind them is longer, so this costs no time.
- **Two chains, by crate.** `store.rs` and the stories' files are each one writer's at a time, and the
  review's order already runs along them.

- **The note is `notes/skein-kv.md`.** The first push named it `notes/kv.md`, and the citation checker
  takes plan files as targets, so a bare `kv.md 3` in `jig/tests/store` resolved to it and `--all` read 1.
  Renamed in the next push. `kv-charges`'s spec still names the first path, because a pushed spec never
  changes: that task and `jig-changed-story` have the old file at their base, so their `--all` reads 1
  until they meet the trunk. Neither touches the note, so the merges are clean.

## Overlaps with the drafts beside this one

None seen: nothing listed for fabrig-scale, qa-out, debt-5 or bugs-temper touches `skein/crates/skein-kv`,
`skein/tests/kv`, `jig/tests/client` or `jig/testing/jig-client-scripted-engine`. The queued drafts
v0.1.0-ladder and sign-in-wait name no files yet.

## Critical path

`jig-changed-story`, `jig-delegate-story`, `jig-story-fixtures`, `jig-story-tidy`: about 210 minutes.
`kv-review`'s chain runs beside it at about 255 minutes, and is the longer: it is the plan's critical path.
Neither waits on the other.
