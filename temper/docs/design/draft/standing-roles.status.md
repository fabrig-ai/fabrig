# standing-roles: status

2026-11-11. Where the design lags or departs from [standing-roles.md](standing-roles.md).

Sections 2, 3, 5 and 9, but for areas and routing, are carried by jig's
`domain/tasks/standing.md` and `../domain/agent/standing.md`, milestone R1
of `whole-story.execution.md`, and sections 6 and 7 by jig's
`domain/tasks/standing.md`, 5.6 and 6, and `../domain/agent/standing.md`,
sections 7 and 8, milestone R3, but for the lags and departures below.

- **Design lags:**
  - 2, 4 and 9: temper's `../domain/findings.md`, sections 2, 6 and 9 name a
    project's areas per repository and one triager per project; areas in
    words with an owner, and the triager's act handing to that owner, are
    not designed, so a standing role's spec words say what it covers
    (`../domain/agent/standing.md`, section 2).
  - 5: no landing rule by path holds a standing role's change that touches
    paths a rule guards (`what-reaches-you.status.md`, 7).
  - 6 and 7: the pages draw no talk and no report: Team's "Talk to it", a
    session's line of what it did, read from its last `NowWritten`, and
    its weekly report, and the terminal's `/talk`, are
    `surfaces.status.md`'s, 6 and 11 (`whole-story.execution.md`, P4c and
    T3); `../domain/agent/standing.md`, sections 7 and 8 carry the engine's
    side and `temper ctl role talk`.
  - 8 and 11: `../client/ctl/tools.md`, 3 has `finding route`, `finding
    backlog` and `finding drop`, but no `finding file` or `need ask`.
  - 10: `stock-ways.md`, section 2 lets a way set how work starts and lands
    alone, and its 3.3 and `../protocol/configuration/settings.md`, 8 bring no
    standing role with a way; `../protocol/configuration/stock.md` holds
    one standing role's charter, 13, for any work, and no triager's,
    prover's or porter's.
- **Design departs:**
  - 3: a session also ends when its rules change, by a move to a newer
    revision of the settings or an amendment of its own instructions,
    since a resumed run keeps the instructions its session began with
    (jig's `domain/tasks/standing.md`, 5.2): R1's design, 2026-11-11.
  - 3: the context's bound counts the bytes of the session's transcript
    window, at four bytes a token, since the engine holds a transcript's
    bytes and not its tokens; the quiet span is the model's declared
    cache life, `cache_life_ms`, an account's longer cache left open
    (jig's `domain/tasks/standing.md`, 5.3 and section 9): R1's design,
    2026-11-11.
  - 2 and 9: a person starts a standing role with `temper ctl role start`
    from template 7, its spec's words saying what it covers, until
    "Hand an area to a role" (section 10) does it by proposal; and its
    person leaving the project moves it to the owner whose request saved
    the roles (`../domain/agent/standing.md`, sections 3 and 6): R1's
    design, 2026-11-11.
  - 6: a talk lapses once 30 minutes pass with neither its person nor
    the role saying anything, and is judged then as left, ending its
    session, since nothing tells the engine that a person has gone; and
    a role holds one talk at a time, a second person's refused while the
    first is open (jig's `domain/tasks/standing.md`, 5.6): R3's design,
    2026-11-11.
  - 7: a session's line of what it did is the first line of the last Now
    it wrote, kept in its activity as `NowWritten`, since a role's runs
    park rather than finish and so leave no result's words (jig's
    `domain/tasks/standing.md`, section 6): R3's design, 2026-11-11.
- **Open:**
  - 3: the context bound's default, half the window, and how far under the
    cache's life the quiet span sits, nine tenths of it, both taken as
    defaults until the first standing roles' numbers say otherwise (jig's
    `domain/tasks/standing.md`, section 9).
  - 6: who may talk with a standing role, and whose instruction may become a
    rule (section 12): R3 takes whoever may send it words today, its
    person and the project's maintainers and owners, and a rule is the
    accepter's of its charter, a deployment's owner
    (`../domain/agent/standing.md`, section 7).
  - 6: a rule for one role whose charter others share: a rule changes its
    charter, so a role under the stock's charter 13 adds it for every role
    on 13 (`../domain/agent/standing.md`, section 7).
  - 12: the weekly report's form: R3 takes a note, described `report` in
    the role's own scope and rewritten whole each week, so that the newest
    alone is kept (`../domain/agent/standing.md`, section 8).
  - 12: a standing role's default model and budget, per stock way.
