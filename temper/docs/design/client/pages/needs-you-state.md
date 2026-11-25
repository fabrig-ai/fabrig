# What Needs you keeps

This document holds what the client keeps, split from [`needs-you.md`](needs-you.md).

## 1. What the client keeps

- **What the client keeps** of what needs the person: in `Pages`, not on
  a page, for this session, gone on reload or on sign-out, none of it a
  stored record, and at most 64 each of the cards marked later, the answered
  lines, the refused answers and the proposals of settings, the oldest going
  first, a line still held never:
  - *The needs' rows,* `Inbox::rows`, beside its cards: every row of the
    needs' groups its address selects, read a page at a time through to
    the last (jig's `domain/people/README.md`, section 5, *Bounded*), at
    most 64, in the inbox's order, oldest first. A filtered address reads
    its needs' pages with `Read::Inbox`, each answer's `after` read on at
    once until none; the unfiltered address takes the watched document's
    window and, where its needs run past it (its `next` set and no update
    in the window), reads the needs as a filtered address does, from the
    first page. Each refresh reads them again; the rows read before, and
    whether they were cut, stand until the new read's last page has
    answered with its value, so no count falls back or gains a "+" while
    it reads (`README.md`'s opening: a page redraws from whole answers
    alone). The read is *cut* where its list is: where it stops at 64 rows
    with a page still after them, or where a page it answered says `cut`
    (jig's `InboxPage::cut`); `Inbox::cut`'s needs' half says so, and
    nothing else sets it. A page that answers without its value (busy,
    unreachable, refused, past the client's reads, its watch ended) is no
    answer: the read stops there, and the rows, their cards and their cut
    stand as the last whole read left them. Before any read has answered
    whole the needs are not known, and nothing a read has answered so far
    is counted: Home's headline says the project's name (`home.md`, 1),
    and no "N+" stands for a read that stopped. A read that ends
    transiently, busy, unreachable or past the client's reads, is asked
    again from its first page while the page is open, waiting one second,
    doubling to at most thirty, as a conversation's activity reads are
    (`README.md`'s opening; `pages::activity::Asking`), the wait back to
    one second once a read answers whole; a refused or ended one waits for
    the next refresh. The rows are what is counted; cards are read for
    the first of them in the inbox's order, as many as the needs' share
    of `card_capacity` (half, the updates' window holding the rest), and,
    on *Needs you*, for the open one (`needs-you.md`, 1, **Needs you**), which takes the
    place of the last of them. A row whose card is not read has no
    severity and concerns no findings (`home.md`, 1, **Findings, drawn**,
    *Counted*).
  - *Later,* the cards marked later. *The open cards* are the needs'
    rows, each with its card where it is read, less those answered, with the approvals
    (`needs-you.md`, 1, *An approval*) and the findings the person holds (`needs-you.md`, 1, *A finding
    that is the person's*), in one order, `findings::Session::all`'s, which
    Home, Home on All projects and this page share
    (`../../draft/home-for-a-person.md`, 3): those marked later last; before
    them, a card about findings by its highest severity, high first, then
    medium, then low, any other card with high's; within each, as before,
    the oldest first, the findings joining the inbox's cards and the
    approvals by when they were filed. A card's severity is
    `findings::Need::severity` (`home.md`, 1, **Findings, drawn**,
    *Counted*): a group's card (`needs-you.md`, 1, *A group of findings*) is about
    findings, ranked by its highest and placed by when it was asked. So a medium finding filed an hour ago comes after a
    question asked a minute ago, and before a low finding filed a day
    ago. The head's "oldest first" stays: within a severity, it holds.
    Their count, which Home's headline and head, Home on All projects'
    headline, this page's intro and head and the terminal's "Needs you N"
    say, is every open card: each open need once, its card read or not,
    as `me` counts its needs, with the approvals and the findings the
    person holds; "N+" only where the needs' rows, `approvals` or the
    findings' read is cut, never because fewer cards are read than rows,
    nor because a read ended without its value. Every place that says it
    (Home's headline and head, this page's intro and head, the rail's count
    on the page, below) takes it from the one `findings::Session::counted`,
    so two of them never say two counts of the same needs.
  - *Answered,* a bounded list: each card's object, project, title,
    label and when. The label is the choice's words, or `Replied: “…”`,
    its words cut at 57 characters plus "…" past 60. A line shows on
    *Needs you* for 10 minutes and on Home for 2, and an answered card
    leaves the open ones at once.
  - *At,* the card a Home row asked for, until it is answered or marked
    later, or the person leaves the page.
  - *The held send.* jig has no retraction, so a choice or a reply is
    held in the client for 5 seconds, a deadline of the domain's as
    `next_deadline` has. The card moves to answered at once, with the
    toast "Answered: LABEL" and its "Undo". Undo restores the card, a
    reply's words with it, unseen until the form opens, and sends
    nothing. At 5 seconds the `Answer` goes, its words the choice's
    or the reply's; after that its line shows no "Undo". A refused answer
    puts the card back with the refusal's words on it, as does an answer
    the engine did not keep. A tab closed inside the 5 seconds sends
    nothing and the question stays open. Accept, Reject, Release, Leave
    held, Take and Hand back send at once, with no Undo, and leave no
    answered line.
  - *A press stays under the finger.* When the inbox's document changes,
    a card still in it keeps the page it shows until its new read answers,
    and keeps it if that read fails. At a filtered address the cards stay
    as they are until the section reads have answered, a section whose read
    failed keeping its own. So a choice on the page stays pressable, in the
    same element, through the read: nothing a change of the inbox does draws
    a card without its choices, or takes one away, but the card leaving the
    rows. A press of a drawn choice that a bound refuses (its words, or 64
    answers already held) moves nothing and says so, in the toast "That
    answer wasn't sent. Press it again.": an answer never goes unsent with
    nothing shown.
  - *The toast,* one at a time, 4.5 seconds, with one optional act; drawn
    with the status role at the end of the frame, in a region named "Notices"
    (skein's `ui/README.md`, 12: a page's child is a landmark) that has
    no look of its own, in the mock's `.toast`:
    fixed, centred, bottom 28 (88 on a phone), an ink ground.
  - *The rail's count:* one standing `inbox` watch while a project's
    frame shows, the rail being what reads it, so that a client on no
    project's page asks nothing more; its needs' rows counted by the open
    project, less those answered, a row counting in each project it shows in
    (jig's `client/domain/contract/documents.md`, 1: `shows_in`), so that a proposal of
    settings counts in the projects it belongs to, and for one of the
    deployment's in each the person owns; "N+" where its needs run past its
    window, its `next` set and no update in the window, as the unfiltered
    address reads it (*The needs' rows*), never because updates run past
    it; and no count before it is delivered. A card marked later still
    counts. The open project's rows of `approvals` add to it, "N+" where
    that document is cut too. On a page whose inbox is filtered by the open
    project alone and known (Home, a project's *Needs you*), the rail says
    that page's count, `findings::Session::counted`'s, the number and "+"
    the page's head says, so the rail and the page never disagree.
    On a page across projects (`../view-navigation.md`, 1, *All projects*) it counts
    what *Needs you* across them shows: each of the needs' rows once,
    whatever projects it shows in, less those answered, and every row of
    `approvals`, "N+" where either document is cut; so it is never the sum
    of the projects' counts, which counts a row of several projects in each.
    `me`'s `needs_total`, with every row of `approvals` added, is shown
    nowhere outside a project: the top bar offers no navigation (`../view.md`, 1).
  - *The approvals,* the forge side's watch of the subject `approvals` (`../forge/records.md`),
    held for the session as `dismissed` is, from when `me` is first delivered
    while the link is live: a watch refused or ended is kept as such, with no
    rows, and not opened again until the next sign-in.
  - *Stop asking,* the one open (`needs-you.md`, 1): its card's object,
    the words typed for a `Words` offer, the act out and its refusal, gone
    when it closes; and the Keep asking out, by its key, until answered.

## 2. Open questions

- **Later and the answered lines, kept** (`needs-you.md`, 1, *What the client keeps*; the
  request of 2026-11-05 for *Needs you* decides session order and no store
  record): a card marked later, or a line answered, surviving a reload or
  reaching another device, which takes a stored record. Default: this
  session's alone.
- **A goal hearing a choice, as a story** (`needs-you.md`, 1, *What the client keeps*;
  `../../domain/agent/agents.md`, 2): that request asks one live walk of a scripted goal
  asking in the line form, a person pressing its recommended choice on Home and
  the goal hearing those words, whose scripts are throwaway. No world keeps it:
  the parser, the stock sentence, the session and the view each hold their own
  side. Default: those tests.
