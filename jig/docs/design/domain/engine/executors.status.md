# executors: status

2026-11-12. Where the tree's code lags or departs from [executors.md](executors.md).

- **Not yet:** 4: a standing task's finish parked (`../tasks/standing.md`, section 2) [in flight: change 103].
- **Not yet:** 4: a standing task's runs resuming only within a session (`../tasks/standing.md`, section 5) [in flight: change 106].

- **Not yet:**  engine slots, hello and placement remain `fleet.status.md` (Not yet, the whole document).  the point-load fences remain `README.status.md`, 4.2 and 5.3.
  1: the brief's missing sections remain `briefs.status.md`, 1. 2: what a
  turn's calls made, written beside the turn. 3: the tools `move`, `succeed`,
  `hold` and `lift`.
   the restart-oracle gaps remain `../testing.status.md`, 9.
   the pending-reasons continuation gap remains `settings.status.md`, 6.
   the settings-holder gap remains `settings.status.md`, 6.
   the ignored restore stories remain `settings.status.md`, 10.

- **Not yet:** 3, *A run's read as its person*: a task query of the
  project that names a person requester reads its ended part from the
  person's index, which spans projects (`reads/ended.rs`,
  `ended_part`), and a page that loads another project's row answers
  unknown whole (`reads/rows.rs`, `person_from_store`), where a party's
  read drops that row. temper's addresses name no requester; no test
  reads such a query.

- **Not yet:** 3, *Of its project alone*: an inbox page filtered by the
  run's project takes its `next` cursor from the last entry of the
  person's whole inbox, before the filter (`reads/rows.rs`, the live
  page's `last`), so whether there is a next page, and the cursor the
  root holds, can come from another project's entry; temper says only
  whether more wait. No test pages a filtered inbox.
