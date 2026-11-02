# reference-m1: contracts

The shapes that cross this plan's families, as its binding calls. A task builds its half to
this note and needs no other family's task for it, except where its spec's `needs` says so.
Each entry names the design that holds it in full; the design wins on detail, this note on
the names. Families: **goal** (the goal's own branch, jig's holds and templates), **change**
(`change` and the change procedure), **settings** (the landing rules as settings, a charter's
default wake policy, the reference's configuration), **bound** (a result's bound).

## C1. The goal's own branch, g

- **Its name:** `temper/<root>/g<task>`, `<task>` the goal's number and `<root>` its tree's
  root (a goal that is its tree's root is `temper/<root>/g<root>`), in its project's home
  repository, under that repository's prefix: a sibling of the `c`, `s`, `l` and `r`
  branches of `temper/docs/design/domain/forge.md`, section 11.
- **Whose:** the goal's, held by the goal task G for its whole life (an exclusive hold,
  refusing), never a change's. M1 keeps g as G closes, whatever way it ends; its deletion
  once a change has landed it is M2's.
- **Made** at the home's default branch's tip as G's hold is taken, by the forge, as the
  effect of the hold's taking; failing, G is held, its words saying why.
- Family: goal.

## C2. A template's own resources

- jig's template names, beside its other defaults, the **symbolic resources** a task filled
  from it holds: each a connector's symbol, resolved once the task has its number, with its
  grant and its hold (jig's `domain/authority.md`, section 4: the mechanism a batch uses for
  "its own branch" today). A template is data for clients: a task filled from it carries the
  symbols in its batch entry, and jig checks the batch as any.
- **The forge's symbol for g:** *its own goal branch*, offered on a tracked task only; a batch
  naming it for an untracked task is refused, naming the symbol.
- The configuration's spelling of a template's symbols, and the Rust names, are the goal
  family's design (jig's `protocol/configuration.md`, 3.1), and this note's C2 line below
  once written. The settings family's template for drafts uses that spelling.
- Family: goal.

## C3. Writing g

- g is **held and a landing branch with a queue** at once: G's delegates land into it, each
  change's merge ordered by g's queue (`forge.md`, section 9), one at a time under G's hold.
- **Writer slots** are taken by what a run or an effect writes: a merge into g takes g's
  writer slot while in flight; G's runs, inspect only, take none.
- **Refused from outside:** a change whose landing branch is held by a task that is not the
  change's requester or an ancestor of it is refused at its batch, naming the holder.
- Family: goal.

## C4. `change`, M1's parameters

M1 adds to `change` (`temper/docs/design/domain/agent.md`, 4.3.1) only these; M2, M3, M5, M7
and M9 add theirs beside them, so the call's shape takes new optional members without
changing these:

- **`landing`:** `"goal"`, the requester's own goal branch (C1), or a branch's name in the
  repository; absent, the repository's default branch, as today. `"goal"` from a requester
  holding no goal branch is refused, saying so.
- **`after`:** the change's dependencies, each a task number or the `name` of another change
  in the same call.
- **Several changes in one call,** each with its own words, `name`, `landing` and `after`,
  made as one batch (jig's `domain/tasks.md`, section 4): all or none.
- The exact JSON spelling is the change family's design (agent.md, 4.3.1). The root checks
  each member as it decides the delegate; a charter's authority says whether its runs may set
  them.
- Family: change.

## C5. Checking and updating by the landing branch's rule

- A change's CI at its head (`forge.md`, 8.2 step 3) and its update when first (8.2 step 6;
  section 10) run only where its landing branch's rule asks them: CI passed, up to date.
- The procedure asks one thing of the landing rules: for a repository and a landing branch,
  `Requires { ci: bool, up_to_date: bool }`, from a `requires` lookup in
  `temper-engine-domain-forge` over the rules as compiled. The change family creates it over
  the rules as they are compiled today; the settings family keeps its signature and meaning
  when it moves the rules into settings.
- **The rule for goal branches:** a pattern on the forge's `branch` segment then `temper`,
  its last segment open (jig's `domain/authority.md`, section 4), asking neither: a change
  into g merges at its head, as a merge commit, once the forge finds it mergeable.
- A merge into g refused for a conflict: the merge job is M7's. In M1 the change is held,
  its words saying the merge conflicts (the change family's design says where).
- Family: change; the rule's source, settings.

## C6. A change's result words (N15)

- After its landing's three lines, its producer's last report, then what the landing passed,
  cut from the end to the result's bound and marked with "…" (`forge.md`, 8.2 step 7).
- The result's bound is the deployment's `tasks.result_bytes`; jig counts a verdict's fields
  apart from it (C8), so a profile may raise it past 512.
- Family: change.

## C7. A charter's default wake policy (N23)

- A charter in configuration carries a **default wake policy** in jig's closed vocabulary
  (jig's `domain/tasks.md`, 8.3); a task made under the charter, from a template or a batch,
  takes it unless its creator narrows or widens it.
- The vocabulary gains **a pace for news:** waking news batched by age, at most once in a
  span (the coordinator's: 30 minutes).
- **The coordinator's:** questions, escalations, failures, every delegate's result, words from
  its delegates and its person's words, at once; news its connectors class as waking, at
  most once in 30 minutes.
- The configuration's spelling and the Rust names are the settings family's design (jig's
  `protocol/configuration.md`, section 3; tasks.md, 8.3).
- Family: settings.

## C8. A result's bound past the settings schema's cap

- `jig_domain_tasks::VERDICT_FIELDS = 512`, the settings schema's `list 512` ceiling; a
  verdict's fields are counted apart from `result_bytes` (jig's `domain/tasks.md`, section 10).
- Family: bound (task `verdict-fields`).

## Names the families write here

Each family's designer adds one line under its entry's heading below as it settles a name
another family uses; nothing else of this note changes after release.

- C2:
- C4:
- C7:
- The reference's configuration (settings family): where its first editions live.
