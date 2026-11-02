# reference-m8: tiers and verdicts on main

Carry out milestone **M8** of `temper/docs/design/draft/reference-case.execution.md`. Its entry in section 3 is the scope. Its section 2 says how a milestone runs: its design and its code go through this plan as one pipeline, and its designers write the design into the trunk, family by family, with the specs that cite it (the repository's `DESIGN.md`).

## The scope (M8's entry)

- **Executes:** `temper/docs/design/draft/reference-landing.md`, section 6: main's tiers, the verdict, L living to cleared. In design terms, `temper/docs/design/domain/after-landing.md`, sections 2 to 6, which are designed and not built, together with whatever of its sections 10 and 11 they need.
- **Design to write:** `after-landing.md`, sections 1 and 3: a tier names the branches it follows, and main's tiers follow main only (`temper/docs/design/draft/notes.md`, N5). The rest is already designed: the families (`temper/docs/design/protocol/store.md`), the `ctl` groups (`temper/docs/design/client/tools.md`), and skein's `Read::Artifact` and `Read::Commits` (skein's `forge.md`, 3.2, and `fake-forge.md`, 2.3).
- **Builds:** temper's tiers, runs, verdicts, and a change living to cleared; skein's reads; jig's part where a procedure the deployment starts needs funding.
- **Configures:** main's tiers, which are the slow suites; the verification bound, two hours.
- **Out of scope:** `after-landing.md` 7 (a red to its goal, `repairs`) and a goal branch's tier are M9's. Sections 8 and 9 (trailers, what the forge shows) are not the reference's. Findings are not built.

## Old code, for reference only

The verdicts line holds most of this build, written against an older tree:
- jig `verdicts/jig-zero-seed-13`;
- skein `verdicts/skein-reads-3`;
- temper `verdicts/2a-set-8`;
- then temper `verdicts/t9-bound` (reruns, flakes, bisection, the bound).

These branches live in hel8's mirrors at `~/src/fabrig/attic/<tree>/mirror.git`, readable with `git -C ... show <branch>:<path>` or `log`/`diff`. The assessment is in hel8's `~/src/fabrig/coord/old-repos-value/`, which also points to the mission's decision list.

They are a reference, never a plan. Whether to use them, what of them, and how is the designers' and implementers' choice, case by case: adapting old code to today's tree may cost more than writing it fresh. The old code follows the landing branch alone, where N5 has a tier name its branches. Implementers cannot reach the attic, so whatever a designer takes from it goes into the specs or notes.

## Walk

fabrig/qa walks M8 as a rung once it lands. A chat's small fix into main, verified: green clears it; a red reruns, blames and bisects (`reference-landing.md`, section 7). Mark the slice that makes this story run `walk = true`.
