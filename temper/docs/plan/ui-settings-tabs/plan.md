# ui-settings-tabs: the plan

From `inputs/request.md`: `Forges`, `Accounts` and a new read-only `Builders` tab in Settings, and paths on `ConnectForge`, `ConnectModel` and `Adopt`, in the greenfield mock's language. The mock draws none of these tabs, so the design is cut ahead of the tasks, saying how its language applies: `temper/docs/design/client/README.md`, 3.5, *Settings*, 5.3 and 11; `client/forge.md`, 5.4; `client/browser.md`, 2.3, 2.5 and 7.2; their status files. No draft is involved. The shapes that cross tasks are in `notes/contracts.md`.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `forges-tab` | | high, 120 | the row's token and rules, the frame's helpers, `Forges` in the language, alone with no project too |
| 2 | `accounts-tab` | 1 | high, 130 | `Accounts` the same, the old accounts page's rules gone |
| 3 | `flow-paths` | 2 | high, 100 | the three paths, `Adopt`'s look, "Start over"'s return to `Forges` |
| 4 | `builders-tab` | 3 | high, 110 | `Address::Builders`, the watch, the tab, its samples, a world's story, the status closed |

## Critical path

All four, as the request asks: ~460 implementer-minutes, then the trunk's end check. The chain is for `temper.css`, the gallery and the goldens, not for the code: only `accounts-tab` builds on another's code.

## Overlaps with the drafts beside this one

- **The request's order:** it says this plan starts once `ui-settings` has landed. This trunk stands on `ui-settings`' trunk, with the frame, the tabs and `ways-session`'s work, but `how-work-runs`, `settings-proposal` and `settings-proof` are still to run. No task needs them. They change `pages/settings.rs`, `look.rs`, `words.rs`, `temper.css` and the gallery, so that each side will meet the other's lines on a catch-up. This plan keeps to its helpers and `Tab` in `pages/settings.rs`, and Builders is a file of its own, `pages/builders.rs`.
- **`temper.css`, `look.rs`, `words.rs`, `frame.rs`, `root.rs`, the gallery's `names.txt`, `samples.rs`, `worlds.rs`, `tests.rs`:** `ui-chats`, `ui-needs`, `ui-welcome`, `ui-goal-page` and `ui-settings` all change them, and none can be moved. This plan keeps to its marker and its `SETTINGS_*` tokens and regenerates on a catch-up (`notes/common.md`).
- **`forge/connect.rs`, `pages/accounts.rs`, `pages/connect_model.rs`, `temper-client-domain`'s `forge/forges.rs`:** `ui-welcome`'s `ready-step` and `sheet-choices` change them. `flow-paths` touches `connect.rs`' `main` and `unfinished` and `removal_told` alone, and `forges-tab` and `accounts-tab` leave `connect.rs` as it is, by scoped rules.
- **`client/README.md`, `README.status.md`, `browser.md`, `browser.status.md`, `forge.md`, `forge.status.md`:** every draft beside edits them. This plan's lines are its own but for 3.5's *Settings* and 5.3's "Workers" sentence.

## What the cut settled

- **With no project, `Forges` and `Accounts` take the language too, with no tabs.** The design said they stay as they are, but the request has the old rules go rather than be overridden. They keep their level-1 heading.
- **The terminal draws no path.** The request wants the flows' path as `Configure`'s, and says the terminal's prints keep their meaning. A path would shift every number on the flows and add nothing to a rail that holds "Settings".
- **The tab's controls take `.btn` by rules scoped under the row,** not by new styles on calls `forge/connect.rs` shares with the welcome, so the welcome's look holds by construction.
- **"Start over" from `ConnectForge` returns to `Forges`,** the one ending with no page of its own: today it leaves for the welcome and on to the inbox. From the welcome it opens the step's choices, as ever. `flow-paths` greps for others.
- **Builders' heading and lede are the cut's words,** since the request gives none: "Where agents work." and "The machines temper’s agents run on, each with its slots and how many are in use."
- **Who reads Builders:** jig gives the `deployment` document to any role in the deployment, so the watch opens for any, though the refusal's words say owners, as the request has them.
- **"None connected" and left lines together:** with hosts kept but none connected, `.empty` stands above their "Not connected" lines, so that no state is dropped. With none kept, it stands alone.
- **`Word::ForgeSummary` takes a capital,** "Forge 1 · Forgejo", as the request draws it. "forge N" elsewhere (ctl's words) stays.
- **Journeys:** `temper-client-journeys` fails a stale journey, and these tasks change pages and a stylesheet. A task reprints what goes stale. `forges-tab`'s spec went out before this was found and doesn't name it, so `accounts-tab` takes up anything it leaves.
- **No samples are promised beyond the request's:** a state no gallery world reaches gets a sample if a few lines do it, or else its view test is named in the report.

## Formats and interfaces that change

No stored format and no protocol. **A public address is new,** `/projects/<project>/settings/builders`, and `Address::Builders` and `forge::forges::Act::StartOver`'s `end` are public Rust, in the client's own crates. jig changes nothing.
