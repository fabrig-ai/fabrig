# ui-ways-rules: plan

Step 2 of `temper/docs/design/draft/stock-ways.md`, section 7: its
section 3, whole (`inputs/request.md`). A project's way becomes its own.
The stock's landing rule moves from the deployment's kind 1 to every
project's kind 2 rule 1. A project's rule may name a branch of each of its
repositories. Draft first's coordinator (charter 5) and goal template
(template 4) join the stock, with the delegation that reaches them.

The cut wrote the design first: `temper/docs/design/protocol/configuration.md`,
13.2, 13.3, 13.4 and 13.9, with `configuration.status.md`; also
`temper/docs/design/domain/forge.md`, 8.9, and
`temper/docs/design/protocol/setup.md`, 8.3, each with its status. The
draft's status no longer lists 3.1 to 3.3 as lagging.

## Parts and tasks

- **rules:** task `forge-rules`, high, about 130 minutes, no needs. The
  forge admits a kind 2 pattern beginning at `branch` and matches it past
  the repository's segments. It holds a rule given as kind 2, scope 0 as
  every project's rule of that number, while the project sets none of its
  own; a removal or a reset brings it back. The root counts such a rule's
  approvers over every repository of the project, and a run's `settings`
  read answers the given rule. Tests: the given rule per project and its
  reset; a branch rule over two repositories, one adopted after the rule;
  the fuzzy branch-against-full pattern test.
- **roles:** task `stock-roles`, high, about 170 minutes, no needs.
  Charter 5 is `inputs/charter5.md` byte for byte, split into shared
  instructions and four parts. Template 4 is a goal for a draft, at the
  stock's models, with its own goal branch. The delegation gains charter 5
  and the merge job. The task also moves every fixture and count that pins
  the stock (eight charters, four templates, fourteen executor kinds).
  Tests: charter 5's parts as written; the stock's delegation, through
  the domain world's goal from the stock's template 4.
- **words:** task `field-words`, medium, about 50 minutes, no needs. The
  form's words and the `landing-rule` help page cover the `branch` form
  and the per-project stock. A settings file and `propose_settings` carry
  either form.
- **stock** (next batch): task `project-stock`, needs `forge-rules` and
  `stock-roles`. The binary gives the forge kind 2's rule 1 for every
  project and no kind 1 rule. Every test and world that assumed kind 1's
  stock moves with it.
- **reference** (next batch): task `reference-rules`, needs `forge-rules`
  and `stock-roles`. The reference's `project.json` sets the project's
  rule 1 to main's, naming a branch, so the stock's rule over every
  branch doesn't hold its goals' branches.

Slice 1 holds rules, roles and words; slice 2 holds stock and reference.
No walk.

## Critical path

`forge-rules` (130) and `stock-roles` (170) start at the first push,
beside `field-words` (50). `project-stock` follows both. The path runs
`stock-roles`, then `project-stock`: about 170 + 120 implementer-minutes.

## Who hears first

The delegation's changes, for the owner of the goal's charters, to review
before landing (its generated settings grant executors by number):

- The stock rules' ceiling and policy 1's ceiling, and so roles 0 to 2,
  delegate to `Charter(5)` and `Job(3)` beside today's kinds: fourteen in
  all (`notes/contracts.md`, section 3; task `stock-roles`).
- Charter 5 and template 4 are stock numbers now. The reference's
  `settings.json` sets its own editions of both over the stock's: charter
  5's second edition, and template 4 with its LLM choice.
- The reference's widening (its `README.md`) still adds charter 7. Charters
  5 and 6, job 3 and role 0 are already the stock's, so its `unique`
  leaves them once.

The landing rules' scope and pattern change, for the TUI's owner and the
same owner:

- Kind 1, the deployment's landing rules, has no stock rule. Its owners
  may add rules, which every project's merges meet beside its own.
- Kind 2's rule 1 is every project's stock: every branch of each of its
  repositories (pattern `["branch"]`, open), enforced, CI, up to date,
  gate 1 blocking at a clean head. A project's owners may loosen it;
  removed or reset, it is the stock's again.
- A kind 2 pattern may begin at `branch`, covering that branch in each
  repository of the project, those adopted later among them. One
  beginning at `forge` keeps its meaning. Kind 1's begin at `forge` alone.
- The forge's read of a project's rule 1 answers the stock where the
  project has set none. jig's settings inventory no longer lists a
  deployment rule 1, and lists no project rule 1 either.

## What was decided

- **The project's rule 1 is given to the forge, not built into it.** A
  constant inside the forge crate, as kinds 6 and 7 have, would add an
  agent's review to every merge in every test world, because rules add.
  So the stock module gives it at each start as an entry of kind 2 for no
  one scope, where it gave kind 1's rule 1 before. A world that gives its
  own forge defaults gives its own or none. The forge tells jig nothing of
  that entry, since jig's defaults name one scope.
- **One matching function.** `covers_branch` decides coverage for every
  reader of held rules. A `branch` pattern is matched past the
  repository's four segments, so a project's rule covers repositories
  adopted later with no change.
- **Charter 5 runs fresh, waiting 10 s.** 13.9's reference waits 0, but
  the root refuses a run policy that waits 0
  (`temper-engine-domain/src/limits.rs`, `run_policy_bytes`), and a
  charter in parts must not resume.
- **Template 4's report is the deployment's result bound,** as every stock
  template's is. The reference's 63,738 bytes stays the reference's.
- **The reference keeps its two deployment rules.** Its project sets its
  own rule 1 (main's, naming a branch) in `project.json`. Moving both
  rules into the project would also move stage one's golden, which names
  the deployment's two rules (`temper/docs/design/domain/testing.md`).
  That golden not naming the project's rule 1 stays a **Not yet** in
  `configuration.status.md`.
- **No wire or stored format changes.** The client wire, the settings
  file and the forge's stored rows already carry any first segment. The
  forge's judge was the only refusal.
- **The web's Settings writes no landing rule,** so "Settings writes
  either form" is ctl's settings file and Ask temper's `propose_settings`,
  both pinned by rows in `field-words`.

## Overlaps with drafts beside this one

None found. `tui-launchpad` works in skein's screen crates. `look-round-19`
works in the web client's pages and stylesheets. `ui-authorize-busy` works
in jig's engine clients. One overlap could not be avoided: if
`temper-client-journeys again` rewrites a journey's step pages
(`temper/docs/design/client/journeys/*/[0-9][0-9].html`) as well as its
`recording`, it meets `look-round-19`'s pages, and those are regenerated
on a catch-up, never hand-merged. `stock-roles` and `project-stock` may
each regenerate the recordings. `project-stock` runs after `stock-roles`,
so the second regeneration starts from the first.
