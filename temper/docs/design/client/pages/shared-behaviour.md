# Behaviour across pages

This document holds across pages, split from [`README.md`](README.md).

## 1. Across pages

- **Links are by number:** a chip, an entry or a line naming a task
  opens its page, from any page (jig's `client/domain/pages.md`,
  section 2).
- **A task's title on a line:** a task's title is its words' first line,
  as jig takes it, and a writer may leave it long. Every line, row and
  card that names a task by its title draws it cut past 80 characters,
  by one function of the views' shared parts, `shared::row_title`
  (`common::chat::row_title`, at `common::chat::ROW_TITLE`, 80): Work's
  goal row's cut (`work.md`, 1, *Its title*), at the last space that
  leaves it at most 80 with "…", or at a character's edge where there
  is none, its punctuation kept. It holds in the browser and the
  terminal alike, and in the TUI's lines from the same parts:
  - *Where:* Needs you's from line, its milestone and goal included,
    and its cards: a question, a proposal (its proposed tasks' headings
    and the question line a reasonless batch takes from them), a held
    task, a person task, an approval ("Approve TITLE on …"), funds, a
    group of findings' asking task (`needs-you.md`, 1); a card's SHORT
    and the need lines built from it; Home's lines, its cards, chats,
    milestones and Decided for you's rows (`home.md`, 1); the inbox's,
    the terminal's cards, their updates and the answered lines; a
    goal's task rows and its milestone's link (`goal.md`, 2); a task's
    plan rows (`goal.md`, 2, **A plan row's cells**) and its lineage
    bar (`task.md`, 2); Work's rows, cards and
    milestone groups (`work.md`, 1); the chats' rows and the chat
    switcher's list; a chat's goal bar, a delegation's card and line
    (`../view.md`, 5) and a proposal's line; the Changes rows; a close's
    delegate rows (`../root.md`, 1.6); a result's card; How it runs'
    budget rows; Tell's About line; the public pages' rows, whose
    "title cut" word follows only a title the cut left whole.
  - *A link's name* is the cut title too, as Work's goal row's is. The
    cut is drawn alone: the domain keeps the title whole.
  - *Whole:* a task's own page's heading, its taskbar's "TN · TITLE"
    and a change's or a person's header (`task.md`), and a goal's title
    in its page's header (`goal.md`, 2). A chat's heading keeps its own
    short form (`chat::short_title`, at 72), a wait's or a dependency's
    name its cut at 40 (`work.md`, 1, *A wait's names*), and the TUI's
    launchpad its cut to the row's room, after this one. The TUI's chat
    cards, a delegated task's and a proposal's, draw the title whole and
    let the screen wrap it (`../tui/pages/chat.md`, 1, *a card*): the
    shared lines keep the cut for the browser's terminal, and
    `shared::proposal_title` gives a proposal's whole.
- **A task's words after its title:** a card or page that draws a
  task's title and then its words leaves the words out when they are
  only the title: whole, not cut, with nothing past their first line
  feed but spaces, tabs, carriage returns and line feeds. One function
  of the shared parts says it, `shared::only_title(&DocumentText) ->
  bool` (`common::chat::only_title`). It holds on the terminal's inbox
  cards, the goal's page's needs bands, which draw
  those cards, and an agent's task's "Task brief" (`task.md`, 2).
  Needs you's held task, funds and person task cards, whose from line
  names the title, draw only the words past their first line, by a
  function beside it (`needs-you.md`, 1, *Its title once*). A
  title the card cuts is then read whole on its task's own page, whose
  heading keeps it. A proposal's card differs, since its tasks have no
  page yet: each proposed task's heading is its title, cut, and the
  words under it are those past the first line, or, where the heading
  is cut, its words whole, so that its whole first line is on the card
  once. A person task in words reads as a question and draws no title
  heading over its words (`needs-you.md`, 1). A goal's page draws no
  words of its goal (`goal.md`, 2, **Not drawn**), and a chat's request
  is its conversation's first message, kept.
- **Away** (the greenfield mock's `home.html`, `phone.html` and
  `settings.html`'s `#away`; `../../domain/forge/attention.md`, 4 and 6;
  `../../draft/what-reaches-you.md`, section 3): a person's switch on a
  project, while which agents decide for that person what the project's
  classes let Away decide, each listed under *Decided for you* (`home.md`, 1). The
  classes are the project's, the switch each person's. It is the rail's and
  Settings'.
  - *Shown* while either holds, one function's answer (`Domain::away_shown`),
    so the rail and Settings agree: the project's classes, kind 8 as `settings.md`, 2
    reads it, hold an item Away may decide (`Classes::away_may_decide`,
    `../../domain/forge/attention.md`, 6), or the party's row there is on, so that a
    switch left on can be turned off. A project on You approve's table has no
    switch, and one with no value of kind 8 has Agents review's table and so
    has one. Until kind 8 is read, or after a lasting refusal, only an on row
    shows it.
  - *The control,* in the rail's foot above Settings, a button, `State::Pressed` while on,
    in `temper-away`, the mock's `.away`: 8 below it, full
    width, 8 10 padding, 1px `--gf-line`, radius 8, `--gf-surface`, 13.5 px
    muted, a 30 by 18 track holding a 14 px knob, by style; on, the ink ground,
    white words, the knob at the right. Its words: "Away" off; "Away until
    08:00" on, `until` by `../view.md`, 6's *clock*; "Away" alone on with `until` none.
    On a phone the foot is hidden: a second button, `temper-away-chip`, the
    mock's `.away-chip`, at the top bar's right (`../view-navigation.md`, 1) beside the project switcher,
    min-height 36, radius 18: "Away: off", "Away until 08:00", "Away" alone
    on with `until` none; the browser's alone. In the terminal the rail's
    button, numbered as any. While its request is out it is disabled.
  - *Its act* sends `SetAway` (`../forge/records.md`): to off, `on` false and the row's
    `until`; to on, `until` none where the row's is none, otherwise the next
    08:00: the first wall after now whose hours in the person's zone (`../view.md`, 6)
    read 08:00, computed by one pure function, `forge::away::next_eight`. A
    row's `until` once past means nothing but that the choice is 08:00. What
    Away decides is the classes', never the act's.
  - *Past `until`* agents are told to ask (`../../domain/forge/attention.md`, 4) and
    the control draws off, with no write: the build asks to wake at
    `until`. `Domain::away_on` holds the rule, the row and the wall.
  - *Its toasts,* Needs' toast slot (`needs-you.md`, 1, *What the client keeps*), on
    the request's answer: on, "Away until 08:00: agents decide within
    PROJECT’s limits." with the act "Limits", to Settings; "Away: agents
    decide within PROJECT’s limits." with `until` none; off, "Away is off:
    questions wait for you again."; refused, "Away didn’t change. Try again."
  - *On All projects* (`../../draft/surfaces.md`, section 2), while the page
    is one across projects (`pages::rail::across`), the switch and the chip
    are the person's projects' together: M, the projects `me` lists, and
    N, those whose row is on at the wall (`Domain::away_on`), counted with
    the earliest `until` of the rows on by one pure function,
    `pages::away_all::across`, which the rail and the chip read:
    - *Shown* while this section's *Shown* holds for any of them, the
      rail's fact on All projects (`../view-navigation.md`, 1).
    - *Its words:* with N 0, a project's off words, not pressed; with N
      M, a project's on words with that earliest `until`, "Away" alone
      where no row on has one, pressed; between, "Away on 2 of 4", N of
      M, not pressed, so that the state says what a press does. The build
      asks to wake at that earliest `until`.
    - *Its act,* `away::Act::ToggleAll`: with N M, every project off;
      otherwise every project on. Each project not already so is sent its
      own `SetAway`, the request *Its act* above sends for that project's
      switch, one decision each, sent one after another, each once the one
      before is answered: the client's exchanges in flight are bounded
      (jig's `protocol/client/README.md`, section 10), and may be fewer than the
      person's projects. One already so is sent nothing. While any is out
      or waits to go, every Away control is disabled and a project's act
      waits, as while a project's request is out.
    - *Its toast,* once every answer is in, says how many changed: "Away
      turned on for 3 projects: agents decide within their limits.";
      "Away turned off for 3 projects: questions wait for you again.";
      "1 project" for one; with some refused or unknown, "Away changed for
      2 of 3 projects. Try again.", of those sent; with none changed, "Away
      didn’t change. Try again." No "Limits": limits are a project's.
  - *On How work runs,* `settings.md`, 2's *What reaches you* draws the classes, what
    Away changes among them and "Away until". "Limits", on the toast and on
    Home, opens How work runs with that section's heading
    focused, so a browser brings it into view: `Address` holds no anchor, and
    the page takes the request from `Pages` once.
- **The approval page** (`../../draft/tui.md`, 4.2;
  `../../draft/tui-remote.md`, 5.3; jig's
  `protocol/client/sign-in.md`, *A sign-in approved by a browser*):
  where a person signed in to the browser approves a terminal's sign-in,
  in the browser alone. Its terminal's side is 7.1.
  - *A place of its own,* `Place::Approve { code }`, at `/terminal`
    (`../web/addresses.md`, 1), beside `Authorize`: no page's address, so the
    pages and the forge's side are left as at any place that isn't
    theirs (`../root.md`, 1.3), and the frame is the top bar alone (`../view.md`, 1). `code` is an
    `ApprovalCode` (jig's wire), or none. Its state is the root's, in a
    module of its own, `approve`, beside `authorize`: made when the place
    is gone to, dropped when it is left, and kept nowhere, the small
    store included. `Domain::approval()` gives it to the view, and the
    root's shown count moves with it.
  - *Opening it approves nothing.* With a code, the page reads the
    approval, `jig-client`'s `ReadApproval { code }`, once the link is
    live and an exchange is free, as the `Authorize` place sends its
    own; until then, and while it is read, it says "Reading CODE…".
  - *The ask's kind decides the press,* as the read answers it (jig's
    `protocol/client/sign-in.md`, *Its kind*), never the page's origin:
    a `returned` ask, with a loopback return, is approved by one press,
    its code read from the address; a `typed` ask, made by the public
    name with none, only by its code typed into the field, on every
    origin, the listener's own address through a tunnel among them; a
    `listener` ask, made by the listener's own address, as on one
    machine, its code read from the address. The page keeps whether its
    code came from the address or was typed: a code from the address
    whose read answers `typed` is dropped, the address replaced with
    `/terminal` as a code that doesn't read is (`../web/addresses.md`,
    1), and the field shown, empty, with no alert; a typed code whose
    read answers `typed` is the ask, waiting.
  - *With no code,* or one the address carried that doesn't read
    (`../web/addresses.md`, 1), it asks for one: a field named "Code" and
    "Check". The field keeps what is typed up to jig's bound on a typed
    code (`CODE_TYPED_BYTES`, 32 bytes), and marks a field typed past
    it; "Check", or Enter, reads it as the address's code is read,
    `ApprovalCode::parse`: letters of the code's alphabet in either
    case, hyphens and spaces ignored. One that reads is read as above;
    one that doesn't, or a field typed past the bound, stays in the
    field with the alert below. A typed code changes no address. The page never
    lists the approvals that wait: the engine reads one by its code
    alone.
  - *Its states,* each in the column (`../view.md`, 1; `../web/look.md`, *Outside
    a project*), its level-1 heading, its words, then its acts, the
    first filled and the others outlined, CODE the code as shown
    (`WDJB-MJHT`), large, in mono, on the surface; "Check" filled beside
    the field, the field named "Code" above them; while deciding,
    "Approving…" or "Refusing…" stands in the acts' place:

    | State | Heading | The column | Its acts |
    |---|---|---|---|
    | asking for a code | "Sign in a terminal?" | "Type the code your terminal shows.", the field | "Check" |
    | a typed code that doesn't read | "Sign in a terminal?" | the same, the field as typed, and the alert "That isn't a terminal's code. Type the eight letters your terminal shows." | "Check" |
    | reading | "Sign in a terminal?" | "Reading CODE…" | none |
    | the ask, waiting, by the listener's address (`listener`) | "Sign in a terminal?" | CODE, large; "Asked AGO from ADDRESS" (below); "Approve only a code your own terminal shows." | "Approve", "Refuse" |
    | the ask, waiting, with a loopback return (`returned`) | "Sign in a terminal on this computer?" | CODE, large; "Asked AGO from ADDRESS" | "Approve", "Refuse" |
    | the ask, waiting, its code typed (`typed`) | "Sign in a terminal?" | CODE, large; "Asked AGO from ADDRESS"; "Approve only a code your own terminal shows."; and, where the ask's source and the reading browser's are both known and differ, "It was asked from another network than this browser's." | "Approve", "Refuse" |
    | deciding | "Sign in a terminal?" | the ask's, "Approving…" or "Refusing…" in its buttons' place | none |
    | approved, handing the sign-in to the terminal (a loopback return's grant answered) | "Approved" | CODE; "Handing the sign-in to your terminal…" | none |
    | approved, waiting for the terminal's take | "Approved" | CODE; "Waiting for your terminal to sign in…" | none |
    | signed in, taken | "Terminal signed in" | "Signed in. You can close this tab." | none |
    | refused | "Sign-in refused" | "This terminal's sign-in was refused. Nothing was signed in." | none |
    | expired | "Code expired" | "This code has expired. Ask again in your terminal." | none |
    | used | "Code already used" | "This code was already approved or refused. If your terminal isn't signed in, ask again there." | none |
    | unknown | "Unknown code" | "temper doesn't know this code. Check the code your terminal shows, or ask again there." | "Type a code" |
    | signed out | "Sign in first" | "This browser isn't signed in to temper. Open the sign-in link temper printed first, then open this page's address again: your terminal shows it." | none |
    | signed out, Google's way given (the view's `Ways`, `google` and an origin) | "Sign in first" | "This browser isn't signed in to temper. Sign in, and this page comes back with your terminal's code." | "Continue with Google", to `ORIGIN/sign-in/google?then=PATH`, PATH the approval's printed address with its code, `/terminal?code=CODE` (`/terminal` with none), percent-encoded as the sign-in page's (`../../protocol/sign-in.md`, section 7) |
    | busy, not ready or lost | "temper couldn't answer just now" | "It is starting or stopping, busy, or out of reach. Try again in a moment." | "Try again" |

  - *Where the ask came from:* AGO is `../view.md`, 6's *ago*, from the
    read's wall time; ADDRESS the ask's source, an IPv4 address in
    dotted decimal and an IPv6 one in RFC 5952's text, as core's
    `IpAddr` prints them, "Asked just now from 203.0.113.7" under a
    minute, "Asked 3 min ago from 2001:db8::7" after; with
    none, "from an unknown address". The read's own source, the reading
    browser's, is said nowhere but in the `typed` row's comparison.
  - *Deciding:* "Approve" and "Refuse" each send `DecideApproval {
    code, approve }` once, when the link is live and an exchange is
    free, and the page is deciding until it is answered: approved,
    refused, used, expired or unknown as the table says. Nothing sends
    a decision by itself, and none is sent again.
  - *The hand-off,* for an approval answered `Approved { loopback }`
    with a loopback return (jig's `protocol/client/sign-in.md`,
    *Approved with a loopback return*): the page is handing, and
    replaces its address with
    `http://127.0.0.1:PORT/approved?grant=GRANT`, the root's `Request::Leave {
    to, replace: true }`, which the browser's shell carries out, so that no history entry holds it: PORT the
    port of that answer, never one from the page's address, the host
    always `127.0.0.1`, never one the ask or the address named, the
    path always `/approved`, GRANT the grant's 32 bytes in base64url
    with no padding (RFC 4648, 5), 43 characters. The grant is held
    for that address alone: never shown, kept, logged or put in the
    small store. The terminal's listener there takes the sign-in and
    sends the browser back to `/terminal?code=CODE`, which reads it
    taken: "Terminal signed in" (`../../draft/tui-remote.md`, 5.1). Until
    the page is left it reads again each second, as approved. A
    browser that never reaches the terminal leaves the approval
    approved and untaken, and it ends expired, signing nobody in. An
    answer approved with no loopback return is approved, as before.
  - *Approved:* the page reads the approval again each second
    (`approve::AGAIN`) while it stays approved, from the answer's
    turn, to learn the terminal's take: taken is signed in, and the
    reading ends. The approval's own expiry ends it otherwise, answered
    expired (jig's `protocol/client/sign-in.md`, *Never taken,
    nothing made*), so a page left open reads at most for the
    approval's ten minutes. A read answered busy, not ready or lost
    meanwhile keeps the page approved and reads again a second later.
  - *Ended:* signed in, refused, expired, used and unknown read no more.
    "Type a code" returns to the field, empty, its code dropped.
  - *Signed out:* the link signed out, before a read or after, or a read
    or a decision answered signed out (which signs the client out,
    jig's `client/domain/client/sign-in.md`, section 1), draws "Sign in first"
    in place of the pages' signed-out paragraph and sign-in page (`../view.md`, 1).
    An answer of not a browser draws it too: the page's exchanges carry
    the cookie (`../web/sign-in.md`, 1), so the answer says they reached the
    engine without it, and the remedy is the same. The printed link's
    sign-in lands on `/welcome` (`../web/sign-in.md`, 1), so the person comes
    back by the address their terminal shows; a Google deployment prints
    no link, and its "Continue with Google" returns to the approval with
    its code, so a person signing a terminal in there is never stuck
    (the look request `outside-pages`). Its `then` keeps the address's
    code whatever the ask: signed out, the page can't read the ask's
    kind, and a `typed` ask's code is dropped by the read after the
    sign-in, as above. The page's codes, acts and
    `then` are the TUI's too (`../sign-in.md`, 1.1): only its states' look and words are
    the browser's.
  - *Busy:* a read or a decision answered busy, not ready or lost.
    "Try again" reads the approval again, from its code, so that a
    decision lost on its way back shows as it stands.

## 2. Open questions

- **Away on All projects after landing** (1, *Away*, *On All
  projects*): no mockup draws "Away on 2 of 4", so temper/ui renders
  `../web/gallery.md`, 2's `all-home-away` and `all-home-away-on` at 1440, 1000
  and 390 px, by day and by night, the words in the switch and the chip
  where the whole story's `web/whole-story/home.html` and
  `phone-home.html` draw them, and walks a press on a box across two
  projects. Owner: temper/ui. Default: each difference a
  finding filed back as a repair.
- **The Away switch and quiet hours** (1, *Away*): "08:00" is the mock's
  end of its quiet hours, a fixed hour here. Quiet hours and notifications
  are later parts, and a setting of the hour would come with them. The
  terminal reads no zone (`../view.md`, 6), so its 08:00 is universal time's.
