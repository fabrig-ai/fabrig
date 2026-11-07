+++
id = "forge-cards"
part = "forge"
kind = "feature"
needs = ["connect-look"]
files = ["temper/crates/temper-client-view/src/forge/connect.rs", "temper/crates/temper-client-domain/src/forge/connect.rs", "temper/crates/temper-client-view/src/words.rs", "temper/crates/temper-client-view/src/tests/connect_forge.rs", "temper/crates/temper-client-domain/src/tests/connect.rs", "temper/tests/client/tests/suite/connect_forge.rs", "temper/tools/gallery/**"]
effort = "high"
minutes = 75
+++

## Goal

`ConnectForge`'s Choose and its "A Forgejo elsewhere" take the welcome's cards and panels, under Settings' path and in its column (pages 11, 12): the steps bar and the "Choose" card go, "Add a forge" over "Connect a forge" goes, "‹ All forges" becomes a quiet "Other forges" beside the main button.

Acceptance:
- Choose: "Connect a forge", its lede, Forgejo's card and GitHub's, with `CONNECT_DEFAULT` on the first not connected, a connected one saying "✓ Connected · ORG/NAME" with "Open Forgejo ↗" (page 11), then "Or a server of your own" with the row "Your Forgejo" and "Connect your Forgejo";
- "Run Forgejo here", on Settings as on the welcome, opens the welcome's sheet (`fn sheet`) over this page, its Cancel back to Choose;
- `Choice::Elsewhere` draws page 12: "A Forgejo elsewhere", the numbered steps, each block in `CONNECT_CMD` with its Copy, "Other forges";
- no `flow::steps` on this page; the path's word is "Connect a forge";
- the cut-short band ("FORGE · not finished", "Start over", "Go on") stays, in a `CONNECT_PANEL`;
- the welcome's Forge step looks as it did; the terminal keeps its headings' meaning;
- focused tests of `temper-client-view`, `temper-client-domain`, the client world's `connect_forge` and `the_committed_journeys_are_their_print` pass.

## Inputs

- `temper/docs/design/client/forge.md`, 5.4, *Connecting a forge*; `temper/docs/design/client/README.md`, 3.5.
- The request's `README.md` and pages 10, 11, 12.
- `temper/docs/plan/look-round-13/notes/common.md`; `notes/contracts.md`, *Look tokens*.

## Owns and may change

- Owns the `files` above, in `forge/connect.rs`: `main`, `body`, `choose`, `elsewhere`, `welcome_choices`. `managed`, `admin_details`, `tokens_details` are `forge-yours`'s; `setting_up`, `ready_card`, `progress_in` are `forge-rows`'.
- May change: `Word::PathAddForge` and tests that read it; `temper-client-view/src/flow.rs`, a helper this page alone called; tests of the changed words in `temper-client-view/src/tests/{settings,model_forge,forges,welcome,home}.rs`, `temper/tests/client/tests/suite/{forges,forges_edges,welcome_edges,github_connect}.rs`, `temper/crates/temper/tests/support/{playground,forge_land}.rs`, `temper/crates/temper/tests/suite/forge_land.rs`: words alone. `temper/tests/client-browser/**`: edit and compile only.
- Option: the forges' section head's button in `forges.rs` says "+ Connect a forge" and the empty band's "Connect a forge" (page 10); make that one-word change here.
- Option: a `Copy` control: reuse the one `pages/welcome_model.rs`'s program panel draws. If it isn't reusable, move it to `flow.rs`, and say so.

## Steps

1. View; a `Place::Connect` that draws with `CONNECT_*`, `Place::Welcome` as it was. A domain `connect::default_choice(...)` with a unit test where the card order needs one.
2. Tests; goldens: `connect-forge-choose` stays named, add `connect-forge-elsewhere` and `connect-forge-connected`, `names.txt` kept sorted; journeys reprint.
3. Compare at 1440, 1000, 420 against pages 11, 12 by a throwaway driver.
4. `cargo fmt`, old forms, clippy, focused tests, commit with the trailer, push.

## Report

As the implementer's text has it, with the differences from the request's pages named.
