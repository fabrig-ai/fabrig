# ui-home-status: the plan

From `inputs/request.md`: Home on any project reads as the greenfield mock's, with what changed since the
person last looked, the landing's line, and the status agent's words. The design is cut ahead of the tasks:
`temper/docs/design/domain/forge.md`, 5.6 and 5.7; `protocol/store.md`, 2 and 3; `client/README.md`, 3.5, 4.6, 8 and 11.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `status-wire` | | medium, 40 | `Looked`, `Watch::Status`, `ProjectStatus` on the wire |
| 2 | `project-facts` | | high, 90 | the families, the count, the look and the document in the forge |
| 3 | `status-charter` | | medium, 60 | stock charter 9 and its host-part figures |
| 4 | `status-read` | 1, 2 | high, 60 | the root and the service carry the look and the document |
| 5 | `home-domain` | 1 | high, 75 | Home watches `status` and looks once |
| 6 | `home-walk` | 4, 5 | high, 60 | the meeting point: the real roots, a fake clock past 30 minutes |
| 7 | `home-words` | 5 | high, 90 | the kicker, the headline, the foot, Chats' sentence, five gallery states |
| 8 | `home-goals` | 7 | high, 60 | Goals in Chats' place |
| 9 | `status-run` | 2, 3, 4 | high, 90 | the agent's run: made, delegated, heard, saved |
| 10 | `home-brief` | 8 | medium, 45 | the lede, the byline, the brief's sentence, `home-morning` |
| 11 | `home-proof` | 6, 9, 10 | high, 90 | Home beside the mock at three widths, a real walk with the fake box |

The request's five tasks map so: its 1 is the design this cut wrote; its 2 is tasks 1, 2 and 4; its 3 is 5, 7 and 8 (Goals' section is in the request's section 2, and needed a task of its own); its 4 is 3 and 9; its 5 is 10. Task 6 and 11 are its proof. They are fanned out where the code allows, not one chain: a task waits only for the code it builds on.

The computed Home (tasks 1, 2, 4, 5, 7) is whole without the agent, and is the part the request calls worth landing alone.

## Critical path

`status-wire`, `home-domain`, `home-words`, `home-goals`, `home-brief`, `home-proof`: ~400 implementer-minutes, then the trunk's end check. `project-facts`, `status-read`, `status-run`, `home-proof` is ~330 beside it. Tasks 1, 2 and 3 start at once. The view, the stylesheet, the gallery and the terminal's goldens take one writer at a time, 7, 8, 10, then 11.

## What the cut settled, for the user to see

- **A change of stored format and protocol:** families 36 `ProjectFacts` and 37 `Looked`; the wire's `Looked`, `Watch::Status`, `ProjectStatus`. No compatibility is kept.
- **`next_seen`, a field the request doesn't name.** The request defines `seen` as the count at `previous`, then says a new sitting sets it to the current `landed`, which would zero the count just as the person returns, so Home would never say what landed, and the request's own walk couldn't pass. The definition wins; `Looked` keeps the count at `latest` beside it (`domain/forge.md`, 5.6).
- **This branch merged main first.** The plan's trunk began at `ui-goals`' integration, without main's `ui-changes`. Main holds the landing log, `views/changes.rs`, `Address::Changes` and the landing's line, which this plan reuses. Without it, the work would be done twice.
- **The status agent is stock charter 9, not 7.** The reference deployment's settings already number its designer 7 and its triage 8, so a stock 7 would run the designer's words there.
- **A status run is kept, not "forgotten once read".** jig forgets a result only for a person's request (`reads/forget.rs`), and this task is the deployment's. It is kept as a tier run's is, at most one a sitting and an hour; `domain/forge.md`, section 20, says so.
- **It is a procedure that delegates charter 9, not a parentless agent.** jig's connector task takes the project's whole ceiling as its authority and can't narrow it; a delegate can, as a foreman's is. An agent reading change titles shouldn't hold merge authority. So a project whose policy's delegation doesn't name charter 9 has no brief, and Home keeps the computed words.
- **"No tools but finish" holds by authority.** The host part declares its fixed tools whatever the charter says (`protocol/configuration.status.md`, 3); with no tools or grants in its authority, the others are refused.
- **A goal's "landed and total changes" are its delegates done and total,** as `ui-goals`' task line counts them; the forge keeps no per-goal count, so the root adds that part of the words.
- **The foot's "Changes" link is drawn,** since main has the page, as the request's condition has it.
- **`queued` and `red` are the Changes document's,** computed by the same code, so the two pages can't disagree.

- **Things the request's literal words would have broken, each noted above, reach the user through the report:** `next_seen`, charter 9, a kept run, a procedure that delegates.
- **The status agent's `running` is stored,** a field the request doesn't name: "none running" has to survive a restart, and a run in flight past an hour counts as none, so a held run can't stop every later one.
- **The real walk has no fake clock:** the client world's has (`home-walk`), a real box hasn't, so `home-proof` waits 31 real minutes, or does it once by hand for the proof if its tier can't hold that. No budget is raised.
- **`ui-needs` is not on this trunk yet,** though the request says this plan needs it. Home keeps the needs sentence it finds, through one function, so a catch-up takes `ui-needs`' words without touching this plan's.

## Overlaps with the drafts beside this one

- `temper-client-view/src/pages/home.rs`: `ui-needs` rewrites most of its needs section, and `ui-chats`, `ui-settings` and `ui-welcome` touch it too. Nothing of this plan's touches it until the engine's half is done; then one task at a time holds it, and its changes keep to the intro and the foot.
- `client-wire-v1.schema`, `store-v11.schema`, `temper-engine-domain-forge/src/lib.rs`: `ui-welcome` and `ui-settings` add to them. Unavoidable: a catch-up regenerates the generated code rather than merging it.
- `temper/docs/design/client/README.status.md`: `ui-needs/needs-proof` and `ui-goal-page/goal-proof` edit it; this plan's lines are its own.
