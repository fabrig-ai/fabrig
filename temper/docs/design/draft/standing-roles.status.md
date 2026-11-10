# standing-roles: status

2026-11-09. Where the design lags or departs from [standing-roles.md](standing-roles.md).

- **Design lags:**
  - 2 and 9: jig's `domain/tasks/README.md`, sections 3 and 5, have no standing
    agent task: an agent task that parks idle with nothing pending is told
    that nothing is awaited and ends failed when its tries run out (jig's
    `domain/tasks/lifecycle.md`, 2; jig's `domain/engine/executors.md`, 4). Only recurring
    tasks and a connector's standing procedures stand today. Nor is an
    agent task exempt from the stall's bound while idle with live delegates,
    or from tries counted over its life (jig's `domain/tasks/lifecycle.md`, 6, and `domain/tasks/README.md`, 6
    ), and a run that answers finished ends its task (jig's
    `domain/tasks/lifecycle.md`, 2) where a standing role's is parked.
  - 2 and 9: temper makes no move of a task to a new requester when a
    member leaves a project; jig's move (jig's `domain/tasks/changes.md`)
    exists, but nothing in temper calls it for a standing role's person.
  - 2 and 9: jig's `domain/authority.md`, section 7 carves a recurring task's
    budget again each period, but no agent task's.
  - 2, 3 and 9: a task's working notes are jig's `draft/notes-memory.md`,
    section 4, adopted on 2026-11-08 for standing roles alone; jig's
    `domain/engine/briefs.md`, section 2 keeps notes by scope alone, with no task's
    own entry read whole in every brief.
  - 3 and 9: jig's `domain/engine/executors.md`, 4 lets a charter alone say whether a
    run resumes or starts fresh, and jig's `domain/tasks/messages.md`, 6 gives a
    charter whose runs resume no parts. Neither has a session, its end by a
    quiet span taken from the account's and model's cache life, a bound,
    closed work or a waking message its parts don't offer, or a Now written
    before every park.
  - 4 and 9: temper's `../domain/findings.md`, sections 2, 6 and 9 name a
    project's areas per repository and one triager per project; areas in
    words with an owner, and the triager's act handing to that owner, are
    not designed.
  - 5: no landing rule by path holds a standing role's change that touches
    paths a rule guards (`what-reaches-you.status.md`, 7).
  - 5 and 9: `../domain/agent/agents.md` has no row for a standing role,
    and `../protocol/configuration/stock.md` no standing role's charter or
    template.
  - 6 and 7: `../client/pages/README.md`, *Team*, draws a project's people
    alone (`surfaces.status.md`, 6), with no standing role's card or
    report, and its 5.3 and `standing-roles.md`, section 6 no conversation with one; Home's
    brief reading the owners' Nows is `surfaces.status.md`'s.
  - 8 and 11: `../client/ctl/tools.md`, 3 has `finding route`, `finding
    backlog` and `finding drop`, but no `finding file`, `need ask`, `role
    list`, `role show` or `role talk`.
  - 10: `stock-ways.md`, section 2 lets a way set how work starts and lands
    alone, and its 3.3 and `../protocol/configuration/settings.md`, 8 bring no
    standing role with a way.
- **Open:**
  - 3: the context bound's default, half the window, and how far under the
    cache's life the quiet span sits, until the first standing roles'
    numbers say otherwise.
  - 6: who may talk with a standing role, and whose instruction may become a
    rule (section 12).
  - 12: a standing role's default model and budget, per stock way.
