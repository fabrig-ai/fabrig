# The view's state

What of the reference UI is not a tree: where the person is, what the
page holds, the requests it sent, what the person is in the middle of,
and its notices. It is step code under the subset, as the client is
(skein/docs/foundation/programming-model.md, section 3), with its own limits, worst case and
stories (9, 10). It keeps no copy of a watch's document: it reads
each from the client, by its handles, when the client reports it
changed. A read's page, which the client hands over once, in the read's
terminal, and keeps no copy of (client/README.md, section 6), the reference UI
keeps until the page that read it is left or the handle's next page
replaces it (3). It decides nothing the engine or the client decides:
what is made, when, and once, is theirs.

## 1. Its surface

```
jig_client_view::Ui: step(now, wall, &Client, Event, &mut Queue<Request>), fire(now, wall, &Client, &mut Queue<Request>), next_deadline()

made with: its limits; the application's navigation and home; its arrangement

Event, in
  from the application:
    Start { records, place, seed }  its own records, as the application kept them; the place the client starts at; its seed
    Went { place } | Left           a place of the reference UI's gone to; a place of the application's
    Act { action }                  a person's act on one of its nodes, as its bindings decoded it (section 5)
    Named { objects, sections }     what the application's page names of jig's, the whole list each time (pages.md, 12.5)
    Client { told }                 what the client tells under jig's asker, the link, and kept records dropped, by count, as the root routes them (client/README.md, section 4; root.md, section 8)
Request, out
  for the client:      Watch | Unwatch | Read | Send | SignOut               the client's events, under its handles and keys
  for the application: Show { place, how } | Save { records } | Erase { records } | ShowPart { part, scope } | PartsLeft
read by the application: ui.sections(), ui.shown()
```

- **A child of the client root,** as any presentation is (root.md,
  section 2). The root steps it with the person's acts and the places
  gone to, which its shell reports, and with what the client tells of
  its handles and keys and of the link, routed by asker (root.md,
  section 5).
- **It asks the client through the root:** each request for the client
  is one of the client's events, which the root gives the client in the
  same step; the reference UI never holds the client, and reads it only
  by the `&Client` the root lends it (`document`, `text`, `pending`,
  `link`).
- **It asks for nothing else of the client's, and reports nothing.** It
  never asks to sign in: signing in is the application's (6). It
  never continues a run's watch: the client does, from the text it
  holds (client/watches.md, 3). A limit that refuses is said where it refuses,
  on the page or in place (2, 4), never as a report beside it.
- **`now` and `wall`,** as the client's (client/README.md, section 4): the
  monotonic instant for its deadlines (7), and the wall time it shows
  the engine's wall times against ("waiting 4 min").
- **Its handles and keys** carry its asker, jig's owner, 65536 (`view.md`, section 3
  ; root.md, section 15), which the client hands back with everything
  it tells of them. It numbers its handles itself, and draws its keys'
  numbers from its seed, fresh for each request it means (client/README.md, section 4 and client/requests.md
  ).
- **Navigation, home and arrangement** are given when it is made, as
  data, and never change after: which of jig's pages the frame offers,
  in what order, which is home, and the sections its arrangement adds
  (pages.md, section 2 and 12.5).
  A navigation naming a page twice, or an arrangement pages.md, 12.5
  refuses, is refused when it is made.
- **`ShowPart` and `PartsLeft`** ask the application to show a part in a
  slot, naming the project or the deployment it shows in, and tell it
  the page's parts have left (root.md, section 6).
- **No loop, no call.** It runs no loop of its own, and never calls the
  application or the client: it is stepped, and asks.

## 2. The place and the page

- **One page at a time.** The page at the place is one of the reference
  UI's (pages.md) or the application's, which it knows only as
  elsewhere, and by the sections and objects of jig's the application's
  page names (`Named`). Pages that are not about the work (starting,
  signing in, a page missing, a client out of date, someone else's
  work: pages.md, section 8) are pages too, whatever the place.
- **`me`, always.** It watches `me` from its start, whatever the page,
  by a handle of its own, which shares the client's own watch of it
  (client/watches.md, 2): the frame's parts, the templates the pages fill, the
  bounds a draft is compared with, and the party whose work is kept
  come from it (contract/documents.md, 1).
- **Going to a page** closes the old page's watches (`Unwatch`), forgets
  its reads and the pages they were handed (3), tells the application
  its parts have left (`PartsLeft`), drops from the working set every
  object no request in flight is about (3), then opens the new
  page's watches and reads (pages.md, 12.5), and asks for its place to
  be shown (`Show`). Requests in flight, `me`, drafts and notices carry
  over.
- **Handles, not generations.** Each watch and read a page opens has a
  handle of the reference UI's; what the client tells of a handle the
  page does not hold is dropped. The client drops what comes for a watch
  closed or a read abandoned, so a handle may serve the next page at
  once, and a `Watch` on a handle already open moves it to the new
  subject (client/README.md, section 6, and client/watches.md, section 1).
- **The link is the client's** (`Link`; client/watches.md, 2), which the
  reference UI judges nothing of: the frame's link part says its state;
  starting, signed out and out of date are also pages not about the work
  (pages.md, section 8).
- **Behind** is the page's: a handle of the page whose state the client
  gives as behind (client/watches.md, 1) makes it behind, and the page says
  what it shows may be old, beside the link's state. On an application's
  page, each section of jig's says it in its own region (pages.md,
  12.5).
- **Full is a refusal the page shows.** A page whose watches or reads
  would pass the reference UI's limits opens what fits, and says what it
  left out; nothing grows. A handle the client ends, or a read it
  answers, refused by the client's own limit (client/README.md, section 6, and client/watches.md, section 1
  ), is one the page left out.

## 3. The working set

- **Objects by key, once.** Each object a card shows is held once, under
  the key the engine names it by: a task, a proposal, an escalation, a
  question, a person task, a result. One object may show in several
  places on a page (an escalation in a task's overview and in its plan);
  it is one object, with one card.
- **Read from the client's documents.** Its facts are the documents the
  client holds for the page's handles (`client.document(handle)`), read
  each time the client reports one changed. The working set keeps only
  what no document of the client's holds: which handles name each
  object, its card's state, for a leaving object, how it ended, and the
  pages its reads were handed.
- **A read's page is kept by the page that read it.** A read has one
  terminal, `Page { handle, page }`, and the client keeps nothing of it
  once it is told (client/README.md, section 6: "A read is one attempt ...:
  the application reads again if it wants"). The page keeps what the
  read is handed, under the read's handle: a project's settings, a page
  of a task's history or activity, an object's document read to say how
  it ended. A read's handle keeps one page; a window's keeps the rows
  its pages put in the window (client/README.md, section 6: "Keeping rows
  across pages, as a list slides while a person scrolls, is the
  presentation's"). What it keeps goes when the page is left (2), or
  when a read on the same handle is answered with a page, which replaces
  it; a read answered with none leaves it, and the page shows why, as
  2 says.
- **Sources, not reference counts.** Each object records which of the
  page's handles name it. Each change stands for the whole of what its
  handle names: an object no handle names leaves, unless a request about
  it is pending or it is leaving.
- **Each object carries the revisions of what may be decided about it,**
  as its document gave them: those the acts of 4 name. Each moves
  only when what is decided on changes, never with spend or a
  delegate's phase.
- **An object no watch carries whole** (a proposal, an escalation, a
  question, a person task, a result, a note) is read, and read again
  whenever a watch that names it says it moved: its task's activity
  moved, its entry in the inbox changed, its chip's phase changed
  (contract/README.md, 5.1). Once no watch names it, it is read once more for
  how it ended.
- **A card's state,** wherever the card shows:
  - *open;*
  - *deciding:* a request about it is pending, shown as asked, never as
    done;
  - *refused:* the answer refused it; the card says why, and stays;
  - *unknown:* its request may or may not have been made; the card says
    so, and where to look (4);
  - *not kept:* its request was never sent; the card says so, and offers
    it again (4);
  - *leaving:* it was decided, answered, withdrawn or ended, by this
    person or another; the card says what happened, by whom and when,
    and leaves after a linger.
    The task page's own header returns to open when its linger ends,
    keeping its final task document and the chip's terminal phase.
- **Leaving, whichever comes first.** A card leaves on its request's
  answer ("you accepted it"), or when no document of the page names it.
  In the second case, a read of the object's document says how it ended
  ("Pat released it at 10:14") before it leaves. While a request about
  it is pending, it stays deciding until the answer, so the person
  learns how their own request ended.
- **Full is a refusal the page shows.** A document naming more objects
  than the working set holds keeps what fits in its order, and the page
  says how many more there are.

## 4. Requests

- **Asked once per intent.** A card's act opens its confirmation (5);
  confirming it sends one request through the client (`Send`), under a
  key of its own, unless one for the same act on the same object is
  pending, which the second press joins. A double press and a retry
  therefore carry the request's one key, and a reload carries it as the
  client keeps it (client/README.md, section 8).
- **One request per act.** What the person sees as one act is one
  request: one of jig's operations, or one compound request
  (contract/requests.md, 4): amending and releasing a held task; words that
  release a stopped chat; closing a chat with what becomes of its live
  delegates; reordering goals. The reference UI never chains two
  requests, so it never has to undo the first when the second is
  refused.
- **Shapes through the helpers.** A chat is started with `shapes::chat`,
  a goal set with `shapes::set_goal`, a chat closed with
  `shapes::close_chat` (client/helpers.md
  ), each filling the project's template as `me` serves it; the
  reference UI sends what they give, and knows no shape of its own.
- **Shown as asked.** The card is deciding from the confirmation until
  the answer: "being decided", "retrying", "waiting for the engine", as
  the client reports the request's progress (`Pending`,
  `client.pending(key)`). Nothing is shown as done before its answer,
  which the client reports only once its commit is durable (client/requests.md
  ).
- **Said before it is sent.** An object's document offers each act:
  made; proposable, and to whom; or refused, and why; setting a goal and
  amending are offered with bounds (contract/documents.md, 3). The form
  and the confirmation compare the draft with the offer's facts (a
  number no larger, a grant among those listed) and say which before
  anything is sent: within them, the operation is sent; beyond what the
  party may do alone but proposable, a `Propose` is sent, the
  confirmation saying to whom; refused, it says why, as when nobody may
  accept the proposal, and sends nothing. That is the engine's word
  read, never authority computed: the engine judges the request again
  when it comes.
- **The answer, applied.** Done: the card shows the outcome (made, or
  proposed, naming the proposal), and leaves if its object stopped
  waiting. Refused: the card says why, from the refusal's facts
  (contract/README.md, section 3), and offers what fits: look at the object
  again, where it moved; change what was asked, where a limit or the
  party's authority refused it; propose it, where a `Create` or an
  `Amend` was refused for want of authority and the refusal says a
  proposal could be accepted (objects.md, section 9).
- **Unknown.** A request the client could not send again within the
  engine's retention may or may not have been made (client/requests.md
  ): the card says so, and where to look.
- **Not kept.** A request whose record the client could not write was
  never sent (client/README.md, section 8): the card says so, and offers it
  again. So it does for one answered unconfirmed, sent before `me` named
  the party (client/requests.md).
- **Words are a request too.** A composer's words are a `Message`, shown
  as being sent while pending, as sent once answered, and as read once a
  committed turn took them (`view.md`, section 9).
- **Asks name what they saw.** A release, and a decision about a held
  task, name its hold's revision; an amendment, the task's spec's; an
  answer to a person task, its own; a note's correction or deletion, the
  note's: each as the document the card was built from gave it. The
  engine refuses one whose revision has moved, saying what it is, and
  the card asks the person to look again (contract/documents.md, 2). A
  priority-only amendment names none, and the first commit decides.
- **Kept work.** A request the client restored at start under jig's
  asker (`Kept`, root.md, section 8) has no card: the reference UI keeps
  no intent across a restart, since its records are its drafts and its
  place (8). Refused, or unknown whether made, it shows as a notice
  (7); made, it shows in what it made. Kept for another party, it is
  counted for the page someone else's work (pages.md, section 8). Kept
  work the client dropped at start, unread, is told by how many
  (`Dropped`; client/README.md, section 8; root.md, section 8): a notice says
  that many requests kept from before were not sent.
- **Bounded.** The intents it keeps are a limit; an act past them is
  refused in place ("too many requests in flight"), never queued; so is
  one the client refuses for want of room.

## 5. Drafts and the confirmation

- **A field** is something a person types into, named by what it is for:
  the new chat's words, a task's composer, the open confirmation's
  reason, an amendment's fields, a new goal's. It keeps the text, and a
  count of the reference UI's own writes (a composer cleared once its
  words are sent, a draft restored), so the view writes a field's value
  only when that count moved and the person's typing is never
  overwritten (`view.md`, section 6).
- **Composers keep their drafts per task,** across pages and reloads,
  among the reference UI's records (8), bounded by the drafts kept;
  the oldest untouched leaves first.
- **One confirmation at a time:** the act, the object and the revision
  the act names, as the object's document gave it when the confirmation
  opened, the choices made (an option of a person task, what becomes of
  each live delegate on closing), and what is missing (a reason; a draft
  its offer's bounds say would be refused, 4). What it says comes from
  the object's document through `facts` (client/helpers.md): what an
  acceptance grants and costs and the pool after, what a cancel ends,
  what a release will do (contract/documents.md, 4), so the view says it
  without asking anything.
- **The confirmation is the reference UI's.** The engine asks for none,
  and an application confirms as it chooses (contract/documents.md, 4); the
  reference UI opens one on every act that decides something, and sends
  nothing until it is confirmed.
- **A revision that moves under an open confirmation** marks it changed:
  a held task's hold, under its release. Confirming it then is refused in
  place, saying what changed, rather than sent. What the act does not
  name (spend, a delegate's phase) changes the confirmation's words, not
  its validity. Bounds that move under an open confirmation, as a newer
  document gives them, are compared with the draft again, and the
  confirmation says what it will be: made, proposed, or refused.
- **A confirmation's fields** are cleared when it closes, confirmed or
  dismissed.
- **Folds the reference UI keeps** leave what they fold out of the tree:
  a plan's level folded into counts, a call folded to its line. They are
  its state's because opening one changes what is read or built. A
  disclosure, whose content is always in the tree, is the shell's to
  fold, and the reference UI never knows it (`view.md`, section 3).

## 6. Signing in and out

- **Signed out:** the link says it (client/watches.md, 2, and client/README.md, 9), and
  the client ends every handle. The reference UI shows the sign-in page
  (pages.md, section 8), keeping the place to come back to; the requests
  in flight wait, parked, in the client.
- **Signing in is the application's:** the printed link's secret or its
  own sign-in's credential, which its shell or its host gives the
  client (client/README.md, section 9), or a browser's cookie, which the
  client never sees. Once `me` names the party, the reference UI returns
  to the place it kept.
- **Signing out,** from the frame's party part, sends the client
  `SignOut`; the reference UI shows the sign-in page once the client
  reports the party signed out.
- **A sign-out that does not reach the engine** leaves the person signed
  in, as the link says (client/watches.md, 2, and client/README.md, 9): the frame says the
  sign-out did not reach the engine and offers it again; nothing is
  shown done before it is (testing.md, 6.3). A sign-in the engine ends
  itself, before the sign-out or after it, ends what was asked: the link
  says signed out, and the reference UI shows the sign-in page, as
  above; it never keeps the work page once the link says signed out.
- **A service** has no reference UI, and never sees a sign-in page.

## 7. Time and notices

- **Wall times are for showing:** the engine's records carry wall times,
  and "waiting 4 min" is the view's, from an object's wall time and the
  step's. While a page shows any, a tick once a minute counts a change to
  what is shown, so the view builds again.
- **Deadlines are the reference UI's:** a card's linger, a notice's
  life, the coalescing of drafts' saves, on the monotonic clock
  (skein/docs/foundation/programming-model.md, section 9). Backoffs and heartbeats are the
  client's (client/watches.md).
- **Notices** are short-lived, for what has no card: a request answered
  for an object not shown, a refusal with nothing to show it on, a
  kept request of jig's refused or unknown, kept work dropped at start
  (4), the link going and coming back. A card's own outcome is on
  the card.

## 8. Its records

- **Its own records:** the drafts, and where the person was (the place).
  Nothing else: requests are the client's to keep (client/README.md, section
  8), and everything shown is read again.
- **Kept through the application.** The reference UI asks the
  application to save and erase them (`Save`, `Erase`), and is given them
  back at `Start`. The application keeps them where it keeps the
  client's small store, under keys of the reference UI's own, apart
  from the client's records, which the client's own saves carry alone:
  in a browser, skein's small store, the tab's session storage
  (`../../protocol/browser/README.md`, section 10); a file, for a terminal.
- **Written with the client's.** The application gives the transport a
  step's records of the reference UI's among the step's writes, with
  the client's, so they are written before the step's exchanges leave:
  a composer cleared because its words were sent is never restored
  beside the kept request that carries them.
- **Versioned** by the reference UI, so a client updated since reads
  what an older one kept, and a record it cannot read is dropped, never
  guessed: a draft lost, a place forgotten for the home.
- **Bounded:** the drafts kept, a draft's bytes and a place's are
  limits.

## 9. Limits and envelopes

The reference UI's limits are its own: its windows, folds and its pages' bounds; for what the client already bounds (watches, reads, parts, documents, rows, text and the rest) it reads the client's limits (client/README.md, section 11), never a copy.

Each is configured and bounds what the view accepts or builds:

- objects in a page's working set; rows per window; turns a
  conversation's window holds; nodes of a plan held; the pages its reads
  were handed that it keeps (3), one per read handle, each priced at
  the client's bytes of a page handed over (client/README.md, section 11);
- account lists at the client's counts (client/README.md, section 11), set
  beside its client limits: the view renders every admitted account
  and source, with no smaller cut;
- its handles (watches open, `me`'s included, and reads in flight) and
  intents kept;
- these, and the objects, windows and pages above, counted for the most
  sections a page may show together: at a place of jig's, its own with
  the arrangement's beside them; on an application's page, every
  section of one task, one project and one inbox filter, with the
  conversations and the deployment (pages.md, 12.5);
- drafts kept; notices;
- bytes a person may write (the engine's own bound on words); bytes of
  any one text shown (a spec, a result, a turn); bytes of a provisional
  turn;
- the linger, a notice's life, the drafts' coalescing.

The view's limits derive from these and the client's, so a tree always
fits (`view.md`, section 10). Containers grow as they fill; fixed scenarios check
the measured heap peak against a named ceiling (testing.md, section 6).

## 10. Its world

The reference UI's stories, in a person's terms, run in jig's client
world with jig's test application and the fake person (testing.md,
section 4), and in the joint world with the engine's domain (testing.md,
section 5): signing in and starting a chat; a held chat released; a
proposal accepted in the inbox while its chat shows it; two tabs
deciding one thing; a reload with requests pending; the engine
restarting under a pending request; a watch dropped and the page behind;
a page left with reads in flight; a draft kept across a reload; a goal
beyond the person's authority, proposed as the confirmation said it
would be; a chat closed with one live delegate cancelled and another
made the person's; goals reordered together; a task's page arranged
with every section together, a proposal accepted in its plan while its
conversation shows it; an application's page showing a task's plan in a
section slot, and a second task's not shown, its slot saying so.

Its referee's rules are testing.md's, section 6: those on the client's
surface, jig's check, and those on what the person saw.
