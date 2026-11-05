# debt-16: the sixteenth debt round

Three parts (`inputs/request.md`), from the spot review of debt-13 (`inputs/debt13-review.md`), read and not run. This
file is brought up to date with each release; the parts still to come are listed at its end.

## Parts and tasks so far

| Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---:|---|
| `wall-due` | `wall-due` | | medium | 15 | finding 5: `expiry <= env.wall` at the call sites |
| `core-small` | `core-small` | | medium | 20 | finding 10, jig's half |

## Findings checked against the code so far

- **5, right.** `Wall` is `Ord`; `wall_due` differs from the comparison only at a saturated `now`, the wrong answer its
  test pinned. Three call sites, two in `jig-test-domain` and one in `ops`, beyond the review's "call sites".
- **10, jig's half, right.** `Core::cold_header` has only its own test; `Core::commit_header` has only tests and
  `cold_header`. `Counters::commit_header` stays: temper's `decision.rs` calls it.

## Still to come

Part 1's findings 2, 4, 8, 9 and the rest of 10; part 2; part 3.
