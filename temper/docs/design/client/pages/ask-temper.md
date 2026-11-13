# Ask temper

## 1. The page

- **The first chat** (`../../draft/ask-temper.md`, section 9;
  `../../draft/configuring.md`, 5.1): Home's first step offers "Tell temper
  how you work", and its "Start" opens Ask temper, as the rail's "Ask
  temper" and the tab bar's "Ask" do (`../view.md`, 3): the person's newest Ask temper
  chat open in the project, as `Chats` reads them, or else `Ask { project
  }` (1). A step says what the person does, and what it opens is
  named "Ask temper". The chat starts with the person's own words, typed,
  or the opening's "How does temper work?" pressed, its fixed words sent
  as theirs (the look request `ask-opening`, 2026-11-12); temper never
  starts it with words of its own. Neither Settings nor the Chats page makes the offer.
- **Ask temper's page** (the mock's `chat.html?id=new&configure=1`, in
  the project's frame, out of Settings as the look request
  `ask-temper-place` places it; `../../draft/ask-temper.md`, section 9):
  `Ask { project }`, at `/projects/<project>/ask`, through no Settings, so
  that it reads as asking temper itself and not as one more chat or a
  setting. It opens that project as `Home` does; with no
  such project in `me` it says what `Home` says. Each main of this
  layout (`temper-session`) is the page's height, in this order:
  - *Its top bar:* the path, a navigation named "Where": on a wide page
    PROJECT, a link to `Home`, followed by "/", then the level-1 heading
    "Ask temper", the page's title too; on a phone "‹", a link named
    "Home", alone before the heading. With a composer its bar names no
    model: what the chat's runs use is the picker's,
    under its composer (`chat/composer.md`, 1, *The work's account and model*): on an
    Ask temper chat, what its next run takes, as any chat's; with no chat
    yet, what one would, template 5's `resolved_default` in `me` (`accounts.md`, 2), and a
    choice made here is the chat's when it starts. It never sets one LLM
    provider's account beside another's model; with no usable account the
    composer says what `accounts.md`, 2 says then, and there is no picker.
  - *The conversation,* a region named "Conversation", its opening a
    page as `Home` is, starting 28 pixels from the conversation's top,
    with no kicker and no heading, so that the ways have the room (the
    look request `ask-temper-ways`, the user, 2026-11-08). Every path,
    mark, way's word and card in it comes from the project's `ways` read
    (`../forge/records.md`; `settings.md`, 2, *A way's acts*), none written by hand; a way's words are
    the read's. Until the read answers, the ways' region holds nothing; if
    its watch ends, it says, muted, "The ways can't be read now.". In this
    order:
    - *the lede,* in ink, "Choose how work runs, or describe how you work
      below.", naming no project, so that describing one's own way to
      temper, in the composer, reads as the other choice; "how work runs"
      is the rail's and Settings' name for it, and no new noun (the look
      request `ask-opening`, 2026-11-12);
    - *the ways,* a region named "Ways of working", 22 pixels under the
      lede: one control of tabs named "Ways of working", the full width
      in a lined frame, a tab for each way in the read's order, from
      more of the person to more of their agents, "You approve", "Agents
      review" and "Draft first", then "Yours" where the project runs no
      stock way; the current way's tab adds "now", small, in mono, in the
      signal's colour; the tab shown is filled. Under it, small, faint and
      in mono, "more of you" at its left and "more of your agents" at its
      right, a line named "From more of you to more of your agents";
    - *the shown way,* 26 pixels under the tabs, a group named by its
      name: its line in ink, the read's `line` ("Yours" has none); its
      path, temper's drawing of a path pressed (`chat/conversation.md`, 3), named "The way a
      change goes", its fallback a stop a line; while a stop is open, its
      card (below); then its foot, the legend at its left, a line named
      "An agent, a person, code; no merge tool above the dotted line",
      small, faint and in mono, "agent", "person", "code" and "no merge
      tool above", each after its mark, and its act at its right;
    - *its act,* as `settings.md`, 2, *A way's acts*, says: "Use this" on a way the
      project doesn't run, filled in ink (the signal's orange at night),
      or "Keep it" on the current one while To begin's first step is
      left; none on "Yours", for a person who may not change settings,
      or on the current way once that step is done; disabled while one
      is out. A refusal stands under the foot in jig's words with the
      forge's reason, as Settings says one under its row (`settings.md`, 2); an act
      answered says its toast (`settings.md`, 2).
    - *"How does temper work?",* in the same row, before the way's act, on
      every tab and whether or not the way has an act (the row then holds
      it alone): a button in the row's outlined look
      (`.temper-ways-acts > button`), its hint "Sends: Briefly, how does
      temper work, from my ask to main? And how do these three differ?".
      Pressed, it starts the chat as "Send" does (below), its first message
      those fixed words, "Briefly, how does temper work, from my ask to
      main? And how do these three differ?", the person's own message,
      shown in the chat as any other; the composer's words are neither
      sent nor changed. It is disabled while "Send" can't start a chat
      (no template 5, or a start out), and goes with the opening once the
      chat opens. On a phone the row's buttons wrap as "Keep it" does
      (the look request `ask-opening`, 2026-11-12).
    - *a stop's card:* a stop pressed, in the drawing or as its line in
      the fallback, opens its card under the path, the drawing's caret
      at the stop (`chat/conversation.md`, 3); pressed again it closes, another stop moves
      it, another tab closes it, as the page's state, in no address. It
      is the stop's `card` from the read, framed as the tabs are: a
      level-3 heading, the stop's words with a capital ("Review"), its
      `who` beside it in grey ("an agent · holds a change until it
      approves"), and, where the stop opens a charter, a gate or a rule,
      a link "Open NAME ›" at its right, NAME the card's `name` ("the
      reviewer"), to what it opens: a charter's role,
      `Role { project, charter }`, a gate's its reviews' charter's role,
      the gate's `charter`, and a rule's page, `Rule { project, kind,
      number }` (`settings.md`, 1, *A role's or a rule's page*); then its `facts`, each label
      small, faint and in mono over against its value ("Model", "Can",
      "Asked to"; a person's role and what they decide; code's checks,
      the queue or the turn). A bar opens no card.

    On a phone the tabs keep one row, smaller; the path runs down the
    page (`chat/conversation.md`, 3); the card has no caret and its facts stack, each label
    over its value; the act fills the foot's width. The terminal has no
    style to tell them apart: the tabs print a line each, the current
    one's words "NAME, now"; the path its fallback, a stop a line, its
    actor in words; the card its heading, line, link and facts; then the
    legend and the act. The opening goes after a first message, as the
    chat it starts opens in its place.
  - *The composer,* a form named "Ask temper", at the
    foot, focused as the page opens: its text area "Ask temper, or tell it
    how you work", that its placeholder too, its text 14
    pixels in from the composer's edge; "Send", the send arrow on ink in a rounded
    square, grey while disabled, which it is while the text is empty;
    under it the chat page's hint, "Enter sends · Shift+Enter for a
    new line". Its picker is on that row, as a chat's: a choice there is this
    chat's alone, and Ask temper's models, charter 11's, stay a setting
    (`settings.md`, 2), which this chat can change. On a
    phone the frame's top bar goes, and with it the model's line and the
    composer's hint, as the configure-chat look request draws it. The tab
    bar stays: this is the phone's Ask tab (`../../draft/surfaces.md`, section
    10), so its tabs close the page under the composer, "Ask" current, the
    composer and its picker standing wholly over them, as on Home (`../view.md`, 3):
    the four tabs, or while To begin shows its three, "Home", "Ask" and
    "Deployment" (the look request `to-begin-two-steps`, whose rail and tab
    bar hold on Ask temper's page as Start opens it). An Ask temper chat
    keeps them so too; any other chat draws none on a phone. The night
    switch stands at the bar's right (`../view.md`, 3).

  The conversation and the composer's foot each keep a scrollbar's gutter
  on both edges and hold one column, at most 720 pixels and centred, so
  that the opening, a chat's turns and the composer share their edges
  whether or not the conversation scrolls, in every mode, the column
  shifted by half a scrollbar where the platform's scrollbars take room. On
  a phone
  neither keeps a gutter and both stand 16 pixels in, the conversation
  scrolling with no scrollbar drawn, the opening 20 pixels from the top.
  At 1366 by 680 the opening, with no stop's card open, and the composer
  show with no scrolling.

  "Send" starts the chat, its words the first message: one act, one
  request (`../README.md`, 3.6), from template 5 (`../../protocol/configuration/stock.md`),
  with its own models, while `me` lists it among the project's templates.
  With none it falls back to no other template: "Send" stays disabled,
  and under the composer "Ask temper isn't set up in this project."; nor
  do the rail and the tab bar offer it (`../view.md`, 3). While it is out, "Send" is
  disabled and the words stay; made, the chat's page opens: it is a chat
  like any other there; refused, jig's words stay under the composer, the
  words kept. "How does temper work?" is the same act with its fixed
  words in place of the composer's: one request, the same template,
  model and refusal, its refusal under the composer, the composer's own
  words left as they were. The chat names no repository. A way's act starts no chat:
  it changes the settings at once (`settings.md`, 2).
- **temper's own way,** the stock page `how-temper-builds-temper`
  (`../../protocol/documentation.md`, 2.2), has no way in from this page
  (the look request `ask-temper-ways`, the user, 2026-11-08): the client
  neither compiles it in nor draws it beside the page; it stays among
  temper's pages, which Ask temper reads (`../../domain/agent/agents.md`, 1).
  Where else it is reached from is `../../draft/own-way.md`'s.
- **An Ask temper chat** is a chat whose chip's executor is the charter
  of template 5 as `me` lists it (`home.md`, 1). In `Chats` it is listed
  with the others, tagged "Ask temper". Its `Chat { task }`, opened from
  there, from its start, from the rail's "Ask temper" or from its
  proposal's review (`settings.md`, 2), has this layout: the same top bar, its path
  "Ask temper /", naming no project, which the rail's switcher names
  (`chat/README.md`, 1, *One layout for every chat*), "‹" on a phone a link named "Ask temper" to
  `Ask { project }`, its heading the chat's number and title as a chat's
  is, then its number and " · Ask temper", its status, its spend, Pause
  or Resume, and Close; the conversation as a chat's in place of the
  starters; the composer of this layout, its text area "Message" as a
  chat's, and none on an ended chat; the rail's "Ask temper" and the tab
  bar's "Ask" are current. Its address stays `/chats/<task>`. A chat from
  template 1 is any chat.

## 2. Open questions

- **Ask temper's ways after landing** (1, *Ask temper's page*; `settings.md`, 2;
  `chat/conversation.md`, 3): temper/ui compares the look request `ask-temper-ways` with main
  at 1440, 1000 and 420 px (`../web/look-requests.md`, 4): each tab, a
  stop's card open on an agent, a person and code, "Yours", a refusal, a
  member with no act, "Keep it" gone once To begin's step is done, at
  night, and the terminal's print. With them, the repairs of its first
  render check: the path's marks and labels with no outline
  (`../web/look.md`), the path at 1000 px 658 by 127.94, the open card
  14 px under it at every width, the gallery's print at 420 down the page
  (`../web/gallery.md`, 1), the reviewer's "Asked to" whole and CI's checks;
  then `look-round-25`'s: the open card at 1000 px and on a phone, four
  tabs two by two at 760 px and under, a stop made only when needed
  outlined with its small words over it, the agent's star and its rises
  in ink, each rise's fade, CI's card naming the workflows, and How work
  runs drawing the same path, with Roles under it (`settings.md`, 2); and a walk
  from each card's "Open NAME ›" to its role's or rule's page
  (`settings.md`, 1, *A role's or a rule's page*), which no request draws and no gallery
  state prints. Owner: temper/ui. Default: each difference a finding
  filed back under the request's trailer.
- **Ask temper after landing** (`README.md`, `../view.md`, 3): try the small chat on ChatGPT and
  Claude Code through a proposal and an owner's Accept, then judge its
  instructions. The other views and sketches join at M3b of
  `../../draft/ask-temper.execution.md`. The page of preferences (`remembered.md`) is
  walked after landing, a line kept in one chat, read in the next and
  deleted there, and its gallery's prints read at a laptop's width and a
  phone's.
- **A fresh Ask temper chat from its composer** (1, *Ask temper's page*): a
  command ("/clear") once the composer takes commands. Default: "Start over" in
  its bar.
- **The reference's picture as a flow** (1, *temper's own way*;
  `../../draft/own-way.md`, 3): the reference shows no figures, the user's word of
  2026-11-06, and a flow is one path, its parts in parallel a caption until they
  branch. Default: the numbered list.
- **The table's QA row** (1, *temper's own way*): it says "Later"
  until QA after landing is built (`../../domain/findings.md`, section 8), then names
  its range as the other rows do. Default: "Later".
