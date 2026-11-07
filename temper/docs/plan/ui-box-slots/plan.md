# ui-box-slots: the plan

The request is `inputs/request.md`. It allows two tasks at most: focused and fuzzy alone, with new cases as rows of existing tables.

## Parts and tasks

| Part | Task | Needs | State |
|---|---|---|---|
| `box-words` | `box-words`: the request's sections 2 and 3 | none | released |
| `box-slots` | `box-slots`: the request's section 1 | none | being written |

Critical path: none. The two tasks share no file and neither needs the other.

## What was decided

### A box's words

The view can already tell a box from `temper serve`. The deployment's read, `DeploymentAnswer.owned`, is true only where the document has `application.machine.owned` and the engine runs under the box (`temper-service/src/system.rs`, `deployment_owned`). So no wire field is added.

`temper box` on a document with no `owned` part reads false and keeps today's words. `temper init`'s box document has the part. That's written down as an open question in `temper/docs/design/client/README.md`, with its default.

### A failed call's answer

A new `Problem::Needs { kind, member }` follows the commit identifier's precedent, in that the answer says what to do. Both members a read's kind can need get it: `number` and `project`.

A kind no setting has, with no `number`, used to answer "member missing: number". It now answers out of range for `kind`, which is what it was already answered when it had a `number`.

The `propose_settings` form reader isn't changed: it has its own words, and `model-catalogue-2/configuring-learns` is on that crate.

No shape crosses between the two tasks, so there is no `notes/contracts.md`.

## Overlaps that could not be avoided

- `temper-engine-smith/src/settings.rs` and `calls.rs`: `model-catalogue-2/configuring-learns`.
- `temper-engine-smith/src/answers.rs`: `ui-proposal-2/chat-revises`, one compiler-forced arm.
- `temper-client-view/src/words.rs`: `limits-fixtures/view-words`.
- `temper/docs/design/client/README.status.md`: `model-catalogue-2/models-proof`, `look-round-13/look-proof`, `ui-proposal-2`.
