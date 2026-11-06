+++
id = "resumed-order"
part = "resumed"
kind = "feature"
needs = ["uncap"]
files = ["hearth/tests/builder/tests/suite/orders.rs", "hearth/crates/hearth-service-builder/src/tests.rs", "hearth/crates/hearth-protocol-builder/src/tests/**", "hearth/docs/development/debt/README.md", "hearth/docs/development/debt/D014-*.md", "hearth/docs/design/domain/render.md", "hearth/docs/design/domain/render.status.md"]
effort = "high"
minutes = 60
+++

# What resumed work's order protects: one more try

## Goal

"Resumed work goes before the next event" is held by `work_pending` and by no test: 384 fixed cases couldn't show what it keeps. Try once more, in about **45 minutes of work**, with the queues now uncapped: pending cache work alongside stdin room, a child's exit and a pipe's end. Either way, the search ends here, and D001 can go.

**The guard stays in both outcomes.** Nothing here removes it from the code you push.

Acceptance, one of two:

- **A test bites.** It's added, named for the order, and fails for that reason with the guard out. `render.status.md`'s **Not yet** on 8.5 ("no test holds that resumed work...") goes, and so does `render.md`, 10's "What resumed work's order protects".
- **None does.** The order stays a named predicate that `render.md`, 8.5 states with no test. A debt item, `hearth/docs/development/debt/D014-<slug>.md`, holds the search: what was tried, on which cases, and what each showed, with D001's 384 cases carried over, so that D001 can be deleted. `hearth/docs/development/debt/README.md` raises its highest ID. `render.status.md`'s entry says so, citing D014 and no longer D001, and `render.md`, 10's entry stays, without its status of "a search still to run": it says what would show the order.

In both: `python3 -I -B tools/citations/citations.py --all` stays at 0, and the time you spent is in the report. Past about 45 minutes with nothing biting, stop and take the second outcome: that is the request's call, not a failure.

## Inputs

- `hearth/docs/plan/hearth-memory-6/notes/common.md`
- `hearth/docs/plan/hearth-memory-6/inputs/request.md`, 3, its last bullet
- `hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "The service-order search": what was tried, on which cases
- `hearth/docs/design/domain/render.md`, 8.5 and 10
- `hearth/crates/hearth-protocol-builder/src/lib.rs`, `Protocol::work_pending`, `resume` and `up`; `read.rs`, `cache_work_pending`
- `hearth/docs/development/debt/README.md`: the entry's format, and the next free ID. Check it's still D014 at your base.

## Owns and may change

- **Owns:** the files under `files`. `Departs from your text:` you edit `render.md`, 10 and `render.status.md`, 8.5, because this task's result is what they state, and only those entries.
- **The search is yours.** The mutations are: take `service.protocol.work_pending()` out of the `told` loop's check in `protocol_up`; then also out of the `file_told` loop's; then also the early return before both. They are temporary, in your slot.
- **Where to look first,** in order:
  1. By reading, not running: what does `Protocol::up` do with `io::Event::Output` (stdin room), `Exited` and a pipe's `Stream` end, when `read.cache_work_pending()` or a closing child has work? Which of them replace state that `resume` was about to use?
  2. A step test in `hearth-protocol-builder/src/tests/`, through `Layer`: enter with the work pending, against resuming first. A difference in `above`, in `below`, or a panic, shows a consequence.
  3. A consequence found, hold it where the guard is: a test in `hearth-service-builder/src/tests.rs` pushing to `service.told` and calling `crate::protocol_up`, or a world in `orders.rs` with a fixed seed, at `bounds::limits()` and a small git chunk.
- **A test at the step level alone doesn't hold the guard.** The guard is in the service, so the test must fail when the guard is out of `protocol_up`. A step test showing the consequence is a finding, worth a line in D014 if no service-level test follows.
- **A bug found** is a finding first. If entering with work pending can do harm the guard already prevents, that's the bite. If the search finds a bug the guard doesn't prevent, report it under `## Left`: don't fix it here.

## Steps

1. Read, then try, in that order.
2. Take the outcome that holds. For the second, D014's sketch:

   ```md
   # D014: <a short claim: resumed work's order has no test>

   - **Kind:** tests
   - **Where:** `hearth-service-builder/src/lib.rs`, `protocol_up`
   - **Seen:** <date> at `<commit>`, in `hearth/docs/plan/hearth-memory-6/inputs/request.md`, 3
   - **Checked:** <date> at `<commit>`
   - **Value:** low (a guard no test can yet fail)
   - **Size:** S
   - **Status:** open
   - **Paid when:** <a test fails with the guard out, or the guard is shown to keep nothing and goes>

   ## What
   ## Why it matters
   ## What pays it back
   ```

   It keeps the search: the 384 cases, what this round added, and what would show the order.
3. Restore every mutation: `git diff` shows the guards as they were.
4. `heavy cargo nextest run -p hearth-service-builder -p hearth-builder-world --test suite` as far as your change reaches, and clippy on what you changed. Commit and push.

## Report

The first line says which outcome. What was tried, each mutation and what it showed, the time spent, what ran, and the CPU seconds of any test added.
