+++
id = "forge-choices"
part = "pages"
kind = "feature"
needs = ["client-reads-deployment"]
files = ["temper/crates/temper-client-domain/src/forge/connect.rs", "temper/crates/temper-client-domain/src/tests/connect.rs", "temper/crates/temper-client-view/src/forge/connect.rs", "temper/crates/temper-client-view/src/tests/connect_forge.rs", "temper/docs/design/client/forge.status.md"]
effort = "medium"
minutes = 45
+++

# Forgejo on this machine is offered from the deployment's read

## Goal

`connect::choices` stops assuming the box: Forgejo on this machine is offered from the deployment's
`owned` (contracts K4), and a kept owned row's offer decides as it does today. An address that names
it with none offered is Choose.

Acceptance:

- `connect::choices(rows, playground, owned: bool)`: the third argument is named for what it is;
  `temper-client-view/src/forge/connect.rs:254` passes `root.forge().deployment()`'s `owned`, false
  while unknown.
- `connect::step(choice, rows, playground, owned: bool)`: `AddressChoice::Owned` is `Shown::Choose`
  when the same rule offers none. The other kinds' Details are unchanged. `connect::welcome_step`,
  which calls it, takes the same argument.
- Domain tests: `no_owned_offers_github_and_a_forgejo_elsewhere` and
  `an_owned_address_with_none_offered_is_choose`.
- View tests, in `tests/connect_forge.rs` (`welcome-inbox` holds `tests/welcome.rs`): with `owned:
  false`, `ConnectForge`'s Choose has no "Forgejo on this machine" card, and the welcome's Forge
  step no "Run on this machine"; with `owned: true`, as today, on empty rows.
- `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-world`
  pass.
- `forge.status.md` loses "5.4: Forgejo on this machine, offered from the deployment's read:
  `temper-client-view`'s `forge/connect.rs` passes `true` to `connect::choices`, and `connect::step`
  shows its Details from the address alone."

## Inputs

- `temper/docs/plan/serve-without-box/notes/common.md`; `notes/contracts.md`, K4 and K5.
- `temper/docs/design/client/forge.md`, 5.4 (Choose; Details).
- `temper/docs/design/client/README.md`, 4.6 (The deployment's read).

## Owns and may change

- The files of the header.
- Callers of `connect::step` and `connect::choices`: the argument alone. `tests/connect.rs:83,87,315,966`
  keep their meaning.
- `temper-client-view/src/tests/connect_forge.rs:167,193` draw `Details(Owned)`: they hold under
  `client-reads-deployment`'s `owned: true`. If a fixture there has no fact yet, answer it.
- The engine's `Offer` and `offer()`: no change.

## Steps

1. `choices` and `step`; a small private function they share for the rule.
2. The view's caller; the tests; the implementer's checks; commit and push.

## Report

As the implementer's text asks.
