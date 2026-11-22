# After-landing effects and storage

This document holds after-landing effects, reads and durable records, split from [`after-landing.md`](after-landing.md).

## 1. Effects, reads and the store

- **A run is a task:** each tier run is a procedure task of the
  forge's, `tier`, made when the run starts, requested by the
  deployment on the project's behalf, funded from the project's spend,
  with the authority the project's policy gives tiers (kinds 6, 7, 9,
  16 and 17 on its repository), as a landing is (`landing.md`, 4.2). It
  owns the run's branch and its probes', its dispatch, reruns, status
  and comments, and ends done with the verdict as its result; its
  branches are released as it closes, by the release a change's branch
  takes (`forge/branches.md`), never by an effect of kind 15, so that no
  grant of it is needed. A fault past `tier_retries`, or a wait for a
  runner past its bound, holds it (`after-landing-runs.md`, section 2). The
  tier's record names its task while it lives, and no longer once it
  closes, however it ends (`after-landing-runs.md`, section 2); at most one per tier and repository
  lives, so they are bounded by the repositories and `tiers`. A tier's
  sequence counts the runs it has numbered, judged or dropped.
- **The run task's words** start with `Tier NAME run N`, then name the
  repository, the full tested commit and the owned `temper/tier/NAME.N`
  branch at creation. Each part is identified separately: the judged
  run, a failed-test rerun and each bisection probe, with its commit and
  branch, and the forge run number and URL once observed. The procedure
  projects these facts through its own task's specification words in a
  due step (jig's `domain/tasks/lifecycle.md`, 3). New part words are saved
  before its effects are issued; the procedure continues at a deadline
  due immediately. The task keeps them after
  completion, branch release and restart. Both ctl `task show TASK` and
  `task show TASK --until ended` follow missing specification chunks within
  their output bounds, so `asked:` retains these identities beyond the
  overview prefix; when exhausted it
  gives a `task text` continuation at the first undisplayed byte.
  Observations not yet projected
  are kept on the tier row until a step acknowledges their task words,
  including across follower replacement and restart.
- **The completed run task's report** names its tier and judged run, and pairs
  each blamed merge commit with that landing's own failed IDs, distinguishing
  owned and suspected failures. Its summary counts inherited failures without
  attributing them to cleared landings. Singular counts say `1 test` and
  `1 landing`; overflow verdicts say `many tests`.
  The existing report contract's `words` bounds this result. Each attribution
  occupies a line, with a whole quoted ID (backslash, quote, CR and LF escaped)
  and, for a retained landing, `change show TASK`, TASK its first
  change, from its `Verification` row, else from the landing log. Lines that do not fit are
  omitted whole; later shorter lines may still fit. An explicit remainder ends
  the report: `and N more failing tests; change show TASK`, naming the first
  omitted retained landing (`1 more failing test` for a single ID).
  If any omitted attribution is `many`, the
  remainder says `many` too. Full IDs remain in the change read's retained
  `Verification`; no separate record or larger task result is needed.
  Where the run's failures are many and its report kept names (`after-landing-runs.md`, section 3
  ), a line `Its report names: "ID", "ID" and K more.` follows the
  attribution lines the report holds, before its remainder, each id
  quoted as an attribution's is and taken whole in the room the
  attributions and the remainder leave, those left out joining its K
  (2026-11-19).
  Under `wait: landed`, it ends with the tier's red filings waiting for
  room (`red-findings.md`, section 7), kept within its bound.
- **Effects:** the tiers' dispatches and reruns are `landing.md`, 11's
  kinds 16 and 17, their branches' creation kind 9, released at the
  task's close; statuses and comments are kinds 6 and 7; each its run's
  task's.
- **A purpose of its own,** tag 8 in `forge/README.md`'s table: a tier's step,
  the repository (16 bytes), the tier (2), the judged run's number in
  the tier's sequence (8), the operation (1: the branch, the dispatch,
  a rerun, a probe's branch, a probe's dispatch, the branch's deletion,
  the status, a comment), and the probe or the member it is for (2), 30
  bytes with the tag. The tier's record keeps its sequence and each
  operation's state, so a restart settles what was in flight.
- **Reads:** skein's `Read::Artifact { run, name, max_bytes }`, a run's
  artifact by name, unzipped (one file, stored or deflated), cut to
  `max_bytes`, for a tier's report the room of `failing_tests` ids of
  `test_id_bytes` each in its format, within the forge client's
  `answer_bytes`; and `Read::Commits { before, after, page }`, the commits
  of a range with their messages, oldest first.
- **In the store:** `Tier`, keyed by repository and tier, its run in
  flight, its last judged tip and failures, its flakes, its red filings
  waiting for room in its project's findings, at most 8, with a count
  of those not kept (`red-findings.md`, section 7), a held run's cause,
  its last fault or its wait for a runner, its run's clock (`after-landing-runs.md`, section 2) and a
  person's hold, its party, its time and its reason, at most 1024
  bytes (`after-landing-runs.md`, section 2); a probe's part keeps the group it probes, its
  `low` and `high` landings by their index in the range and its tests,
  and the groups waiting, each with its own, named by their places in
  the run's failures, so their bound is the failures' and not a
  name's per landing (`after-landing.md`, 5.2); a judged run's
  posting while its task posts (`after-landing.md`, section 9): the run, its tip, its
  link, the status's check and words, the verdict's blame of each
  landing of the range, and each comment's pull request with the
  landing it is for, or the red landing it repairs and the repairing
  pull request, and the index of the next write, erased by the task's
  last post or its close; the judged run's many names (`after-landing-runs.md`, section 3), set
  at its whole part's read and taken by the next run's start, so that
  its posting, done before that start, reads them there;
  for a tier following goal branches, each goal branch it follows,
  with its goal, its last run's tip, failures and start, and the branch
  its run in flight is for, within the forge's `holds`;
  `Verification`, keyed by repository and landing commit, for temper's
  own landings under `wait: cleared` alone (`after-landing.md`, section 6), made with the landing's `Landed` row and erased
  when its last change closes, so bounded by `changes`: the changes it
  landed, each tier's verdict (cleared, owns failures, or suspected of
  them), the failures blamed on it and the commit that repaired it; the
  `Change` family's landed state means verifying, and `Red` and
  `Cleared` join it; a change keeps the commit of the red landing it
  repairs (`repairs`). Through the baseline's rules (`landing.md`, section
  12).
- **The landing log,** a family of its own, `LandingLog`, per adopted
  repository, oldest first: each landing until every tier following its
  branch has judged it, at most `landings_pending` such, and the last
  `landings_kept` landings that every such tier has judged (*Pending and
  kept*, below), each with its commit,
  the branch it moved, the engine's wall time when it saw the move, its
  changes (none for a push from outside temper), each with its task, its
  pull request's number, so that a late verdict's comment reaches it,
  and its title, at most 256 bytes, since a change's close takes its row
  and temper's `Changes` page still names it (`../client/forge/README.md`,
  section 3; a visitor's public queue merges read its commits and tasks
  alone, 11.2 there), its goal, the tracked ancestor of the change's
  requester that the change's row names (`after-landing.md`, section 7, *A member's goal*),
  and that goal's milestone, where it has one (`milestones.md`), so
  that blame names them once the change's row is gone, with the goal's
  person, title and `Draft:` path and the findings its pull request's
  body names, from the change's row (`red-findings.md`, sections 3 and
  6), the sign-offs counted at its merge, at most
  `SIGN_OFFS`, each its person, head and time (`forge/change/gates.md`,
  2, *Sign-offs kept at landing*), its trailers within `trailers` and
  `trailer_bytes`, and each tier's verdict: an entry for each tier
  following its branch, by the project's settings in force as it is
  logged, `verifying`, set at that tier's verdict to cleared by run N,
  or red by run N with the count of distinct tests blamed on it, owned
  or suspected, and their ids, or many with its run's names where it has
  them (`after-landing-runs.md`, section 3), for `read_ci` (`after-landing.md`, section 9); a ring, written at each move of a watched landing
  branch (`after-landing.md`, section 8), at each merge of temper's, when a change sees its
  pull request merged by someone else or by the forge's own queue, and at
  each verdict, so that a landing's record outlives its changes' close and
  a range's judging. Every adopted repository logs, and its log is erased
  with it. A commit logged already gains the change, within the forge's
  `landing_members`, and no second landing; its tiers' entries stay.
- **Pending and kept:** a logged landing is pending while one of its
  entries is `verifying`; one no tier follows has none, and is judged
  as it is logged. Logging a pending landing when `landings_pending`
  are pending drops the oldest pending one: its tiers still judge it from
  their own records, by commit (`after-landing-runs.md`, section 2), with no member to comment
  on (`after-landing.md`, section 9). A landing whose last entry is judged joins the judged
  ones, and the oldest judged one past `landings_kept` leaves. A verdict
  on a landing the log holds sets its tier's entry, adding it where it
  has none, and saves the log in the decision that keeps the verdict;
  one on a landing the log no longer holds writes nothing there. So the
  log holds at most `landings_kept + landings_pending` landings.
  - *Read from before:* a landing logged before the log kept its tiers'
    verdicts reads, at the restore, `verifying` in each tier of its
    repository whose record still holds it to judge (its pending range
    or its run in flight's), and judged in the others; a member logged
    before it kept goals reads with no goal and no milestone, and one
    logged before it kept their persons with no person, title, draft
    or findings named.
  - *A tier gone from the settings* starts no run (`after-landing.md`, section 11), so a
    start drops each `verifying` entry of a tier its repository's
    settings no longer name; a verdict that tier's run in flight still
    gives sets its entry as any verdict does.
  - *A landing's entries* are at most the forge's `tiers`, as a
    verification's are (2026-11-19). A verdict that would add one past
    it first drops a judged entry of a tier the repository's settings
    no longer name; where there is none, it writes nothing there. A log
    read from before that holds more is brought within the bound the
    same way at its restore.
- **Limits,** the forge's: `landings_kept` per repository (16) and
  `landings_pending` per repository (64), their sum at most the store's
  256 (`../protocol/configuration/limits.md`); `trailers` per landing (8) and `trailer_bytes` per line
  (200); `tiers` per repository (2 in `small`),
  `failing_tests` per run (64, 16 in `small`) and `test_id_bytes` (512),
  each at most what a finding's signature holds, 64 ids of at most 512
  bytes (`findings.md`, section 4), which a start checks
  (`../protocol/configuration/limits.md`), and within the store's 1,024
  bytes of an id (`../protocol/store-versions.md`, 1); and a tier's name at most
  32 bytes. A many's names, wherever kept, hold at most
  `failing_tests` ids. A tier row's `many` and a landing log's red
  entry keep at most 64 of them, the store's ceiling (2026-11-19).
  A tier row's `many` uses `list 64 ShownTest`,
  `ShownTest { test_id: bytes 1024 }`, which the store's wide text
  (`../protocol/store-versions.md`, 1) leaves alone: no older layout holds them.
  A tier row is within 223 KB of the store codec's `u32` worst case at 759 landings,
  which 1,024 names of 1,024 bytes pass; 256 landings of 32 tiers each
  keeping 1,024 ids of 1,024 bytes pass the store codec's `u32` worst
  case, and 64 is the stock `failing_tests` and a finding's signature's
  (`findings.md`, section 4). Past it the row or entry keeps the lowest 64 in
  byte order; its count still counts every blamed test, and a many's
  names add the ids left out to their `more`, so `read_ci`'s remainder
  (`after-landing.md`, section 9) says how many more there are. The entry keeps its ids, or
  a many's names, in one list of at most 64 in the store, a many adding
  only its `more` and `cut`. With the entries bound by `tiers`, the log's
  derived worst case stays within the store's admitted value at the
  stock limits; 32 entries of two lists each would not (169 MB against
  103 MB, 2026-11-19).

A tier's record keeps at most `failing_tests` ids of its last judged
failures, each within `test_id_bytes`; a run past either is many, as
`after-landing-runs.md`, section 3 says. It keeps at most `failing_tests` distinct flake IDs. Once full
it keeps counting the IDs already retained and drops newly seen flake IDs,
without holding the tier (2026-11-04).

## 2. Later

- **Older public queue merges:** public history beyond the landing log's
  ring, or beyond the tasks jig's public index hands, is the client's
  question (`../client/forge/README.md`, section 8). Default: none; keeping a
  landing in the log grants no public access to it (`after-landing-storage.md`, section 1).
