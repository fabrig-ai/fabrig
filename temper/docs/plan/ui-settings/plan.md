# ui-settings: the plan

From `inputs/request.md`: Settings' frame, How work runs and Repositories, as the greenfield mock draws them, on a new setting of temper's, kind 7. The design is cut ahead of the
tasks: `temper/docs/design/protocol/configuration.md`, 13.2, 13.4 and 13.9; `protocol/store.md`, 2; `client/README.md`, 3.5, 5.3, 5.13, 10 and 11; `client/browser.md`, 2.3, 2.5 and 7.2; `client/forge.md`, 9;
`client/tools.md`, 4 and 5.3; the status of `draft/configuring.md` and `draft/template-places.md`.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `project-templates` | | medium, 25 | jig's `Core::project_templates` |
| 2 | `ways-kind` | 1 | high, 150 | kinds 6 and 7, the route, a project's read |
| 3 | `ways-ctl` | 2 | high, 110 | the entrances take them; ctl, the file, the agent's tool and words; the reference's `project.json` |
| 4 | `settings-frame` | | high, 170 | `Address::Repositories`, the tabs, Repositories, `Forges` and `Accounts` in the frame, "Sign out" |
| 5 | `ways-session` | 2, 4 | high, 140 | the session's watch and reads, the acts' requests, the rail and tab bar, `drafts_offered` |
| 6 | `how-work-runs` | 5 | high, 170 | the tab's sections and live controls |
| 7 | `settings-proposal` | 6 | high, 110 | a proposal of settings' card, and the read of its values |
| 8 | `settings-proof` | 3, 7 | high, 100 | the meeting point, renders beside the mock at three widths, the status closed |

The request asks for three to five tasks; this is eight. It lists six parts, and the engine's and the proof's each split for a reason: `project-templates` is jig's and a branch stays in its tree, and `ways-ctl` has crates of
its own, so that it runs beside the client's chain and takes nothing from it. The chain the request wants for `temper.css`, the gallery and the goldens holds: 4, 5, 6, 7, 8, each `needs` the last.

## Critical path

`project-templates`, `ways-kind`, `ways-session`, `how-work-runs`, `settings-proposal`, `settings-proof`: ~695 implementer-minutes, then the trunk's end check. `settings-frame` runs from the start beside the engine
and joins at `ways-session`; `ways-ctl` runs beside the chain and joins at the proof.

## Overlaps with the drafts beside this one

- **`temper.css`, `frame.rs`, `look.rs`, `words.rs`, `pages.rs`, `root.rs`, the gallery's `names.txt`, `samples.rs`, `worlds.rs`, `tests.rs`, and the terminal's goldens:** `ui-goals`, `ui-needs`, `ui-chats`, `ui-changes` and
  `ui-welcome` all change them, and none can be moved. This plan keeps to its marker in `temper.css` and its own `pages/settings.rs` and `forge/ways.rs`, and regenerates on a catch-up (`notes/common.md`).
- **`client-wire-v1.schema`, `store-v11.schema`, `settings.json`:** `ui-changes` and `ui-welcome` change the schemas: each adds lines of its own, and the generated code regenerates. `settings.json` isn't changed here.
- **`client/README.md`, `README.status.md`, `browser.md`, `browser.status.md`, `configuration.md`:** every draft beside edits them. This plan's lines are its own but for 3.5's *Settings*, 5.3 and 5.13, which the request hands it; `ui-goals`' Goals in 5.3 will need a catch-up.
- **The rail by what has landed:** Goals, Changes and the Away switch may or may not be on the trunk when `ways-session` runs: its spec says what to do in each case, and `ui-goals`' and `ui-away`'s requests say what they do if this lands first. `settings-proof` reads the trunk once more.
- **The request's order:** it says this plan starts once `look-round-5` and `ui-changes` have landed. `look-round-5` has; `ui-changes` hadn't when this was cut, and no task needs it: fab merges it in as it lands.

## What the cut settled

- **Kind 6 is built here.** The request reads it as live, and 13.2 designs it, but the forge declares kinds 1 to 3 alone. `ways-kind` makes it a setting, and the `change` that names no route takes it. **So a `foreman`
  on the tab changes the work's questions from then on:** a change of behaviour, by design, off by default.
- **Kind 7 takes the number** though `draft/template-places.md` names 7 and 8 for offers: that draft is adopted by no one, and its status now says its kinds take the next free numbers.
- **Kinds 6 and 7 have no stock row.** A stock default names one scope and a project made later has its own number, so their stock is a constant of the code, read when no value is stored (`configuration.md`, 13.4). A removal returns a project to it.
- **The reference's kind 7 is a file of its own, `docs/examples/reference/project.json`, not `settings.json`** as the request says: `settings.json` is applied before the case's project exists, and jig refuses a project's item then. Its `way` is 4, the reference's goal.
  The walk that applies it is `reference-walks`' to add, by 13.9.
- **A read of a project's kinds takes a role in that project, and a proposal's items keep a role in the deployment:** those items span scopes. A project's owner with no deployment role sees the card's kinds 6 and 7 named as ctl names them, not their values (5.13).
- **Kind 7's controls are disabled together while one is out:** they write one record, so two at once would lose one; the request said only "its control". The value asked shows meanwhile, and the answer is held at once, so the next press starts from it.
- **`drafts_offered`:** the request calls it `ui-chats`', and it isn't on `ui-chats`' trunk. `ways-session` adds it, public, and whoever draws "+ Draft" calls it.
- **"Sign out did not reach the engine." stays on the rail's foot too.** The request moves it with "Sign out", from the stub. The tab gets it, and the rail's stays: it is 6.6's, and a person with a lost sign-out may be on any page.
- **`.setting`'s switch and radio are `Role::Check` and `Role::Choice`,** with a fallback pre-approved in `how-work-runs` if skein's mapping won't take the look.
- **Section 11 closes** How work runs' place and look, its docking and Settings' own page. The model's line that left Home stays in no place: nothing in the mock's tab draws it.
- **Formats that change:** temper's store family 20 and the client wire gain the two values, in place; existing stores keep decoding, since an enum gains variants. `ChangeSettings` is now sent by the pages, for the tab's rows alone: 5.13's exception.
