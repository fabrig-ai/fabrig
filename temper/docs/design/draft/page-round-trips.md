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
  a batch, admitted as one exchange, its answers streamed back as they are
  made.
- **A row carries what the page draws of it** (section 4). This settles jig's
  open question on bundling parts: a task query names an application's part,
  and each row comes with its own. No read is made per row, and no read waits
  for another row's answer.
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
  - *Outside it:* a person's acts (Older, opening a goal), values the stream
    brings later, a lost connection's retries, and answers busy.

## 3. One exchange, many messages

- **A batch,** `POST <prefix>/v1/batch` in jig's HTTP mapping (jig's
  `protocol/client/default-http.md`, section 1): its body a list of reads,
  watches and unwatches, each the record its own route takes, at most the
  client's bounds on reads and watches at once (jig's
  `client/domain/client/README.md`, section 11) and the route's byte bound.
  Keyed requests keep their own route, each with its key, its backoff and the
  answer kept for it.
- **Its answer streams.** `200`, its body a sequence of replies, each framed
  by its length and naming its place in the batch, written as each is made:
  live state's at once, the store's as they load, so that one slow read
  holds back nothing else. The client end reads the body a piece at a time,
  as it reads the stream (jig's `protocol/client/default-http.md`, section 2),
  and gives each reply to its message as it comes. A watch's values come over
  the stream, as now.
- **Each message's answer is its own.** One answered busy, refused or not
  served is that message's alone, handled as on its own route. A batch lost
  is each message lost: its watches opened again after their backoffs, its
  reads ended unreachable (jig's `protocol/client/default-http.md`,
  section 4).
- **Admitted as one.** A batch takes one call of the mount's room and one
  exchange of `clients.exchanges_per_client`, whatever it holds, so the
  stock's two per sign-in don't answer the rest busy; each message within it
  meets the component's other bounds, a stream's watches among them, as it
  would alone.
- **The link sends one whenever more than one message is ready.** At each
  iteration the client's link takes every read, watch and unwatch it holds
  ready and sends them as one batch, in the order the page asked for them; a
  message alone takes its own route, as now. `EXCHANGES` stays eight.
- **The stream names the one it replaces.** A reload opens its stream with the
  identity of the stream it held before, and the engine ends that one first,
  so no `stream-gone` is sent and waited for before the stream.

## 4. Rows carry their parts

- **jig's open question, settled** (jig's `client/domain/contract/README.md`,
  section 9, *Bundling parts*): a task query, read or watched (jig's
  `client/domain/contract/README.md`, 4.1), may name an application's part,
  by its owner, its kind and its bytes per row. The engine's root asks the
  application's connector for each row's part as the row is made for an
  answer or a delivery, and the row carries it as the family's bytes, or
  none. jig reads nothing of it.
- **A part changes only with its row.** A part is made from its row's task
  and that task's delegates alone (its document, its notes, its delegates'
  lines, the application's records of it), each of whose changes moves the
  row, a note written moving its task's activity, so that a watch delivers
  the row again with its new part and keeps no mark of its own.
- **Bounded by its kind.** A part's bytes per row are its kind's, fixed by
  the application and within the window's bytes; a part that would pass them
  is cut by its connector to what fits, as a headline is cut, never dropped
  for a read per row.
- **temper's parts,** each from what the page draws today:
  - *a goal's row:* whether it has a report, its newest report's headline and
    when it was written, and the milestone that gathers it, if any. Work's
    and Home's report reads, one at a time, go, and so do the reads of a
    goal's requester to learn whether it is a milestone.
  - *a chat's row:* its document's description, the first of its notes not
    `work:`'s, and the goal it started, if any. The project's notes by page go.
  - *a milestone's row:* whether it is whole or so far, its totals (done,
    failed, closed, live, ended, its last landing, its spend), and its goals'
    chips, at most its card's. The milestone's task read, its totals read, its
    goals' watch and their ended pages go.
- **temper's own rows name theirs too.** The `changes` subject's queued,
  in-flight and landed rows (`../client/forge/records.md`, *A project's
  changes*) each name the change's goal, the chat it came from and its
  milestone, each by number and title, as the engine knows them from the
  change's task; the change's task reads go. The trials' rows carry their
  running trial's line, so the frame follows a running trial without a watch
  of its task.

## 5. One fact, one read

- **`me` comes with the stream.** Every stream watches `me` from its opening:
  its hello carries `me`'s value, later values come as a `me` watch's do, and
  the client opens no watch of `me`. What waits for `me` (the templates' labels,
  the party) is ready when the stream is.
- **The party's own and the deployment's need no `me`.** Their reads and
  watches name no party (jig's `client/domain/contract/README.md`, 5.5), so
  they go in the first batch with everything else.
- **No narrower twin.** A watch whose subject differs from an open one only by
  a smaller window is served by the open one: the rail's facts that a project
  has goals or milestones come from the page's lists while a page holds them.
- **No reread on opening.** A list's ended group read at opening is current;
  its live watch's first value asks for no second read, and a later delivery
  reads the ended group again only when a row has left the live group since.
  A setting read in the same batch as the settings' watch is made at the same
  state, so it is read again only when a delivery names a newer revision.
- **Nothing opened that the page doesn't draw.** Work opens no chips watch for
  its chats' goals, and no frame's read holds the page back.

## 6. Drawing what has arrived

- **The frame draws with the stream.** The rail and the bar draw once the
  hello's `me` is in, before the batch's answers.
- **Sections draw top down, each as its list comes.** A section draws once its
  own list and those of the sections above it are known, so nothing the person
  is reading moves when a later one arrives; the link puts the page's messages
  in the batch in the page's order, so the top's answers tend to come first.
  No section waits for another's details, for the people's read, or for the
  frame's own reads. Today Work's heading and every section wait for the
  milestones', goals', chats', notes', changes' and drafts' lists together
  (`../client/pages/work.md`, section 1).
- **The heading waits only for its counts,** its line held so that nothing
  below it moves when it comes; the kicker, the lede and the filters draw at
  once, as now.
- **A row is whole when first drawn.** What its card draws comes with it
  (section 4), so no headline, milestone or provenance fills in later and moves
  a row; a signer's name, from the project's people, comes in the same batch.
- **The terminal and the TUI** draw the same tree from the same client root,
  and gain the same.

## 7. Proving it

All focused, with no walk or measure in a plan.

- **The budget is a test.** On the client root, with an engine double that
  answers each wave's exchanges at once, a cold open of Work counts its waves
  and its requests until it is drawn whole and sends nothing more: on a small
  project and on one at the client's bounds (32 goals live and ended, 16 chats,
  16 milestones with their goals, 64 changes queued and landed, 64 landed
  drafts), both within the budget and equal. The budget is one constant, beside
  the page.
- **The batch:** replies in the order made, one slow read holding back none;
  one exchange and one call admitted for many messages; a lost batch as each
  message lost; a busy message alone retried.
- **Parts:** a row delivered again when its part changes, a report written
  among them; a part cut to its bytes.
- **Recordings:** the journeys the changed wire touches are recorded again with
  the change that moves it.

## 8. Upgrades

- jig's client wire and HTTP mapping and temper's client family move their
  versions, unbridged: a page of an earlier version is refused by name and
  reloads to the engine's bundle (`redeploys.md`, section 8).
- Nothing stored changes: parts are made when a row is, and kept nowhere. The
  single-message routes stay, for a message alone.

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
