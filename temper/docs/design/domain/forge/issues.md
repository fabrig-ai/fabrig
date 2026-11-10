# Issues

The forge stays readable: issues show goals and their milestones, pull
requests their changes, for people who prefer to read there and for the
history they keep if temper is unplugged. Issues are projections (jig's
`domain/model.md`, 8.1), written and never read back:

- **One per goal,** in its project's home (jig's `domain/model.md`, 6.1),
  created when the task becomes tracked. Its body is the goal and its
  plan as a task list, which the forge renders with its progress,
  rewritten as the plan changes, at most once per `interval` and only
  when what it projects changed.
- **Milestones as comments,** each once: the plan accepted; each revision
  and its reason, from the tree's history; each change landed or held;
  each report the goal's agents mark as worth keeping ("why not library
  X"), and the design agreed, in their words; the goal finished, after
  which the issue is closed.
- **Pull requests refer to it** ("Part of `ai/temper#12`"), so the forge
  links each change to its goal, across repositories too.
- **Findings** are `../findings.md`'s: labelled issues whose labels temper
  reads back, the one exception to "never read back" here.
- **A projection, not a procedure** (jig's `domain/connectors.md`,
  section 8). The `issues` child domain is fed each goal's state, its
  plan and its milestones by the root: it is part of tracking, not work
  anyone asked for, so it has no task of its own. Its effects are keyed
  by the goal and what they write, made with the authority the project's
  policy gives its goals' projections (jig's `domain/authority.md`,
  section 6), in its home; the last ones, the closing milestone and
  closing the issue, are made as the goal closes.
- **It fits its bounds, and never holds the goal.** The issue's body,
  its edit and its milestones have the forge's bounds (the issue
  policy's body, plan items and milestones; the client's operation
  bytes); what it projects is cut to them, never refused: the goal's
  words to their share, each plan item's text to an equal share of what
  is left, marked cut; past the plan items' bound, the last item says
  how many more. Comments are written once and never edited, so past
  the milestones' bound the newest wait rather than the oldest fold:
  milestones are posted one each while more than two of the bound's
  slots remain; when the goal finishes, one comment counts those that
  waited and gives the last one's words, then the finishing milestone,
  then the close. What is posted is always a prefix, so nothing new is
  stored. A projection is a reading of the goal, not its work: a write
  of it that still fails holds the projection alone, retried after the
  policy's interval while the goal lives and settled failed when the
  goal closes, never holding the goal, its runs or its close.
  A creation lost to a crash and not found again (`README.md`, section 4: never
  sent again by temper on its own) keeps the projection's at most once:
  a milestone comment so lost counts as posted, and the projection goes
  on to its next milestone, the finishing one and the close; the issue's
  open so lost fails the projection, which has nothing to write on, and
  settles failed at the goal's close.
- **A full outbox defers it.** A projection whose next write meets the
  outbox at its bound (`README.md`, section 4) waits for room, neither failed nor
  settled: it is tried again after the policy's interval, as a failed
  write is, and goes out at the first try that finds room; a restart
  tries it at once, as it does every projection with nothing in flight.
  Nothing asks for it sooner: once a step has met the full outbox, a
  projection due at once in that step, by a write's outcome, a release
  or its timer, waits the interval too while the outbox stays full, so
  the step ends.
- **Issues others open** in an adopted repository are participating
  objects, which may become intake later.

## 1. Questions on the forge

Off by default: a project's setting turns it on.

- **A question to a person on the forge:** a person task asking in
  words (jig's `domain/people/README.md`, section 7), addressed to a party
  whose forge account adoption mapped (`adoption.md`), is projected as an
  issue in the repository it concerns (its goal's changes' repository,
  else the project's home), provided that account is on that repository's
  forge: a party known on another has no mapped account here. It is
  labelled `waiting-on-person`, carrying the
  question's words and the asking task, keyed by the person task (a
  purpose of tag 10: the task, 8 bytes).
- **The person answers in a comment.** A comment on that issue by the
  mapped account, the first after the issue's last projection, is the
  person task's answer, admitted as that party's `Answer` (jig's
  `domain/people/acts.md`, 1) at the revision the projection showed, which
  the issue's body carries and temper's record keeps; the inbox shows
  the same question, and whichever answer commits first wins, the other
  refused as decided. An amendment, or an address to another party,
  moves the revision: the issue is edited to show it (and closed, if
  the new party has no mapped account), and a comment made before the
  edit is refused as moved, with a comment saying so.
  The issue is subscribed as a participating object (`README.md`, section 7) for
  that account's comments alone.
- **Closed when the question ends:** answered, by either way, with a
  comment saying so; cancelled; or ended with its asker's cancel.
- **The one exception** to `README.md`, section 7's rule that what people write on
  temper's objects reaches no task: one issue per question, the mapped
  account only, its first comment only, nothing else read.
