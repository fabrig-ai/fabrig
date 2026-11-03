# D040: One name means several types

- **Kind:** naming
- **Where:** `jig-domain-core/src`, `jig-client-domain/src/contract/versions.rs`, `jig-domain-tasks/src/placed.rs`, `jig-protocol-store/src/patching/sets.rs`, `jig-client-view/src/owner.rs`, `tests/views/Cargo.toml`, `testing/jig-test-system`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T9, A0 F7, A2 F6, A5 F7, A8 F9, A9 F2, A10 F12)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (T9: 13 public type names defined twice in core; crate names)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (a name leads to one concept)
- **Size:** M
- **Status:** open
- **Paid when:** no public type name is defined twice in one crate, each crate-crossing name (`Owner`, `Flight`) has one meaning or a qualified one, and each test package's name says its kind.
- **Measure:** `rg -o --no-filename '^pub (struct|enum) ([A-Z][A-Za-z0-9]*)' -r '$2' jig/crates/jig-domain-core/src -g '!**/tests/**' -g '!tests.rs' | sort | uniq -d | wc -l` (19 at base)

## What

- 19 public type names are defined more than once inside `jig-domain-core` (13 at the review): `Ask`, `Event`, `Key`, `Limits`, `Record`, `Settings` and others;
- four types named `Flight` in the core (`jig-domain-core/src/effects.rs`, line 107; `jig-domain-core/src/routing/flight.rs`, line 9; `jig-domain-core/src/routing/party/key_routes.rs`, line 55; `jig-domain-core/src/routing/application_routes.rs`, line 17);
- five public `Owner` types: the client contract's envelope owner, tasks' placed owner, the store's patch-set owner, the view's skein-ui owner, and the core's forget owner;
- `jig-client-views-world` (`tests/views`) is the engine's views world named as a client crate; `jig-test-system` is a fake.

The review's "historical" (about 1,040 uses) is paid: 0 at base.

## Why it matters

A reader has to look at the import to know which type a name is; renames and searches hit
the wrong one.

## What pays it back

Rename per concept, starting with the core's doubles and the `Flight` types (with D002);
rename the two test packages.
