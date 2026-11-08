# follows: status

2026-11-08. Where the design lags or departs from [follows.md](follows.md).

- **Design lags:**
  - 2: no design document holds a follow as a project's setting, beside
    the landing rules as settings (`../domain/forge.md`, 8.9), nor its kind
    in `../protocol/configuration.md`; `../domain/forge.md`, 8.1's
    parameters have no list of the source landings a follow change
    follows, as `repairs` names a red landing
    (`../domain/after-landing.md`, section 7).
  - 3: no design says who makes a follow change, or where the follow's
    queue is kept across a restart (`../protocol/store.md`); jig's standing
    procedure (`domain/connectors.md`, 6.6) is the nearest shape. The owner's
    merge, hold and skip have no tool or act of their own
    (`../domain/agent.md`, 4.3). jig holds a change whose dependency is
    cancelled until an amendment removes it (jig's `domain/tasks.md`, 5.5):
    no design amends the next follow change's dependency in the same
    decision as a merge or a skip.
  - 4: a gate's dispatch (`../domain/landing.md`, 7.1) has no input for
    the followed commit.
  - 5: `../client/README.md` draws no lag or parity: Home, Team and How
    work runs are `surfaces.md`'s, and lag with them.
  - 6: the forge follows the landing branches of a project's owned
    repositories; a context repository's branch named by a follow is
    not followed (`../domain/forge.md`, section 4 and 5.2).
  - 7: no design document holds a project's dependency on another.
  - 9: `../domain/testing.md` has no world with a follow, and
    `../../development/acceptance.md` no follow story.
- **Open:**
  - 3 and 10: a follow's owner asks its person in the target's project;
    whether it may ask the source's project across projects.
  - 8 and 10: the pilot's check runs vsr's tests unchanged against
    vsr-rs through `include/vsr.h`'s C interface, rather than ported
    tests; the user's to confirm.
  - 2: a follow's changes wait for their owner's word by default
    (`owner`), rather than starting in order (`code`); the user's to
    confirm.
