# skein-world-costs: what every world pays to count its heap and keep its wire's trace

Every tree's worlds run under `skein_heap::Counting`, and many script their peers through `skein_world::stream::Wire`. Every tree's focused and fuzzy suites have limits of 30 and 120 CPU s (`skein/docs/foundation/testing-strategy.md`, 8 and 8.1), and several are over. This plan is 8.1's second question for these two: they belong, so make them cheaper at the cause. A fix here reaches all five trees.

Profiled with callgrind on main, inclusive instructions:

| Cost | Where | Share |
|---|---|---|
| `Counting`'s `alloc` and `dealloc` | `skein/testing/skein-heap/src/lib.rs` | 38% of a median `jig-core-world` focused test, 33% of a `jig-client-protocol-world` test, 32% of a `jig-ops-world` fuzzy test. `malloc` and `free` themselves are about 2% |
| `alloc::fmt::format` from `Wire::answer` (8.2%) and `Wire::take` (4.14%) | `skein/testing/skein-world/src/stream.rs` | 13.67% of 153M instructions, over 1,292 calls, in `client::account_keys::a_key_ask_carries_its_value_into_the_secrets_store` of `jig-client-protocol-world` |

## What's wanted

**1. `Counting`, with the same counts.** `allocated` makes at least five `LocalKey` reads or writes per allocation and `freed` at least two, and each is several calls in an unoptimized build. `padded` calls `Layout::from_size_align`, with its checks, on both paths. The lead: one thread-local that holds the counters, reached once per call, and no `Layout` round trip where its conditions are already known. Keep what the crate's comments require: reaching it never allocates, never fails and never panics; const-initialised and without a destructor; nothing allocates while the highs are borrowed. `alloc` still answers null past what a layout can describe. **`tests/heap` passes unchanged:** no test there changes, and every count, peak and hand-out stays exact.

**2. `Wire`'s trace, read only when a test fails.** `take` and `answer` push a formatted `String` on every step, including `format!("fragment {bytes:?}")` of the raw bytes. The lead: keep what a failure needs in a cheap form, and format it when something reads it. It's a public field, read by other trees' worlds (`wire.trace.clear()`, for one): change its callers in this plan, whichever trees they're in. What a failure prints stays as useful as it is today.

**3. The same pattern elsewhere in skein's world helpers, where a profile shows it.** Profile one median test of each of `skein-forge-machine-world`, `skein-llm-world` and `skein-tls-world` once 1 and 2 are done. Largest first; stop when the next cause's gain is under about 1% of a test. A cause in a test's own fixture isn't this plan's: another plan takes those.

**4. Report before and after, in instruction counts,** for the jig tests named above and the three skein tests, with the cause of each and whether it's fixed. A cost the code fix can't reach gets a line with its share: a `[profile.dev.package.skein-heap]` override is the user's to sign off under section 8, with those numbers, so don't add one.

At most three tasks, one for each of 1, 2 and 3. `walk = false`. Effort medium.

**Overlaps:** many plans add worlds that use these. Expect catch-ups. `skein-json`'s string paths are another plan's, and so is what a process costs before a test starts: leave both.

## Sources

Read by section, too large to be a source: `skein/docs/foundation/testing-strategy.md`, 8 and 8.1.

- `skein/testing/skein-heap/src/lib.rs`
- `skein/testing/skein-world/src/stream.rs`
- `skein/tests/heap/tests/suite/main.rs`

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, or one of their stories. No walk, render check or throwaway driver.
- **Find the cause with a profile before changing anything** (section 8, "A slow test is understood, never hidden"). `valgrind --tool=callgrind` on one test by name gives instruction counts that a loaded machine doesn't move; quote them before and after. CPU seconds on these machines swing by a third or more between runs.
- **No budget, timeout, deadline or build setting changes.** Test builds stay unoptimized.
- **Nothing is lost.** No test leaves its suite, and no assertion weakens, in this plan.
- **Decide, don't ask.** A small edit just outside a task's file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
