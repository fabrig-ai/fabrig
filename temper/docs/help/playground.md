# The playground

Kind: concept

The playground is temper with nothing of yours: a repository on an in-memory forge and a scripted model, made when you choose it and removed when you leave it, so a chat runs through real tasks.

## What it is

- **Its parts:** the playground's forge, a fake Forgejo held in
  memory, with the repository `playground/widget`; the scripted
  model, an account the deployment holds; and the project
  `Playground`, which adopts the repository. They are made when the
  playground starts, and removed when it is reset.
- **What is real:** the engine, the worker and the agents' processes;
  the forge, its git and the model are fakes.
- **The scripted model** answers three requests, as typed: `Add CSV
  export to widget` delegates a change to the repository;
  `Add a dark theme to widget` and `Fix widget's empty search state`
  each explain that nothing changes. Any other request is answered
  that it is not one of these. Only the playground's project uses
  it, and the playground's project uses no other model.
- **Under a box,** the fakes run as one of its groups of child
  processes; elsewhere, the deployment must reach fakes of its own.

## How it changes

- **Started:** `playground start` makes the forge, the account and
  the project, waits for the forge, and adopts `playground/widget`.
  It prints `running` once the playground runs. It is offered while
  no forge of yours is kept.
- **Running:** a chat in the project `Playground` asks for one of the
  three requests (temper help chat). Every page says it is the
  playground, and offers nothing to add.
- **Reset:** `playground reset`, or "set up temper" on any page's
  banner, deletes the project with its chats and tasks, removes the
  forge and the scripted account, and leaves temper as it was before
  the playground started: the welcome opens at its Forge step. A
  reset cut by a restart goes on at the next start, and so does a
  playground the deployment finds running when it starts, since the
  in-memory forge kept nothing.

## What temper guarantees

- **One call, again until done:** a start or a reset that fails
  prints the step that failed, and the same call again finishes what
  is not done, never repeating what is.
- **No account of yours is used, nor changed:** the playground's
  project runs on the scripted model alone, and an account you added
  stays when the playground is reset.
- **Its work is real work:** its chats, tasks and change have the
  states, acts and reads of any other (temper help task), until the
  reset erases them.

## Acting on it

- **ctl:** `playground start` starts it and `playground reset` resets
  it, as a party with a role in the deployment; `chat start` in the
  project `Playground` asks it for a change; `task show` and `change
  show` follow the work.
- `forge setup` makes the same repository, `playground/widget`, on
  the owned Forgejo of a box (temper help forge).

## Limits

- `refused: not offered` (exit 2): the deployment has no playground,
  or a forge of yours is kept.
- A failed start or reset exits 1, naming its step; make the same
  call again.
- The scripted model does no work beyond its three requests.
