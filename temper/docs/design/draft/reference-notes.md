# Notes behind the reference drafts

Working notes, 2026-11-01. Nothing here is adopted. The evidence and the
fab-to-temper map behind `reference-code-production.md`,
`reference-landing.md` and `reference-case.md`, which name neither fab nor
fabrig: these notes do, and the drafts stand without them.

## 1. The run

`playground-reset-r6` on hel8 (`~/src/fabrig/fab`): draft
`temper/docs/design/draft/playground-reset.md`. `ready = false`, so it never
landed and never caught up with main. It covers code production only. Base
time T0 = 2026-11-01 09:18:21Z, 721 events, ending 15:41.

| When | What |
|---|---|
| T+0:00 | cut call starts (Claude, 53 min, 147 turns, peak context 382,693; designer sub-agents, read-only surveys) |
| T+0:27 | first implementer (`sv-skein-connection`) |
| T+0:27 to 0:54 | cut pushes 9 times, 8 to 9 batches |
| T+1:05 | first mid check, red |
| T+1:29 | an answer call creates `sv-skein-retire`; `fe-remove` needs it |
| T+1:30 to 2:00 | peak, 14 implementers |
| T+1:36, 2:00, 2:14, 2:55, 3:41, 4:27, 5:17, 6:03 | the other 8 mid checks; only 2:00 green |
| T+4:16 to 4:52 | 5 merge refusals (conflicts), 5 refusal foremen, 5 rebase runs |
| T+6:19 to 6:23 | end check green, 4.3 min |

Numbers:
- 63 tasks cut plus 1 added; 4 parts (`fs`, `rf`, `pd`, `pg`), 4 slices, all direct; 5 crate families (`pd`, `fe`, `sv`, `bx`, `cl`).
- Specs 2.6 to 21 KB, 492 KB in all (mean 7.7 KB); notes 99 KB (`contracts.md` 22 KB, `common.md` 3 KB).
- Estimate 6,065 implementer-minutes; critical path 1,080 min; about 1,567 implementer-minutes spent.
- 185 runs: 121 implement (72 done, 45 question, 4 failed), 49 foreman (19 done, 30 could not settle), 10 check, 5 rebase. 54 early runs.
- 135 trunk merges: 72 tasks, 44 calls, 14 answers, 5 repairs.
- 54 questions (45 plus 9 parks): 40 answered by 35 answer calls, 14 by foremen, almost all "rerun once the dependency publishes".
- Refusal foremen said `fix` 5 of 5 in r6, 26 of 27 in the whole log.
- Task merges per 30 min: 5, 13, 12, 12, 8, 4, 5, 3, 4, 1, 2, 3.

## 2. The landing evidence

No replay landed. Sources: the queue's record on hetzner8 (10-30 20:41Z to 11-01
19:19Z), `phase-case/integration` (readied 11-01 17:13:57, landed 17:17:58),
the 10-31 manual playground landing (23 conflicts, 4 repairs, 56 min), the
after-landing and QA events of the log.

- 130 batches: 128 of one, 1 of three, 1 of none. 121 green, 3 red, 3 error, 2 conflict, 1 collateral. 125 landed, 5 rejected.
- 125 landings: 89 `calls/` (doc and spec commits), 36 code. Thin sample.
- Selection: 91 none, 30 full, 5 tests, 4 docs. Timed green runs (33): median 182 s, p90 285 s, max 315 s.
- Errors were infrastructure: a dropped fetch and two cleanups.
- Whole-log rejections: rebase 21, gate:size 11, merge 8, test 7, gate:shape 2, default 1, fetch 1, paths 1, ratchet 1.
- After landing: 29 tier runs, mostly green; 17 `tiers.red`, all one live-smith login-type run. No real new red with suspects.
- QA: 41 walks, 65 findings; triage 44 backlog, 15 path, 1 drop; 15 fixes.
- Codex-kit: 17 of 24 merges met an old-form baseline conflict, settled by a recount.

## 3. The map

| fab | temper/jig |
|---|---|
| tick, dependency order, activation | jig's core |
| plan | the task tree |
| cut | the goal's first run, then its parts' first runs |
| answer call | a fresh run of a part's designer, on an escalation |
| foreman | nothing: kept, Accept by `## Left`, options in the spec |
| refusal foreman, rebase run | a resolution |
| spec `## Answers` | a spec amendment |
| `notes/`, plan files | a handed branch lands them on the goal's branch; specs are task words |
| trunk | a goal's own branch, a landing branch under a rule with no gates |
| light merge | a landing by turn with nothing to prove |
| early start | a change that starts kept, its workspace with its dependencies' heads merged |
| mid check | a tier following the goal's branch, with a cadence |
| repair-n | a change into the goal's branch, made by code (forge.md, 9's shape) |
| catch-up | an update of the landing change, on a cadence |
| catch-up-n | a resolution as a change into the goal's branch |
| old-form recount | the project's prepare, on a job |
| end check | the gate at the candidate |
| ready, queue | end of the keep, then landing by turn |
| third red, plan.stopped | a hold, escalated to the goal |
| retire | the goal's last run, a change into main |
| run report `Done:`/`Question:`/`Failed:` | a change result, a typed "needs a decision", a failure |

## 4. Decisions taken in the drafts, and why

- **G above parts and L.** L must not go ready while parts of the plan are still unwritten. Dependency on every part gives that with no new mechanism, since a task ends after its delegates (jig tasks 5.6). L cannot need its own parent (a circle), so the top is G, a real judgement run (the thin cut), and parts are delegates. Family designers as delegates also fit the user's 11-01 word on efforts: charter main Opus 5.5 at xhigh, sub-agents (surveys) Opus 5.5 at high, since smith names a sub-agent's model once, so two efforts of one model cannot both be sub-agents'.
- **Questions by hold,** as agreed: the change still asks no one.
- **Red on the branch makes its repair in code,** the user's 11-01 pose: LLMs for judgement, code for mechanisms. fab's tick does the same.
- **A resolution or repair during production is a change into the branch,** as fab's catch-up-n and repair-n, so landings into the branch never wait for an agent.
- **By turn at both levels,** not queue: gates dispatch-only, so no CI on every task's pull; tier and gate share keys; built on Forgejo.

## 5. The after-pass against the code

Done 2026-11-01 by one independent reviewer; the design-changing findings were
read in the code before applying them. Where it changed the drafts:

- Designers cannot write: `agent.md`, 4.3 gives a goal's runs inspection alone. First applied by making c0 an ordinary change with a producer; reverted 11-02 on the user's word that the design leads and code may change. A designer writes contracts and skeletons as code, delivered mid-run to its run branch and handed to a change (the rail's own condition: `modify` only with delivery). Recorded as CP-14.
- L cannot make its repairs today: `base_repair` is refused for a task requester (`temper-engine-domain/src/route/forge_route.rs:1786`), and a change's kinds name no procedure (`:2152`).
- Every prepare counts toward `updates`, which holds at three (`temper-engine-domain-forge-change/src/domain.rs:901`), so a busy goal branch would wake a designer.
- Hold words reach the immediate requester alone (`temper-engine-domain/src/route/forge_route/holds.rs:26`); past a procedure a task reads only that a decision is needed.
- A change's 6 tasks and depth 2 are fixed in the code (`forge_route.rs:2167`), not settings.
- Amending a dependency onto a held change is built, so the old CP-11 went and later ids moved down one.
- A run outside a change starts at the default branch (`temper-engine-domain-forge/src/items.rs:101`), so a designer's checkout at B is a gap.

Not traced: L leaving a live landing when a dependency is added (`landing_route.rs:157` guards the start only). It is LD-10, unverified beyond that.
