+++
id = "walk-waits"
part = "walk-waits"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/tests/fake_backends/walk/goal.rs",
  "temper/crates/temper/tests/fake_backends/walk/away.rs",
  "temper/crates/temper/tests/fake_backends/walk/status.rs",
  "temper/docs/design/domain/testing.status.md",
  "temper/docs/plan/test-speed/figures/walk-waits.md",
]
effort = "high"
minutes = 100
+++

# The fake-box walks wake on what they wait for

## Goal

Four walks idle for most of their wall (best wall against median CPU, in s): `a_goal_decides_while_away_then_waits_on_home_after_overturn_and_off` 23.7 against 4.1; `a_plans_project_has_its_brief_on_a_return_past_the_sitting` 52 against 32; `home_on_the_fake_box_keeps_a_landing_count_after_box_and_client_restarts` 18.4 against 6.3; `a_chats_goal_on_the_fake_box_shows_its_report_in_the_goals_list` 11.3 against 2.6. Find what each waits for, from a timeline and not from reading, and make it wake on the event, or on the short paces the status walks set.

Acceptance:

- **A timeline first:** `figures/walk-waits.md` holds, for each of the four, its wall by range between its `ctl` acts, read from its `box.log` and the acts' times, naming at least 90% of the idle wall and what each range waits for. Home's status walk is slow on the short paces already: its table says why.
- **Each wait repaired where it lies:**
  - an event that exists: the walk waits on it (`--until`, a printed line);
  - only the forge's poll brings the news: the walk runs on the status walks' paces, from one helper they share;
  - a wait in the product that a person meets too (a poll where a wake exists, a settle that outlasts its reason): red first in the lowest tier that shows it, then the fix, and the figures say a person met it.
- **The oracle stands:** no assertion of the four walks is weakened, and no bound is raised.
- **Figures** before and after: each walk's wall and CPU over at least three runs each, and the tier's at both ends (`notes/common.md`). A wait with no cheap repair stays, with its figure and reason, in `testing.status.md`'s entry.
- `testing.status.md`: the second sentence of the 2.6 entry the cut wrote (the walks' paces) goes, or says what stands. The first (one test at a time) is `tier-threads`'s: leave it.

## Inputs

- `temper/docs/design/domain/testing.md`, 2.6 (*A walk wakes on what it waits for*).
- `temper/docs/help/reference/limits.md`, the `forge.client.*` lines.
- `temper/crates/temper/tests/fake_backends/walk/status.rs`, near lines 25 to 34 and 119: how the status walks prepare, set the paces and reopen.
- `temper/docs/plan/test-speed/notes/common.md`.

## Owns and may change

Owns the files in the header.

Sketch: a helper on `Walk`, in `walk.rs` (`tier-threads` and `box-small/link-hours` touch this file too, so add it as its own small hunk and change nothing else there):

```rust
impl Walk {
    /// Reopens on short forge paces after a first run has prepared the box (client's wait: testing.md, 2.6).
    fn short_paces(&self);
}
```

Readers and neighbours, and what's allowed there:

- `walk.rs`: that helper alone.
- `temper/crates/temper/tests/support/**`: a wait's own lines, where a timeline shows the harness is what idles (a fixed sleep, a poll of its own).
- **Product code a cause lies in,** in any temper crate, with its test. A cause in jig or skein is reproduced and fixed in that tree's lowest tier and is the report's first item.

Options, each decided:

- If a first run can take the paces without a second start of the box (a walk reopening costs a box's start), do that; otherwise reopen, as the status walks do, and name the cost.
- If a short pace changes what a walk shows (it fills the journal with reads: `status.rs` leaves room for that), take the longest pace that keeps the walk under its old CPU, and say which.
- If a walk's CPU rises with a shorter pace, keep the change only where wall falls by more than the CPU rises, and report both.
- `a_plans_project…` spends 32 s of CPU: its CPU isn't this task's. Report what it's spent on, in a line, if the timeline shows it.

Run the `process` binary's fake-box walks (`-E 'test(/walk|fake_backends/)'`, check the count) after the change, and the whole tier for the figures.

## Steps

1. Measure the tier at the base, then the timelines. Commit nothing yet.
2. Repair the biggest wait first; each walk's narrow test after each change.
3. A new test that pins a rule bites: break it by hand, see it fail, restore.
4. Measure again; write the figures.
5. `cargo fmt`, `old_forms.py`, clippy on the packages changed, the tests. Commit and push.

## Report

Each walk's wait in a line, before and after; which were a person's too; what stays and why.
