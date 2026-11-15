# A requester reaches its change's live producer

Draft, 2026-11-15, on the user's word. temper/dev builds it, after `kept-change-answers.md` has landed. On 11-14, a goal's change passed up its producer's question. The goal answered "hold your saved work; when the other change lands on g, I'll message you", and the producer parked. When the other change landed, nothing woke the goal, and once a person had nudged it, it had no way left to reach the producer. Its words to the change were answered "Words do not answer". A second answer to the same question was refused "no reference". An amendment of the change's spec never reached the producer. The producer sat parked for 47 minutes, until a person messaged it directly (P49).

The designed path would have worked: name the existing change as a prerequisite in the asking change's `after`, then answer. The answer then waits for the prerequisite to land, and the producer's next run starts from g's tip with that change merged in (`temper/docs/design/domain/forge/change/questions.md`, 1, *Early starts*). The charter only says to make a new prerequisite, so the goal promised a message instead. Two engine gaps then left the promise with no way to be kept.

## 1. The charter names an existing prerequisite, and never promises

- **Today:** the stock coordinator charter (`temper/docs/charters/stock/coordinator.md`, *A question a change passes up*) says: "A prerequisite the answer needs: make it, add it to the asking change's whole `after`, and answer that it waits for it."
- **Wanted:** the line covers a prerequisite that already exists, a sibling change of the goal's included. Use the change that carries it, or make one. Add it to the asking change's whole `after` before answering, and answer that it waits for it. Never answer with a promise to send word later as a stand-in for a dependency. Keep the charter's other lines, and its assembly and size tests, as they are.

## 2. An amendment reaches the change's live producer

- **Today:** jig commits a change's amended spec and sends the change procedure an `Amendment` message with the reason (jig `docs/design/domain/tasks/changes.md`). The change's relay skips `Amendment` (`temper/crates/temper-engine-domain/src/route/forge_route/relay.rs`, `take`). The producer works from the spec it was given when it was made (`temper/crates/temper-engine-domain/src/route/forge_route.rs`, `change_delegate` and `production_words`), so a running or parked producer goes on to the old words, across attempts, until it ends.
- **Wanted:**
  - When a change's spec is amended while it has a live producer, running or parked, the producer is told, with the amendment's reason and the change's new revision, and its next attempt works to the amended words.
  - Prefer the change amending its producer's spec with the words it would give a new producer. jig's amendment then tells the producer as it tells any executor, and its next brief reads the new spec. If a procedure can't amend its delegate within jig's rules, relay the reason as words naming the revision and how to read it, and say why in the status file.
  - An amendment never answers or clears a question.

## 3. A requester's words reach the change's live producer

- **Today:** a change answers its requester's words with fallback feedback ("Relays questions … Words do not answer"), and the words go no further (`relay.rs`, `take`; `questions.md`, 1). The design deliberately relays only questions up and answers down.
- **Wanted:**
  - A change with a live producer passes its requester's words to that producer as words, naming who sent them, and tells the requester they were passed.
  - With no live producer, the requester gets today's fallback.
  - Words never answer or clear a question. A producer kept by its own open question stays kept, and the change's reply to its requester says which question still waits.
  - Passing words is durable like the answer relay: kept on the change's record through a step with no message room, a refused step and a restart, and delivered once.
  - Revise `questions.md`, 1, and `temper/docs/design/domain/forge/change/README.md` where they say what a change does with its requester's words and amendments.

## 4. Out

- A goal's wake on its own change landing into g. That landing news is `Kept` and doesn't wake a goal, by design (`temper/docs/design/domain/forge/README.md`, 5.3). A goal that must hear of it subscribes to the change, as the charter already says.
- Answers to a kept change: `kept-change-answers.md`.
- A second answer to an answered question stays refused.

## 5. Tests

Cheap: focused and fuzzy suites only; walk = false.
- Focused stories in temper's engine domain tests, near `temper/tests/engine/domain/tests/suite/forge_routes/questions.rs` and `goal_questions.rs`:
  1. A change's producer asks. Its requester adds an existing sibling change to the asking change's `after`, then answers. The answer reaches the producer only once the sibling has landed, and the producer's next run starts from the branch tip containing it.
  2. After its question is answered, the producer parks. The requester's words to the change reach the same producer and wake it.
  3. The requester amends the change's spec while the producer is running, and again while it is parked. The producer is told each time, and its next attempt works to the amended words.
  4. Words sent while the producer's question is still open leave the question open, and the requester is told which question waits.
  5. Words passed with no message room, then across a restart, are delivered exactly once.
- Keep the early-start checkout assertions (`temper/crates/temper-engine-domain-forge/src/tests/early.rs`).
- The charter's assembly tests (`temper/crates/temper-charter-set/tests/assembly.rs`) pass with the revised line.

Run only the new and affected tests, by name.
