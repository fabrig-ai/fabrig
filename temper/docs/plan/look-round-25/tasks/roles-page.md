+++
id = "roles-page"
part = "roles"
kind = "feature"
needs = ["how-drawing", "roles-view"]
files = [
  "temper/crates/temper-client-domain/src/pages/roles.rs",
  "temper/crates/temper-client-domain/src/pages/how.rs",
  "temper/crates/temper-client-domain/src/tests/pages/how.rs",
  "temper/crates/temper-client-domain/src/tests/pages/project_pages.rs",
  "temper/crates/temper-client-view/src/pages/how.rs",
  "temper/crates/temper-client-view/src/pages/settings.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/how.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tests/client/tests/suite/ways.rs",
  "temper/tools/gallery/src/worlds/ways.rs",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/settings.txt",
  "temper/docs/design/client/journeys/chat-draft/**",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 150
+++

# Roles on How work runs

## Goal

The request's item 2 (`temper/docs/plan/look-round-25/inputs/request.md`), on the page: How work runs gains Roles, every agent the project's work runs, a row each with its name, when it runs and its model, opening to what it can do, what it is asked and a link to its instructions, read from the view `roles` that `roles-view` built.

Acceptance, by `temper/docs/design/client/README.md`, 5.13, *The page* (*Roles*) and *Its parts*, and `temper/docs/design/client/browser.md`, 2.5, *How work runs' look* (*Roles*) and 7.2's `how-draft`, as this cut wrote them, and `temper/docs/plan/look-round-25/notes/contracts.md`, *The view `roles`*:

1. **The read** (`temper-client-domain/src/pages/roles.rs`): the page state that reads `charter N` for `Role` and `rule K/N` for `Rule` reads `roles` for `Settings { project }` too, with the same lifecycle (once as the page opens, again at a newer settings revision, its handle dropped on leaving); `view_address` gives `roles` there. No new watch, act or address.
2. **Its part** (`pages/how.rs`): `Facts` gains `roles: bool`, the read drawn as a `w::View::Roles` with at least one role; `Parts` gains `roles: bool`, `parts` setting it from the fact alone (`README.md`, 5.13, *Its parts*).
3. **The section** (`temper-client-view/src/pages/how.rs`, `roles(b, root, w, bindings, project)`, called by `pages/settings.rs` between `how::path` and `how::budgets`): while `parts.roles`, a section "Roles", its head's right "every agent PROJECT runs"; a `Role::Disclosure` a role, as `confirm.rs` and the frame draw one, its summary the card's `name`, its `who` and its "Model" fact's value, opened its "Can" and "Asked to" facts as terms (as `views.rs`' `facts` draws a card's), then "Read its instructions ›", a link (`Act::Go { address: Address::Role { project, charter } }`). Closed as the page opens.
4. **Words and look:** `words.rs` keys from 5120 (`notes/common.md`): "Roles", "every agent {name} runs", "Read its instructions ›". `look.rs`: the reserved slots 648 to 652 (`notes/contracts.md`, *`look.rs` and `words.rs`*) become `temper-how-roles`, `temper-how-role`, `temper-how-role-line`, `temper-how-role-model` and `temper-how-role-more`, with their constants. `temper.css` under `ui: how`: `whole-story.css`' `.rows`, `.row` (as a role's summary) and `.role` rules (its lines 103 to 106 and 257 to 272), values and all, on those tokens and the disclosure's markup as skein draws it, and its `@media (max-width: 760px)` rows.
5. **Rows, no new test function:**
   - `temper-client-domain`: `tests/pages/how.rs`' parts table, rows with and without a `roles` answer; the role page's read test (`tests/pages/project_pages.rs` or where `Roles::configure` is tested, `rg -n 'view_address|Roles::' temper/crates/temper-client-domain/src/tests`), a row for `Settings { project }` reading `roles`.
   - `temper-client-view`, `tests/how.rs`: a case with a `roles` answer of three roles: the section's rows in order, a disclosure opened showing "Can" and "Asked to", the link's act; with none, no section; in the terminal the rows print.
   - `temper-client-world`, `tests/suite/ways.rs`, its How work runs rows: the page reads the engine's `roles` over the stock project and shows "Chat", "Ask temper", "Coding agent" and "Status agent", in order (the meeting point of `roles-view` and this page).
   - The gallery: `how-draft`'s hand-made world (`temper/tools/gallery/src/worlds/ways.rs` or where the `how` worlds stand) answers `roles` with Chat, Ask temper, Coordinator, Coding agent, Foreman and Status agent; its test's assertions for `how-draft` see them. No state added.
6. **Prints:** the journey `chat-draft` reprinted (`notes/common.md`, *Prints*); the terminal's `settings.txt` golden regenerated if How work runs is in it.
7. **Status:** delete the **Not yet** entries naming `roles-page` in `temper/docs/design/client/README.status.md` (5.13) and `temper/docs/design/client/browser.status.md` (2.5 and 7.2). No other design line.
8. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-client-domain`, `-p temper-client-view`, `-p temper-client-gallery`, `-p temper-client-shell-web`, `-p temper-client-shell-terminal`, `-p temper-client-journeys`, `-p temper-client-world` and `--profile fuzzy -p temper-client-world`; clippy on every package changed.

## Inputs

- `temper/docs/plan/look-round-25/notes/common.md`, `temper/docs/plan/look-round-25/notes/contracts.md`.
- The request's item 2.
- `temper/docs/design/client/README.md`, 5.13, *The page* (*Roles*), *Its parts*; 3.5, the pages' table's `Role`; 5.16, *Read when shown*.
- `temper/docs/design/client/browser.md`, 2.5, *How work runs' look*; 7.2, `how-draft`.
- `temper/docs/design/domain/forge.md`, 8.13, *The project's roles*.
- The target: `temper/docs/design/client/web/whole-story/how.html`, its section "Roles" (what follows each line's colon is P3e's, not drawn), and `whole-story.css`, the lines named.

## Owns and may change

Owns the files above. Reader files: a match on `Address` that the read's widened arm forces, its arm alone. Not the path's drawing (`pages/ways.rs`, `paths/**`), nor the engine.

Options, each decided:

- **A role whose card has no "Model" fact:** the summary's right is empty.
- **The disclosure's open state:** skein's native disclosure, held by the shell as `confirm.rs`' is; no page state of the domain's, no address.
- **The Settings frame's other tabs** (Repositories) read no `roles`: only How work runs' tab does, as `how::facts` reads the ways for it alone.

## Steps

1. Read the Inputs, `pages/roles.rs` (both crates), `how.rs` (both), `settings.rs`' How work runs body, `confirm.rs`' disclosure and the tests named.
2. The read (1) and the part (2) with their domain rows; bite: leave `Settings` out of `view_address`, see the row fail, restore.
3. The section (3), words and look (4), the view's, world's and gallery's rows (5), the prints (6), the status (7), the runs (8); `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; commit; `python3 -I -B tools/citations/citations.py --base origin/main`.
4. Commit with the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| Roles: every agent the project runs | landed (`pages/how.rs`, from the view `roles`) |
| A role's line, "When you look at Home after landings" for the status agent | landed, the engine's words |
| What a role does, past when it runs | deferred to P3e (`temper/docs/design/draft/whole-story.execution.md`) |
| A standing role's "Its area on Team ›" | deferred to P4b and R1 (`whole-story.execution.md`, section 3) |
| `chat-draft` journey | reprinted |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4) |

## Report

The implementer's report: the read and the section as drawn, the rows and the bite with its failure, the prints regenerated, the runs' counts, and the account as committed.
