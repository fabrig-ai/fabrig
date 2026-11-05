# ui-home-status: the plan

From `inputs/request.md`: Home on any project reads as the greenfield mock's, with what changed since the
person last looked, the landing's line, and the status agent's words. The design is cut ahead of the tasks:
`temper/docs/design/domain/forge.md`, 5.6 and 5.7; `protocol/store.md`, 2 and 3; `client/README.md`, 3.5, 4.6, 8 and 11.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `status-wire` | | medium, 40 | `Looked`, `Watch::Status`, `ProjectStatus` on the wire |
| 2 | `project-facts` | | high, 90 | the families, the count, the look and the document in the forge |

More follow in later pushes: the root and the service, the client's watch and look, the view, the charter, the run.

## Critical path

`project-facts`, then what follows it. To be filled as batches land.

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

## Overlaps with the drafts beside this one

- `temper-client-view/src/pages/home.rs`: `ui-needs` rewrites most of its needs section, and `ui-chats`, `ui-settings` and `ui-welcome` touch it too. Nothing of this plan's touches it until the engine's half is done; then one task at a time holds it, and its changes keep to the intro and the foot.
- `client-wire-v1.schema`, `store-v11.schema`, `temper-engine-domain-forge/src/lib.rs`: `ui-welcome` and `ui-settings` add to them. Unavoidable: a catch-up regenerates the generated code rather than merging it.
- `temper/docs/design/client/README.status.md`: `ui-needs/needs-proof` and `ui-goal-page/goal-proof` edit it; this plan's lines are its own.
