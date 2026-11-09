# projection-full-outbox: contracts

The run's binding calls, between `room-wait` (the forge connector,
`temper-engine-domain-forge`) and `requeue-backstop` (the root,
`temper-engine-domain`). Design: `temper/docs/design/domain/forge.md`, 6
(*The outbox is bounded*) and 12 (*A full outbox defers it*).

## The wait (`room-wait`)

A projection whose next write is an effect while the outbox holds its
bound (`d.entries.len() == env.limits.entries`) waits for room, by the
idiom `fail_projection` already keeps for a failed write's retry, minus
the failure:

```rust
// temper-engine-domain-forge/src/domain.rs, private
fn wait_for_room(d: &mut Domain, env: &Env<Limits>, goal: u64, out: &mut Queue<Request>) {
    let when = skein_lib::Wall::from_nanos(
        env.wall.as_nanos().saturating_add(env.limits.issue_policy.interval.as_nanos()),
    );
    d.projection_due.insert(goal, when).expect("one deadline per issue");
    d.full_outbox_met = true;
    emit(out, Request::ProjectAfter { goal, when: Some(when) });
}
```

- The row stays as it is: not `failed`, no `pending`, its `desired` kept
  (already saved above the decision when it changed). No `Save`, no
  `ProjectionFailed`, no `ProjectionSettled`.
- The other half of today's condition, an entry number already in the
  outbox (`d.entries.contains_key(&number)`), keeps `ProjectAfter { goal,
  when: None }`: the root asks again with a fresh number.
- The timer fires as every projection timer does (`fire`, `ProjectAfter
  { when: None }` once due); a restart re-arms it (`Stored::Issue`'s
  restore, a row with nothing in flight). Both exist; neither changes.

## The step's mark (`room-wait` makes it, `requeue-backstop` reads it)

On the forge's `Domain` (`temper-engine-domain-forge/src/domain.rs`):

```rust
/// Volatile: a projection met the full outbox since the root's step began
/// (domain/forge.md, 12). Never saved; false after a restore.
full_outbox_met: bool,

impl Domain {
    /// Whether a projection met the full outbox since the root's step began
    /// (temper/docs/design/domain/forge.md, 12).
    #[must_use]
    pub fn full_outbox_met(&self) -> bool;

    /// The root's step begins: forget what the last one's projections met.
    pub fn begin_step(&mut self);
}
```

Set only by `wait_for_room`; cleared only by `begin_step`; false in the
constructor.

## The backstop (`requeue-backstop`)

The root calls `domain.forge.begin_step()` beside `domain.changed()` at
the top of each of its public entries that start a step: `step`, `fire`
and `release` (`temper-engine-domain/src/domain.rs`).

The root's `forge::Request::ProjectAfter { goal, when: None }` arm
(`temper-engine-domain/src/route/forge_route.rs`) pushes
`Work::Forge(forge::Event::ProjectDesired { entry, goal })` as today
unless

```rust
domain.forge.full_outbox_met() && !domain.forge.effect_room(&env.limits.forge, 0)
```

in which case it pushes no work and arms the projection's timer instead:
`domain.forge.projection_after(goal, when)` with `when = env.wall +
env.limits.forge.issue_policy.interval` (saturating, as `wait_for_room`
computes it). The `when: Some(..)` arm and the `pending` check stay as
they are.
