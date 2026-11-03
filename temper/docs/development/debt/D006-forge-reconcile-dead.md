# D006: The forge reconcile path is dead: its verb, its host and `setup.orgs`

- **Kind:** dead code
- **Where:** `temper/src/forge.rs`, `temper/src/commands.rs`, `temper-forge-setup-shell/src/reconcile.rs`, `temper-forge-setup/src/limits.rs`, `temper-forge-setup/src/given.rs`, `temper-service/src/limits.rs`
- **Seen:** 2026-11-03 at `41d63da8c3`, in `temper/docs/design/protocol/README.status.md` (its Departs entry 4), `temper/docs/design/protocol/setup.status.md` (its Departs entry for 5 and 6.5) and `temper/docs/design/protocol/configuration.status.md` (its Departs entry 7): "a code task removes them"
- **Seen:** 2026-11-03 at `41d63da8c3`, while checking those entries: the shell's reconcile host has no product caller (new here; the status files name the verb, not the host)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (about 600 lines and a command gone; the help's table loses a verb that only refuses)
- **Size:** S
- **Status:** open
- **Paid when:** `temper forge reconcile` is not a command, `temper-forge-setup-shell` has no `reconcile` module, no limit or setup place is named `setup.orgs` or `orgs`, and the three status entries are gone.
- **Measure:** `wc -l < temper/crates/temper-forge-setup-shell/src/reconcile.rs`: 524

## What

The managed and connected forms that `temper forge reconcile` served are
gone (`temper/docs/design/protocol/setup.md`, 5 and 6.5). What is left:

- **The verb.** `temper/src/forge.rs` still parses `reconcile N` and
  answers it at once with "forge N is kept by the service: connect it
  from the client"; `temper/src/commands.rs` lists it ("Reconcile a
  managed or connected Forgejo still in its old form"), and two tests
  pin it (`temper/src/tests.rs`, `temper/src/tests/commands.rs`).
- **Its host.** `temper-forge-setup-shell/src/reconcile.rs`, 524 lines:
  `reconcile`, `Settings`, `Running`, and the forge files' decimal
  translation (`user_item`, `user_file`, "temper-15 removes the forge
  files and this decimal-file/item translation"). Nothing outside the
  crate's own tests calls it (`tests/suite/reconcile.rs`,
  `tests/support/mod.rs`); the binary links the crate for `command`,
  `prepare`, `run`, `fetch`, `parent` and `process` only.
- **`setup.orgs`.** `temper-forge-setup/src/limits.rs` keeps `orgs` (8 in
  the small profile) and its `setup.orgs` override, `given.rs` names
  `Place::Orgs`, `Org` and `OrgsLimit`, and `temper-service/src/limits.rs`
  lists the override; every setup entry's `orgs` is empty
  (`given.rs`, `machine.rs`).

The engine's setup host keeps its own `SetupPlan::Reconcile` through
skein's machine; that is live and not this item.

## Why it matters

A person reading `temper help` finds a command that only refuses; a
reader of the setup shell meets 524 lines no deployment runs, kept
green by their own tests.

## What pays it back

One task: remove the verb, its help row and tests, the shell's
`reconcile` module and its tests, and the `orgs` limit, places and
override; strike the three status entries. skein's `given::Place` names
orgs too (`skein_forge_setup::given`); whether skein drops it is skein's.
D005 is the other half of the old forms.
