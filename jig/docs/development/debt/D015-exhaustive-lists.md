# D015: Long or-pattern lists stand where a classifier on the type would do

- **Kind:** complexity
- **Where:** `jig-domain-people/src`, `jig-domain-core/src`, `jig-domain-tasks/src`, `examples/ops/domain/src`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T7 "exhaustive lists", A1 F15, A3 F8, A4 F5, A10 F9, A12 F10, F12)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (T7, "classifiers belong on the owning type")
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (a new variant is added in one place, and files shrink)
- **Size:** M
- **Status:** open
- **Paid when:** a question asked of a vocabulary in several places (is it new work, which activity is it, which rules apply) is a method on the type, and callers match on its answer.
- **Measure:** `rg -c '^\s*\| ' jig/crates/jig-domain-people/src -g '!**/tests/**' -g '!tests.rs' | awk -F: '{s+=$2} END {print s}'` (986 at base)

## What

Exhaustive matching is the house style, and it is right for routing. But the same lists
are written again and again to ask one question:

- people's production code has 986 or-pattern continuation lines, mostly re-matching an already routed `Ask` or `Outcome` (1,061 at the review, by its own count);
- the core's has 3,165, `ops`'s 580 (the same command on `jig/crates/jig-domain-core/src` and `jig/examples/ops`), including 38 `core::Held` variants listed in `ops`'s intercepts;
- tasks names `MessageKind::` 464 times in production; `impl MessageKind` (`boundary.rs`, line 2225) exists but most callers still match the variants themselves.

## Why it matters

Each list keeps "a new variant breaks the build" only by being edited, so one new act or
event touches 20 to 50 files (the quality run's count), each under a
`match_same_arms` or `too_many_lines` exception.

## What pays it back

Accessors and classifiers on the owning type (`Event::new_work()`, `Stored::activity()`,
`MessageKind::rules()`), sub-enums for people's `Ask` and `Outcome` families; callers
match the answer. Lowers D016 and D017 too.
