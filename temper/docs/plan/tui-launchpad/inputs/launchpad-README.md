# launchpad (hand-made)

Hand-made frames, not a recording. They show what the TUI could show after its first screen, drawn from `../../../../draft/tui.md`, section 3, at 80 by 24 in the dark theme. The values are written in by hand (the project Hello, csv-export, the status agent's brief), so no reprint will equal them.

The glyphs and words are the browser's chat terminal's (`../../../README.md`, 3.5, *A chat's terminal*): the person's words on a band after "❯", the agent's after "✦", calls folded under "▸", the running line "✦ working" with no clock, the commands' words, and the quiet line under the box. The chat reads as `../../../../draft/chat-reading.md` says: a run of calls is one line in words with its time, and the answer only grows, whole, its end a small block after the last word.

- `01`: the launchpad. A framed header with the project and the deployment, the status agent's brief in words, where you left off, what needs you, the composer at the foot, and the quiet line: "/ for commands", what needs you, the model and effort, and the link's state.
- `02`: `/` typed in the launchpad's composer. The commands it offers, in the browser terminal's order and words, the first one selected, in the quiet line's place. `/font` is the browser's alone, and `/exit` closes temper.
- `03`: a question typed in the launchpad's composer, then Enter. It opens a new chat, T2, whose top line is the browser terminal's. Then the person's words, the run's line, the answer still growing, the running line above the box, and the box taking a reply.
- `04`: the same chat later. The answer, committed in place without moving, the person's reply, and the agent's confirmation under the box as a list, as the browser terminal draws `/close`'s: its question as the head, the recommended choice first and selected, "Tell it something else" last.

Not drawn: before the first call or word, the three dots. While a call runs, the run's line counts what has ended and names the call being made ("reading change 2…").

Open:
- Once the composer has the focus, `q` types. Closing is Ctrl+C or `/exit`.
- The colours are the current theme's. The terminal's own background is a separate request.
