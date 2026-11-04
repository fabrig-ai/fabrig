# Request: follow-ups from the spot review of debt-4

From fabrig/debt, 2026-11-04 (the user's continuous tech-debt process). A read-only review of
debt-4's skein-kv and jig D028 changes found two defects in the store and two stories that cannot
fail. `inputs/kv-scripted-review.md` holds the findings; nothing in it was run, so each claim is
checked against the code before it is acted on. Two independent parts.

Every part, on its own branch:

- checks each finding against the code as it stands now. A finding that is wrong or has moved on
  is skipped, and the report says why;
- writes acceptance searches narrowly, as earlier rounds learned;
- stops and asks if a change needs a design change, a breaking public interface change or a
  stored-format change;
- keeps `tools/citations/citations.py --all` at 0.

**A test that pins a rule must be able to fail on that rule.** For each test or story added or
changed here, break the rule by hand and watch it go red, then say in the report what was broken
and what failed. "It passes" is not the check.

**All parts must be in the plan when the cut ends.**

## 1. skein-kv: Part A of the review

Findings 1 and 2 come first, each with a test that fails on today's code:

- finding 1: a put-and-erase churn on distinct keys under sustained load, showing `charges` staying
  bounded. The test must fail before the fix;
- finding 2: an `io_error` mid-snapshot, asserting `open_files() == 0`.

Then findings 3 to 8, in the review's order. In finding 6, leave the eight relabelling handlers as
they are. The on-disk format is unchanged, and tests/kv's crash and golden tests pass unchanged. Aim:
`store.rs` and its modules end near the 1,250-line cap that debt-4's tasks set, about 200 lines down,
less what the two fixes add.

## 2. jig: Part B of the review

Findings 1 and 2 come first: D028 was paid on the promise that these rules are held, so they must be
held for real. Finding 1 is a story that goes red when the `Changed` arm is replaced. Finding 2
is a story that goes red when `requester: by` comes back, or, if that costs too much, the story and
`Facts::DelegateEscalation` are deleted and `testing.status.md` records the gap.

Then findings 3 to 8. The net line change of this part should be negative.
