# github-repair-race: the plan

From `inputs/request.md`: the replacement github_repair story in temper's real loop ends on a
`change show --until held` that can match hold 1 before the replacement repair runs, and under
load fails with "story completion obligations: undeclared engine task 1: Running". M7a's trunk
fixed it in `063edf0d7e` (`inputs/fix.patch`). One direct part, one task, as the request asks:
the same commit on main.

## Parts and tasks

- **replacement-wait** (`temper/docs/design/domain/testing.md, 2.4`), slice 1.
  - `tasks/replacement-wait.md`, medium, ~40 minutes, needs nothing: `git cherry-pick -x
    063edf0d7e` (one file, +8 -2, blob `9610423b40`), then the test alone five times and the
    `suite` binary three times under heavy, one command at a time.

## Critical path

`replacement-wait`, then fab's end check on the trunk.

## Overlaps

- `reference-m7a` carries the same commit (`reference-m7a/repair-1`). At the cut main's
  `github_repair.rs` is `063edf0d7e`'s preimage blob (`a2e050d61c`), so the pick reaches the
  same blob as M7a's trunk and the two merge cleanly. The spec keeps the bytes as picked
  (no reformat; at the cut the fix's file was rustfmt-clean).
- No other draft beside this one names `temper/tests/real-loop/**`.

## Calls made

- No design change: the fix is a test's wait, inside what `domain/testing.md, 2.4` says.
- The repeated runs depart from `AGENTS.md` 6's "no unchanged rerun", stated in the spec: the
  request asks for them, to show the race gone under load.
- A failure of this test in any run stops the task with a question, rather than a second fix
  on main that would part from M7a's.
