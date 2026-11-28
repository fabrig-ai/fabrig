# A project's charters

Provisional, 2026-11-28, from temper's
`temper/docs/design/draft/ways-charters.md`, section 3 (temper's
`temper/docs/design/draft/ways.execution.md`, W3). A project may keep
charters of its own: words that stand, for that project's tasks, in
place of the deployment's charter of the same name, run by a
deployment's charter's settings with its overlay laid over them. The
application gives them as text it has read and judged; the core keeps
them by the project's revision, binds new tasks to them, and composes a
run's instructions from them. A charter's settings are `settings.md`;
an overlay, `overlays.md`; a run's preparation, `executors.md`, 1; a
part and its reasons, `../tasks/messages.md`, 6. What temper puts
through it is temper's `temper/docs/design/domain/agent/project-charters.md`.

## 1. In one page

- **A project's set** (2): the application gives the core, per project,
  numbered sets of charters, each a name, the deployment's charter whose
  settings it runs by, and its words: instructions and parts. No person
  sets one.
- **Held** (3) while it is its project's newest, or a live agent task
  records it; then it goes.
- **Bound by name** (4): a task made in a project, naming a charter
  whose name the project's newest set holds, is made the project's
  charter of that name, its charter the one that charter's settings
  name. Every task records its project's newest set. A creation may ask
  for the deployment's words instead.
- **A run's words** (5): the bound charter's fields, as any task's; its
  instructions and parts the project's, then the deployment's overlay's.
- **Bounds** (6): a set within jig's charter bounds at its entrance; the
  composed charter within them at each binding.
- **Read** (7) as a setting's read naming a project; **stored** (8) in
  the settings family; **no version moves** (9).

## 2. A project's set

- **Given by the application,** in a decision of its own making, as one
  input to the core: the project, the set's `revision`, its `source` and
  its charters. No client request sets one, and no person.
  - `revision`: the application's number for the set, from 1, above
    every set the project was given before.
  - `source`: at most 256 bytes of the application's words saying where
    the set was read (temper names the repository and the commit). Reads
    and a run's record show it (7; `../tasks/lifecycle.md`, 12.1); jig
    never reads it.
  - **its charters,** each a `name` (`settings.md`, 1, *Names*), unique
    in the set; `settings`, the name of the deployment's charter whose
    settings it runs by; its `instructions`, text; and its `parts`, each
    a part as a charter's (`CharterPart`: its reasons and its text). A
    charter names no field: models, tools, hosts, waiting, resume and
    budgets are its settings charter's.
  - A set may hold no charter: its project then binds nothing (4).
- **The project's newest** is the set it was last given. A set whose
  revision isn't above the newest's is refused as stale, and one for a
  project that isn't live as unknown; neither changes anything.
- **Checked at its entrance** against jig's bounds alone (6). Its
  `settings` names aren't judged then: each binding resolves them
  against the newest settings (4), since settings change while a set
  stands.
- **Why text, given:** jig reads no repository. The application reads
  its files, judges them and parses them, and gives jig words in a
  charter's own shape, so the core composes them as it composes an
  overlay, and a host receives assembled text as from any charter.

## 3. Held

- **What holds a set:** being its project's newest, or being recorded
  by a live agent task (4). A procedure's or a person's task records its
  project's newest and holds none, as it holds no settings revision
  (`settings.md`, 3).
- **Dropped** in the decision that leaves it held by nothing: the one
  that ends its last holder, or the one that makes a newer set its
  project's newest. It leaves memory and the store together.
- **Bounded by what holds them:** each live agent task holds one set,
  so the sets held are at most the projects and the live agent tasks,
  each within its entrance's bounds (6). A set costs its records' puts
  when it is taken and their erasures when it is dropped, priced in the
  core's writes bound as charter versions are (`settings.md`, 8).
- **A project's erasure** erases its sets.

## 4. A task's binding

- **Every task made in a project records its project's newest set,**
  its `charters`, the set's revision, or 0 where the project has none,
  as it records the newest settings revision, whatever its parent's.
  A task in no project records 0.
- **Bound by name:** a task made naming charter N, in a project whose
  newest set holds a charter named as N is named at the newest settings,
  is made the project's charter of that name: its charter is M, the
  charter its `settings` names at the newest settings, which is N where
  `settings` names N; and its **scope** is that name. M is the bound
  charter: every check of a creation's charter (offered, its LLM
  choice and speeds against the charter's models, its delegation) judges
  M, as `charter(M)` gives it, and every site that reads a task's
  charter afterwards (the tasks child, the fleet, a run's calls, its
  placement, its finish) reads M's version through `charter_of`. The
  scope changes the words alone (5).
- **Its refusals:** a binding whose `settings` names no charter the
  newest settings offer is refused as not offered, naming the project's
  charter and its `settings`; a composed charter past a bound is refused
  so too (6). Both are refused before authority is checked, as a
  charter not offered is (`settings.md`, 3), so the run or the party
  reads the true cause.
- **Bound again at admission:** a creation whose checks wait on a
  connector's answer (its inputs, its holdings) is bound again when it
  is admitted, against the newest settings and set then. Where the bound
  charter is the one its checks judged, the task is made with the newest
  set and settings; where it differs, the creation is refused as not
  offered, and a new request is checked against the charter it now
  binds. The checks are never replayed against a replacement: a
  connector's answer stands for the charter it was asked for.
- **Unbound:** a creation naming a charter that has no name, or whose
  name the project's newest set lacks, makes its task as before, under
  its charter's own words.
- **The deployment's words, asked:** a creation may ask for the
  deployment's charter whatever its project holds, its charter scope
  *deployment*; the default is *project*. Its task is made unbound. An
  application asks it for a task whose words are its owners' to say:
  temper's floor review (temper's
  `temper/docs/design/domain/agent/project-charters.md`, 5).
- **Kept for the task's life:** its scope and its `charters` never
  change. A move to the newest settings (`settings.md`, 4) moves its
  settings revision alone: its fields and its overlay's text are then
  M's at the newest, its project's words still its set's.

## 5. A run's words

- **A bound task's run** is given its charter version's fields (M's at
  the run's revision, `settings.md`, 3) and the project's words: its
  instructions are the project's charter's instructions, then, where
  M's version holds an overlay whose instructions aren't empty, two line
  feeds and the overlay's, every byte as written, the overlay's alone
  where the project's are empty; its parts are the project's charter's,
  then the overlay's, in order, each selected by its reasons as any part
  is (`overlays.md`, 1). A charter M set whole holds no overlay and adds
  no words.
- **Instructions a task owns,** from an amendment, win as they win over
  any charter's (`../tasks/changes.md`).
- **Nothing else differs:** a resuming charter's run is sent its
  instructions alone, as any; the brief, the documents and the tools
  are M's and the task's.

## 6. Bounds and refusals

- **A set at its entrance:** at most `tasks.charters` charters; each
  charter's words, its instructions and parts counted as a charter's
  are, within `core.run_bytes`, its parts within `tasks.charter_parts`,
  each part's reasons not empty. A set past one is refused whole,
  naming the charter and the bound, and the project's newest stays.
- **A composed charter at each binding:** the charter section 5 composes counts
  as any charter does (`settings.md`, 8): its stored policy and parts
  within `core.run_bytes`, its parts within `tasks.charter_parts`. Past
  either, the creation is refused as not offered, naming the project's
  charter, its set's revision and the bound, until a set, a settings
  change or a limit's change makes it fit. A task already made keeps
  its words.
- **The answer to the application:** a set refused comes back to the
  application as the core's refusal, naming the project, the revision
  and the fault: stale, unknown, or the bound and the charter.

## 7. Reads

- **A charter as a project's new task would read it:** a setting's read
  of a charter (`settings-store.md`, 1;
  `../../client/domain/contract/reads.md`, 1) may name a project. It
  resolves the binding of 4 at the newest settings, as a creation in
  that project would: the read's setting and value are the bound
  charter M's, as its read is today, its overlay beside it
  (`overlays.md`, 6), and beside them the project's charter the task
  would take, whole: its name, its set's revision and `source`, its
  `settings`, its instructions and its parts. Where nothing binds, the
  read is the charter's own, with no project's charter. A read naming a
  project names no revision: it is of the newest.
- **Who may read it:** whoever may read the setting and the project.

## 8. Stored, and at a start

- **A set is kept in jig's settings family,** at keys of its own beside
  a charter's versions (`../../protocol/store/keys.md`, 1.4), keyed by
  project and revision: its `source` and its charters. It is put in the
  decision that takes it and erased in the one that drops it (3). A
  project's newest is the highest revision kept for it.
- **A task's `charters` and scope** are fields of its record, and a
  creation's charter scope a field of what makes it, each added with
  its default: 0, none and *project*.
- **At a start** the core restores its sets with its settings, before
  any connector restores, and drops in the start's settings commit any
  set that is no project's newest and that no live agent task records.

## 9. Upgrades

- **No version moves:** the settings family gains keys and a value
  type, the tasks family's record its two fields, a creation its charter
  scope, a setting's read its project and its answer the project's
  charter, each converted by the plan its fingerprints derive
  (`../../protocol/store/layouts.md`, 2; `../../protocol/client/versions.md`,
  1). A store an earlier build wrote holds no set, so its tasks read as
  recording none, unbound, and run as before.
- **Hosts and workers** see nothing new: a bound run's instructions and
  parts are text, as any charter's.

## 10. Testing

Focused, in the core's and the tasks child's tests; fuzzy where it says.

- **Sets:** taken; stale and unknown refused; past each bound refused
  whole; dropped when no longer newest and unheld, kept while a live
  agent task records it, dropped at that task's end; restored at a
  start, and an unheld one dropped there.
- **Binding:** a task recording the newest set; one bound by name, its
  charter M where `settings` names another; a creation asking the
  deployment's words unbound; a `settings` not offered and a composed
  charter past a bound refused, naming the project's charter; a move to
  the newest settings keeping the project's words.
- **Words:** a bound run's instructions and parts with M's overlay and
  with M set whole, its parts selected by reason, a task's own
  instructions winning.
- **The read:** with a project, bound and unbound.
- **Fuzzy:** sets, creations, ends and restarts in any order: every
  live agent task's set held, no set held that nothing holds, a
  project's newest revision never going back.

## 11. Open questions

- **A piece of work's set:** each new task takes its project's newest;
  a piece of work keeping its starting set for every task it starts,
  until amended, is `temper/docs/design/draft/ways-recipes.md`, 2
  (`temper/docs/design/draft/ways.execution.md`, W4).
- **Composed in place:** a charter made of several of a project's files
  for one recipe, keyed by the recipe and the files
  (`temper/docs/design/draft/ways-charters.md`, 2; W4).
