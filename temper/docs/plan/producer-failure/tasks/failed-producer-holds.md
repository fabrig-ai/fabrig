+++
id = "failed-producer-holds"
part = "failed-producer"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain-forge-change/src/domain.rs", "temper/crates/temper-engine-domain-forge-change/src/boundary.rs", "temper/crates/temper-engine-domain-forge-change/src/tests.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/tests/engine/domain/tests/suite/forge_routes/producer_failure.rs"]
effort = "high"
minutes = 60
+++

## Goal

A producer that delivered and then finishes `failure` goes to `opening` today, so
partial work opens as a pull request, and an empty title reaches the forge (422,
`open pull request: empty operation`). Make it hold at producing, with its failure's
words on the hold, its pushed work on its branch and no pull request. A release
makes a fresh producer. No empty title reaches a forge by any route.

Acceptance:

- A producer that pushes, then ends failed: the change is `Held { was: Producing,
  why: Failed }`, the hold's words are the failure's, no `OpenPull` is written.
- Released, it makes one fresh producer (one more of its tasks), whose workspace
  starts from the pushed branch; that producer's success opens the pull with its title.
- Released without the authority for one, it holds by the authority rule.
- A repair and a resolution that pushed and then failed hold, their head not taken
  (`delegated` reads so today; pin it, fix it if the pin shows otherwise).
- An `Effect::Open` write for a row with an empty title carries `Update repository`.
- A head with no title and no pull request never opens from `producing`.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.1 (the pull's title) and 8.4 (*A release*).
- `temper/docs/design/domain/forge.md`, 8.11, *Early starts*: an early producer's
  failure holds nothing, and that stays.
- `temper/docs/plan/producer-failure/notes/common.md`.

## Owns and may change

Owns: `producing` and `released_state` in
`temper-engine-domain-forge-change/src/domain.rs`; `Facts` in its `boundary.rs`;
`change_facts`, `change_write` and the title default in
`temper-engine-domain-forge/src/domain.rs`; the tests below.

May change:

- `temper/crates/temper-engine-domain/src/route/forge_route.rs`:
  `initial_change_title`, only to use the shared default constant.
- Any literal of `change::Facts` the compiler names: the new field only.

Options, pre-approved:

- If the `None` arm's `Status::Failed` in `producing` can no longer fire, keep it
  listed (every variant is listed) or restructure the match; your call.
- If a released resolution holds again on its stale `Failed` (`released_state` keeps
  `Resolving { requested: true }`), that is not this task's: report it under `## Left`
  unless your pin for a resolution needs it. Then give it the repair's arm and say so.
- If `titled_change`'s wait can be dropped because `producing` now covers it, leave it:
  it keeps the stall deadline (8.1).

Not yours: `temper-engine-domain/src/route.rs`'s `keep_produced_title` (a failed
producer has no title to keep), `relay.rs`, any stock text, jig.

## Steps

Sketch, in `temper-engine-domain-forge-change/src/domain.rs`:

```rust
// Early arms unchanged: an early producer's failure holds nothing (8.11).
if requested && standing.delegate == Status::Failed {
    // Whatever it delivered (domain/forge.md, 8.4).
    return hold(change, Hold::Failed);
}
match facts.branch {
    Some(head) if facts.titled => { /* today's move to Opening */ }
    Some(_) if !requested => request_producer(change, facts, standing, now),
    Some(_) => waiting_on_agent(change, facts, now, limits),
    None if !requested => { /* today's */ }
    None => { /* today's */ }
}
```

`released_state`: `State::Producing { requested: true } if why == Hold::Failed` becomes
`State::Producing { requested: false }`, the head and `last_head` conditions gone.

`Facts` gains `pub titled: bool`: the row's title is known, or it has a pull request.
`change_facts` sets `titled: !row.title.is_empty() || row.pull.is_some()`. A catch-up
and a handed branch have a title from their making, so they open as they do today. The
crate's test `facts()` sets `titled: true`.

`change_write`, `Effect::Open`: a title that comes out empty takes the default. One
constant in `temper-engine-domain-forge` holds `Update repository`, used by
`produced_title`, `initial_change_title` and `change_write`.

Tests, failing first:

1. `temper/tests/engine/domain/tests/suite/forge_routes/producer_failure.rs`, the world
   the request calls the forge world for this story, since the core and the connector
   meet there: `a_producer_that_delivers_then_fails_holds_at_producing`. Producer pushes
   (`change_producer.rs`'s `produce` shows the push), finishes
   `TaskResult::Failure { reason }`. Assert the hold, its words, `row.pull.is_none()`,
   no open-pull write. Then `a_released_failed_producer_with_a_head_makes_a_fresh_one`:
   release; a second producer is assigned; it finishes a change with a title; the pull
   opens with that title.
2. `temper-engine-domain-forge-change/src/tests.rs`, beside `failed_producer`:
   failed with `facts.branch = Some(HEAD)` holds; released, the decision is
   `Delegate(Produce)`; released with a lacking authority holds `Authority`; `Early`'s
   failure with a head still holds nothing; a repair and a resolution with a moved head
   and `Failed` hold; `titled: false` with a head and `requested: false` asks a producer.
3. `temper-engine-domain-forge/src/tests.rs`: `change_write` with an empty title gives
   `Update repository`.

Each new test bites: remove its rule once, see it fail for that reason, restore.

Then narrow tests, `cargo fmt`, clippy on the changed packages, the focused tests of
`temper-engine-domain-forge-change`, `temper-engine-domain-forge`,
`temper-engine-domain` and the `temper/tests/engine` worlds (the rule changes what a
shared step means, so its readers run), commit, push.

## Report

The failing test's first output; the cause, confirmed or corrected; whether a
resolution's release was touched; what is left.
