# A project's waits

Provisional, 2026-11-26. The Waits page, reached from Work: every live
wait in a project, as jig's `waits` read lists it (jig's
`client/domain/contract/reads.md`, 1.2), for a person following what
has stopped; and the words in which a page says a wait, which a
milestone's goal rows say too (`milestone.md`, 3). Written from
`../../draft/bounded-waits.md`, 9.

## 1. Its page

- **Its address:** `Waits { project, filter }`, at
  `/projects/<project>/waits`, its filter, `WaitsFilter { kind:
  Option<WaitKind>, mine: bool, due: Option<u32> }`, in its query:
  `kind=KIND`, KIND a kind as jig names it (`party`, `sign-off`);
  `mine=1`; `due=SECONDS`, from 1 to 2,592,000, 30 days; any of them
  together (`../web/addresses.md`, 1). A query that doesn't parse is
  the address's as any page's is.
- **Reached from Work:** Work's filters hold "Waits", a link to it
  (`work.md`, 1, *Its filters*). Work makes no read of waits, so the
  link has no count and is drawn always.
- **Its read:** the `waits` read of the project, its filter the
  address's, `mine` sending the signed-in party's number as its
  `party` and `due` its seconds as nanoseconds. It reads the first page
  when the page opens, again on a move to it, a reload, or its
  "Refresh"; "More waits", while the answer has a cursor, reads the
  next page and appends it, a person's act. It is not watched (jig's
  `client/domain/contract/reads.md`, 1.2, *Read, not watched*). A cold
  open is drawn whole within Work's budget record, `Budget { waves: 2,
  requests: 2 }` (`work.md`, 1, *Its budget*): the stream, then one
  batch carrying the read with the frame's and the session's.
- **Its head:** the kicker "PROJECT · Work"; the level-1 heading
  counted from the rows read, "N waits", "One wait", or "Nothing
  waits." with none, "N+" while the answer has a cursor, the number in
  words from one to nine and in digits from ten, as Work's heading;
  at its right, muted, "overdue first, then by bound". Then its
  filters, chips as Work's, each a link: "All", "Mine", "Due within an
  hour" (`due=3600`), and, under a kind, that kind's name; the current
  one on the signal's ground. Then "Refresh".
- **Its rows,** `temper-wait-row`, in the read's order, one a wait: its
  task's title, a link to its `Task`, "TN" where the chip holds no
  title; then the wait's words (2), the hearer's among them; a row whose
  wait is overdue marked "Overdue" in the signal's colour. The kind's
  name is a link to this page filtered by it.
- **Empty:** "Nothing waits on PROJECT." with no filter; "No wait
  matches." under one.
- **Before its read answers** it claims nothing: the heading's line is
  held as Work's is (`work.md`, 1, *Its head*). A read refused or busy
  says so as a page's read does, and "Refresh" reads again.
- **The terminal:** the view is shared: the head, the filters as links,
  then a line a row.
- **Its test,** focused, in `temper-client-view`'s suite, with the
  engine double Work's budget test uses: rows drawn in the read's order
  with their words, an overdue one marked; "More waits" sending the
  cursor and appending; each filter's address sending its filter; the
  empty words; a cold open within the budget record.

## 2. A wait's words

One function of the client's words, shared by this page and a
milestone's goal rows, says a wait (jig's `domain/engine/waits.md`, 2)
from its record and the view's clock (`../view.md`, 6), redrawn as the
clock moves:

- **What for,** by kind: an entry, its kind and number at its holder,
  "question 3 at Sam", the holder "you" for the reader, a member by
  name as Work names who signed (`work.md`, 1, *Who signed*), "role R",
  or "task N"; "question N to task R"; "task N" for a dependency or a
  delegate; "resource NAME", with "held by task T" where one holds it;
  the next run's wait as the task's page says it; "news: subscriptions
  S, T", "a timer at TIME" or "nothing armed"; a procedure's step and
  the things it names, in the forge's words for their codes
  (`../../domain/forge/waits.md`, 3), a code it doesn't know by its
  number.
- **Since:** "waiting 3 h", its age as a span (`../view.md`, 6).
- **Its bound,** to come: "passes up in 20 min", "holds in 4 h", "is
  overdue in 2 h", "gives up in 5 min"; past: "overdue for 2 h" for an
  *overdue* bound, the others "due now", since their outcome is taken
  at once. With none: "no deadline", "standing", "behind task N" or "an
  operator's hold".
- **Who hears,** on this page alone: "then HOLDER hears", the holder in
  the words of *What for*, where one would.

Joined by " · ": "question 3 at Sam · waiting 3 h · passes up in 20
min · then task 812 hears".

## 3. Open questions

- **A watch of the waits,** with marks as bounds pass: if a person
  keeps the page open to follow a stall, the page could read again as
  its first row's bound comes. Default: "Refresh" and a move to the page.
