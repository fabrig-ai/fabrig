# ui-p4a-team: what every task keeps

Every spec of this plan names this note; where a spec says otherwise, it
says why and wins.

- **The request's rules bind:** `temper/docs/plan/ui-p4a-team/inputs/request.md`,
  its section *For every task of this plan*, read whole. Focused and fuzzy
  alone run; a browser, process or live story is written and compiled
  (`heavy cargo check -p PKG --tests`), never run.
- **The design is this cut's:** `temper/docs/design/client/README.md`, 3.5,
  the pages' table's `Team { project }` row and *Team*, whole;
  `temper/docs/design/client/browser.md`, 2.3's `Team` row, 2.5's *Team's
  look* and 7.2's rows `team`, `team-alone` and `night-team`. A spec cites
  the parts it builds.
- **The contracts:** `temper/docs/plan/ui-p4a-team/notes/contracts.md`.
  Build your half to it; a shape there that your code can't meet is a
  question, never a change of the shape.
- **The look is the whole story's mockups:**
  `temper/docs/design/client/web/whole-story/team/team.html` for a project
  of people, `team.html` beside it, and `whole-story.css` (`.col.wide`,
  `.intro`, `.kicker`, `.headline`, `.lede`, `.section`, `.shead`,
  `.grid3`, `.box`, `.small`, `.muted`). Draw only their People section
  and intro: the areas' table, a role's card, "+ Invite", "+ Hand an area
  to a role" and the next step's card are later milestones' (P4b, P4c),
  and nothing stands in their place (`whole-story.execution.md`, 2).
- **Other plans' files:** the rail and Work are `ui-p1-rail`'s; Home and
  Needs you are `ui-p2-home`'s. Change nothing in `temper-client-view`'s
  `frame.rs`, `pages/home.rs`, `pages/needs.rs`, `pages/inbox.rs`, or
  `temper-client-domain`'s `pages/home.rs`, `pages/needs.rs`,
  `pages/inbox.rs`, `pages/rail.rs`, `pages/work.rs`, `pages/parts.rs`,
  `pages/all_home.rs`, but the compiler-forced arms of `Address::Team`
  and what a spec names. Only `team-rail` touches the rail.
- **The engine is out of scope:** no change to `temper-engine-domain`,
  `forge_route.rs`, jig or any wire schema. A read Team lacks is named in
  the status, never added.
- **The TUI is temper/tui's:** in `temper-client-tui` and
  `temper-client-shell-tui`, the forced arms of `Address::Team` alone,
  each as `Builders`' arm (the launchpad; `Home`'s projection in
  `host.rs`).
- **Step crates' rules** (`AGENTS.md`, section 2): no `_` arm and no
  `matches!` in `temper-client-domain` or `temper-client-view`; a new arm
  of `Address` goes in every exhaustive match. Use `if let` to ask one
  variant.
- **Status lines:** your spec names the **Not yet** entries your task
  closes in `temper/docs/design/client/README.status.md` and
  `browser.status.md`. Remove exactly those, whole, once your code holds
  what they say is missing; change no other line of `docs/design/`. This
  departs from the implementer's text, which closes `docs/design/`.
- **Reader repairs:** a test of your package or a direct dependent that
  your change's correct behaviour trips is yours to repair, keeping what it
  checks for everything else (the request's rule), and named in the report.
- **A red that isn't yours:** a focused test of a package you didn't
  change that fails on your branch runs once on your base (`git stash` is
  shared: use a second worktree or a WIP commit, never bare `git stash`).
  Failing there too, it's listed under `## Left` with its log's lines.
- **Tests stay cheap:** temper's focused and fuzzy suites are over their
  budgets already (`temper/docs/design/domain/testing.status.md`), and a
  plan that adds cost is rejected at the gate. A new case is a row of the
  table test beside it; a new test function only where a spec names it.
  Name each new test's CPU time in the report.
- **Phrase keys and style tokens:** Team's words take `Word` ids 4900 to
  4949 alone (`temper-client-view/src/words.rs`, each variant in its four
  places, beside its area's words, never at a list's end). Team's style
  tokens are appended to `look.rs`'s `NAMES`; where a catch-up meets
  another plan's tokens at the same numbers, renumber Team's, never
  theirs. `temper.css`'s rules go under `/* ==== ui: team ==== */`, placed
  just before `/* ==== ui: settings ==== */`.
- **Goldens, prints, the gallery's `names.txt` and `tests.rs`:** on a
  catch-up, regenerate rather than hand-merge; keep `names.txt` sorted as
  it is.
- **Before the push:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`
  (and each other tree's you touched); `heavy cargo clippy -p PKG ...
  --all-targets -- -D warnings` for every package changed; the focused
  tests of every package changed and of its direct dependents; commit,
  then `python3 -I -B tools/citations/citations.py --base origin/main` (it
  reads `HEAD`).
