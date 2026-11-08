# standing-roles: status

2026-11-08. Where the design lags or departs from [standing-roles.md](standing-roles.md).

- **Design lags:**
  - 2 and 9: jig's `domain/tasks.md`, sections 3 and 10, have no standing
    agent task: an agent task that parks idle with nothing pending is told
    that nothing is awaited and ends failed when its tries run out (jig's
    `domain/tasks.md`, 5.2; jig's `domain/engine.md`, 7.4). Only recurring
    tasks and a connector's standing procedures stand today.
  - 2 and 9: jig's `domain/authority.md`, section 7 carves a recurring task's
    budget again each period, but no agent task's.
  - 2, 3 and 9: a task's working notes are jig's `draft/notes-memory.md`,
    section 4, a draft; jig's `domain/engine.md`, section 10 keeps notes by
    scope alone, with no task's own entry read whole in every brief.
  - 3 and 9: jig's `domain/engine.md`, 7.4 lets a charter alone say whether a
    run resumes or starts fresh, and jig's `domain/tasks.md`, 8.6 gives a
    charter whose runs resume no parts. Neither has a session, its end by a
    quiet span, a bound or closed work, or the warm wake that writes the Now.
  - 4 and 9: temper's `../domain/findings.md`, sections 2, 6 and 9 name a
    project's areas per repository and one triager per project; areas by path
    with an owner, and routing to that owner, are not designed.
    `../domain/after-landing.md`, section 7 tells the project's triager of a
    red of no goal's, not its area's owner.
  - 5 and 9: `../domain/agent.md`, section 8 has no row for a standing role,
    and `../protocol/configuration.md`, 13.4 no role's charter or template.
  - 6 and 7: `../client/README.md`, 5.3 and section 6 have no Team page, no
    role's card and no conversation with a role; `../domain/forge.md`, 5.7's
    status agent reads no owner's Now.
  - 8: `../client/tools.md`, 5.3 has `finding route`, `finding backlog` and
    `finding drop`, but no `finding file`, `need ask`, `role list` or
    `role show`.
  - 10: `stock-ways.md`, 3.3 and `../protocol/configuration.md`, 13.10 bring
    no role with a way.
- **Open:**
  - 3: the quiet span's and the context bound's defaults, 50 minutes and half
    the window, until the first roles' numbers say otherwise.
  - 6: who may talk with a role, and whose instruction may become a rule
    (section 12).
