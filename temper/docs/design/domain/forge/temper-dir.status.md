# temper-dir: status

2026-11-28. Where temper's code lags or departs from [temper-dir.md](temper-dir.md).

- **Not yet:** 2, 3: nothing reads `.temper/` from a landing tip, and the store keeps no repository revision [in flight: change 22425].
- **Not yet:** 4: no change waits for temper's check of `.temper/` at its own head, under the queue or before a turn [in flight: change 22426].
- **Not yet:** 4: no turn judges its candidate's `.temper/` [in flight: change 22427].
- **Not yet:** 5: kind 4 has no `temper_dir_sign_off` [in flight: change 22419].
- **Not yet:** 5: no change under the queue waits for an owner's sign-off of `.temper/` [in flight: change 22426].
- **Not yet:** 5: no turn's member waits for an owner's sign-off of `.temper/` [in flight: change 22427].
- **Design departs:** 2: a home without method is not watched for moves; a
  first `.temper/ways.toml` added from outside temper is read at the next
  engine start or the next landing temper sees on that home, so a project
  without `ways.toml` runs exactly as before (the goal's requester, 2026-11-28).
- **Design departs:** 4: before comparing, the step a change may join a turn
  or land at reads, pinned to the change's head and its base's tip, whether
  either holds method; where neither does it makes no comparison and the
  change lands as before, so a project without `ways.toml` makes no comparison
  it didn't (the goal's requester, 2026-11-28).
