# Decided for you

This document holds decided for you, split from [`home.md`](home.md).

## 1. Decided for you

- **Decided for you** (`../../draft/what-reaches-you.md`, section 6;
  `../../domain/forge/attention.md`, 5): what agents decided for the person, any task's,
  a goal's, a chat's or a delegate's, whether or not the person was away.
  Each row shows for 24 hours from its `at` on Home and on *Needs you*,
  then lives only in its task's history, where its call keeps its answer.
  - *Its documents:* the subject `decided` (`../forge/records.md`) of each project the open
    page shows, held by `Pages` while it is open, `pages::decided`: Home's
    project; *Needs you*'s filter's project, or, unfiltered, the shown card's
    project and then `me`'s projects in its order, at most `DECIDED_PROJECTS`,
    4, in all, each watch counted in the client's `watches`. Other pages hold
    none. A watch stays from one page to the next while its project does, so
    "Answer in a row" keeps Home's. A watch refused or ended has no rows. The
    same documents' `asked` rows give a need its reason (`needs-you.md`, 1, *Why it
    reached you*).
  - *A row shows* while its `at` is less than 24 hours before the wall; the
    build asks to wake when the oldest stops showing. With none showing, the
    page draws nothing of this; nothing of it is drawn while To begin shows.
  - *The region,* named by its heading: a `section-head`, the level-2
    heading "Decided for you · 4", every row showing counted, "4+" where a
    document is `cut` or, unfiltered, `me` holds more projects than are
    watched; at its right "Limits", to the project's Settings, on Home and on
    a filtered *Needs you*, and on the unfiltered *Needs you* "for 24 hours,
    then in each task's history", muted, as the whole story's `needs.html`
    draws its head, a row being any task's. The region is named by that
    heading on both pages. Then `temper-decided` (the mock's `.decided`, a
    top rule), a `temper-decision` a row (`.decision`), at most 7, the newest
    kept, oldest first: mono "TITLE · 01:05", TITLE the task's, its `at` by
    `../view.md`, 6's *clock*, and on the unfiltered *Needs you* "PROJECT · TITLE ·
    01:05"; a row whose task is under a milestone, by its read document's
    lineage (`milestones::lineage`, `milestone.md`, 1), "Store shape › TITLE
    · 01:05", the milestone's name first (`../../draft/milestones.md`, 5.1),
    after PROJECT where PROJECT is said; muted, the question; `temper-decision-took` (500), what it took;
    `temper-decision-allowed` (13.5 px, muted), what allowed it (below).
    The agent's why is left to the task's page, which keeps its call
    whole (`../../draft/home-for-a-person.md`, 3): no row draws it. At
    the right, by the first that holds:
    - the task ended: `temper-decision-late` (`.late`), "landed Sat 19:01",
      its ended time, or "ended Sat 21:22" for one that did not end done.
      A landed one takes `temper-decision-landed` too, whose `::before`
      draws a tick, "✓" in `--gf-good`, before "landed", by style alone,
      so the terminal prints "landed Sat 19:01";
    - overturned: `.late`, "Overturned: it asks you again";
    - the task live: "Overturn" (`.btn.small`);
    - its read not delivered, or refused: nothing, TITLE reading "Task N".

    An overturned row's took is struck through, in `--gf-faint`
    (`temper-decision-overturned`), ended or not. Each shown row's task is
    read (`Read::Task`), as the inbox reads its cards, within `reads`, a read
    kept from one page to the next while its row shows.
  - *What allowed it,* one sentence from the row's reach, the object
    alone: "Allowed: OBJECT." for a reach of class `decide`, and for `ask`
    with `away`, OBJECT the reason's subject as Stop asking names it
    (`needs-you.md`, 1): so "Allowed: test-only changes." and "Allowed:
    wording choices." The kicker and the headline of an Away spell say
    that the person was away, so the row doesn't. A set's name is the
    project's classes' (`settings.md`, 2), "a path set" until they are read. A reach that
    does not decode draws no sentence.
  - *The same row everywhere:* these parts are the row's, one function's
    (`decided::decision`), on Home, Home on All projects and Needs you.
  - *On Home* (`home.md`, 1, *Home on any project*), after *Needs you*. While the party's Away row there is on, or its spell ended less
    than 24 hours before the wall, the intro takes the spell's form, in the
    intro's one function, whether or not a row shows:
    - *the kicker:* "While you were away · 23:10 to now", or "· 23:10 to
      08:03" once off, by `../view.md`, 6's *clock*: `since`, then `off`, or `until` once
      past;
    - *the headline:* "57 changes landed." where `landed` less the row's
      `landed` is above 0, "One change landed." for 1, nothing for 0; then
      "Four things were decided for you." in number words to "Nine", digits
      after, or "One thing was decided for you." in place of the needs
      sentence; with no row showing, the needs sentence keeps its place,
      so "57 changes landed. Nothing needs you.", or the needs sentence
      alone where nothing landed. The lede and the byline as *Home on any project* has them;
    - *Needs' head:* "Still needs you · 2", or "Still needs you" with none,
      and the rest of *Needs you* as it is.

    With no such spell, the row undelivered or the `away` watch refused or
    ended among them, the intro and the needs' head are *Home on any
    project*'s, and the region alone says what was decided.
  - *On Needs you,* after the card's `.next-up` and its *Answered*, and in
    the done state after the answered lines, before "Home"
    (`needs-you.md`, 1).
  - *Overturn* sends `Overturn` (`../forge/records.md`), from either page, for a row it shows
    whose task is live and that is not overturned. Answered, the client sends
    the task its existing message (`Ask::Message`), in words: "I overturn your
    decision. QUESTION You took TOOK. Ask me again with message, as a
    question, and wait for my answer."; sent, the toast "Overturned: it asks
    you again, under Needs you." Its question comes as any other. A refused
    message (the task ended between) leaves the row marked, and the toast "It
    has ended; a new goal can undo it." `Overturn` refused, overturned or
    unknown: no message, the toast "That was overturned already." The
    session's marks hold on both pages.
  - *In the terminal* the region prints as a heading and a paragraph a row,
    its parts one readable run of words, Overturn a numbered button.

## 2. Open questions

- **Decided for you across many projects** (`home.md`, 1, *Decided for you*): the
  unfiltered *Needs you* watches at most four projects' `decided`, the shown
  card's first, within the client's watches; a person in more reads the
  others' rows on each project's own pages, the count marked "+". A subject
  of the party's rows across the deployment would lift the bound; default:
  four.
