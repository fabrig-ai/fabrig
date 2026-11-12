# Chats

Kind: concept

A chat is a conversation between a party and an agent in a project: an untracked task of the label chat, which replies, parks between messages, and asks for changes that other tasks make.

## What it is

- **A chat is a task** labelled `chat`, made from its project's chat
  template, template 1 by default, or another offered chat template:
  its charter, its budget rule and its model.
  It is untracked, has no deadline, and keeps its transcript.
- **Ask temper,** template 5 under charter 11 in the stock, explains temper
  and this project's settings, and turns how you describe your work into
  a proposal of settings. Describe it in your words, paste a text, or
  name a document in an adopted
  repository. The agent reads the settings and asks one question at a
  time until it can propose. What temper must enforce becomes settings;
  what agents must follow stays in their instructions or in documents
  their charters name. An owner decides the proposal; the agent changes
  nothing itself (temper help settings).
- **A design session,** template 6 under charter 12 in the stock: the
  drafter designs one piece of work with you, one question at a time,
  keeps the draft as the chat's document and starts nothing. You start
  a goal from the draft. A session opened again goes on from its draft
  and its open points.
- **A failed settings proposal** creates no proposal to accept. Its activity
  summary says "failed to propose a change" and counts failed attempts
  separately from successful proposals; expand the activity to read the refusal.
- **Its conversation** is what `chat show` reads: each turn's words, from
  the party and from the agent, the brief supplied to each run, and each
  run's start and end.
- **A reply is not a result.** The agent's words answer the party,
  and the chat goes on: it ends only when its party closes it.
- **A change is another task's.** The stock chat can inspect its workspace;
  its tools cannot write, edit or remove files. Asked for a repository
  change, the chat first reads the code it touches with the forge's reads:
  `read_branch` for its head, then `read_file`. It asks for the change
  with the host tool `change`, giving the party's request and only code
  it has read. If a fix's named fault is absent, it changes nothing and
  says what the code has. The coder also writes nothing and finishes
  successfully with a no-change explanation for an absent fault, never
  adding a feature in a fix's place. If the change's words describe the code wrongly, the coder
  builds the requested addition in the code there is, under its names;
  it ends with a no-change explanation only when the addition cannot be
  built there. Production errors still hold the change for recovery.
  A change task, the chat's delegate, writes the change, opens its pull
  request and lands it; the chat hears its result (temper help task). Until a
  delegate reports, the chat says it is started, nothing more. If the change
  finishes successfully without changes, the chat tells the party that nothing
  changed and explains why using the delegate's conclusion, without claiming
  landing, implementation or tests. When the change holds, the chat tells the
  party why, in the hold's words,
  and that the party decides: release it or cancel it (temper help
  change).
- **Work agreed first.** Asked to agree a plan before anything is
  changed, the chat's run never calls `change` for it, never writes the
  plan or asks for the go-ahead in its own reply, and never sets `advisor`:
  it starts a goal at once with `goal`, `agree` set and the party's
  words whole, even where they ask for an advisor first (temper help
  goal), and
  says a question will wait in the party's inbox. The goal asks its plan with
  `message`, then waits for the answer, and finishes only once every
  agreed change has landed; the question waits in the party's inbox; the party
  answers it there (`task answer`), the answer reaches the goal itself,
  and only then does the goal call `change`. The chat never answers
  the goal's question: an answer it sends is refused, naming that its
  party answers it in their inbox. In the chat's conversation the
  goal's question reads `G asks (question Q; answer with message target=G, form=answer, question=Q): ...` and, once answered,
  `Person answered task G's question Q in their inbox; task G has the answer and does the work`.
  When you answer in your inbox, the chat hears it with your words; the work that asked does the work.
  The chat says what
  landed with the pull request and merge commit its result's `Landed`
  line names. It describes what changed only as the result or pull
  request says, never by restating the party's request as done, and
  never names a check or test the result doesn't name.
- **An advisor, and changes in order.** Asked for an advisor first, the
  goal, never the chat, starts one with `goal` and `advisor` set: temper opens its brief
  with a line saying it is an advisor, which reads the forge and
  reports, and it can change nothing. The goal names what it found in
  its plan. Agreed changes are made one at a time: one that builds on
  another is asked for only after the first has landed. Asked for a
  survey that changes nothing, the chat starts a goal the same way,
  which asks an advisor and reports, changing nothing.

## How it changes

- **Running, then parked:** a run answers, waits a while for more
  words, then parks; the chat waits for news, `Waiting for news`, and
  a message makes it due for its next run.
- **Closing, then closed:** closing moves the live direct delegates
  the party names to the party, cancels the others at once, then
  gives the chat's run its template's grace, 5 seconds in the stock
  template, and cancels it. A closed chat is not reopened.

## What temper guarantees

- **Words between turns:** a message sent while a run works reaches
  it at the next turn's boundary.
- **Long chats continue:** when saved history cannot be resumed, the
  next run starts fresh with recent conversation text and pending
  messages. If that history cannot be read, a retry continues with the
  pending messages alone. An agent failing to start does not by itself
  mean the history is unusable: retries keep that history and your pending
  words. Repeated agent failures can still hold the chat as out of tries.
  The operator's run-failure log keeps the startup cause and diagnostic;
  release retries the task (temper help task).
- **Each delegate decided at the close:** none is left without an
  owner; the ones not moved are cancelled with the chat.
- **The transcript stays** after the close: `chat show` and `task
  show` with `--run` read it.

## Acting on it

- **In the client:** Work in the project's rail holds its Goals and Changes
  filters and the way to Chats. To start one, use **+ New draft or change**
  on the rail and choose Draft or Change; with no drafts offered, use
  **+ New change**.
- **Read a run's brief:** `temper ctl DIR chat show TASK` prints the
  gathered brief, including charter document sections and their recorded
  commits. Follow `chat show TASK --before CURSOR` as printed to reach
  older transcript pages. `task text` reads a task's specification or
  instructions; those fields do not establish which document content a
  run received. For individual turns, `task activity TASK` gives the
  attempt and turn as `A.T`; `task turn TASK A.T` reads its parts, with
  `--part PART --from OFFSET` to follow a cut part's continuation.
- **ctl:** `chat start` starts one in a project and `chat show` reads
  one; `task message` sends it words; `chat close` ends it, and the
  same `chat close` again gets its first answer; `task show` on its
  task with `--until replied` returns once it has answered or waits on
  someone; `task tree` shows the change tasks it started.
- **To describe how you work,** start Ask temper:

  ```
  temper ctl DIR chat start --project PROJECT --template 5 TEXT --key KEY < /dev/null
  ```

  On the web, choose **Ask temper** in the project's rail or **Ask** in
  the tab bar to open its newest open chat, or start a new one.
- **The stock chat's runs** use the host tools `change`, `goal`,
  `delegate`, `decided` and `message`, and read these pages with `read_docs`.
  Ask temper reads with `see`, keeps its chat's document with `document`,
  keeps a line about you across your chats with `remember` when you ask
  it to, and proposes settings with `propose_settings`. On the web, the
  lines it keeps are on **What temper remembers about you**, in your
  menu, where you change or delete them.

## Limits

For the configuration members that need a restart, read `temper help configuration`.

- `chat start` uses the project's chat template unless `--template`
  names another of its chat templates; a template that is no chat's
  starts none.
- temper does not promise a final summary before the close's grace
  ends.
- A chat whose request goes past what the party may start alone is
  proposed instead, and starts only once accepted.
