# A chat in a terminal

The look request `chat-terminal`, held: an experiment the user is weighing. Hand-made, since no journey
reaches it. Page 01 is a recorded chat with a new button; the terminal over it holds that chat's words.

| Page | Shows |
|---|---|
| 01 | The chat as it is, with `>_` beside Chat, Split and Draft. It opens the terminal, full screen |
| 02 | The terminal |
| 03 | With the draft beside it |
| 04 | `/` opens the commands |
| 05 | `/model` lists its choices |
| 06 | While the agent works |

The pages are live: the buttons, `Ctrl+\`, the commands and PgUp work, and a message sent is answered by
a line saying nobody answers. The look is `terminal.css`, the behaviour `terminal.js`. `/theme`, between dark and
light, and `/font`, between DejaVu Sans Mono and IBM Plex Mono, are there to compare them.

`fonts/` holds the two weights of DejaVu Sans Mono, which a landing moves to the shell's `files/fonts/`. Its
`README.txt` says where they come from and how they were cut.
