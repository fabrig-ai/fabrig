# ui-goal-asks: contracts

The run's binding calls; both tasks build to them and neither needs the other's code.

## The person task a goal asks with

`temper/docs/design/domain/agent.md`, 8.2. The goal's call, as charter 1's paragraph spells it:

```json
{"batch":[{
  "executor":{"kind":"person","person":N},
  "spec":{"words":"<the question, in the line form of 8.2>"},
  "contract":{"kind":"report","words":4096},
  "authority":{"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":0},"notes":0}
}]}
```

- In jig's terms: `tasks::Executor::Person(tasks::PersonAddress::Person(N))`, `tasks::Contract::Report { words: 4096 }`, requester `tasks::Party::Task(goal)`.
- In the client's: `contract::PersonTask` with `address: PersonAddress::Party(N)`, `contract: TaskContractDocument::Report { .. }`, `requester: Requester::Task(goal)`.
- **A person task in words** is one addressed to a person (`PersonAddress::Party`) whose contract is a `Report`, whatever its requester. Only that shape reads as a question; a role's, a verdict's and an approval's keep today's card.
- The answer is `contract::AnswerOn::Person { task, revision, answer: contract::PersonAnswer::Words(text) }`. It ends the person task with `tasks::TaskResult::Report { words }`, and the goal's next run has a message of kind `tasks::MessageKind::Result(tasks::ResultKind::Report ..)` with those words, from `tasks::Party::Task(person_task)`. jig already does this; neither task changes jig.

## The permission

`temper/docs/design/domain/agent.md`, 8.2, *Its permission is a role's*. `authority::Executor::Role(r)`: rules' and each project's ceiling name roles 0, 1 and 2; a role's authority names its own alone; a goal a chat calls names none. A world fixture that wants a person-started goal to ask sets `Executor::Role(0)` at those three places for an owner of role 0, as `person_task_configuration` in `temper/tests/client/src/fixtures.rs` does.

## Not changed

`message`, `authorize_tool`, jig's kinds and `asks`: a chat as a sheet goal's requester is not this plan. A goal's `asks` stays set only by a chat's `goal` with `agree`.
