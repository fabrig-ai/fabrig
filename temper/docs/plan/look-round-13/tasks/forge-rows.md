+++
id = "forge-rows"
part = "forge"
kind = "feature"
needs = ["forge-yours"]
files = ["temper/crates/temper-client-view/src/forge/connect.rs", "temper/crates/temper-client-view/src/forge/forges.rs", "temper/crates/temper-client-domain/src/forge/connect.rs", "temper/crates/temper-client-domain/src/forge/forges.rs", "temper/crates/temper-client-view/src/flow.rs", "temper/crates/temper-client-view/src/tests/forges.rs", "temper/crates/temper-client-domain/src/tests/forges.rs", "temper/tests/client/tests/suite/forges*.rs", "temper/tests/client/tests/suite/github_connect.rs", "temper/tools/gallery/**"]
effort = "high"
minutes = 75
+++

## Goal

Setting up lives in the row: from a forge's connect on, a managed Forgejo, and GitHub's return, the address is `Forges` and the forge's row is open with its live steps (page 15). The Ready step goes: once connected, the row and `Forges`' line say what was added, as `Accounts`' does.

Acceptance:
- `ConnectForge { forge: Some(Forge(n)) }` at `SettingUp` or `Ready` moves to `Forges`, its address replaced, row `n` open; on a reload of `Forges` the row is folded unless it is still setting up;
- the row, setting up: summary "Your Forgejo", then "Forge 2 · Forgejo · Setting up · work/hello"; `CONNECT_LIVE`, a done step plain, the current `aria-current`, a later one `CONNECT_LATER`; a failure's words and its "Try again" in the row;
- GitHub's return comes back to `Forges`; a forge with no accepted installation keeps its install act, in the row;
- connected, a line under the section's head, `CONNECT_CONFIRM`: "Connected FORGE." with a quiet "Adopt a repository" when a project is open;
- `setting_up` and `ready_card` draw only for the welcome; what nothing calls goes, `flow::steps` among them once nothing calls it;
- the welcome's forge step looks as it did, its own Ready stands;
- focused tests of `temper-client-view`, `temper-client-domain`, the client world's `forges*`, `github_connect` and `the_committed_journeys_are_their_print` pass.

## Inputs

- `temper/docs/design/client/forge.md`, 5.4, *Connecting a forge*, *Setting up*; `temper/docs/design/client/README.md`, 3.5.
- The request's `README.md` (*What goes*, *Bigger than a look*) and pages 10, 15.
- `temper/docs/plan/look-round-13/notes/common.md`; `notes/contracts.md`, *Look tokens*.

## Owns and may change

- Owns the `files` above.
- May change: `temper-client-domain/src/pages.rs`, the address change, the one call; `Act::GitHub { back }`'s `back` at its call sites; tests of the changed flow in `temper-client-view/src/tests/{connect_forge,settings,model_forge}.rs`, `temper/tests/client/tests/suite/{connect_forge,welcome_edges,playground}.rs`, `temper/crates/temper/tests/support/{playground,forge_land}.rs`, `temper/crates/temper/tests/suite/forge_land.rs`; `temper/tests/client-browser/**`: edit and compile only.
- Option, the row's open state: a page-side `Forges.opened: Option<u16>` read when the row is drawn, since a person's own opening stays the browser's. If the row's folding is entirely the browser's, use a state the builder gives a `details` at creation, and say so.
- A forge that fails at setup keeps every act and word it has today.

## Steps

1. Domain: the move, with tests in `temper-client-domain/src/tests/forges.rs`: each `Shown` that moves, an install still asked staying with its act, a welcome that doesn't move.
2. View; `flow.rs`'s now-unused helpers go; tests; goldens: `forges-setting-up`, `forges-fetching`, `forges-failed` print the row, `forges-added` is new; journeys reprint.
3. Compare at 1440, 1000, 420 against pages 10, 15 by a throwaway driver.
4. `cargo fmt`, old forms, clippy, focused tests, commit with the trailer, push.

## Report

As the implementer's text has it, with the differences from the request's pages named.
