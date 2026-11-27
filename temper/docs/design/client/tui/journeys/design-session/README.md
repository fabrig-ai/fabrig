# design-session

A version-2 `walk`, `after project-landing`, in one process on the
playground's composition, with both shipped clients and no terminal
(`../../../journey-walks-world.md`, 1, and `../../../journey-walks.md`, 6). Hello has the owned `work/hello`
repository and the OpenAI sign-in account shown as "your ChatGPT plan".
The browser approves the TUI. `/draft Plan a farewell line for hello.`
opens a design session whose drafter writes `farewell-line.md`, asks
where the line belongs, and offers three choices. Ctrl-o cycles half,
all, folded and half again. Sending "1" chooses "At the end of README.md";
the drafter settles Open questions. `/goal` starts the draft's goal by
Hello's first way, Task by task, and returns to the chat.

Its `scripts.json` supplies the drafter, the answer and the started goal's
report. The chat is T2 in project 1. Unlike the browser's `chat-draft`,
this session creates note 2 directly; its answer replaces two lines at
line 15 of revision 1, making revision 2. The goal is T3, cued by
`farewell-line\n\nImplement this draft.`. Its report demonstrates the
flow; the scripted model changes no repository.

Each number has an `.html` page and a `.txt` frame:

- `01`: Hello's launchpad, "Nothing needs you.", Start's `/draft`,
  `/change` and `/ask`, and the empty composer. No key link: pasting
  the opening words is typing.
- `02`: `/draft Plan a farewell line for hello.` typed on the launchpad.
  Enter leads to the session.
- `03`: half, the filename beside the conversation, all first-draft
  sections marked new, and the drafter's three choices below the box.
  Ctrl-o widens the pane.
- `04`: all, the draft from its head through Purpose, Checks and Open
  questions, the choices still below the box. Ctrl-o folds it.
- `05`: folded, the conversation and choices alone, with no document.
  Ctrl-o opens the half pane again.
- `06`: half again with "1" typed; the quiet line replaces the choices.
  Enter sends "At the end of README.md".
- `07`: the second reply says nothing blocks the work. The pane starts
  at Open questions, the only section marked new, now placing the line
  at README.md's end. No key link: the next paste is typing.
- `08`: `/goal` typed and offered below the box. Enter opens its prompt.
- `09`: "Start a goal from farewell-line.md. How should it run?", with
  Task by task first and active, Plan, tasks, land whole next, then
  Cancel. Enter starts Task by task.
- `10`: the returned chat, still half, with the revised section marked
  new and its goal linked. Typing `/goal` no longer replaces the quiet
  line with a command offer; those words are cleared before the mark.
  The last page has no key link.

The frames are 80 columns by 24 rows, dark with no band. The launchpad
shows build `0.0.0 (000000000000)` and `127.0.0.1:10001`; the session's
rule names Hello, the shortened title, T2 and that service address.
The quiet line shows `Live · Hello · fake` on the launchpad and
`Live · Hello · drafter · fake` in the session, with no effort label.
Where choices or a command prompt do not replace it, its right side
says `pane: half · Ctrl-o widens`; at this width the linked goal's part
is left out. Pages without a key link lead on by their neighbours.
The journeys' writer owns the pages and accompanying files.
