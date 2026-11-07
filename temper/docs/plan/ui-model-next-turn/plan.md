# ui-model-next-turn

A started chat's change of model or effort is what its next turn's request uses. The request: `inputs/request.md`.

## The cause, as read

Read from the code, not yet run: the first test of `choice-next-turn` is the proof.

A temper chat is an Open run. After a reply it listens for its charter's waiting time, the stock's 10 seconds, then parks. A model change is an `Amend`, and `jig/docs/design/domain/tasks.md`, section 7 gives a new choice to the next run. So within that span:

- **The amendment is a word.** `MessageKind::Amendment` wakes at once (`jig/crates/jig-domain-tasks/src/wake.rs`, `rule`), so the words "Change next run" are relayed into the listening run, whose old model answers them, and the listening starts again.
- **The person's next words reach the same run,** on the old model. That fits what `look-round-11`'s proof saw.
- **On a parked chat,** the same word wakes a run nobody asked for.

`resumed_model` in smith is not the fault. It does what `smith/docs/design/domain/session.md`, 8.2 says: a run opened on a new model first compacts with the window's own model, then answers on the new one.

## What the plan does

In `jig/docs/design/domain/tasks.md`, section 7, for a task its requester ends:

- An amendment that changes only the LLM choice adds no word: it wakes nothing and is heard by nobody.
- A live run is marked behind its choice, and hears nothing more. A turn under way finishes, and a run left alone parks as ever.
- The next message stops the run, by the path a narrowing uses, and the next run takes the new choice and that message.

## Parts and tasks

One part, `next-turn`, one slice, no walk.

| Task | Tree | Needs | Makes true |
|---|---|---|---|
| `choice-next-turn` | jig | none | The rule, its stored bit, tests in `jig-tasks-world` written first |
| `next-turn-story` | temper's tests | `choice-next-turn` | The provider's requests follow, in `temper-system-world`: said while listening, said after parking, effort alone, a turn under way, and a chat nobody changes |

**Critical path:** `choice-next-turn`, then `next-turn-story`.

## Decided here

- **A stored field, because the cause needs it.** `TaskRecord.llm_changed`, in tasks-v9's `TaskRun`, in place. A live run's models aren't stored anywhere a task can compare: `RunProof` and the fleet's `Assignment` don't hold them. A derived comparison would also stop a run when settings or an account moved, against the request's item 4. Comparing with `chosen_policy` was the alternative, rejected for that.
- **The stop comes with the next message, not with the pick.** A pick during a long reply must not cancel that reply (`a run in flight keeps its own models`). Only a message that arrives for a run behind its choice stops it, and that does cut a turn under way: the core can't tell listening from a turn, and the person has both changed the model and said more. What the turn committed stays, and its unread words go to the next run.
- **One request of the next turn still names the old model after a model change:** the summary, which `session.md`, 8.2 designs. The answer names the new one. An after-landing check of the fake's first request alone would see the old name and read it as this fault, so `tasks.md`, section 7 says so.
- **Effort:** the way from the assignment to the provider's body exists at every link. The fake's `Query` has no effort field, so `next-turn-story` holds it at the assignment and names a test per link. A new run with a new effort alone keeps its window.
- **Across LLM providers, or to a window too small (the request's item 3):** the two documents it names don't say. What smith and jig do stands, as the plain thing, recorded in `temper/docs/design/draft/model-catalogue.status.md`, under **Design departs**. No change.
- **A task keeps the model it has (item 4):** only an amendment marks a run. `a_chat_nobody_changes_keeps_its_run_and_its_model` holds it.
- **Tasks its result ends** are left as they are: the request is about a started chat. Their LLM amendment is heard as any amendment.

## Overlaps

Whichever lands second catches up. None of these is waited for.

- **`look-round-11`:** none in code. Its note "A change applies from the next turn." becomes true with this plan. `temper/docs/design/client/README.md`, 3.5 is left alone: "from the chat's next run" stays true, and `look-round-11` and `look-round-13/look-proof` are editing that file.
- **`model-catalogue-2/smith-limits`:** this plan changes no smith code, so `route.rs` stays theirs. `next-turn-story` declares a second model in the agent's configuration, which they will take away.
- **`model-catalogue-2/process-fixtures`** changes `temper/tests/system/src/**`: unavoidable, and kept to a `mod` line and a small helper edit, with the rest in new files.
- **`model-catalogue-2/catalogue-store` and `model-surface`** change `jig-protocol-store`'s schemas, generated code and goldens: their settings family, this plan's tasks family, and `generated/layouts.rs` shared. On a catch-up, regenerate.
- **`model-catalogue-2/models-proof`** changes `temper/docs/design/draft/model-catalogue.status.md`: this plan adds one entry and touches none of theirs.
