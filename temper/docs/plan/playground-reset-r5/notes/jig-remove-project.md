# Part `jig-remove-project`: a project's deletion in jig

The part's own shapes, binding on its tasks (`jp-*`), inside
`notes/contracts.md` section 1 (which wins where they differ). The draft
is `temper/docs/design/draft/playground-reset.md`, section 5; survey S4
(`notes/surveys/s4-jig-projects.md`) is the map of today's code. Project
numbers are `u32`, as every jig type has them (the contracts' table
writes `u64`; `jp-acts` settled `u32`).

## 1. The deletion's row (core family)

In `jig-domain-core`, a new module `src/removal.rs` (task `jp-row`):

```rust
/// A project's deletion, kept until its last step (draft playground-reset.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Removal { pub project: u32, pub step: Step }

/// Where a deletion stands. Every step is idempotent: a start resumes the
/// kept step from its beginning, and what an earlier run erased is gone.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Step {
    /// Its live tasks cancelled; waiting for each to end.
    Ending,
    /// Its ended tasks found by a paged load and each erased whole.
    Erasing,
    /// Its closed ledgers loaded, folded into the totals and erased.
    Spend,
    /// Its notes' entries and index lines loaded and erased.
    Notes,
    /// Its rows the core holds erased in one decision, then this row.
    Own,
}

/// What outlives a deleted project: its spend, in the deployment's and its
/// parties' totals (draft 5, "what outlives it"), never per project or task.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Spender { Deployment, Party(u64) }
```

- `CoreRecord::Removal(Removal)`, `CoreKey::Removal(u32)`: the core
  key's tag 10 then the project widened to eight bytes; **live** (restored
  at start with the core's rows, before the people family's policy pages:
  the core child's tag 0 sorts first). Bound: `limits.authority.projects`.
- `CoreRecord::Spent { spender: Spender, spent: u64 }`, `CoreKey::Spent(Spender)`:
  tag 11, then 0 for the deployment or 1 and the party's number; **live**;
  bound: the people child's party limit plus one. Cumulative totals: jig
  has no deployment period, so "for the period" is the sum over every
  removed project's periods (a `## Design` point for the draft).
- Both in the core family's schema `core-v6.schema`, changed in place
  (jig's `protocol/store.md`, 5: no version moves before the first release).
- In memory: `Core::removals: Map<u32, Removal>` and `Core::spent:
  Map<Spender, u64>`; `Core::removing(project) -> bool`.

## 2. The steps

1. **Closing decision** (`jp-close`), on `Act::RemoveProject { project }`
   (granted `remove_project`) or `Ask::RemoveProject { project }` (an
   owner of the project): refused `Unknown` for a project not live or
   already removing. In one decision: saves `Removal { project, step:
   Ending }`; drops the project from `core.projects` and `core.names` (it
   leaves every list; its name is free; the number counter is untouched);
   tells people `people::Event::ProjectClosed { project }` (its roster
   dropped and its `Roles` row erased, its inbox entries dropped); pushes
   `Request::ProjectRemoved { project }` (the root holds it until the
   commit); calls `removal::steps::begin` (the cancels); answers
   `Outcome::ProjectRemoved { project }`. The authority's policy, the
   permission and template maps and the `Policy` row stay until `Own`, for
   the tasks winding down; nothing outside the deletion reads them for a
   removing project. From this decision every entrance naming the project
   (an ask, an act, an adoption or its answer, a message, a task, a
   settings change, a connector's fact) answers as for an unknown project.
2. **Ending** (`jp-steps`): each live task of the project whose requester
   is not a task (a root) and which is not already closing is cancelled
   as `jp-acts`' `Act::Cancel` cancels it (delegates at once, then the
   task with the grace: the grace of the current settings' conversation
   template for a conversation, as temper's demo end picks it, none for
   any other), by the deployment's compound, in batches within the
   compound's bounds. Each `tasks::Request::Ended` of a removing project's
   task checks again; when no live task of the project is left, the step
   becomes `Erasing` and the first page is loaded.
3. **Erasing** (`jp-steps`): a paged load, `reads::Scope::ProjectEnded {
   project }`, of the tasks family's `Ended` rows naming the project, in
   task order; each page's tasks erased through `jp-erase`'s
   `erase::erase_task` (one decision per page), the next page loaded after
   that commit; the last page moves to `Spend` (`removal::rows::begin`).
4. **Spend, Notes, Own** (`jp-rows`): `Spend` loads `Scope::Spend {
   project }` pages, folds each page's closed ledgers through
   `tasks::Event::ForgetLedgers` into `Spent` rows and erases them;
   `Notes` loads the project's notes (`Scope::Notes`, `Scope::ScopedNotes`
   for its goal and resource scopes) and erases each index line whose scope
   names the project and each entry no other line names; `Own`, in one
   decision: `tasks::Event::ForgetProject { project }` (its live ledgers
   folded and erased, its pending person proposals, placed holds and
   period schedule dropped), `accounts::Event::ForgetProject { project }`
   (each account's `Use::Project(project)` dropped and saved),
   `people::Event::ProjectErased { project }` (retained `Answer`/`Acted`
   rows naming the project erased, so a retry answers as one past its
   retention does), the `Policy` row erased and the authority's policy
   dropped (`authority::Event::Dropped`), the core's maps and projections
   naming it dropped, then the `Removal` row erased.

The internal loads go through one helper of `jp-row`: a `reads` pending
entry with `Destination::Removal { project }`, emitted as
`Held::ReadLoad`; `Event::ReadLoaded` for it calls `removal::loaded`,
which hands the page to `steps::loaded` (step `Erasing`) or
`rows::loaded` (steps `Spend`, `Notes`). Such a load names party 0 and
skips the party's visibility check.

**Resumed at each start** (`jp-row` hooks it, the step tasks fill it): at
the first decision after the start opens, each kept `Removal` resumes its
step from the beginning: `Ending` cancels again any root not closing,
`Erasing`/`Spend`/`Notes` load their first page again, `Own` runs again.
At start, a `Policy` row of a removing project restores its policy to
authority alone, never to `projects` or `names` (`jp-close`), so no start
serves a half-deleted project.

## 3. The connector-facing shape

`Request::ProjectRemoved { project }` (contracts, section 1), held until
the closing commit, is the one thing the application hears. jig keeps no
adoption row (a connector owns its adoptions: S4, section 1) and no
project-scoped setting (jig's settings vocabulary writes scope 0 alone:
S4, section 1), so no connector ask is needed beyond that request: a
connector drops its adoptions and the settings of its kinds for the
project on it (temper's: part `remove-forge`). Afterwards, an adoption
answer or resource fact naming the project is refused or dropped as for
an unknown project. A connector's kept `RemoveProject` act sent again after
the deletion has ended (its retained answer erased at `Own`) is answered
`Refused(Unknown)`: temper treats that answer as done.

## 4. The modules and their owners

`jig-domain-core/src/removal.rs` (`jp-row`) declares `mod close; mod
steps; mod rows;`, each file with its module doc and the entries the
others call, with these bodies until their task fills them:

| Entry | Stub body (`jp-row`) | Filled by |
|---|---|---|
| `close::close(core, env, work, out, request, project)` (unused until routed) | refuses `NotOffered` | `jp-close` |
| `steps::begin(core, env, work, out, project)` | nothing | `jp-steps` |
| `steps::ended(core, env, work, out, project)` | nothing | `jp-steps` |
| `steps::loaded(core, env, work, out, project, result)` | nothing | `jp-steps` |
| `rows::begin(core, env, work, out, project)` | erases the row, drops the entry | `jp-rows` |
| `rows::loaded(core, env, work, out, project, result)` | nothing | `jp-rows` |
| `removal::resume(core, env, work, out)` | dispatches each kept row by step to `steps::begin` / `rows::begin` | (fixed) |

Signatures follow the core's routing helpers (`&mut Core, &Env<Limits>,
&mut Queue<Event>, &mut Queue<Request>`); `jp-row` fixes them and its
report names them.

## 5. Tests

- `jig-core-world`, `suite` binary: `project_removal.rs` (`jp-close`),
  `project_removal_steps.rs` (`jp-steps`), `project_removal_rows.rs`
  (`jp-rows`).
- `jig-conformance-world` (`jig/tests/conformance`): a case per family
  and the referee's end check that no durable row names the project
  (`jp-conform`), with the deletion cut by a crash after each step's
  commit, resumed to the same end.
