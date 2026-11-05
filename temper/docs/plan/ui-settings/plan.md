# ui-settings: the plan

From `inputs/request.md`: Settings' frame, How work runs and Repositories, as the greenfield mock draws them.
This is the first push: the engine. The client's tasks follow in the next batches.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `project-templates` | | medium, 25 | jig's `Core::project_templates` |
| 2 | `ways-kind` | 1 | high, 150 | kinds 6 and 7, the route, a project's read |

## What the cut settled, so far

- **Kind 6 is built here.** The request reads it as live, and 13.2 designs it, but the forge declares kinds 1 to 3
  alone: `ways-kind` makes it a setting, and the `change` that names no route takes it.
- **Kinds 6 and 7 have no stock row.** A stock default names one scope and a project made later has its own number,
  so their stock is a constant of the code, read when no value is stored (`configuration.md`, 13.4).
- **The reference's kind 7 is a file of its own, `docs/examples/reference/project.json`, not `settings.json`** as the
  request says: `settings.json` is applied before the case's project exists, and jig refuses a project's item then.
  Its `way` is 4, the reference's goal.
- **Formats that change:** temper's store family 20 and the client wire gain the two values, in place. Existing
  stores keep decoding: an enum gains variants. No compatibility is kept beyond that.
