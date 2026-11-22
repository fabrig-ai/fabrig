# whole-story: the reasons behind it

This holds the reasons for `whole-story.md`, moved from [`notes.md`](notes.md), section 12.

The reasons behind `whole-story.md` and the four drafts written with it,
from the user, 2026-11-08.

- **One front end.** "I'll be the first user of temper and ideally
  temper would be my only frontend, replacing most of the things we now
  run by hand." The pieces were there, the dogfooding, the web becoming
  an overview of where things stand and what needs the person, the TUI
  for focused sessions, but not one story joining them.
- **Some things stay outside.** A deployment's operations and the care
  of its logins have no equivalent in temper; they belong to fabrig.ai's
  later Run.
- **A middle ground between sessions.** "Very long running sessions
  should be probably avoided, but maybe we need a middle ground between
  function call-like sessions like the ones producing the code and long
  running ones." Offered standing roles with short sessions that end by
  writing their state, the user: "yes". The measure behind it: on the
  long sessions we run by hand, 74% of their own spend on 2026-11-07 went
  on wakes after the prompt cache had expired.
- **Room for every point of the spectrum.** The vision must also fit
  "other ways of using temper, perhaps at smaller scale", so the pages
  fill as a person hands over more, rather than being drawn for the
  largest case, and two more profiles were drawn: Pair and Team.
- **A pilot that isn't temper.** The user's first: cowsql's vsr, a
  solid C implementation of Viewstamped Replication replacing cowsql's
  raft layer, ported to Rust line by line, continuously, "unsafe is
  fine, guarantees would come from C tests and QA, also replayed on the
  Rust version"; the port, in a repository of its own, will replicate
  skein-kv. On the same deployment, as a new project: a brand new
  deployment would show the same pages, deployment-wide settings aside.
- **Greenfield pages.** "Take it as greenfield, doesn't matter what we
  have now, let's design optimizing for the vision", the rail, Chats and
  Settings included. Shown the sketches, the user asked for them in the
  design whole, and for all of it by v0.1.0: "we have resources (hw and
  subscriptions), so let's try to move fast and parallelize when
  possible", hence the tracks of `whole-story.execution.md`.
