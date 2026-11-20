# Pages in few round trips

Draft, 2026-11-17, from a session with the user that day: "I noticed that
opening https://dev.fabrig.ai/projects/2/work now takes almost a minute on my
slow mobile connection. It performs dozens or maybe hundreds of requests
(judging from the network tab). What can we do about it?" Of the directions
offered, "1 and 2 is a good start": a page's round trips fixed whatever the
project's size, and a page that draws what has arrived. The bundle's own weight
is not this draft's (section 9).

## 1. In one page

- **Work costs two round trips after its bundle, whatever the project's size**
  (section 2): the stream, whose hello carries `me`, then one exchange that
  carries everything the page and its frame open. A focused test holds Work to
  it (section 7).
- **One exchange carries many messages** (section 3). jig's HTTP mapping gains
  a batch, its members reserved as they would be apart, its answers streamed
  back as they are released.
- **A row carries what the page draws of it** (section 4). This settles jig's
  open question on bundling parts: a task query names an application's part,
  and each row comes with its own, marked when what it is made from changes.
  No read is made per row, and no read waits for another row's answer.
- **One fact, one read** (section 5): no watch beside a wider one of the same
  subject, no reread on opening, nothing of the party's own waiting for `me`,
  nothing opened that the page doesn't draw.
- **A page draws what has arrived** (section 6): each section as its list
  comes, top down, and each row whole from its first drawing.

## 2. Today, and the budget

- **What a cold open of Work sends today.** A survey of the code at
  `73d3fc2a69` counted it; the evidence is outside this repository, and the
  figures are read from the code, not measured. Besides the bundle and the
  stream, about 28 exchanges are made for the frame and the page's lists,
  then one per goal row for its report (up to 32, live and ended), one per
  distinct change in the changes' rows (queued, in flight and landed, not only
  this week's), five to eleven per drawn milestone (its watch, its task, its
  totals and its goals' ended pages), one per goal whose requester isn't yet
  known to be a milestone, and the project's notes by page. About five more
  are rereads that the lists' first deliveries ask for. By that count, a
  project with 32 goals, 40 changes and four milestones makes about 130.
- **Why the count turns into a minute.** The page keeps eight exchanges in
  flight (`temper-client-shell-web`'s `EXCHANGES`), and the goals' reports are
  read one at a time, live rows first, each waiting for the last: 32 goals are
  a chain of at least 35 round trips. At 300 ms a round trip that is ten
  seconds, at 800 ms half a minute, before the bundle's own time on a slow
  link. Reads past the client's bound of 32 at once are refused locally, and a
  change's task so refused is not read again, so a big project's rows can stay
  drawn without their details.
- **The budget.** A cold open of Work, a reload among them, under any of its
  filters (Goals and Changes share its reads, `../client/pages/work.md`,
  section 1), is drawn whole in **two waves of two requests** after its page
  and bundle, whatever the project's size: the stream, then one batch.
  - *A wave* is every exchange the client has sent and that waits for an
    answer, all answered at once; a page drawn whole and sending nothing more
    has taken its last.
  - *Drawn whole* is every row of the page's windows, with everything its card
    draws, and the frame's rail and bar.
  - *Outside it:* a person's acts (Older, opening a goal); values the stream
    brings later; a lost connection's retries; answers busy; and, on a reload,
    the keyed requests the small store kept, which go on their own routes
    beside the batch, with their keys, and hold nothing of the page.

## 3. One exchange, many messages

- **A batch,** `POST <prefix>/v1/batch` in jig's HTTP mapping (jig's
  `protocol/client/default-http.md`, section 1): its body a list of reads,
  watches and unwatches, each the record its own route takes. Its members and
  its bytes have bounds of the engine's, which it checks. Keyed requests keep
  their own route, each with its key, its backoff and the answer kept for it.
- **Its answer streams.** `200`, its body a sequence of replies, each framed
  by its length and naming its member, written in the order they are
  released: a read's once every commit it reflects is durable (jig's
  `client/domain/contract/README.md`, section 4), a watch's opening once
  admitted or refused. One slow read holds back nothing else. The engine holds
  at most the route's bound of replies unwritten, and writes them as the
  connection takes them. A watch's values come over the stream, as now.
- **The shells deliver a body as it comes.** The client end reads a batch's
  body a piece at a time, as it reads the stream (jig's
  `protocol/client/default-http.md`, section 2), and hands each reply to its
  member at once: skein's browser fetch and jig's native connections give an
  exchange's body in pieces, not only at its end, so the early answers reach
  the browser, the terminal and the TUI early.
- **Each member's answer is its own.** One answered busy, refused or not
  served is that member's alone, handled as on its own route. A batch cut
  short leaves its answered members as they are; each unanswered one is lost
  as it would be alone: a read ends unreachable, and a watch's opening or
  closing with no answer drops the stream, whose watches the client opens
  again (jig's `client/domain/client/watches.md`, section 1). The batch holds
  its exchange's place until its body ends.
- **Admitted as one exchange, reserved as many.** A batch takes one call of
  the mount's room and one exchange of `clients.exchanges_per_client`. Each
  member reserves what it would alone, a read's or a watch's place and its
  bytes, against per-sign-in bounds of reads and watches that are sized to
  the client's own (32 reads and 31 watches at once, jig's
  `client/domain/client/README.md`, section 11). A batch admits nothing the
  same messages sent apart could not, and the stock's two exchanges per
  sign-in no longer answer a page's reads busy.
- **The link sends once per settling.** The client root routes each event of
  an iteration and settles what follows from it (the hello, `me`, the frame's
  and the page's demands) before its link sends anything. Then the link sends
  what it gathered: one batch while it holds more than one read, watch or
  unwatch, its members in the order they were asked for, and a message alone
  on its own route, as now. `EXCHANGES` stays eight.
- **The stream names the one it replaces.** A reload opens its stream naming
  the stream its small store kept, and the engine ends that one first, its
  sign-in's credential checked as `stream-gone` checks it (jig's
  `protocol/client/wire.md`, section 2), so no `stream-gone` is sent and
  waited for before the stream.

## 4. Rows carry their parts

- **jig's open question, settled** (jig's `client/domain/contract/README.md`,
  section 9, *Bundling parts*): a task query, read or watched (jig's
  `client/domain/contract/README.md`, 4.1), may name an application's part,
  by its owner, its kind and the owner's version. The engine's root asks the
  application's connector for each row's part as the row is made for an
  answer or a delivery. The row carries it in its owner's versioned envelope,
  as an application's read is answered; a version not served or a kind
  unknown answers that part so, typed, and the row comes whole without it.
  jig reads nothing of a part's bytes.
- **Made as reads are.** A part is made from live state or the store, its
  loads bounded and admitted as a read's, its visibility its row's, and its
  row is released only once every commit the part reflects is durable. A part
  that can't be made comes as failed, typed, and its row draws without it
  until the row comes again; no read per row stands in for it.
- **Marked with its row.** Each kind names what it is made from, among its
  task's line, its task's notes, its delegates' lines and the application's
  own records of the task. A commit to any of them marks the row in every
  watch whose query carries that kind, and moves the ended group's mark
  (section 5) where the row has ended. A note made, edited or deleted is such
  a commit: today a note moves no task's line (jig's
  `domain/engine/views.md`, section 3), so this mark is new.
- **Bounded by its kind.** A part's bytes per row are its kind's, fixed by
  the application within the window's; what it holds of a list is bounded in
  rows too, in a stated order, saying how many more. A connector cuts a text to
  what fits, saying so; it never drops a part for a read per row.
- **temper's parts,** each from what the page draws today:
  - *a goal's row:* whether it has a report, its newest report's headline and
    when it was written, and the milestone that gathers it, if any. Work's
    and Home's report reads, one at a time, go, and so do the reads of a
    goal's requester to learn whether it is a milestone.
  - *a chat's row:* its document's description, the first of its notes not
    `work:`'s. The project's notes by page go. The goal a chat started stays
    the party's own fact, which the dismissed facts' watch already carries in
    the same batch.
  - *a milestone's row:* whether it is whole or so far; its totals (done,
    failed, closed, live, ended, its last landing, its spend), exact over all
    its goals, made from jig's records and the forge's, both named as its
    inputs; and its goals as its card draws them, each with its title, its
    state, the done and total of its delegates and its report's headline, at
    most 16, the live by priority and then the ended latest first, and how
    many more. The milestone's task read, its totals read, its goals' watch and
    their ended pages go, and a milestone's goals outside the project's goal
    windows still draw.
- **temper's own rows name theirs too.** The `changes` subject's queued,
  in-flight and landed rows (`../client/forge/records.md`, *A project's
  changes*) each name the change's goal, the chat it came from and its
  milestone, each by number and title, as the engine knows them from the
  change's task, and a landed row names its signers as the project's people
  name them; the change's task reads go. The trials' rows carry their running
  trial's run wait, a change of which marks the trials' subject, so the frame
  follows a running trial without a watch of its task.

## 5. One fact, one read

- **`me` comes with the stream.** Every stream watches `me` from its opening.
  Its hello holds the engine's served ranges first, as now, so a client out of
  range is told so before it decodes anything else; then `me`'s value, in
  jig's versioned envelope at the version the stream's opening names. Later
  values come as a `me` watch's do, and the client opens no watch of `me`.
  What waits for `me` (the templates' labels, the party) is ready when the
  stream is.
- **The party's own and the deployment's need no `me`.** Their reads and
  watches name no party (jig's `client/domain/contract/README.md`, 5.5), so
  they go in the first batch with everything else.
- **No narrower twin.** A watch whose subject differs from an open one only by
  a smaller window is served by the open one: the rail's facts that a project
  has goals or milestones come from the page's lists while a page holds them.
- **No reread on opening.** Each is read once at opening, and again only when
  a mark says it moved:
  - *a list's ended group:* its watch carries the ended group's mark, a number
    the engine moves whenever its ended group would read otherwise (a task its
    query selects ending, an ended row's part changing), given with the
    opening's ended read and with every delivery. Comparing live rows can't
    tell, since deliveries coalesce (jig's `client/domain/contract/README.md`,
    5.2).
  - *the needs' selection:* read again as `me`'s `needs_total` or the inbox's
    mark moves, never for the goals' first delivery.
  - *a setting:* its read answers with the revision it read, and the settings'
    watch names each slot's revision; a slot is read again only when the watch
    names a newer revision than its read did, whichever came first.
- **Nothing opened that the page doesn't draw.** Work opens no chips watch for
  its chats' goals, and no frame's read holds the page back.

## 6. Drawing what has arrived

- **The frame draws with the stream.** The rail and the bar draw once the
  hello's `me` is in, before the batch's answers.
- **Sections draw top down, each as its list comes.** A section draws once its
  own list and those of the sections above it are known, so nothing the person
  is reading moves when a later one arrives; the link puts the page's messages
  in the batch in the page's order, so the top's answers tend to come first.
  No section waits for another's details or for the frame's own reads. Today
  Work's heading and every section wait for the milestones', goals', chats',
  notes', changes' and drafts' lists together (`../client/pages/work.md`,
  section 1).
- **The heading waits only for its counts,** its line held so that nothing
  below it moves when it comes; the kicker, the lede and the filters draw at
  once, as now.
- **A row is whole when first drawn.** What its card draws comes with it,
  its signers' names among them (section 4), so no headline, milestone,
  provenance or name fills in later and moves a row.
- **The terminal and the TUI** draw the same tree from the same client root,
  and gain the same.

## 7. Proving it

All focused, with no walk or measure in a plan.

- **The budget is a test.** On the client root, with an engine double that
  answers each wave's exchanges at once, a cold open of Work counts its waves
  and its requests until it is drawn whole and sends nothing more: on a small
  project and on one at the client's bounds (32 goals live and ended, 16 chats,
  16 milestones with their goals, 64 changes queued and landed, 64 landed
  drafts), both within the budget and equal; into each of its filters, Goals'
  Needs among them; and a reload with kept requests, the page within its budget
  and the kept requests on their own routes beside it. The budget is one
  constant, beside the page.
- **The batch:** replies in the order released, one slow read holding back
  none, and each delivered to its member as its piece arrives; one exchange
  and one call admitted, each member reserved as alone; a batch cut short, its
  answered members kept and its unanswered ones lost as alone; a busy watch
  opening retried after its backoff and a busy read answered busy, as on their
  own routes; a batch's member and byte bounds refused by the engine.
- **Parts:** a row delivered again when what its part is made from changes, a
  note made, edited or deleted among them; an ended row's part moving its
  group's mark; a part failed, cut, or of a version not served, its row whole
  without it.
- **Marks:** an ended group read again only when its mark moves, a task that
  started and ended between two deliveries among them; a setting read again
  only for a newer revision, in either order.
- **Recordings:** a journey the changed wire stales is recorded again with the
  change where the change can, as R1 to R3 did, and otherwise removed,
  recording and prints, and recorded again with `again` after landing
  (`../client/web/journeys.md`, 8).

## 8. Upgrades

- jig's client wire and HTTP mapping and temper's client family move their
  versions, unbridged: a page of an earlier version is refused by name and
  reloads to the engine's bundle (`redeploys.md`, section 8). The hello keeps
  its served ranges first, so that refusal stays readable whatever `me`'s
  layout.
- A part is versioned by its owner, as the owner's reads are, so temper's
  parts move with temper's family and jig's task rows with jig's.
- Nothing stored changes: parts and marks are made when a row is, and kept
  nowhere. The single-message routes stay, for a message alone.

## 9. Not this draft's

- **The bundle's weight:** Brotli beside gzip, a smaller module, and files named
  by their own bytes, so that a stylesheet's change doesn't move the module's
  address (skein's `ui/web/bundle.md`).
- **A second visit drawn at once** from answers kept in the browser: the small
  store keeps requests and the stream's identity, and no answer.
- **A page composed by the engine,** one read per page: it ties the engine to
  the client's pages, where batches and parts stay jig's and temper's
  mechanisms.
- **More exchanges in flight:** batches make eight enough, and a raise would
  meet the engine's per-sign-in admission first.
- **Home's budget** (`../client/pages/home.md`, section 1): Home gains from
  sections 3 to 5 at once, its goals, chats and milestones being Work's, but
  its inbox cards are still read one per card (section 10).

## 10. Open questions

- **Home and the inbox's cards:** each card reads its question, proposal or
  task, sometimes its source's too. Default: the inbox's rows carry what their
  cards draw, as task rows carry their parts, and Home takes the same budget
  in a draft of its own, once Work is within this one's.
