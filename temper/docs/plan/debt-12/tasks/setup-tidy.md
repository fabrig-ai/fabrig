+++
id = "setup-tidy"
part = "owners"
kind = "feature"
needs = ["link-surface", "link-ending", "llm-tidy", "forge-tidy"]
files = ["skein/crates/skein-forge-setup/src/host.rs", "skein/crates/skein-forge-setup/src/host/connection.rs", "skein/tests/forge/setup/src/host.rs"]
effort = "high"
minutes = 25
+++

# Setup's host reads the link's face, and its close bound is the rule's

## Goal

Setup spells "is the close due" twice, `link_env` in seven lines and each optional body in a `match` under an
`#[expect]`, and its test allows two `close_timeout`s where the rule is one. About 20 production lines go. This
is the plan's last task, held back on purpose: `repository-copies-1` changes these files, and a later merge is a
smaller one.

Acceptance:

- `rg -n 'next_deadline\(\) \{' skein/crates/skein-forge-setup/src/host.rs` finds nothing.
- `rg -n 'manual_map|Upload::new|wall: env.wall,$' skein/crates/skein-forge-setup/src/host/connection.rs` lists
  neither the upload's `match` nor `link_env`.
- `a_tls_close_without_room_returns_the_ended_runs_slot_within_its_bound`'s bound holds one `close_timeout`, and
  fails with `Link::close`'s `until` doubled.
- `heavy cargo nextest run -p skein-forge-setup -p skein-forge-setup-world` passes, no other test changed.
- `heavy cargo clippy -p skein-forge-setup -p skein-forge-setup-world --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/debt-12/notes/common.md`
- `temper/docs/plan/debt-12/notes/contracts.md`, 2 and 3
- `temper/docs/plan/debt-12/inputs/link-review.md`, findings 5, 9 and 10's second sentence
- `skein/docs/design/link.md`, 1 and 3.3

## Owns and may change

- **Owns:** the files in this spec's `files`, in the lines named below.
- **Options:**
  - If an `#[expect(clippy::manual_map, ...)]` also covers another `match` in its function, it stays, and the
    report says so.
  - If the tightened bound fails, measure what `deadline` is against `ended`, and stop and ask: the rule is one
    `close_timeout`, and a test that needs two is a finding, not a typo.
  - If `repository-copies-1` moved these lines, follow them by name.

## Steps

1. `host.rs`: `c.link.due(env.now)` in `drive`, `c.link.due(now)` in `ready`, each replacing a `match` on
   `next_deadline`.
2. `host/connection.rs`: `link_env`, as the contracts' section 2. `upload: skein_link::Upload::of(encoded.body),`
   at both constructors, and the two `#[expect(clippy::manual_map, ...)]` that spoke of them go.
3. `upload`, keeping the order of what it does:

   ```rust
   let (send, next) = upload.room(limits::http_transport(&env.limits, self.link.encrypted()).send);
   if next == Down::Finish {
       self.upload = None;
   }
   self.http_down(env, http::Request::Upload(send));
   self.http_down(env, http::Request::Upload(next));
   ```

4. `skein/tests/forge/setup/src/host.rs`, `a_tls_close_without_room_returns_the_ended_runs_slot_within_its_bound`:
   one `.saturating_add(world.env.limits.io.close_timeout)` goes from `bound`. Break it: double `until` in
   `Link::close`, see it red, and restore.
5. Fmt, clippy, tests. `python3 -I -B scripts/old_forms.py` in `skein/`. Commit and push.

## Report

The report of your implementer text, and: what you broke and what failed; net production lines; and, as the
plan's last task, the plan's production total, the sum of `git diff --shortstat <the plan's base, cf6766c426> --
PATH` over `skein-link/src`, and each owner's files this plan changed, tests apart. The request expects about 65
lines lower.
