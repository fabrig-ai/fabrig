# ui-goal-page: the plan

From `inputs/request.md`: a goal's page, its acts and Start goal, drawn as the greenfield mock's `goal.html`.

The trunk was made from `ui-goals`' and carries its unlanded commits; neither `ui-goals` nor `ui-needs` has
landed. So the plan's option `after = ["ui-goals", "ui-needs"]` holds this trunk's end check and landing until
they have, and task `needs-base` merges `ui-needs`' integration into this trunk, so the work needn't wait. The
cut added `after` itself: fab's trunk was made as the option's text describes, without the option.

## Parts and tasks so far

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `needs-base` | | high, 75 | `ui-needs`' merged code on this trunk |
| 2 | `made` | | medium, 60 | `Task.made`, from the record through the wire to the client |

The design and the page's tasks follow in the next batches.

## Critical path, as it stands

`needs-base` (or `made`), then the page's chain.

## Overlaps with the drafts beside this one

- `needs-base` meets `ui-goals`' and `ui-needs`' own lines: the two conflict in about twenty files, one to three
  hunks each. Whichever of them lands second meets the same conflicts on its catch-up; this trunk's next catch-up
  then settles any difference between the two resolutions.
