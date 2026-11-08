# tui-launchpad: the account's rows

The landing's last commit carries the account (`temper/docs/design/draft/look-requests.md`,
5): one row per page change and per state the request's message names, each landed;
differs, and why; or deferred, to whom and where it is tracked. The task `again`, the
plan's last, checks each row below against the trunk it starts from and writes the table
into its last commit's message as it then stands, a row whose result moved said so. The
expected result is this cut's; the build decides.

| Item | Expected |
|---|---|
| 01: the framed header, "temper · PROJECT" in the border, the address at its end | landed (`screen-looks`, `tui-launchpad`) |
| 01: the status agent's brief in words, "written by the status agent HH:MM" at the end | landed, from Home's facts (`tui-launchpad`) |
| 01: with no brief new to the person | Home's headline, as the web's Home says it, with no byline (`view-shared`, `tui-launchpad`): a state the frames don't draw, decided by the design (`temper/docs/design/client/README.md`, 6.14) |
| 01: "Where you left off": the person's most recent chat and what it waits on | landed, from Home's live chats (`tui-launchpad`); differs in its words: the chat's state as Home's Chats rows say it ("Waiting for you"), not "waiting on your words", so the TUI and the web say one thing |
| 01: "Where you left off": ", draft" after the chat's title | differs: left out, since drafting is not the TUI's yet (`temper/docs/design/draft/tui.md`, 3, *Drafting*) and Home reads no chat's document |
| 01: "Where you left off": the person's newest goal | deferred: Home holds the project's goals by priority, not the person's newest; tracked in `temper/docs/design/client/README.md`'s Later for the TUI, owner temper/tui |
| 01: "Needs you", "? TITLE: QUESTION" | landed, from Home's needs (`tui-launchpad`) |
| 01: the box, "❯ Ask temper", between two rules | landed (`screen-looks`, `tui-foot`) |
| 01: the quiet line, "/ for commands · N needs you" and "MODEL · EFFORT · ● STATE" | landed (`tui-foot`) |
| 02: `/` lists the launchpad's commands, in the browser terminal's order and words, the first active | landed (`screen-lists`, `tui-foot`) |
| 02: the active command drawn by a band alone | differs: an options menu's active item carries the focus mark and accent over its band, one rule for every list, as 04's choice is drawn |
| 02: `/exit` closes temper; `/font` left out | landed (`host-launchpad`, `tui-foot`) |
| 03: a question from the launchpad opens a new Ask temper chat | landed (`domain-home`, `host-launchpad`) |
| 03: the top line, "✦ PROJECT / TITLE TN" and the address | landed (`tui-chat`) |
| 03: the person's words on a band after "❯" | landed (`screen-looks`, `tui-chat`) |
| 03: a run of calls as one line in words with its time, under "▸" | landed, in the browser terminal's words (`view-shared`, `tui-chat`) |
| 03: the answer growing whole, its blocks drawn once, its end a small block | landed as the browser's terminal draws it (`tui-chat`); "drawn once" as `chat-reading.status.md`, 3 has it |
| 03: numbered lists numbered | landed (skein's reader and renderer already; `tui-chat`) |
| 03: the running line "✦ working", no clock | landed (`view-shared`, `tui-foot`) |
| 04: the answer committed in place, the person's reply | landed (`tui-chat`) |
| 04: the question under the box: its head, the recommended choice first and active, "Tell it something else" last | landed (`tui-chat`); departs from the browser's terminal, which keeps the choices' order and ends with "Later" |
| State: the three dots before the first call or word | landed (`tui-chat`) |
| State: a call running, the run's line counting what ended and naming the call | landed (`view-shared`, `tui-chat`) |
| State: no project: the launchpad says so | landed: "temper" alone in the border, "No project yet" and how to make one on the web (`tui-launchpad`) |
| State: no project: the composer still asks temper | differs: the box takes commands alone, its placeholder "/ for commands", since Ask temper's chat is a project's by its template (`temper/docs/design/draft/ask-temper.md`, 9); `temper/docs/design/client/README.md`, 6.14, *No project yet* |
| State: nothing needs you, "Needs you" and the count left out | landed (`tui-launchpad`, `tui-foot`) |
| State: offline or signed out, the quiet line's right end | landed (`tui-foot`) |
| State: the light theme, by the same tokens | landed (`tui-surface`) |
| State: a narrow terminal, the header's address first, then the model | landed (`screen-looks`, `tui-launchpad`, `tui-foot`) |
| Replaces: the first screen's two lines and "q to quit"; `q` types once the box has the focus | landed (`tui-launchpad`, `host-launchpad`) |
| Journey: `sign-in-approved` recorded again, ending on the launchpad | landed (`again`) |
| Journey: `launchpad-chat`, the launchpad and a chat on a fake box | landed (`again`), or the reason it could not be |
| The render check and the walk | follow the landing, outside the plan, on main; fab's after-landing check, whose findings come back as repairs under this trailer |
