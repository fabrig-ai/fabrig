# reference-m9: reds to the goal; g's tier

Carry out milestone **M9** of `temper/docs/design/draft/reference-case.execution.md`. Its entry is the scope, quoted below. Section 2 there says how a milestone runs: design and code go through this plan as one pipeline, and its designers write the design into the trunk, family by family, with the specs that cite it (`DESIGN.md`).

In short: a goal branch's own tier, which judges no landing and sends its reds to the goal as news; a red after landing reaching its goal as news; the coordinator's triage; and `repairs`, a change naming the red landing it repairs, which clears it.

It builds on what is on main:
- M8: main's tiers, verdicts, blame and bisection, a change living to cleared;
- M1: the goal's own branch g;
- M2: L, the goal's landing.

## The entry

### M9. Reds to the goal; g's tier

- **Executes:** `reference-code-production.md`, section 10 (g's tier,
  its reds as news, triage, a repair into g while merges go on);
  `reference-landing.md`, section 6 (a red to its goal, triage, a repair
  into main naming it, cleared by it): `after-landing.md`, section 7.
- **Design:** `after-landing.md`, sections 1 to 3 (a goal branch's tier,
  judging no landing, its reds as news); `agent.md`, 4.3.1 and
  `after-landing.md`, section 7 (`repairs`).
- **Builds:** temper's g's tier, a red's news to its goal, `repairs`.
- **Configures:** g's tier (the gate without its slowest suites,
  following goal branches, at most once per 20 minutes); the
  coordinator's charter: triage along the way and after landing, notes
  on what it learns.
- **Old code:** temper `verdicts/t9-bound`'s red news to goals and
  `repairs`; g's tier and triage were never built.
- **Walk:** steps 20 and 25 to 27; `reference-landing.md`, section 9's
  third story.

## What runs beside it

Wave 2's M3, M5, M6 and M7b are still building, with `configuring-p2`, `look-welcome-chats` and `first-run-projects`. Several plans add to the same sections (`reference-case.execution.md`, section 4). M9's are `agent.md`, 4.3.1 (`repairs` among `change`'s parameters), `after-landing.md`, sections 1 to 3 and 7, and the coordinator's charter. Add M9's own part and leave the others' alone: a compatible addition, never a rewrite. A designer meeting another plan's text there builds on it as it lands on main and reaches this trunk by catch-up.

A budget or timeout is the user's (skein's `docs/foundation/testing-strategy.md`, 8.1): a role never raises one.

## Old code, for reference only

temper `verdicts/t9-bound`, in hel8's mirror `~/src/fabrig/attic/temper/mirror.git` (`git -C ... show <branch>:<path>`); the assessment is in hel8's `~/src/fabrig/coord/old-repos-value/`. Its red news to goals and `repairs` are commits a60054d5b and c62b86964. It is a reference, never a plan. Whether to use it, what of it and how is the designers' and implementers' choice, case by case: adapting old code may cost more than writing it fresh. Implementers cannot reach the attic, so whatever a designer takes goes into the specs or notes.

## Size

One cut is at most 90 minutes. Release early, family by family. If the whole cannot be designed in one cut, release what holds together and end with a `Question:` naming the rest.

## Walk

fabrig/qa judges and walks it once it lands. Mark the slice that makes the entry's story run `walk = true`.
