+++
id = "env-with"
part = "env-with"
kind = "feature"
needs = []
files = ["skein/crates/**/*.rs", "skein/tests/**/*.rs", "skein/examples/**/*.rs", "skein/docs/development/debt/D003-env-rewraps.md"]
effort = "medium"
minutes = 40
+++

# skein: an `Env` of the same clocks is `env.with(limits)`

## Goal

Pay `skein/docs/development/debt/D003-env-rewraps.md`: no skein code builds an `Env` from another `Env`'s `now` and `wall`
by hand. The same goes for a `skein_shell::Now`, which has `Now::with` beside it and nearly no callers.

The item's own measure can't reach 0: it also matches the two definitions and nine sites whose clocks aren't one value's.
Those are listed below and stay.

Acceptance:

- From the repository root, this reads 75 at this plan's base and 0 after:

  ```
  rg -cP 'Env \{ now: ([a-z_.]+)\.now, wall(: \1\.wall)?,' skein -g '*.rs' -g '!**/skein-lib/src/env.rs' -g '!**/skein-shell/src/clock.rs' | awk -F: '{s+=$2} END{print s+0}'
  ```

- The item's measure, `rg -c 'Env \{ now: [a-z_.]*now' skein -g '*.rs' | awk -F: '{s+=$2} END{print s}'`, reads 11, the
  sites below (or fewer, if a trunk merge removed one).
- Every test passes unchanged. The branch's net lines fall, and no production line rises but where `rustfmt` rewraps one.
- `skein/docs/development/debt/D003-env-rewraps.md` is deleted.

## Inputs

- `temper/docs/plan/debt-17/notes/common.md`
- `skein/docs/development/debt/D003-env-rewraps.md`: its counts are older; this spec's stand.
- `skein/docs/design/lib.md`, 9

## Owns and may change

- **Owns:** only the sites the acceptance search finds, and each file's now-unused import. Touch nothing else in a file
  you open. `Env::with` is `with<M>(&self, limits: M) -> Env<M>`; `Now::with` is `with<L>(self, limits: L) -> Env<L>`.
- **Stay as they are** (their clocks aren't one value's, or they define the form):

  | Site | Why |
  |---|---|
  | `skein-lib/src/env.rs`, `skein-shell/src/clock.rs` | the definitions |
  | `skein-fake-forge-github/src/server.rs`: `now: h.env.now, wall: h.now` (three) | tests move `h.now` and `h.env.wall` apart; `with` would change the wall |
  | `skein-sim/src/tests/given_listener.rs` | `sim.now()`, `sim.wall()` |
  | `skein/tests/fake-peers/tests/forge/fixtures.rs` | `forge.now`, `self.sim.wall()` |
  | `skein/tests/forge/github/src/fake.rs` | `self.sim.now()`, `self.sim.wall()` |
  | `skein/tests/forge/protocol/src/world.rs` | `joint.now()`, `joint.wall()` |
  | `skein/tests/llm/tests/suite/fake_memory.rs` | advances time, `..env` |
  | `skein/tests/llm-connection/tests/suite/failures.rs` | `self.now`, `pki::VALID` |

- **Review each site:** one whose `now` and `wall` turn out to come from different values stays, and your report names it.
  `skein-fake-forge-github/src/inline.rs` reads `wall` from a local, which the line above builds `env` from: that one
  converts.
- **A field of the same env:** `env.with(env.limits.http)` compiles for a `Copy` field; `skein-link`'s `link.rs` already
  writes it. `macro_rules!` sites in `skein-fake-peers/src/forge/layer.rs` convert too.
- If a site's source is a temporary or a `Copy` `Env` copy that won't borrow, bind it first, and say so in the report.
- Five test files of `skein/tests/conformance/real` lose their last use of `Env`: drop the import.
- Shorthand `Env { now, wall, .. }` built from parameters is not this item. Leave it.

## Steps

1. Convert crate by crate, in commits small enough to read: `env.with(limits)` for an `Env`, `now.with(limits)` for a
   `Now`. A search-and-replace with a look at each site; the compiler finds the rest.

2. `heavy cargo nextest run -p skein-llm-connection -p skein-llm -p skein-fake-forge-forgejo -p skein-fake-llm-protocol -p
   skein-fake-peers -p skein-fake-forge-github -p skein-forge-connection -p skein-forge-setup -p skein-oauth-accounts -p
   skein-json -p skein-forge-protocol-world -p skein-forge-machine-world -p skein-forge-setup-world -p skein-kv-tests`, adding any
   package the search names that this list lacks. `heavy cargo check --tests -p skein-conformance-real -p forgejo-conformance -p
   skein-ui-example-server`: the real tiers aren't this task's to run. **This departs from the implementer's text:** no public
   item changes, so direct dependents are not run.

3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy` naming every package you changed,
   `--all-targets -- -D warnings`; delete the item; the citations check. Commit and push.

## Report

The common note's items, both searches' readings, and every site you left beyond the table. A rewrite: nothing to break
by hand.
