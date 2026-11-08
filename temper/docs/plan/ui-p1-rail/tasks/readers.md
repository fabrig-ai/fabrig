+++
id = "readers"
part = "rail"
kind = "migration"
needs = ["rail", "work-page", "deployment"]
files = ["temper/tests/client/src/**", "temper/tests/client/tests/live_text.rs", "temper/tests/client/tests/suite/*.rs", "temper/tests/client-browser/src/lib.rs", "temper/tests/client-browser/tests/browser_all/*.rs", "temper/tests/one-process/src/**", "temper/tests/real-loop/tests/**", "temper/crates/temper/tests/process.rs", "temper/crates/temper/tests/support/person.rs"]
effort = "high"
minutes = 150
+++

# The stories outside the view walk the new rail, Work and Deployment

## Goal

The stories of other packages that press the rail's old entries, Settings' tabs
or the Builders words walk the client as it now is: "Work" and its filters for
Goals, Changes and Chats; the new box's "Change" (or "+ New change") for "+ New
chat"; "How work runs" and "Deployment" for "Settings"; "Workers" for "Builders".
What each story checks stays. The client world's focused stories run; the
browser's, the real loop's, one process's and `temper`'s process stories are
written and compiled, not run (`notes/common.md`).

Acceptance:

1. No story or helper in `files` presses or finds a rail entry, a switcher item
   or a Settings word the client no longer draws; each finds the new one by
   `temper/docs/design/client/README.md`, 5.3 and 3.5 (*Work*, *Settings and
   Deployment*), keeping its own checks.
2. Starting a chat from the rail is one helper per harness, named for what it
   does (`start_change`), opening the new box's menu and pressing "Change" while
   the project offers drafts, else pressing "New change".
3. `temper-client-world`'s stories that read kind 7's effect on the rail
   (`settings.rs`'s `a_switch_on_the_tab_changes_the_rail_and_a_member_is_refused`
   and its neighbours) read it on Work's filters now (3.5, *Work*, *Its filters*),
   each keeping its member's refusal.
4. `heavy cargo nextest run -p temper-client-world` passes (focused profile, and
   `--profile fuzzy` for its fuzzy suite); `heavy cargo check --tests` passes for
   `temper-client-browser-world`, `temper-one-process-world`,
   `temper-real-loop-world` and `temper`.

## Inputs

- `temper/docs/plan/ui-p1-rail/notes/common.md`.
- `temper/docs/design/client/README.md`, 5.3 (*The rail*, the phone's tab bar,
  *Projects in the rail*), 3.5 (*Work*, *Its filters*; *Settings and Deployment*,
  *Its tabs* and *Workers*).
- The lead's survey of the presses, a lead and not a list to stop at:
  `temper/tests/client/src/fixtures.rs` ("Settings", around 493);
  `tests/live_text.rs` ("New chat", 31); `tests/suite/open_chat.rs` (29, 57),
  `choice.rs` (13), `stories.rs` (30, 58, 100, 118), `chat.rs` (205-208, 323),
  `models.rs` (92, 109, 279, 301), `task.rs` (490, "Goals"), `chat_accept.rs` (69),
  `settings.rs` (50, 57, 74, 152), `adoption.rs` (85), `accounts_edges.rs` (84-90),
  `forges_edges.rs` (831-837, 999), `welcome_m3.rs` (114), `builders.rs` ("Builders
  · 1", the owners' line); `temper/tests/client-browser/src/lib.rs` (`start_chat`,
  `playground_chat`), `tests/browser_all/browser_pages.rs` (21, 31, 104, 167-185),
  `browser_playground.rs` (81), `browser_accounts.rs` (114, 126);
  `temper/tests/one-process/src/stories.rs` (61); `temper/tests/real-loop/tests/live_text.rs`
  (28), `suite/terminal.rs` (24, 64, 143), `suite/github_land.rs` (33),
  `suite/model_check.rs` (47), `suite/sign_in.rs` (22, 46);
  `temper/crates/temper/tests/process.rs` (94); `tests/support/person.rs` (838).
  Search each for "New chat", "Settings", "Goals", "Changes", "Chats", "Builders"
  and "Sections" for the rest.

## Owns and may change

Owns: the stories and helpers in `files`, their presses, finds and helpers alone.

Not this task's: any product code; the view's, the gallery's and the domain's
tests (their tasks repaired them); the terminal goldens and the journeys
(`regenerate`). The red-78 story's waits
(`browser_pages.rs`'s `history_keeps_the_chat_focus_and_filter_across_reload_and_back`)
stay: a wait on the rail's "New chat" link becomes a wait on the new box (by its
name, "New draft or change"), drawn in the same build.

Pre-approved options:

- **If a story reaches Chats through the rail:** press "Work", then the filter
  "Chats" (`Chats { filter: All }`); in a browser story, the same.
- **If a story reaches Goals or Changes through the rail:** "Work", then the
  filter; where the filter is hidden by kind 7 or empty, open the address the
  harness can open directly, as the story's other pages do.
- **If a story's check of the rail's order is its point:** rewrite it to 5.3's
  order, keeping what else it checks.
- **If a story fails for a reason other than a press:** run it once on the trunk's
  base before this task's commits; failing there too, it goes under `## Left`.

## Steps

1. Find every press and find (Inputs), per package.
2. Add `start_change` to each harness that starts chats from the rail, and use it.
3. Rewrite each press; keep each story's checks.
4. Run acceptance 4; read each failure's log first.
5. Commit and push: the message says the stories walk the rail, Work and Deployment
   as the client draws them.

## Report

Follow the implementer's text. Also list each story changed, by package and test
name, and each one compiled and not run.
