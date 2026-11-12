# follows: status

2026-11-12. Where the design lags or departs from [follows.md](follows.md).

- **Design lags:**
  - 9: `../testing/README.md` has no world with a follow, and
    `../../development/acceptance.md` no follow story.
- **Design departs:**
  - 2 and 3: a standing role's follow changes are the delegates of a
    follow task the role requests and funds, as a goal's closer is its
    goal's, not the role's own: code makes them, so a procedure stands
    between; their producers' questions reach the role from that task in
    the question form, and their holds as escalations past it, and an
    edit of the owner makes a new follow task that remakes the queue's
    changes (`../domain/forge/follows.md`, sections 2 and 4). Decided by
    goal 1985's design, 2026-11-12.
  - 4: a red after landing of a follow change is filed by code with the
    first area its follow's owner owns, so it reaches the owner with no
    agent's run, rather than handed on by the triager's judgement; an
    owner that owns no area gets it through the triager
    (`../domain/forge/follows.md`, section 6). Decided by goal 1985's
    design, 2026-11-12.
  - 5 and 7: Home's What follows what draws a box a follow, "follows
    SOURCE" over the holding project's name, its line "2 behind · check
    passes at 9c04d7e1a2b3", where this draft's line is "vsr-rs follows
    vsr · 2 behind · all C tests pass on Rust": no setting holds what a
    check checks; and a box a dependency, "depends on vsr-rs" over the
    dependent's name, beside the follows rather than in one chain from
    vsr, since the `follows` document names a follow's source by its
    repository, not its project (`../client/pages/home.md`, 1,
    *What follows what*). Decided by goal 1243's design, 2026-11-12.
  - 2: a follow change's words carry the landing's commits, their
    messages, its files in the follow's paths and a link to its
    comparison, not its diff: no forge read gives a range's patch, and
    the producer reads the diff whole in its read-only checkout of the
    source (`../domain/forge/follows.md`, section 4); the goal's
    designer's recommendation, 2026-11-11, asked of the user.
  - 2 and 3: a person's follow's changes are the delegates of a follow
    task the deployment requests, not the person's own, since code makes
    no task a person requests without their act: a held follow change
    reaches the project's escalation role, and its owner reads it in the
    follow's lag (`../domain/forge/follows.md`, section 4); the goal's
    designer's recommendation, 2026-11-11, asked of the user.
- **Open:**
  - 3 and 10: a follow's owner asks its person in the target's project;
    whether it may ask the source's project across projects. The design
    takes the default: no question leaves the target's project
    (`../domain/forge/follows.md`, section 8).
  - 8 and 10: the pilot's check runs vsr's tests unchanged against
    vsr-rs through `include/vsr.h`'s C interface, rather than ported
    tests; the user's to confirm.
  - 2: a follow's changes wait for their owner's word by default
    (`owner`), rather than starting in order (`code`); the design takes
    the default (`../domain/forge/follows.md`, section 2), the user's to
    confirm.
  - 4 and 10: the followed commit reaches the check as a gate's input,
    `temper_follows`, the draft's default
    (`../domain/forge/follows.md`, section 6).
