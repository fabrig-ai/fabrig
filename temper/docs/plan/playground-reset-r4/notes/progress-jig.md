# Progress: the jig designer

## Specs

final: tasks/jig-acts.md
final: tasks/pd-store.md
final: tasks/pd-cancel.md

## Parts

- `project-deletion`: a project deleted whole, jig's (draft 5; jig's `domain/people.md` 5.2-5.3, `domain/tasks.md` section 7 and its retention class, `protocol/store.md` 11.1).

## Critical path

(in progress)

## Decisions

- jig-acts refuses the new request and acts *not offered* at the people child's entrance (unsaved, people.md 5.3), so no new value reaches jig's store before `pd-store` moves it once (D2).
- `RemoveProject` (request and act) answers a new `Outcome::ProjectRemoved { project }`; `Act::Cancel` answers as `Close` does (the compound's outcome).

## Contract problems
