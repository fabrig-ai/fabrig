# playground-decisions: contracts

The lead's binding calls on the shapes that cross trees or tasks. A task builds to them; a
change to one is a question.

## C1. A removed forge's repositories' names, kept while a kept task names them (decision 2)

The rule is `temper/docs/design/domain/forge.md`, 4.1 ("each of its repositories' rows once no
kept task names it"). Today the forge connector drops a task's `Names` row at its release, and
nothing tells any connector when a task is erased.

- **jig tells every connector when a task is erased:** the core asks each numbered connector
  `connector::Ask::Unname { task }` (`jig/crates/jig-domain-core/src/effects/connector.rs`, its
  shape unchanged, sent by nothing today) in the decision that erases the task whole: a deleted
  project's erase walk (`routing/task/erase.rs`), a task forgotten once read
  (`reads/forget.rs`), and, once built, retention at the horizon. Its meaning: the task is
  erased, so nothing it named need be kept for it. Applications that keep nothing for an ended
  task ignore it, as jig's test domain and the ops example do today.
- **temper's root maps it:** `Ask::Unname { task }` becomes `forge::Event::Unnamed { task }`
  (`temper/crates/temper-engine-domain/src/route/forge_route.rs`, where it is ignored today).
  `Event::Unnamed`'s meaning becomes "the task is erased".
- **temper's forge connector keeps a released task's names:** at a task's release its watches,
  holds and subscriptions go as today, and its `Names` row stays, as the names of a kept task, in
  the same stored record and key (`Stored::Names`, `Key::Names(task)`; no schema change), until
  `Event::Unnamed { task }` erases it. A kept row holds no watch, is never revived by `Rename`,
  and is read by `erase_removed_repositories`, so a removed forge's repository row stays while
  any live or kept task names it. A start restores kept rows: reconcile accepts a `Names` row of
  a task jig does not hold live.
- **Its bound** is the designer's (C1 leaves it open): a kept row must not take a live task's
  room under `Limits::tasks`.

## C2. A deleted project's conversations, cancelled with their close's grace (decision 3)

The rule is jig's `jig/docs/design/domain/people.md`, 5.2 ("a conversation with its grace") and
the user's decision: the grace is the chat template's, as
`jig/docs/design/client/domain/objects.md`'s Close gives it; the engine reads no template, so
the request carries it, as a close's `Cancel` does.

- **`RemoveProject` carries a grace:** `RemoveProject { project, grace: Option<Duration> }`
  on jig's client wire (the `RemoveProject` record of `client-v19.schema`, in place), in the
  client contract's `Ask::RemoveProject`, the people child's `Ask::RemoveProject`, and the
  deployment's act `Act::RemoveProject` (people.md, 5.3, whose subject gains "a grace within the
  tasks child's bound, or none", as `Close`'s has). A grace past the tasks child's bound is
  refused at admission, as a cancel's is.
- **The deletion keeps it:** the deletion's row (`StoredDeletion { project, step }`, people's
  store schema, in place) gains `grace`, so a deletion a restart cut cancels its remaining
  conversations with it.
- **The cancel step:** each live conversation of the project is cancelled as the deployment's
  `Close` composes one: each live delegate cancelled, then the conversation cancelled with the
  grace (with a fresh message number, as `Close` takes one); every other live root with its tree,
  with no grace; a root already finishing under a grace is not asked again.
- **Clients fill it from `me`:** the chat template's grace, as `shapes::close_chat` fills a
  close's (`jig/crates/jig-client/src/shapes.rs`).
- **temper's playground reset gives its chat template's grace:** the forge connector's
  `ResetAct::RemoveProject { project }` stays as it is; temper's root, translating it into the
  deployment's act (`temper/crates/temper-engine-domain/src/route/playground.rs`), fills `grace`
  from its chat template's current value, as it reads template 2 for a goal
  (`route/goal.rs`). The application reads its template; jig's core never does.
