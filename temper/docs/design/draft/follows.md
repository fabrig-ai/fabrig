# Follows

Draft, 2026-11-08, from a session with the user that day. A project may
follow another's branch: each landing there that touches the paths it
names makes a change here, which a standing role owns, and a check proves
the two still agree. It is the mechanism the whole story's pilot needs
(`whole-story.md`, 5.2): vsr's C, ported to Rust line by line, continuously.
It adds to `../domain/forge.md`, 7.1's topics and 8.1's parameters, and it
wins where they differ until its plans fold it in. temper/migration carries
out the follow, its changes and its check; temper/ui, its lag and parity on
Home, Team and How work runs (`surfaces.md`, sections 3, 6 and 7).

## 1. In one page

- **A follow is a project's setting** (section 2): a source, a branch of a
  repository of another project, with the paths that matter; a target, a
  repository and branch of this project; its owner, a standing role
  (`standing-roles.md`, section 2); and its check.
- **Code follows, the owner ports** (section 3). Code hears each landing on
  the source's branch, keeps those that touch its paths, in order, and makes
  one follow change for each, with the landing's diff, message and link as
  its words. The owner starts it, merges it with the next ones, holds it
  with a reason, or skips it with one. It asks a person only what
  `what-reaches-you.md`, section 2 sends to one.
- **Follow changes land in the source's order.** Each waits on the one
  before it, so a held change holds the rest, and the lag says why.
- **The check proves agreement** (section 4): a gate of the target's, for
  example the source's own tests at the followed commit, built against the
  target. A red at the gate is the change's to repair; a red after landing
  is a finding its owner triages.
- **Lag and parity are numbers code keeps** (section 5): landings behind,
  and the check's state at the newest followed commit. Home, Team and How
  work runs show them in a line.
- **One deployment** (section 6): the target's project adopts the source's
  repository as context, so its role reads it. A source on another
  deployment is later.
- **Depends on** (section 7) names a project another builds on, and shows
  it. Nothing moves on its own in v0.1.0.
- **The pilot** (section 8): vsr-rs follows vsr, a line-by-line port of a C
  library to Rust, checked by vsr's tests built against it. fabrig depends
  on vsr-rs, which will replicate skein-kv.
- **Its acceptance** (section 9): focused and fuzzy tests in worlds, with
  the fake forge and the fake model, and a story on a box for v0.1.0.

## 2. A follow

A follow is a setting of the target's project, proposed and accepted as
every setting is (`configuring.md`, 5.2). It holds:

- **Its source:** a project of the same deployment, a repository of that
  project, a branch of it (its landing branch, main, when none is named),
  and path prefixes; none means every path.
- **Its target:** a repository of this project that temper writes, an
  owned one (`../domain/forge.md`, section 4), and the branch its follow
  changes land into: the repository's landing branch, unless it names
  another.
- **Its owner:** a standing role of this project (`standing-roles.md`,
  section 2), or a person. The follow changes are its delegates, and it
  hears what goes wrong with them.
- **How its changes start:** `owner`, the default, where each waits for
  its owner's word; or `code`, where each starts once the one before it
  has landed, and the owner wakes only for a question, a hold or a red.
- **Its producer:** the charter and LLM choice a follow change's producer
  runs under, as `change` names them (`../domain/agent.md`, 4.3.1). For a
  port, the charter carries the port's rules: line by line, the source's
  names and order kept.
- **Its check:** a gate of the target's repository (`../domain/forge.md`,
  8.3), named by the follow, which every follow change carries (section 4).
- **The follow change's words,** made by code, never by a model: the
  source landing's commit, its message, its pull request's link where it
  has one, and its diff over the follow's paths, cut to the change's bound
  with a link to the whole. A change following several landings carries
  each, in order.
- **What a follow change names:** as `repairs` names the red landing a
  change repairs (`../domain/after-landing.md`, section 7), a follow change
  names the source landings it follows, by commit, checked by the root as
  it decides the delegate. Its landing is what makes them followed.

## 3. What code does, what the owner judges

A follow is close to a tier (`../domain/after-landing.md`, section 3): both
follow a branch's tip. A tier's verdict judges the landings in its range; a
follow's answer is a change in another repository. A tier covers several
landings with one run; a follow makes one change a landing, because a port
is per change, and leaves merging them to its owner.

Code, with no model:

- **Hears the source's landings** on the topic of landings on a
  repository's branch (`../domain/forge.md`, 7.1), read as 7.2 reads them:
  temper's landings and those it did not make alike, with the files each
  touched.
- **Keeps those that touch the follow's paths,** in the source's order: the
  follow's queue, in the store, kept across a restart. A landing whose
  files are unknown, as 7.3's overlap can be unknown, counts as touching
  them: a follow change for nothing costs its owner a skip, a missed one
  costs parity.
- **Makes the follow change** for each, its words as section 2 says, a
  delegate of the owner's, depending on the follow change before it (jig's
  `domain/tasks.md`, 5.7). With `owner` it waits for its owner's word; with
  `code` it starts when its dependency lands.
- **Keeps the lag** (section 5) from the queue and the landed follow
  changes, with each held change's reason.

Its owner, at a wake (`standing-roles.md`, section 3):

- **Starts it** as code made it.
- **Merges it** with the next ones into one change that names them all,
  when they touch the same code. A role that waits for the source's next
  fix to the same functions, to port both together, is doing this.
- **Holds it** with a reason, which the lag shows: "waits for the handoff
  fix".
- **Skips it** with a reason, for a landing that needs no port: a comment,
  a change to the source's build alone. A skip counts as followed, and the
  follow lists it with its reason.
- **Asks** what it can't settle, as `what-reaches-you.md`, section 2 sends
  it: in the pilot, a source construct with no direct form in the target
  goes to the source's protocol core's owner, a person.
- **Hears its producers' questions first,** as a change's agents' questions
  reach its requester (`../domain/forge.md`, 8.11).

## 4. The check

- **A gate of the target's,** defined as any gate is (`../domain/forge.md`,
  8.3), which the follow names and every follow change carries beside the
  target's landing rule.
- **Given the followed commit:** dispatched as a gate is
  (`../domain/landing.md`, 7.1), with the newest source commit the change
  follows as one more input beside its commit and key, so its workflow can
  fetch the source there. For a port, the source's tests at that commit,
  built against the target.
- **A red at the gate** is the change's, repaired as any change's is
  (`../domain/forge.md`, 8.4), its producer told which of the source's
  tests fail. A change that keeps failing is held, and its owner hears it.
- **After landing,** the target's tiers follow its main as any repository's
  do (`../domain/after-landing.md`, section 3): the slow suites, the
  source's fuzzing replayed on the target's build. A red there is filed as
  a finding (`../domain/findings.md`, section 4) and routed to the follow's
  owner by its area (`standing-roles.md`, section 4).
- **Proof stays in tests.** Code doesn't compare the follow change's diff
  with the source's: the gate's tests and the agents' review are the check,
  as on any change.

## 5. Lag and parity

- **Lag:** the source landings in the follow's paths that no landed follow
  change names and no skip covers, counted, oldest first, each held one
  with its reason. Zero when caught up.
- **Parity:** the check's state at the newest followed commit: green, or
  the number of the source's tests failing on the target, read from the
  gate's report as a tier's run is read (`../domain/after-landing.md`,
  section 4).
- **Where they show:** on Home's "what follows what", one line a follow,
  "vsr-rs follows vsr · 2 behind · all C tests pass on Rust"
  (`surfaces.md`, section 3); on Team, the owner's row and card (section
  6); on How work runs, the follow's settings with its lag (section 7).
- **Code computes them,** in the decisions that change them, never a
  model. The status agent's brief may quote them.

## 6. Across projects

- **Within one deployment.** The source is a repository of another project
  of the same deployment, on any forge the deployment keeps.
- **Read as context.** The target's project adopts the source's repository
  as context (`../domain/forge.md`, section 4), so its tasks may read it and
  its owner's runs check it out read only beside the target. A follow of a
  repository the project hasn't adopted is refused.
- **A context repository's landings** are heard for its follows: the forge
  follows the source's branch for as long as a follow names it, as it does
  for a tier (`../domain/after-landing.md`, section 3).
- **Who makes one:** an owner of the target's project, by accepting a
  proposal. The source's project is told nothing and gives nothing:
  reading is all a follow does there.
- **Later:** a source on another deployment, or on a forge the deployment
  doesn't keep, read through its public reads alone.

## 7. Depends on

- **A relation between two projects,** a setting of the dependent's:
  "fabrig depends on vsr-rs".
- **Shown** on Home's "what follows what" beside follows (`surfaces.md`,
  section 3), so the person sees the chain: vsr, followed by vsr-rs, on
  which fabrig depends.
- **Nothing automatic in v0.1.0:** a landing in the dependency changes
  nothing in the dependent. A role of the dependent may read it as context
  and decide.
- **Later:** a dependency bump, a follow whose change moves the dependent
  to the dependency's newest landing, checked by the dependent's own gates.

## 8. The pilot: vsr to vsr-rs

The whole story's pilot (`whole-story.md`, 5.2) is a follow.

- **The source:** vsr, a C11 library at github.com/cowsql/vsr: a
  deterministic, sans-I/O core with an io_uring I/O layer, built with
  autotools, its CI running ASan and UBSan, `-Werror`, clang-tidy,
  cppcheck, a fuzz smoke test and coverage. cowsql moves from its raft
  layer onto it.
- **The target:** vsr-rs, a repository of its own, in a project of its own
  on the same deployment, a line-by-line port to Rust. `unsafe` is fine:
  its guarantees come from vsr's tests and QA replayed on the Rust build,
  not from the type system.
- **The follow:** vsr's main, its paths `src/` and `include/`, into
  vsr-rs's main, owned by Porter, a standing role. Its producer's rules:
  port line by line, keeping C's names, order and comments; a construct
  with no direct Rust form goes to the protocol core's owner.
- **The check:** vsr's tests at the followed commit, built against vsr-rs
  through the C interface of `include/vsr.h`, which vsr-rs exports.
  vsr-rs's tiers replay vsr's fuzz targets on the Rust build.
- **Continuous:** each landing on vsr's main in those paths becomes a
  change in vsr-rs, so vsr-rs is never more than its lag behind.
- **Depends on:** fabrig depends on vsr-rs, which will replicate skein-kv.
  fabrig's high availability starts there.

## 9. Its acceptance

Focused and fuzzy tests alone, in worlds with the fake forge and the fake
model (`../domain/testing.md`, sections 4 and 5).

- **Focused,** in a world with two projects on the fake forge, the target's
  adopting the source's repository as context, and a follow owned by a
  scripted standing role:
  - a landing in the source touching the paths makes one follow change
    that names it, the landing's message and diff its words; one outside
    the paths makes none; one whose files are unknown makes one;
  - two landings make two follow changes, the second waiting on the first;
    the first held, the lag is 2, with its reason;
  - the owner merges two into one change naming both, and skips one with a
    reason: each counts as followed;
  - the check is dispatched with the followed commit; red, the change is
    repaired as any; green, it lands and the lag falls to zero;
  - the engine stopped and started with three in the queue: none lost, none
    made twice;
  - a follow of a repository its project hasn't adopted is refused.
- **Fuzzy:** random runs of source landings, in and out of the paths or
  with unknown files, the owner's starts, merges, holds and skips, reds at
  the check, and restarts. Always: each landing in the paths is followed or
  skipped once, never twice; follow changes land in the source's order;
  the lag equals the landings neither followed nor skipped.
- **The story for v0.1.0,** written in `../../development/acceptance.md`
  and walked after landing, never by a plan: on a box, two repositories on
  its forge in two projects, and a follow from one to the other owned by a
  role. A landing in the source makes a follow change, its check runs, it
  lands, and the lag goes to zero. A second follow change, held by its
  owner, shows lag 1 with its reason on Home.

## 10. Open questions

- **The owner's own hands:** whether a role's session may produce a small
  follow change itself, rather than a producer for each.
- **The followed commit's input:** a gate's new input, or the source
  fetched by the target's workflow from the change's words.
- **Tests linked or ported:** vsr-rs exporting `include/vsr.h`'s C
  interface, so vsr's tests run unchanged against it, is the pilot's way.
  Porting the tests too would test the Rust interface as well, at the cost
  of a second port to keep.
- **The other way:** a fault the target finds in the source's logic is a
  finding filed on the source's project, for its owner, rather than a
  follow back.
- **More branches:** a follow of a branch other than main, a release
  branch, and a follow started at an old commit, with the history before
  it as a backlog.
