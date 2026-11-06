# chat-reading-d: a numbered list is a numbered list

The fourth of four parts of `temper/docs/design/draft/chat-reading.md`: section 6 alone, in skein's Markdown. The user took each recommendation there, 2026-11-06. It's independent of the other parts, so it goes now. Read the draft's section 6 and do what it says; if something doesn't hold when you get there, say so in `plan.md` and take the nearest thing.

Failing tests first, in skein's tier for its Markdown: an ordered list renders as one, its numbers as written, in the web and the terminal where skein renders both; a line that merely starts with a number and isn't a list stays text. skein names no temper, and the kit's rules hold (`skein/AGENTS.md`). `skein`'s Markdown isn't on hearth's surface: say so if it is. If temper's stylesheet needs a rule for it to read right, name it, and leave it to the look's owner if it falls in `temper-client-*`. fabrig/ui has priority.

One task. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). `walk = false`: the user reads it on a real chat.
