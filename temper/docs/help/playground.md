# The playground

Kind: concept

The playground is temper's demo: a repository on a fake forge and a scripted model, started with one call, so that a chat and its change run through real tasks with no account of yours.

## What it is

- **Its parts:** a fake Forgejo, with the repository
  `playground/widget`; provider 3, a scripted model, with a key the
  deployment holds; the project `Playground`, which adopts the
  repository. `temper init` writes the model and the fake forge's
  place, `application.machine.fake`, into the configuration; the
  forge, the account and the project are made when the playground
  starts.
- **What is real:** the engine, the worker and the agents' processes;
  the forge, its git and the model are fakes.
- **The scripted model** answers three requests, as typed: `Add CSV
  export to widget` delegates a change to the repository;
  `Add a dark theme to widget` and `Fix widget's empty search state`
  each explain that nothing changes. Any other request is answered that
  it is not one of these, and that other work needs a connected model.
  Its account is used by the project `Playground` alone.
- **Under a box,** the fakes run as one of its groups of child
  processes; elsewhere, the deployment must reach fakes of its own.

## How it changes

- **Started:** `playground start` makes the fake forge, numbered as
  any forge, and starts it; adds the model's key as a deployment
  account; makes the project `Playground`; waits for the forge; and
  adopts `playground/widget` into the project. It prints `running`
  once the playground runs.
- **Running:** a chat in `Playground` asks for one of the three
  requests (temper help chat). A start of the deployment starts the
  fakes again and keeps the account and the adoption.
- **Reset:** `playground end`, or "set up temper" on the client's
  banner, deletes the project `Playground` with its chats and tasks,
  removes the fake forge, and removes the scripted account. The
  deployment is then as it was before the start, and the client opens
  its welcome at the Forge step. It prints `reset`.

## What temper guarantees

- **One call, again until done:** a start that fails prints the step
  that failed, and the same call again finishes what is not done,
  never repeating what is.
- **No account of yours is used:** its model is the scripted one, on
  the deployment's own key.
- **Its work is real work:** its chats, tasks and change have the
  states, acts and reads of any other (temper help task).
- **A reset ends whole:** one a restart cut goes on at the next start.

## Acting on it

- **ctl:** `playground start` starts it, as a party with a role in the
  deployment; `chat start` in `Playground` asks it for a change; `task
  show` and `change show` follow the work; `playground end` resets it.
- `forge setup` can make the same repository, `playground/widget`, on
  the owned Forgejo of a box (temper help forge).

## Limits

- `refused: not offered` (exit 2): the deployment has no place for the
  fake forge, or keeps a forge of its own: the playground is offered
  only before one is set up or connected.
- A failed start exits 1, naming its step: `the fakes did not start
  (REASON)`, `the account refused`, `the adoption refused` or `the
  forge not connected in time`; make the same call again.
- A reset keeps nothing of the playground's work: its chats and tasks
  are erased.
- The scripted model does no work beyond its three requests.
