# ui-p1-rail: what every task keeps

Every spec of this plan names this note; where a spec says otherwise, it
says why and wins.

- **The request's rules bind:** `temper/docs/plan/ui-p1-rail/inputs/request.md`,
  its section *For every task of this plan*, read whole. Focused and fuzzy
  alone run; a browser, process or live story is written and compiled
  (`heavy cargo check -p PKG --tests`), never run.
- **The contracts:** `temper/docs/plan/ui-p1-rail/notes/contracts.md`. Build
  your half to it; a shape there that your code can't meet is a question,
  never a change of the shape.
- **The look is the whole story's mockups:**
  `temper/docs/design/client/web/whole-story/` (`work.html`, `pair/work.html`,
  `team/work.html`, each page's `nav.rail`, `whole-story.css`), over the
  greenfield mock where they differ (their `README.md`). The design text wins
  over a mockup on a mechanism.
- **Home and Needs you are another plan's** (`ui-p2-home`, beside this one).
  Change nothing in `temper-client-domain`'s `pages/home.rs`, `pages/needs.rs`,
  `pages/inbox.rs` or `temper-client-view`'s `pages/home.rs`, `pages/needs.rs`,
  `pages/inbox.rs` but the compiler-forced arms of a type you changed.
- **The engine is out of scope:** no change to `temper-engine-domain`,
  `forge_route.rs` or any wire schema. A read the page lacks is named in the
  status, never added.
- **Step crates' rules** (`AGENTS.md`, section 2): no `_` arm and no
  `matches!` in `temper-client-domain` or `temper-client-view`; a new arm of
  `Address` goes in every exhaustive match. Use `if let` to ask one variant.
- **Reader repairs:** a test of your package or of a direct dependent that
  your change's correct behaviour trips is yours to repair, keeping what it
  checks for everything else (the request's rule). The exceptions are named in
  your spec's `## Owns and may change`: the terminal shell's goldens and the
  journeys are `regenerate`'s, and the client world's, browser's, real
  loop's, one process's and `temper`'s process stories are `readers'`, so
  you leave those stale and list them under `## Left`.
- **A red that isn't yours:** a focused test of a package you didn't change
  that fails on your branch runs once on your base (`git stash` is shared:
  use a second worktree or a WIP commit, never bare `git stash`). Failing
  there too, it's listed under `## Left` with its log's lines.
- **Status lines:** your spec names the `Not yet` entries your task closes in
  `temper/docs/design/client/README.status.md` and `browser.status.md`. Remove
  exactly those, whole, once your code holds what they say is missing; change
  no other line of `docs/design/`.
- **Tests stay cheap:** a new case is a row of the table test beside it; a
  new test function only where no table holds its kind. Name each new test's
  CPU time in the report (`heavy cargo nextest run ... --no-capture` prints
  it, or the profile's summary).
- **Before the push:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`
  (and each other tree's you touched); `heavy cargo clippy -p PKG ...
  --all-targets -- -D warnings` for every package changed; the focused tests of
  every package changed and of its direct dependents; commit, then
  `python3 -I -B tools/citations/citations.py --base origin/main` (it reads
  `HEAD`).
