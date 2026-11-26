# A landing that can move: contracts

2026-11-26. The shapes the changes of the goal building
[`landing-liveness.md`](landing-liveness.md) share, and no other area
reads (`../draft/design-economy.md`, 4.3). They bind as that document
does. The goal's last change deletes this document once the code holds
them.

## 1. The policy's check and verdict

In `temper_engine_domain_forge_landing`'s boundary, beside `Hold`,
`Decision` and `Facts`. The check's change makes them.

```rust
/// What fails a live landing's check (landing-liveness.md, 2.1 and 3.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Stuck {
    /// A member still in it, neither left nor landed: held, or, `held` false,
    /// its task closing, ended or gone.
    Member { change: u64, held: bool },
    /// The live landing, another, that holds its train.
    Train { landing: u64 },
    /// Its wait, which names no wake that can come.
    Wake { on: Awaited },
}

pub enum Hold {
    // every kept variant, in its place, then:
    /// Held by the check's verdict, something having landed (landing-liveness.md, 3.2).
    Stuck(Stuck),
}

pub enum Decision {
    // every kept variant, in its place, then:
    /// Ended by the check's verdict, nothing having landed: the root asks jig to
    /// cancel the landing's task with the cause's words (landing-liveness.md, 3.2).
    End(Stuck),
}

pub struct Facts {
    // every kept field, then:
    /// The train's holder where it is another live landing
    /// (landing-turn.md, 2); none for a single landing.
    pub train_holder: Option<u64>,
}
```

## 2. The step's hearing

In `temper_engine_domain_forge_vocabulary`'s `LandingHeard`, beside
`held`. The wakes' change makes it; the root fills it as it fills `held`.

```rust
pub struct LandingHeard {
    // every kept field, then:
    /// Members whose task is closing, ended or not among jig's live tasks
    /// (landing-liveness.md, 4). The top reads each as `MemberFacts::cancelled`.
    pub ended: Box<[u64]>,
}
```

## 3. The shared ending

In `temper_engine_domain_forge`'s `landing` module, crate-private. The
endings' change makes it from what a cancelled landing's closing does;
the start's change calls it for a row whose task is gone.

```rust
/// Ends a live landing row as its task's closing does (landing-liveness.md, 5):
/// phase `Cancelled`, members told or left ready, index entries and own
/// asks gone, runs no longer followed, train woken, history closed. A row
/// already ended is left as it is.
pub(crate) fn end_row(d: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>);
```
