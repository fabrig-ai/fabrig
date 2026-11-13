# A chat's composer

## 1. Its composer

- **Where the focus stands on a chat** (the request `ui-composer-focus` of
  2026-11-06):
  - *Started by sending:* a chat a person has just started opens with its
    composer focused, as soon as its composer shows (skein's `ui/web/README.md`, 5):
    from a new chat, its draft, `Ask` and a suggested request alike.
    "Start goal" is none of these: it opens the goal's page, its heading
    focused. A start refused stays on its page and moves nothing.
  - *Opened any other way,* its composer asks for nothing. From a list or a
    link, a new entry, the chat opens on its heading, so that a person reading
    with a screen reader or the keyboard lands on what the page is, though the
    heading is drawn after the page (`../../web/README.md`, 2.3). An address, a reload,
    Back and Forward make no new entry, and take the focus to no heading
    (skein's `ui/web/README.md`, 5).
  - *After a send,* by Enter or by "Send", whether or not it went out, the focus
    is in the composer, on a chat's, a goal's and an agent's task's (`../task.md`, 4) alike:
    "Send" disables as the words clear, and a disabled button holds no focus.
    "Pause", "Resume" and "Write it up from this chat" send too, and ask for nothing.
  - *While the rename's form is open* it keeps the focus (`../../view.md`, 3), and a composer
    that has asked for it takes it back as the form closes.
  - *On a phone* the same acts ask the same, as a phone's messaging app does:
    the keyboard stays up after a send, and a chat opened from a list raises
    none (`../../README.md`, section 5). The terminal draws no focus.
- **The work's account and model.** The model and effort sit at the composer,
  on every chat: a new chat's, Ask temper's page's, a chat's and an Ask
  temper chat's (the look request `model-picker`). A choice is the task's LLM choice,
  from the chat's next run (jig's `domain/tasks/README.md`, section 3 and `jig/docs/design/domain/tasks/changes.md`).
  - *The picker* is on the hint's row, at the composer's right edge: a
    disclosure named "Model" (skein's `ui/README.md`, its disclosure), which folds
    away, described by its closed line. A browser shows the line and a chevron;
    the terminal and the fake person read "Model: LINE" and find its content
    whole. On a phone the hint goes and the picker stays.
  - *Its closed line* says the model and effort Create or the next run would
    take, with no account: the draft's LLM choice, or with none the supplied
    `resolved_default` of the chat's template (jig's `client/domain/contract/documents.md`,
    1), an Ask temper chat's its charter's template, 5, any other's template 1;
    "the default model" where neither names one. "Claude" goes before the shown
    name of a model of the `anthropic` provider and no other provider's name
    goes before one, so "Claude Opus `../accounts.md`, 2 · high" and "GPT-`../../line.md`, 1 Sol · high". Its
    effort is the one the choice names or, where it names none, that model's
    default in the offer. What a task holds stays its own: a model the offer
    leaves out is still named, and its wait says why (`../accounts.md`, 2).
  - *With no composer,* a closed chat, one with a result or ended, says it in its
    bar instead, in the bar's muted form where the number was: "T2 · GPT-6 Astra ·
    high", in the closed line's shown names, in the browser and the terminal
    alike. It uses the latest named completion kept by the thread
    (jig's `client/domain/client/conversations.md`), the effort only where
    that turn asked for one. With no model read it says what its
    next run would have taken: its LLM choice, or with none its template's
    `resolved_default`, as *Its closed line* has them. "T2 · Using the default
    model" is only where neither names one, where temper can't know. A run
    with no readable named turn can leave an earlier model or this preview;
    neither proves what that run used (`../../README.md`, section 5). Until the
    conversation is read to its end the bar says the number alone, and names no
    model it would take back. A closed agent's task says the same words without
    the number, which its heading holds, in its taskbar (`../task.md`, 2), with no line or
    picker at its composer. A goal's page draws no such line. A phone's bar shows
    no byline, so a closed chat's words are left out there as its number is.
  - *The menu* rises from it, by what pays, drawn from the project's offer in
    `me` (jig's `client/domain/contract/documents.md`, 1), working nothing out, so that
    no page shows what the offer leaves out. In order:
    - A heading per account of a model's `accounts`, in the offer's order: its
      provider, "Claude" for `anthropic`, and muted what pays, "your
      subscription" for an `anthropic` plan or program, "your ChatGPT plan" for
      an OpenAI plan, "your API key" for a key. An account the accounts document
      doesn't hold takes its provider's name alone. Two headings of one provider
      that would read alike say who each account is instead (*An account*, `../accounts.md`, 2). A model
      with no accounts, the playground's, goes under its provider's name.
    - Under each, a button per model `picker` shows, by its shown name, pressed
      where it is the choice, which draws a tick; "default", muted, on the model
      `resolved_default` names, under its first account. Pressing one takes it at
      its default effort; where its provider has two accounts, also preferring
      the account it is under. Pressing the row marked "default" returns to the
      draft's own choice where that names the same model or none.
    - A provider with no model in the offer is one line and no model, the
      stock's included: its name and "Connect a plan or key ›", a link to
      `ConnectModel` with no way chosen; or, where the person holds an account of
      it that has stopped being usable, its name and that account's own fix as
      the accounts page words it ("Sign in again ›"), a link to `Accounts`. The
      playground's provider has a line only while the playground runs.
    - "Effort", a group of the chosen model's efforts as the provider says them,
      the chosen one pressed; none where the model declares none.
    - "All models" and how many the offer holds, where the picker shows fewer.
  - *All models* takes the menu's place: "‹ Models", back to it; a field "Find a
    model", matching any part of a shown name or a name, in any case; every model
    of the offer, by account as above; and "Choose what the list shows, in Deployment",
    a link to `Models { project }` for the picker's project, Deployment's tab
    (`../../view.md`, 3), the rail naming no Settings (`../../../draft/surfaces.md`,
    section 2). A model whose `new_until`
    is later than now has `.temper-model-tag`, "new", beside its shown name.
  - *A started chat:* the menu opens with "A change applies from the next turn."
    One press of a model or an effort applies it, an amendment or, where the task
    offers only that, a proposal. While it is sent that line says "Changing…";
    refused, it says why, and the choice is the chat's own again.
    An accepted amendment changing only the LLM choice sends no message and
    starts no run. A turn under way keeps its choice; the person's next words
    stop a live run behind the choice and start the next on the new choice,
    with those words retained and no try spent (jig's `domain/tasks/changes.md`
    ).
    A model change in a resumable window first asks the saved window's model,
    when configured, for a summary, then the new model for the answer.
    A changed effort alone keeps the window. Across an endpoint, dialect,
    account or backend boundary, or without the saved program session, the
    conversation's text, calls and results carry over, with opaque replay
    dropped and the summary on the new model (smith's `domain/session.md`,
    8.2). A provider change chooses an account again (jig's `domain/tasks/changes.md`
    ). A smaller window follows the same compaction rules. If the
    conversation still cannot fit, the chat is held with its transcript kept
    and nothing cut: "the run no longer fits the model's window of N tokens"
    (jig's `client/domain/contract/documents.md`, 5). The person can choose another
    model on the held chat.
  - *A chat whose choice can't be changed:* the same words with no chevron, not a
    button. No usable model: `../accounts.md`, 2's "No model yet" above the composer, and no
    picker.
  - *The terminal:* the same list as lines, its headings and models in order, then
    "Effort:" and its efforts beneath.

  A task's wait for its account and a refused next run stay
  above the composer. An agent's task's page, which keeps its
  own composer (`../task.md`, 2), keeps the line (`../accounts.md`, 2) and the picker together in it. A refused next run says "Next run refused
  · WHY", WHY the refusal's cause in the words of jig's reference view
  (jig's `client/domain/client/helpers.md`, 1), each sentence ending with a
  full stop: for a host's hello, "An engine host was refused at HH:MM"
  or "A worker was refused at HH:MM" (`../../view.md`, 6's clock), then its cause, in
  the words jig's reference view gives a hello's refusal. A run no worker
  can ever take, a refused hello with its figures among them, says `../accounts.md`, 2's
  "No worker can take this: ..." instead. A refusal is not a reply: the chat
  starts nothing, and keeps its composer, so the person can choose
  another model. The scripted model is offered where `me` serves it,
  in the playground's project alone (`../../forge/family.md`, 7). Setting a goal names its planner's model and
  effort, the template's by default. A goal's page draws no model, effort
  or account line; that presentation is deferred in `../../README.md`, section 5. An open
  agent's task's page shows them in its composer's line, as a chat's (`../task.md`, 2);
  a task waiting
  for its account
  says so, with the account's acts (`../accounts.md`, 2; jig's `domain/authority.md`,
  8.3).

## 2. Open questions

- **Run-wait words after landing** (`../README.md`, `../accounts.md`, 2; the request
  `ui-wait-words`): temper/qa checks a charter grown past its admitted
  worker's bounds on the page and in the terminal, alongside ctl's
  `next run:` explanation. Default: the fleet, core and wire tables,
  and the view tables for every bound in both shells; no browser story,
  walk or render check within the plan.
- **A closed page's latest-run model** (`../README.md`, `../task.md`, 2; the request
  `ui-model-line`): a run with no committed turn, a tail page with no turn,
  or an undecodable or nameless newest turn can leave an earlier model or
  a next-run preview. Keeping the latest run's model independently of
  readable turns is jig's `client/domain/client/README.md`, section 15. Default:
  the last named completion, then the task's choice or its template's
  `resolved_default`; until caught up, no model line. A turn's missing
  effort stays unknown, never filled from today's offer.
- **The closed model line after landing** (`../README.md`, `../task.md`, 2; the request
  `ui-model-line`): checking the muted header and taskbar in a browser and
  a real Ask temper chat remains after landing. Default: the view tables
  cover both shells and the joined client story carries smith's bytes to
  the terminal's closed header; no browser, walk or render check is a
  condition for completing the plan. A phone retains its hidden byline.

- **A press that starts a chat, on a phone** (1, *Where the focus stands on a
  chat*; the request `ui-composer-focus` of 2026-11-06): the tree has no width, so
  a suggested request's press asks for the composer's focus there too,
  and a browser that lets a press's activation outlive the answer may raise
  a keyboard the person did not ask for. Telling them apart takes a kit mechanism,
  a focus request that raises no keyboard (skein's `ui/web/README.md`, 5). Default: the
  same focus at every width.
- **A start refused, by its press** (1, *Where the focus stands on a chat*; the
  request `ui-composer-focus` of 2026-11-06): the send arrow disables while a start
  is out, and a disabled button holds no focus, so a start refused after a press
  leaves the focus where the browser put it, and the person clicks the composer to
  try again. That request does not ask for it. Default: a refused start moves
  nothing.
- **A composer that comes back** (1, *Where the focus stands on a chat*; the
  request `ui-composer-focus` of 2026-11-06): a composer's request rides its insert
  (skein's `ui/web/README.md`, 5), so a chat that loses Message and regains it, after a
  send or "Change something", takes the focus again, from a person typing elsewhere
  on that page at that moment. Default: not guarded.

- **The composer's selected account and source** (`../accounts.md`, 2): jig's
  `PayerReadiness` could carry the account and source its selection
  chose (`jig-domain-core`'s `catalogue.rs` already calls `select`).
  The view could then read jig's choice for every payer without
  translating selection over the viewer's own account slice. Default:
  name an account only in that slice, with the readiness cross-check
  and expiry fallback in `../accounts.md`, 2; other work names the model alone, with
  the playground provider's exception.
- **A wait said twice** (`../README.md`, `../accounts.md`, 2): on a chat's page the bar and the wait's own
  line both say "Waiting for an account", and with no usable account "No model yet
  · Connect a model" stands beside the wait's "Connect a model". Default: so, until
  a walk reads them.
