+++
id = "oauth-one-owner"
part = "oauth-owner"
kind = "feature"
needs = ["oauth-baseline"]
files = ["skein/tests/oauth-accounts/src/world.rs", "skein/tests/oauth-accounts/src/sign_in.rs", "skein/tests/oauth-accounts/src/sign_in/**", "skein/tests/oauth-accounts/src/private.rs", "skein/tests/oauth-accounts/src/private_cuts.rs", "skein/tests/oauth-accounts/src/accounts.rs", "skein/tests/oauth-accounts/src/shared.rs", "skein/tests/oauth-accounts/src/sign_out.rs", "skein/tests/oauth-accounts/tests/suite/**"]
effort = "high"
minutes = 120
+++

# The sign-in owner is a sub-owner, with one `Process`

## Goal

Step 1 of skein's D048. `sign_in` keeps its own `Client`, host, `private_io` and `Process` beside `world`'s; 77 of their 230 long lines are the same. `sign_in`'s owner becomes a fourth sub-owner of `world::Client`, as `accounts`, `shared` and `sign_out` are, and there's one `Process`.

Acceptance:

- `rg -n 'pub (struct Client|enum Process)' skein/tests/oauth-accounts/src` finds each once, in `world.rs` (twice each at the cut). The `Process` wrappers of `private_cuts.rs` and `sign_in/cuts.rs` are `oauth-one-restart`'s.
- `world::Client` holds `pub(crate) sign_in: Option<crate::sign_in::Owner>`, beside `accounts`, `shared` and `sign_out`.
- The baseline passes, unre-recorded but for what a trunk merge moved before your first edit.
- `heavy cargo nextest run -p skein-oauth-accounts-world` passes, with no assertion changed.
- The net is at least 120 lines fewer, outside the baseline.

## Inputs

- `temper/docs/plan/debt-29/notes/oauth-world.md`, whole: the table of what must survive.
- `skein/docs/development/debt/D048-oauth-accounts-world-copies.md`.
- `skein/tests/oauth-accounts/src/accounts.rs`, `Owner` and `client`: the form of a sub-owner.
- `temper/docs/plan/debt-29/notes/common.md`.

## Owns and may change

Owns the files in the header, for this change. A test file changes in how it reaches a process and nothing else.

- **`world::Process` gains `Browser`;** `Process::client()`, `issuer()` and `browser()` accessors spare the about 40 existing `match` sites a new arm, since `match_wildcard_for_single_variants` is denied here. Each site may take its accessor.
- **`Contracts`:** `sign_in::Contracts`, held as a `Vec`, and `world::Contracts` both stay, since `tests/suite/referee.rs` reads them, and so do both `Fact` enums.
- **`Store` dispatches to the sub-owner.** `world.rs`'s `private_io` pushes world facts and contracts unconditionally today, so that arm tests `self.sign_in` first. Today's hooks don't cover it.
- **The loop:** use `world`'s, since its close triggers push mid-loop. The cut checked `room() < 2` equals `MAX_OUT_DOWN`, and `0..ROOM` against `0..asked.len()` doesn't matter unless a sign-in turn queues over 64 requests. `operations()` differs by +4 with a file driver; the only reads the cut found are for hosted children and the real ring. If the baseline moves on any of these, that's the cause to look at first.
- **The unsettled world grant is harmless:** `Ledger` has no `Drop`, and the `accounts` sub-owner already works this way.
- **If `tests/suite/memory.rs` measures `sign_in::Client`,** its pin may grow as `world::Client` holds more. One that grows stops and asks fabrig/debt, with the measurement.

## Steps

```rust
// src/world.rs
pub struct Client {
    pub(crate) accounts: Option<crate::accounts::Owner>,
    pub(crate) shared: Option<crate::shared::Owner>,
    pub(crate) sign_in: Option<crate::sign_in::Owner>,
    pub(crate) sign_out: Option<crate::sign_out::Owner>,
    // ...
}

pub enum Process {
    Client(Box<Client>),
    Issuer(Box<peer::Peer>),
    Browser(Box<crate::sign_in::Browser>),
}

// src/sign_in.rs: what's left is the owner and its hooks.
// A client is made as accounts::client makes its own:
fn client(/* .. */) -> world::Client {
    let mut client = world::Client::with_limits(world::Story::Rotate, Transport::Plaintext, &tuned, room);
    client.sign_in = Some(Owner { /* .. */ });
    client
}
```

`sign_in`'s free functions `start`, `observe` and `restart`, and hooks for accepted, file events, pending work and the deadline, are what `world::Client`'s host calls. The names bind unless a site reads better otherwise.

Run the baseline first; then after each move of a part. Then the note's **Common to the tasks**. Commit, push.

## Report

As your text says, with the search's reading, the baseline's last result, whether a trunk merge moved it, any memory pin, and the net lines.
