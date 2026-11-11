# A chat

## 1. Its frame

- **One layout for every chat** (the greenfield mock's `chat.html`): every
  chat and `NewChat` take the layout of Ask temper's page
  (`temper-session`; `../ask-temper.md`, 1, *Ask temper's page*). The rail stays
  on every chat: the user's choice in `configure-chat-in-settings`, which
  supersedes the mock's rail-less chat. On a phone, at 760 px or less, rail
  and tab bar go, but for the rail's night switch, at the chat's bar's right
  (`../../view.md`, 3), and the tab bar of Ask temper's page and of an Ask temper chat, the
  phone's Ask tab (`../ask-temper.md`, 1, *Ask temper's page*). A chat differs from an Ask temper chat only in its path: a
  chat's path is "PROJECT / Chats /", "‹" on a phone a
  link to `Chats`; an Ask temper chat's is "PROJECT / Ask temper /", "‹" on a
  phone a link to `Ask { project }`, the rail's "Ask temper" current.
- **A chat's bar** holds its path, its level-1 heading named "TN · TITLE", the
  chat's task number and its title (a chat's first message, whole), and at its
  right, in this order:
  1. *Its number:* "T4", the same pixels as the mock's `.byline`. The browser
     alone draws it, as the terminal's heading says it; on a phone it goes.
     The model and effort are the composer's, and the account leaves the bar
     (`composer.md`, 1, *The work's account and model*). An Ask temper chat's is a chat's,
     then " · Ask temper", in the look of its tag in Chats.
  2. *Its status,* quiet: 12 px, faint, in lower case by the look alone.
     It is the colour of `.state-needs` while it waits for the person.
     A run that can't start says so ("Waiting for an account"), that colour
     too, and has no dots: the dots say a run is working, never that it
     can't start (jig's `client/domain/client/helpers.md`, 1).
     Every ended chat says "Closed", with no time: its end's line has the time
     (`../../README.md`, 3.6, *Closed ends on one line*).
     A chat the person paused says "Paused", quiet; one held for any other
     reason says "Held", and its card says why (1, *A person's pause*).
  3. *Pause or Resume, and Close,* while allowed, each quiet and small. The
     mock has neither; they are the only way to pause or end a chat.
  4. *Its own spend,* "N spent · of B", in the view and hidden in the
     browser.

  The bar's right part is a flex row.
  After the number, once a chat has a document, its mode group; with none,
  "+ Draft"; then "Start goal…" or its goal's link; on an Ask temper chat
  "Start over" (`document.md`, 1).
- **A chat's heading is short** (the request `chat-title`): it shows its
  title's short form, which a browser draws alone and the terminal after
  "TN · ". The short form is worked out in the view, in one function
  (`pages/chat.rs`):
  - *its words:* the title from its first character that isn't white space
    (a space, a tab, a carriage return or a line feed), up to its first line
    feed or through its first sentence's end, whichever comes first, with the
    white space at its end dropped. A sentence ends at a `.`, `!` or `?`
    followed by white space or by the title's end, so a `.` inside a word
    ("`../README.md`", "lib.rs") ends none;
  - *its length:* at most 72 characters, counted as Unicode scalar values.
    Past that, it is cut on a word: the words before the last space or tab
    within its first 72 characters, with any `,`, `;`, `:` and white space at
    their end dropped, then "…", so that the heading is at most 72 characters.
    Where that leaves no words, a first word of more than 71 characters, it is
    cut within it after its 71st character;
  - *the whole title* stays in the heading's name, "TN · TITLE", and where
    the short form isn't the whole title it is the heading's hint as well,
    which a browser shows on hover (`skein/docs/design/ui/README.md`, 3.2); a title
    shown whole has no hint. The person's message, the task card and the stored title keep it
    whole. Only a chat's heading is short: an agent's page keeps "TN ·
    TITLE" whole.

  The path yields to the right part: the crumbs take `flex: 1 1 320px`, so
  the right part keeps the bar's row while the path has 320 px, and the
  heading's ellipsis takes up the rest. Below that the row wraps, and a
  phone keeps the heading above the controls.
- **A person's pause** is a chat held by a placed hold of that person's own,
  mode Stop, over the chat's own subtree: the chip's hold names it, not lifted,
  its owner the party reading (jig's `client/domain/contract/documents.md`, 6). Nothing
  else is a pause: a hold of another's, of a task above it, or with a reason of
  its own says "Held", with its card and why, and has no "Resume": the card
  keeps its acts.
  - "Pause" is `Ask::Stop`, while Stop is allowed and the chat isn't paused.
    "Resume" takes its place on a paused chat where Release is allowed, and is
    `Ask::Release` naming the hold's revision. Each is sent at once (`../../README.md`, 3.6). The
    status says it happened: an accepted Pause or Resume draws no status under
    the conversation; a refusal says why there.
  - **Sending resumes.** A message to a paused chat, where Release is allowed,
    is one act: `Ask::Compound` of its `Message`, then a `Release` naming the
    hold's revision (jig's `client/domain/contract/requests.md`, 4; jig's
    `domain/tasks/changes.md`: no message releases a task, and words to a
    task its party stopped come with a release). The composer's hint says so
    while it is paused, "Sending resumes this chat.", in place of its usual
    words. It commits whole or not at all: a refusal, the hold's revision moved
    or the words refused, leaves the chat paused and nothing in its inbox, and
    the composer says why as for a lone message. Any other chat's message is a
    lone `Message`.
  - An agent's task's Pause and Resume are these acts (`../task.md`, 4); a message to it
    resumes nothing (`../../README.md`, section 5).
- **A chat's status,** in its bar and on its line of the Chats
  page, is jig's, narrowed as the mockups' "Where each value comes
  from" says: "waiting on 1 delegate", "waiting on N delegates" while
  it waits on its delegates, N its chip's live delegates, as a goal's
  header says it (`../goal.md`, 2). On the chat page, the delegated tasks' documents
  reconcile that count while deliveries catch up: a card still running
  counts until its final document arrives, and an ended card no longer
  counts even if the chat's document still lists it. It says "waiting on
  your words" while it waits on news and its wake policy wakes it on
  words (its document's `wake`), from
  the person signed in, who requested it; jig's word otherwise. While
  a question a delegate asked it waits in the person's inbox, jig's wait
  is on the person, before its delegates, and it says "Waiting for
  you". A line of the chats page reads the same wake policy, from its
  chip's (`OwnerChipDetails`' `wake`), and says "waiting on your words"
  as the bar does: a chat at rest after its reply, which only its
  person's words or Close move on (jig's `domain/tasks/lifecycle.md`, 9). A
  chat whose host refuses its start says jig's "Cannot start yet", and
  the feedback above its composer says why (below); a line of the Chats page holds
  no run wait, so it is never refused, and says jig's word for its
  stage (jig's `client/domain/client/helpers.md`, 1). A chat held on a
  configured bound (jig's `Exceeded` hold) says jig's phrase for that
  hold, as its hold card does: "Held: the model's reasoning went past
  its configured allowance of 2048 bytes". A chat the person paused says
  "Paused", on its bar and on its line on Chats and on Home (1, *A person's
  pause*). Any other hold says jig's "Held", and the chat's page says why.

## 2. A new chat

- **A new chat** (the greenfield mock's `chat.html?id=new`): `NewChat`,
  at `/chats/new`, for the open project. The domain's `Chats` state holds
  its words and start, and `NewChat` opens that state with the page's
  watches, so that `Act::StartChat`, `Act::ChatWords` and `Act::Suggest`
  are the page's, as they are Ask temper's page's. On the layout every
  chat takes:
  - *Its bar:* the path "PROJECT / Chats /", "‹" on a phone a link to
    `Chats`, and the level-1 heading "New chat". No project yet, in the
    playground's start, the path is "Chats /" alone.
  - *In the thread,* centred, `.meta-line`: "Say what you want; the agent
    works in OWNER/NAME.", OWNER/NAME the project's home repository, read as
    Home reads it; or, with none, "Say what you want. PROJECT has no
    repository yet, so the agent can talk and plan but not change code."
  - *The composer,* the session's: its name and placeholder "What do you
    want done?", focused as the page opens, the hint "Enter sends ·
    Shift+Enter for a new line". There is no "Start" button: the send arrow
    is the act, grey while the words are empty or the composer cannot send.
    Sent, while it is out, the arrow is disabled and the words stay; made,
    `Chat { task }` opens in place of this address; refused, jig's words
    stay under the composer, the words kept.
  - *With no usable model,* "No model yet · Connect a model" sits above the
    composer (`../accounts.md`, 2).
  - *The playground's start* (`../../forge/family.md`, 7, `../welcome.md`, 2): its status, its failure and "Try
    again" and its context line draw above the `.meta-line`'s place, and its
    suggestions as `.starters` under it.
  - *With a draft,* `NewChat { draft: true }`, `/chats/new?draft=1`: "+ Draft" in
    the bar (`.btn.small`, while `drafts_offered`, `document.md`, 1)
    moves there. The empty document in Split, `.meta-line` "A chat that writes a
    draft. Say what it should hold.", the placeholder "What should this draft
    hold?". The first message starts a design session from template 6 where `me`
    lists it among the project's templates (`../../../domain/agent/agents.md`, 4), and
    otherwise a chat from the project's chat template, either with, under the
    person's words after a blank line, "Write this up as a draft beside us.";
    those words are visible as sent. The chat opens in Split.
- **A suggested request** (`chats.html`) starts a chat with the
  cue's words as its first message, as the composer's send does: one
  act, one request (`../../README.md`, 3.6), the suggestions and the send disabled while
  it is pending. The engine puts that message in the run's brief, and
  the scripted model finds the cue there (`../../../protocol/setup/playground.md`), so
  the chat plays the cue's script; the composer's text is left as it
  was.

## 3. Open questions on its frame

- **The chat heading's rendered look** (`../README.md`; the request `chat-title`):
  temper/ui compares the request's page and named states with the shipped
  view after landing, at 1440, 1000 and 420 px, including open disclosures
  and menus (`../../web/look-requests.md`, section 4). Default: the focused
  short-title and heading tables; they establish no rendered equality.
- **A model-written chat title** (`../README.md`): a later request may replace the
  first-sentence short form. Default: derive the heading in the view and
  keep the stored title whole, as the `chat-title` request asks for now.

## 4. Open questions on a new chat

- **The playground's suggested requests** (`../welcome.md`, 2): the look requests
  (`../../web/journeys.md`, 7) drew the chats page without the playground's
  invitation, its suggestions among it; the user kept the suggestions
  while tests walk the playground by them, to revisit (11-03).
  Default: `.starters` under the playground's start, on `NewChat`.
