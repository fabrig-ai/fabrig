# Plainer panels: what crosses a task

The run's binding calls. The request is
`temper/docs/plan/plainer-panels/inputs/request.md`.

## The word

- `jig_client::words::WaitingWord::Held`, reached as
  `Word::Waiting(WaitingWord::Held)`: phrase number 124, English `held`, no
  holes. Number 33 stays unused. `held` opens both forms the request leaves
  open, so this word does not choose between them.
- `Word::waiting(&WaitsOn)` maps nothing to it: `WaitsOn` stays as it is, on
  the wire and in the engine.

## The lines of `task show`

Text only. `--json`, its digest and the views keep what they print.

| Person task | Prints |
| --- | --- |
| ended, its result's details naming `chosen_by` | `answered by: PARTY` |
| holding party, live or closing or ended with no `chosen_by` | `taken by: PARTY` |
| closing, no holder | neither line |
| any other, no holder | `taken by: none` |

`PARTY` prints as `acts::party` prints it: `you`, or the other party's words.

| Task | `waiting on:` prints |
| --- | --- |
| `task.hold` is `Some`, `waits_on` is `Nothing` | its hold, in the form the user chose |
| `task.hold` is `Some`, `waits_on` is anything else | what it prints today |
| `task.hold` is `None` | what it prints today |
