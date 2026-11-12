# goal-runs-itself: status

2026-11-10. Where the design lags or departs from [goal-runs-itself.md](goal-runs-itself.md).

Sections 1 to 8 are carried by jig's `domain/tasks/lifecycle.md`, 2 and 11, and
jig's `protocol/configuration.md`, 3.2; `../domain/forge/change/README.md`, 1, and `../domain/forge/branches.md`, 1 and 2
; `../protocol/tooling.md`, 4.2; `../domain/worker/jobs.md`;
`../protocol/configuration/engine.md` and `../protocol/configuration/settings.md`, section 2 and `../protocol/configuration/stock.md` and `../protocol/configuration/reference.md`; skein's
`forge/README.md`, 3.2; and fabrig's `DESIGN.md`, section 2 and
`tools/close/README.md`. `goal-runs-itself.md`, section 10's two defaults are the design's.

- **Design departs:**
  - 3: the closer is named by template 4's `ended_by`, `{"closer":
    {"connector": 1, "code": 6}}` (jig's `protocol/configuration.md`,
    3.2), not by a `closes` member of kind 11: every way of making a goal
    from a template carries the template's `ended_by` into the task,
    while kind 11 is read by no way that makes a task and says only, for
    the pages, what a template's charter does. Kind 11 keeps `design`
    and `close_out`, the reference's template 4 at design alone. Decided
    by the goal's designer, 2026-11-10, for the user's review.
  - 4: fabrig's close command, as temper's contract has every `close`
    command, stages its edits and temper commits them, one parent, as
    the merge job's command stages and temper commits
    (`../protocol/tooling.md`, 4.1 and 4.2), so that a command needs no
    git identity of its own. Decided by the goal's designer, 2026-11-10.
  - 4: a goal writes its tags `[in flight: change N]` with its changes'
    task numbers, which `change` gives only once a change is made, so an
    entry written before its change is tagged in the goal's next
    delivery (`../protocol/configuration/reference.md`, charter 5's cut).
    Decided by the goal's designer, 2026-11-10.
  - 6: charter 5's wake policy takes results `last_or_failure`, and a
    goal its closer ends is woken by a failure alone, its last result
    making its closer instead (jig's `domain/tasks/lifecycle.md`, 11); a
    designer's result and a person task's result reach it by the watch
    it subscribes to each. Decided by the goal's designer, 2026-11-10.
  - 7: charter 7 stays the reference's designer's charter, which the
    designers section 7 keeps run under; only its close-out goes.
    Decided by the goal's designer, 2026-11-10.
