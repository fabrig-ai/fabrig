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

## Answers

2026-11-06 (temper/ui). One more for this task, from the user's review after the cut began: look-requests is now at 86358b6de5, the same request's commit (find it by its Look-request: connect-flows trailer). In its connect.css, a select in Settings draws its own chevron, 14px in from the border, with its text lined up with the fields' under it: the rule under the comment 'A select's arrow sits as far in as its text'. Apply it to every select in Settings, not only the connect pages, so they match. temper/ui widens this task's files to temper-client-shell-web's temper.css for that rule, under this plan's marker comment. Show 16.html's select in your screenshots at 1440 and 420. The user called 16.html perfect apart from this, so change nothing else of it.

2026-11-06 (temper/ui). The user's rule, 2026-11-06: a plan executed by fab runs focused and fuzzy alone, and is complete when it lands. Walks and render checks follow landing, and file findings back to fab. So drop this task's throwaway-driver comparison and screenshots: don't start a browser to compare. temper/ui runs the render check after the plan lands.

## Rebase (2026-11-07, step merge, foreman look-round-13/forge-rows/foreman-1)

merge the forge-row and composer-picker changes, then recount their shared tables
The merge conflicts are in `temper/crates/temper-client-view/src/words.rs` and `temper/tools/gallery/src/samples.rs`.
The branch shares base `d1190606f7` with integration; subsequent changes include composer picker `50e7171ce1`, model cards `845d47a7be`, and main reconciliation `a67f82b87d`.
In `words.rs`, retain both the branch’s forge vocabulary replacements and integration’s model-picker vocabulary and hole-bound changes.
Resolve the shared declaration counts from the combined vocabulary, using `english_declarations()` to verify them; do not choose either side’s counts wholesale.
Both sides remove one `ALL` entry, so the combined array length is 1296 at the inspected integration tip `8548f37bff`.
In `samples.rs`, retain `forges-added` alongside all integration model-picker and connection samples, disclosure mappings, and domain setup.
The combined sample count is 294 at that integration tip; preserve integration’s scoped `large_stack_arrays` expectation.
Keep `names.txt` consistent with the combined sample table. Regenerate stale gallery goldens and journey prints through their tools.
Verify the resolved vocabulary, gallery names, forge flow tests, and `the_committed_journeys_are_their_print` through heavy, within focused and fuzzy only.
Preserve the branch’s Settings select-chevron changes. No browser comparison or screenshots are required.
