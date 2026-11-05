# fake-signin-model: the plan

From `inputs/request.md`: a fresh fake box's ChatGPT sign-in came back to Ready with no model and no
word why, which stopped `look-round-3`'s `record-landing`. This draft finds and fixes that, closes
Ready's silence, and records `project-landing` on Home. `look-round-4` follows it on the same
journey, so it stays small.

The design change is one sentence, in `temper/docs/design/client/README.md`, 3.5: Ready says any
refusal or failure in 5.5's words, with its **Not yet** in `README.status.md`. No contracts cross the
parts, so there is no `notes/contracts.md`.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `signin-model` | | high, 150 | the failing fake-box walk, the fault's fix, Ready's words for every ending |
| 2 | `record-landing` | 1 | high, 75 | `project-landing` on Home, reprinted, with its docs |

## Critical path

`signin-model`, then `record-landing`: 225 implementer-minutes. The second needs the first's merged
code, since the walk can't pass Ready without it.

## Overlaps

- `temper/crates/temper/tests/fake_backends/walk.rs`: `repository-copies-1/copy-walk` changes it too.
  `signin-model` keeps its own walk in a new file and adds three small hunks to this one.
- If the fault is in `temper/crates/temper-fake/src/backends.rs` or
  `temper/crates/temper/src/fake_backends.rs`, it meets `repository-copies-1/fake-box`, merged on that
  trunk. Which file the fix takes isn't known until the failing test says.
- `temper/docs/design/client/journeys/**`: `fetch-progress-1/view-fetched` touches `welcome-*`, and
  `record-landing` only `project-landing`, but for a reprint of `welcome-playground/02.html`.
- `to-begin-store/dismissed-facts` touches `temper-client-domain` tests, `signin-model` only if the
  fault lies in the client's reading.
