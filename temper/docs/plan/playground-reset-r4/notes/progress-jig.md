# Progress: the jig designer

## Specs

final: tasks/jig-acts.md
final: tasks/pd-store.md
final: tasks/pd-cancel.md
final: tasks/pd-sweep.md
final: tasks/pd-wire.md
final: tasks/pd-people-gone.md
final: tasks/pd-close.md
final: tasks/pd-erase-task.md
final: tasks/pd-project-rows.md
final: tasks/pd-delete.md
final: tasks/pd-client.md
final: tasks/pd-conformance.md

## Parts

- `project-deletion`: a project deleted whole, jig's (draft 5; jig's `domain/people.md` 5.2-5.3, `domain/tasks.md` section 7 and its retention class, `protocol/store.md` 11.1).

## Critical path

Minutes from the run's start, each task starting when its `needs` have merged:

- jig-acts 50 -> pd-store 110 -> pd-cancel 145; pd-people-gone 160; pd-wire 110 (beside pd-store)
- pd-close (needs pd-cancel, pd-wire, pd-people-gone) 160 -> 220
- pd-sweep (after jig-acts) 110 -> pd-erase-task (needs pd-sweep, pd-store, pd-people-gone) 160 -> 215
- pd-project-rows (needs pd-close, pd-erase-task) 220 -> 270
- pd-delete 270 -> 330 (the fixed id: a deletion whole from here; the playground's reset `needs` it)
- pd-conformance 330 -> 385
- pd-client (needs pd-wire) 110 -> 150, off the path

Critical chain: jig-acts, pd-store, pd-cancel/pd-people-gone, pd-close, pd-project-rows, pd-delete, pd-conformance: about 385 implementer-minutes.

## Decisions

- jig-acts refuses the new request and acts *not offered* at the people child's entrance (unsaved, people.md 5.3), so no new value reaches jig's store before `pd-store` moves it once (D2).
- `RemoveProject` (request and act) answers a new `Outcome::ProjectRemoved { project }`; `Act::Cancel` answers as `Close` does (the compound's outcome).
- The answer is given at the deletion's decision, not at its end: temper's kept `RemoveProject` is answered once the project is closed, while its erasure goes on; a resend after the deletion is whole answers *unknown* (its retained answer is erased at the end), which D4 takes as done.
- jig's store moves in `pd-store` alone: people v13 to v14, core v6 to v7; tasks stays v9. The client wire is appended in place to client-v19 (no version move), as 235df29ae0 did; D2 binds the store only.
- The kept deletion is a core family, `Removal { project, stage: Ending | Erasing | Releasing }`; its cursors and the connectors' request tokens are volatile, and each stage begins again at a start.
- Erasure is core-driven: the core loads typed pages of rows (`reads::Sweep`: `Ended { after }`, `Task(task)`, `Notes { project }`; the store's `Load::Sweep`) and erases them by key in its own decisions; transcripts go by a new core `Write::DropTranscript`. An ended task's `Ended` record is erased last, so a restart finds it again and finishes. skein-kv has no range erase, and an ended task's rows are not all in memory or named by its record.
- Closed at once means more than the draft's order says: at the decision the project leaves `Core.projects`, its name is free, its authority policy is dropped, and people erase its roles and policy copy (`ProjectClosed`). The draft lists the policy copy and roles among the rows erased last; doing it at once is what makes every request naming the project fail as unknown, and the number is never reused.
- The deletion's cancels give no grace (draft 5's "with its grace" read as the cancel's own rule; a grace would only delay the reset).
- Spend kept in totals (draft 5, D5): jig has no deployment or party total apart from the project (s3, 2). The project's current period and pool ledgers are closed into `ClosedLedger` rows and kept; per-task ledgers are erased (draft: "never per task"). The conformance check exempts `ClosedLedger` rows of the project.
- Deleting a project is the project's owners' (`people::Role::Owner` in it), never a policy permission; the deployment's by the `"remove_project"` grant.
- An account whose uses name the project drops that use; one left with none keeps `Some([])` and is not removed.
- jig has no settings of a project's scope (s3, 4): nothing to erase there; connectors erase their own kinds on `ReleaseProject`.
- `pd-sweep`, like jig-acts, changes temper's root and service at their call sites (`CoreRange::Sweep`, the new core write), so a deletion reaches temper's store.
- Words the lead may want in jig's designs (no test reads them, so no docs task): `protocol/store.md` 4.4 (the core's `Removal` family; `Load::Sweep`; the core's transcript drop) and 11.1 (an ended task erased whole at once: the mechanism, no horizon); `domain/people.md` 5.1-5.3 (`RemoveProject`, the acts table's `RemoveProject` and `Cancel` rows, retained answers naming a deleted project erased); `domain/tasks.md` (erasure at once); `domain/connectors.md` (`ReleaseProject`/`ProjectReleased`); `protocol/configuration.md` 3 (the grants `remove_project`, `cancel`).

## Contract problems

None. (Contracts section 4 holds as cut; D5's spend is met by keeping the project's closed ledgers, under Decisions.)

done
