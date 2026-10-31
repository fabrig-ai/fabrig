+++
id = "cl-gallery"
part = "client"
kind = "feature"
needs = ["cl-welcome", "cl-play-view", "cl-connect-view", "cl-remove", "cl-project"]
files = [
  "temper/tools/gallery/**",
]
effort = "medium"
minutes = 50
+++

## Goal

The gallery (`temper-client-gallery`) shows the client's new states and
none of what went: a fresh store's welcome and Forge step with kind
cards, an unfinished owned Forgejo and an unfinished GitHub (Go on,
Start over), a Remove confirmation per kind, the playground's pages with
their one-line banner and nothing to add, "Leaving the playground…", and
the project settings page with Delete (an owner's, typed and not) and
another party's. The demo's ended chats, its halves' banners and the
setup strip go from its samples.

Acceptance (`temper-client-gallery`, `src/tests.rs`):
`names_are_the_promised_ones` (with `names.txt` updated: the new names,
the gone ones removed), `each_state_holds_what_its_name_says` (each new
sample's page holds its named state: the confirmation's words, the
banner's link, the leaving line, the Delete control),
`print_writes_every_page_and_the_bundle`, and every other test of the
package.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/tasks/cl-connect-view.md`,
  `cl-remove.md`, `cl-play-view.md`, `cl-welcome.md`, `cl-project.md`
  (the states and their words).
- `temper/docs/design/draft/playground-reset.md`, 4, 5, 6.1-6.4.

## Owns and may change

Owns `temper/tools/gallery/**`. Builds its worlds from the client
domain's public items and its fixtures' forge rows and playground reads
(wire records); never edits the client crates.

## Steps

1. **Worlds (src/worlds.rs, src/worlds/github.rs):** forge rows as
   records (numbered from 1, `finished` set as each state needs), no
   configured slots; playground reads running and resetting
   (`PlaygroundAnswer`), none; an owner's project document offering
   `ActKind::RemoveProject`, and a member's.
2. **Samples (src/samples.rs, `N`):** new, named in the samples' style:
   `welcome-empty`, `welcome-forge-unfinished-here`,
   `welcome-forge-unfinished-github`, `forges-remove-here`,
   `forges-remove-github`, `forges-remove-managed`, `chats-playground`,
   `accounts-playground`, `connect-model-playground`,
   `chats-playground-leaving`, `project-settings-owner`,
   `project-settings-owner-typed`, `project-settings-member`; the
   samples of the demo's end, its halves' banners and the setup strip
   go; `welcome-forge-details`' `Choice::Forge(3)` becomes the kept
   record's number or `Choice::New(NewForge::Here)`.
3. **Checks:** `cargo fmt`; `heavy cargo clippy -p temper-client-gallery
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-client-gallery`.
4. Old forms from `temper/`; commit and push.

## Report

- The samples added and removed.
