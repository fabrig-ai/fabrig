# debt-8: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of each tree your task touches. A task stays in its tree, but for the
  files its spec names in another one, each edited only as the spec says.
- **The request** is `temper/docs/plan/debt-8/inputs/request.md`. The user answered `d001-scope` with (c):
  `Now::Activate` stays in its decision, so a release stays one commit, and only `Now::PrepareAgent` leaves.
- `temper/docs/plan/debt-8/inputs/held-core-prepare-agent.md` was written on a task that is dropped. Your spec
  replaces it: read it only for background. The branch `origin/debt-5/core-prepare-agent` is for reading only;
  never merge or cherry-pick it. Its commits sit on the dropped task, and its Resume gate and its `expect` are
  wrong on main (`notes/contracts.md`).
- **Behaviour stays unchanged.** `release` in `temper/tests/engine/domain/tests/suite/forge_routes/reviews.rs`
  stays as it is, and so does every test's visible events and outcome. If one changes, stop and ask: never
  retime a test or rewrite an assertion to make the change pass. A test that named a removed variant takes the
  next step as a root would and keeps its assertions about what follows.
- **Stop and ask** on a stored format, a wire shape, a golden file, or a public item the spec does not sketch.
  The sketched changes to `core::Now` and `core::Event` are the ones the user's answer asks for.
- **Size caps in tasks are the cut's estimates, not user budgets.** Compact first. If a cap still can't be met
  without losing wanted behaviour, an answer call may raise it by up to 25%, stating the measured size and why.
  Beyond that, ask. Budgets and timeouts stay the user's.
- **A test that pins a rule must be able to fail on that rule.** Break the rule by hand, watch the test go red,
  restore it, and report what was broken and what failed.
- **Status files and design text are the spec's.** This departs from the implementer's text ("Not yours: any
  `docs/design/`"): a task edits exactly the status files its spec lists, in the way it lists. Drafts stay
  untouched. A status file's form is `DESIGN.md`, 2: its date line is today's (`date -u +%F`), and it holds only
  the gaps as they stand after your change. Remove an entry the change closes; where an entry names several
  gaps, remove only the words for those closed.
- **Citations.** Before pushing, run `python3 -I -B tools/citations/citations.py --base origin/main` from the
  repository root and fix what it lists in your files; `--all` reads 0 at this plan's base and stays 0.
- **Comments and design text say what is true now,** in the design's words, with no debt item's ID in code.
- **Old-form counts only fall:** `python3 -I -B <tree>/scripts/old_forms.py`, with `--write` only to lower one.
- **Markdown read by tests:** `.config/gate/read-by-tests.txt` lists it. Nothing in this plan edits such a file.
- One commit per coherent step (the code and its tests; then the status and debt text), each saying why.

## What runs

A change of meaning runs its readers' tests. Each count is above zero:

```
heavy cargo nextest run -p jig-domain-core -p jig-protocol-store -p jig-test-domain -p jig-test-service -p jig-test-store -p jig-ops-domain
heavy cargo nextest run -p jig-core-world -p jig-ops-world -p jig-client-world -p jig-client-protocol-world
heavy cargo nextest run -p temper-engine-domain -p temper-service
heavy cargo nextest run -p temper-engine-domain-world -p temper-conformance-world -p temper-system-world
heavy cargo clippy -p jig-domain-core -p jig-ops-domain -p jig-test-domain -p temper-engine-domain --all-targets -- -D warnings
```
