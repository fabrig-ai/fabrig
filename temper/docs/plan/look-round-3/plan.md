# look-round-3: the plan

From `inputs/request.md`: land the look request `project-home`, `0197eaea31` on
`origin/look-requests`, by `temper/docs/design/draft/look-requests.md`, sections 3 to 5, with the
two draft changes it needs: Ready adopts the first repository, and offers its name. The trunk is
`look-round-3/integration`, so it never meets the person's branch.

The design is cut ahead of the tasks this time, as the request asks: the first push holds
`temper/docs/design/client/README.md`, 3.5 (*The welcome*, *Home*, *Settings*) and section 11,
`protocol/setup.md`, 6.7, `client/forge.md`, 5.4, and the mockups' status.

## Parts and tasks

Written so far; later pushes add the rest.

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `ready-name` | | medium, 60 | Ready's offered name |
| 2 | `project-addresses` | | high, 75 | `Home` and `Settings`: paths, opening their project, plain pages |

Still to write: Ready's adoption and its landing on Home; Settings; the rail, the switcher and the
tab bar; Home's facts; Home's column; the reprint and the recording; the account.

## Critical path

`temper-client-view` takes one writer at a time, so its tasks chain.
