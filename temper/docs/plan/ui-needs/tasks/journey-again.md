+++
id = "journey-again"
part = "needs-frame"
kind = "feature"
needs = ["needs-frame"]
files = [
  "temper/docs/design/client/journeys/welcome-playground/**",
]
effort = "medium"
minutes = 40
+++

# `welcome-playground`, recorded again

## Goal

The standing watch (`needs-frame`) changed what the client asks where a project's frame shows, so
`welcome-playground`'s recording is stale. Record it again, so that the journeys' test is green. The
pages themselves change only by the rail's "Needs you" link.

Acceptance:

- `heavy cargo run -p temper-client-journeys -- again welcome-playground` has recorded it, on your
  builder, on `temper box --fake-backends` as `temper/docs/design/client/journeys.md`, 8.1 says, and
  `heavy cargo run -p temper-client-journeys` then exits 0.
- `the_committed_journeys_are_their_print` passes (`temper/tools/journeys/src/tests.rs`).
- Any other journey `needs-frame`'s report names as stale is recorded the same way.

## Inputs

- `temper/docs/plan/ui-needs/notes/common.md`.
- `temper/docs/design/client/journeys.md`, 8 and 8.1.

## Owns and may change

Owns `files`, and any other journey's directory the reprint names as stale. May change
`temper/docs/design/client/journeys.status.md` for what the new recording changes there.

Options, each with its default:

- `again` refuses the journey (a `drawing` act, a mark that caught the page moving): report `Failed:`
  with its words, committing nothing. Don't walk it anew; the plan goes on without this task, and the
  trunk's end check will show the red.
- A journey's pages other than the rail's link and the new times and ports differ: say what in the
  report. They are what the view prints, not yours to change.
- If the walk stops on the sign-in fault `temper/docs/design/client/journeys.status.md` names, stop as
  in the option above.

## Steps

1. Run `again`, reading its log on any failure, as `AGENTS.md`, 5 says.
2. The reprint; the journeys' tests: `heavy cargo nextest run -p temper-client-journeys`.
3. Check by `git diff --stat` that only the journeys' directories moved. Commit and push.

## Report

As the implementer's text asks, with the builder, and what differs on the pages.
