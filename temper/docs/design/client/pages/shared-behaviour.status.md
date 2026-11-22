# shared-behaviour: status

2026-11-22. Where the tree's code lags or departs from [shared-behaviour.md](shared-behaviour.md).

- **Not yet:** 1, *A task's title on a line* and *A task's words after its
  title*: a goal's and a task's rows and the lineage bar draw titles
  whole, and an agent's task's brief repeats one-line words [in flight:
  change 6356].
- **Not yet:** 1, *A task's words after its title*: Needs you's held,
  funds and person task cards draw their words whole after the from line
  that names their title, not only those past their first line [in
  flight: change 7579].

- **Design departs:** `README.md`, *Away*, `goal.md`, 2, and `../view.md`, 6, and `settings.md`, 2, from the request `ui-away`
  of 2026-11-05, each decided by its plan's cut, for the user to see:
  -  every clock in the browser reads in the person's zone, where the request
    asks it of Away's 08:00, because `TimeForm::Clock` is one form and read
    universal it would say "Away until 06:00". The offset is read once, at start,
    so a change of daylight saving shows at the next load, and
    `forge::away::next_eight` takes that one offset, so the night the clocks change
    its 08:00 can be an hour off. The terminal stays universal (`../README.md`, section 5).
  - the whole document: the chip is a second button in the rail, the phone's top bar, where the
    request has it in the banner: a project's page has no banner (`../view.md`, 3). The
    terminal's is the rail's button, where the request has its `navigation`, which
    a project's page does not draw.
  - the whole document: "Limits" focuses the section's heading, by a count in `Pages`, and a
    browser brings it into view. `#away` is no anchor: `Address` holds none, so no
    address reaches the section.
  - the whole document: the toast names the project, where the request's words have "fabrig’s".
    "Away: agents decide within PROJECT’s limits.", "Away didn’t change. Try
    again." and "That was overturned already." are words the request does not name.
  - the whole document: a row whose task has ended says "landed…" or "ended…", overturned or not,
    its took still struck, where the request's list has "Overturned: it asks you
    again" first: a refused message leaves the row marked, and its task won't ask.
  - the whole document: Home's away form is not drawn while To begin shows, where the request has
    it with any row showing.
- **Not yet:** *Away*, `../forge/records.md`: with the `away` watch refused, ended or not
  delivered yet the switch draws off, but its act is still offered, where `../forge/records.md`
  has it not offered: `temper-client-view`'s `frame.rs`, `away`, and
  `pages/settings.rs`, `away_until`, draw their controls enabled, and
  `temper-client-domain`'s `pages/away.rs`, `act`, then sends nothing and says
  nothing, so a press does nothing. Seen by reading; no test holds it and no
  reason is recorded.

- **Departs:** *Decided for you*, its *Overturn*: an `Overturn` whose outcome is no
  answer, unknown after a restart or its link lost, toasts "That was overturned
  already." and sends no message (`temper-client-domain`'s `forge/decided.rs`,
  `overturned`), where the section has those words for a refusal: if it
  committed, the row stands marked and its task has heard nothing. A message that
  fails for any reason, a lost link's too, toasts "It has ended; a new goal can
  undo it." (`pages/away.rs`, `overturn_told`), where the section has them for a
  task that ended: a live task then hears nothing, and its marked row says
  "Overturned: it asks you again" and offers no second Overturn. Seen by reading;
  no test holds either and no reason is recorded. The message's question is cut
  at a character boundary to fit 512 bytes (`forge/decided.rs`, `overturn_words`),
  where the section has it whole, because the smallest deployed composer admits
  that (`../view.md`, 4).

The document, *Away*, *On All projects*, and `../view.md`, 3's Away row have no known
gap, by plan `ui-away-all`: `pages::away_all::across` counts `me`'s
projects, those on at the wall and the earliest `until` of those on, and
whether Away shows for any, which `rail::facts` reads on a page across
projects; `frame.rs`'s `away` draws the switch and the chip from it,
"Away on N of M" between, and binds `away::Act::ToggleAll`, whose
`away_all::act` builds each changing project's request by
`pages::away::request`, which a project's own act calls, and sends them
one after another, toasting `Toasted::AwayAll`, or `AwayRefused` with
none changed. The domain's `tests/away.rs`, its
`across_counts_the_projects_on`, `toggle_all_sets_each_project_that_differs`
and `a_second_toggle_waits`, `tests/pages/rail.rs`'s "across" row, the
view's `the_switch_shows_with_ways_away` and the gallery's
`all-home-away` and `all-home-away-on` (`../web/gallery.status.md`, 2) hold
it. No render of the words is on record and no client-world story
presses the switch across projects: the render check and the walk remain
`../README.md`, section 5's after-landing work (*Away on All projects after landing*).

