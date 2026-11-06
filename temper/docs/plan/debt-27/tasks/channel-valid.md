+++
id = "channel-valid"
part = "channel-valid"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-worker-link/src/limits.rs", "jig/crates/jig-protocol-engine-workers/src/limits.rs", "jig/docs/development/debt/D011-channel-mechanics-copied.md"]
effort = "medium"
minutes = 10
+++

# Both worker sides ask `skein_channel::valid`

## Goal

Pay jig's D011. The item asks for a validity function in skein-channel, and it's already there: `skein_channel::valid(schema, role, limits)` is `schema.check(role, limits).is_ok() && capacity_bound(limits).is_some()`, and `capacity_bound` is the two copies' body, line for line. No skein change.

Acceptance:

- The measure reads 0: `rg -n 'fn channel_capacity' jig/crates | wc -l`.
- Neither crate's `valid` calls `schema.check` or `channel_capacity`: each calls `skein_channel::valid` with its own role, `Initiator` in the worker link and `Responder` in the engine's side.
- The comment "The channel has no separate validity API at this pin" has gone with the copies.
- `jig/docs/development/debt/D011-channel-mechanics-copied.md` is deleted.

## Inputs

- `jig/docs/development/debt/D011-channel-mechanics-copied.md`, whole.
- `skein/docs/design/channel.md`, 10.
- `temper/docs/plan/debt-27/notes/common.md`.

## Owns and may change

Owns the files in the header. The item's "what it holds" asks nothing more: both copies were only ever read through `.is_some()`, so no capacity function is needed beside `valid`.

If `Role` or another import goes unused, remove it.

## Steps

In `jig-protocol-worker-link/src/limits.rs`, `schema.check(Role::Initiator, &channel).is_ok() && channel_capacity(&channel).is_some()` becomes `channel::valid(&schema, Role::Initiator, &channel)`; its `use` already names `skein_channel as channel`. In `jig-protocol-engine-workers/src/limits.rs` the same with `Role::Responder`, by whichever path reads best beside its `use skein_channel::{Role, Schema}`.

`heavy cargo nextest run -p jig-protocol-worker-link -p jig-protocol-engine-workers`; `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-worker-link -p jig-protocol-engine-workers --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`. Commit, push.

## Report

As your text says, with the measure's reading and the net lines.
