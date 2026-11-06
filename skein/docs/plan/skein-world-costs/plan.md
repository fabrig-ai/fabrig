# skein-world-costs: what every world pays to count its heap and keep its wire's trace

The request: `inputs/request.md`. `skein/docs/foundation/testing-strategy.md`, 8.1's second question for two things that belong: make them cheaper at the cause. No budget, timeout or build setting changes, and no design text changes: the form of the counters and the trace is the code's, and `skein/docs/design/testing.md`, 5, stays true.

## Parts and tasks

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `counting` | `tasks/counting.md` | | `Counting` with the same counts, one thread-local reached once per call; the six tests named and read before and after |
| `wire-trace` | `tasks/wire-trace.md` | | `Wire`'s trace as `stream::Trace`: steps in a cheap form, formatted when read |
| `world-helpers` | `tasks/world-helpers.md` | `counting`, `wire-trace` | Three skein tests profiled, skein's helpers fixed largest first, and `skein/docs/development/world-costs.md` |

Three tasks, one for each of the request's items 1, 2 and 3. Item 4 is measured by each and written up by `world-helpers`. `walk = false`, and effort medium, as the request says.

Critical path: `wire-trace` or `counting`, whichever ends later, then `world-helpers`. The first two run side by side: neither builds on the other's code, and neither touches the other's files.

## What this plan decided

- **`Trace` keeps `Vec<String>`'s method names** (`len`, `clear`, `push`, `append`, `is_empty`), and adds `lines()` for text. Most readers compile unchanged, in this plan and in plans in flight, so the catch-ups the request expects are few. The edits left are type paths in `skein-llm-connection-world` and `skein-oauth-accounts-world`, and a word in two hearth comments. The other trees' readers only read `len`.
- **A replay compares steps and doesn't format them.** `assert_replays` becomes generic in its trace, as its callers name no type parameters. Reading the trace into `Vec<String>` to compare it would have kept the cost in `skein-llm-connection-world`, a passing path.
- **`take` copies a `Send`'s bytes, and still frees its box where it does today.** A `Meter` counts that free as handed out, so a trace that kept the box would charge the step for what it handed out, and would break memory tests. Bytes go in one store, each step holding how many, so there's no allocation per step.
- **`clear` frees capacity.** `hand.rs` clears so that a scripted peer's transcript isn't charged to the component, and today's `clear` frees every line. A store kept as large as the largest answer would be a new charge.
- **`len()` counts the same steps.** Four files read it as progress (`hooks.rs`, `hand.rs`, `net.rs`, `peer.rs`), and `net.rs` for a buffered fragment by name. `room delayed` counts too.
- **`Rng` draws don't move,** or every pinned seed changes course.
- **`counting` chooses the medians,** and the later tasks read its names. Its before arm is the pristine base, as it starts first, and so is the account's before column. `wire-trace` reads only the request's one named test, as it runs side by side and has no other names yet.
- **`refused` is a new test in `tests/heap`**, in a file of its own, with one `mod` line. The request says no test there changes, and none does, but nothing holds null past a layout's limit, and `counting` rewrites that path. It goes through `Vec::try_reserve_exact`, since the suite forbids `unsafe`.
- **`from_size_align_unchecked` may not pay:** its precondition check still runs in an unoptimized build. `counting` decides by the profile, and keeps the safe form where `unsafe` saves nothing.
- **"World helpers" means crates under `skein/testing/`.** `skein/tests/<world>/` is a test's own fixture, and `skein/crates/*` is the code under test: each gets a line with its share. A result of no fix is a good outcome for `world-helpers`.
- **The account goes in `skein/docs/development/`,** as the figures retire with the plan.
- **`valgrind` missing from a machine is a `Question:`**: no install, as machines are the user's. It wasn't on the machine this plan was cut on. The request's figures came from it, but the request doesn't say where, and this cut did not check the builders.

## Overlaps

None of this plan's files appears in the `Drafts beside this one` lists. These cuts were in flight when this was cut, with no files listed, and may add readers of `Wire::trace`: `jig-world-cost`, `smith-test-cost`, `limits-worlds`, `ui-fuzzy-joint` and `browser-cap`. A reader that counts, clears, pushes or appends compiles as it is. One that takes `wire.trace` as a `Vec<String>` takes a type path or `.lines()` at catch-up.

## Not this plan

`skein-json`'s string paths, what a process costs before a test starts, a cause in a test's own fixture, and any `[profile.dev.package.skein-heap]`: `world-helpers` gives what that override could reach, for the user to sign off.
