+++
id = "settings-proof"
part = "settings-proof"
kind = "feature"
needs = ["settings-proposal", "ways-ctl"]
files = [
  "temper/tests/client/tests/suite/settings.rs",
  "temper/tests/client/tests/suite/main.rs",
  "temper/tests/client/tests/suite/configuring_chat.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-view/src/pages/settings.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/protocol/configuration.status.md",
  "temper/docs/design/protocol/store.status.md",
  "temper/docs/design/draft/configuring.status.md",
]
effort = "high"
minutes = 100
+++

# The meeting point, the proof and what's left

## Goal

A person's accepted proposal of kind 7 shows in `me`'s project, against the real engine. The tab and its neighbours render as the mock does at three widths. The plan's status lines are closed.

Acceptance:

- **The world's story,** `temper/tests/client/tests/suite/settings.rs`, on `configuring_chat.rs`' `propose` and `accept`:
  - `an_accepted_proposal_of_a_projects_ways_shows_in_its_rail`: a configuring agent proposes kind 7 all shown; the owner accepts; the project's rail shows Changes and Goals, as far as the trunk draws them, and the tab's switches read on; the chats' `drafts_offered` is true.
  - `a_switch_on_the_tab_changes_the_rail_and_a_member_is_refused`: the owner's switch sends, the rail changes; a member's is refused, in jig's words, the switch back.
  - `a_projects_questions_set_on_the_tab_reach_its_next_change`: set `foreman`, then a `change` naming no route takes it.
- **The status words:** read the trunk once more (`how-work-runs`, *Status*, *While you're away*, the Ways' function, Goals and Changes in the rail, the mock's `tabbar`): whatever has landed since `how-work-runs` moves it to the first true, and `goals::ways` replaces any private copy; the catch-up's arms are this task's.
- **The proof,** by `temper/docs/plan/ui-settings/inputs/request.md`, *The proof*: the page beside the mock's at 1440, 1000 and 420 wide: `settings.html` scrolled whole (a window 2400 tall); `settings.html?project=hello`; each switch off, the rail changing with it; the switcher open on the page; Repositories at the three widths, with rows and with none. Screenshots of both, by name, in `~/src/fabrig/ui-settings/` on the builder, outside any worktree, the report saying which builder. Never committed; a throwaway render script stays untracked and is gone before the push (`notes/common.md`, *Render proofs*). A difference that's a bug in this plan's rules is fixed here; one that isn't, listed with its reason.
- **What the plan leaves open** goes in the status: `README.status.md`, `browser.status.md`, `configuration.status.md`, `store.status.md` carry no line this plan closed, and `configuring.status.md` keeps its *5.1*.
- `python3 -I -B tools/citations/citations.py --base origin/main` shows none broken.
- The tests pass and bite.

## Inputs

- `temper/docs/plan/ui-settings/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5 *Settings*, 5.3, 5.13, 10. `temper/docs/design/client/browser.md`, 2.5, 7.2.
- `temper/docs/plan/ui-settings/inputs/request.md`: this task reads it whole, and goes through it section by section for anything the other tasks' reports left. **A departure from the implementer's text:** you read the request.
- `temper/tests/client/tests/suite/configuring_chat.rs`.

## Owns and may change

Owns `files`; `configuring_chat.rs` only to share `propose` and `accept` with the new module. May also change any view file of `temper-client-view/src/pages/` and `temper.css` under its `ui: settings` marker, for a bug the proof finds in this plan's own work, each named in the report.

Pre-approved:

- A difference the mock's data or scenario makes (its fabrig project, its fonts over the network, its budgets): not a bug.
- A gap you find against the request that's bigger than a fix of a rule or a few lines: report `Done:` with it under `## Left`, naming its section: the plan's reader takes it up.

## Steps

1. The world's stories; `mod settings;` in `suite/main.rs`.
2. The status words.
3. The proof.
4. The status files.
5. `cargo fmt`; old forms; clippy for what you changed; nextest of `temper-client-world`, `temper-client-view`, `temper-client-gallery`; `heavy cargo check -p temper --tests`. Commit and push.

## Report

As the implementer's text says: which builder; each screenshot pair; each difference and why it stays; every section of the request and how it ended.
