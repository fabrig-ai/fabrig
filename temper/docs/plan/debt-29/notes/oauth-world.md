# debt-29: one owner harness in the oauth-accounts world

For `oauth-baseline`, `oauth-one-owner`, `oauth-one-driver`, `oauth-one-restart` and `oauth-close`, which carry skein's D048 as far as the first three of its seven steps. Read `skein/docs/development/debt/D048-oauth-accounts-world-copies.md` whole, and `skein/docs/foundation/testing-strategy.md`, 10. The world is `skein/tests/oauth-accounts`, the package `skein-oauth-accounts-world`, and it's ordinary Rust.

Line numbers below are the ones the cut's survey saw: find a site by its text.

## The baseline

Most sign-in tests are `drop(run(..))`, so the `Judge` is their only check. `oauth-baseline` adds what holds behaviour still across these tasks: `tests/suite/baseline.rs`, a digest of what each story does, recorded inline.

**It's a temporary instrument, not a tier's test.** Each of its tests is `#[ignore = "D048 baseline, deleted by oauth-close"]`, so no tier runs it, and it moves no test out of any tier. `oauth-close` deletes it.

Run it, through heavy, with:

```sh
heavy cargo nextest run -p skein-oauth-accounts-world --test suite --run-ignored only -E 'test(/^baseline::/)'
```

**Each of the tasks after `oauth-baseline` runs it first,** right after fab's trunk merge, before any edit. Red there means the trunk moved a kit: re-record it, and report which commit moved it. **It runs after each step of your work too:** it stays green, and it's never re-recorded to make a refactor pass. A digest that moves because of your edit is the common note's "stop and ask": nothing in these tasks changes what a story does.

What keeps a digest comparable across commits:

- **Spawn order stays.** A trace's pid follows it.
- **Each `Fact` enum keeps its name and variants.** Its `Debug` text is what's digested, so `sign_in::Fact` and `world::Fact` aren't merged.
- **Operation order stays.** The harness allocates no kernel op tokens, but moving a close moves the trace.

## What must survive

Neither pair of owners is a superset of the other. These differences stay, as fields or hooks:

| Where | `world` | `sign_in` |
|---|---|---|
| Owner | close triggers, `stop_at_*`, `recovered`, the 40 s `Released` close | `restart`, the paste, the `Accepted` hook, `work_pending`'s terms, the `again_at` deadline, `opened_private`, `stored_private` |
| `Store` | pushes world facts and contracts | its own, so `Store` dispatches to the sub-owner |
| Driver's wire seeds | `seed`, `seed + 1`, for life | reseeded per connect with `seed + 4 + posts` and `seed + 5 + posts`, the server rebuilt |
| Driver's trace order | client then server, at the end | appended at each close and at the end |
| Requests served | POST only, `/token` or `/revoke` | GET documents too |
| Per-loop work | `protocol_files`, TLS, `Released` | the visit script, the paste, the callback wire, listeners |
| Exit | `posts <= 1` | exact posts, documents, `preferred_failed` |
| Restart wrapper's close | `private_cuts.rs` closes the signal when empty | `sign_in/cuts.rs` also requires `!work_pending` |

**`PasteThenListener`'s tiers use different predicates:** the browser waits for `Fact::Taken` (`paste.rs`), the wire for `pasted` (`protocol.rs`), which is true one observe turn earlier. Unifying them retimes the story. Leave them apart.

## What stays out of these tasks

- **Story branches stay:** `story ==`, `!=`, `matches!` and `match`. Steps 4 to 7, rows for fixtures, faults, scripts and expected values, aren't in this plan; `oauth-close` writes why into the item. Branches that collapse because two copies became one are fine, and no other is touched.
- **The `Story` enums stay,** as do the public `run`, `protocol`, `facts`, `registration` and `key_set`, and both `Contracts` (`tests/suite/referee.rs` reads them).
- **Nothing outside `skein/tests/oauth-accounts`:** `skein/crates/skein-oauth-accounts`, `skein/testing/*` and `.config/*` are only read. The new wrapper stays local, though `tests/io` has similar forwarders.
- **`tests/suite/memory.rs`'s pins** hold. One that shrinks is updated and reported; one that grows stops and asks fabrig/debt, whichever of the two `Client`s it measures.

## Common to the tasks

- One writer at a time across `src/`: `oauth-one-driver` and `oauth-one-restart` run side by side after `oauth-one-owner`, and their files are disjoint but for one `pub mod` line in `src/lib.rs`.
- The test run, at the end of each: the baseline command above; `heavy cargo nextest run -p skein-oauth-accounts-world`, which is the world's tests and its budget. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-oauth-accounts-world --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`.
- The world's own test run is over budget only as it was before: this plan's temporary tests are ignored, and these tasks add no run to a tier. If it's over, that's red: read `skein/docs/foundation/testing-strategy.md`, 8.1, and report it with measurements.
