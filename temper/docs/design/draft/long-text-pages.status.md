# long-text-pages: status

2026-11-09. Where the design lags or departs from [long-text-pages.md](long-text-pages.md).

- **Design lags:** none. `../client/tools.md`, 6.4, 11.1 and 12 carry parts 1 to 3; the code lags as `../client/tools.status.md` says.
- **Design departs:**
  - Item 1: a read that prints a cut text inside a larger answer, `task show`'s asked words, `task turn` without `--part`, `task activity` and `chat show`, is no paged read and keeps its end line alone; the paged read its call names says so first (`../client/tools.md`, 6.4). The cut's call, 2026-11-09.
  - Item 2: the client's text stays 32 KiB, named apart from a result's text: jig's wire takes at most 64 KiB of it (`task_text_words`), short of a page with its lines, and `task text` joins the engine's chunks, so no client bound had to rise. The step test of `../client/tools.md`, 11.1 holds as that document's section 12 says it: a result's text within one document's bytes, a page within a turn's, the client's text within a result's (`../client/tools.md`, 11.1 and 12). The cut's call, 2026-11-09.
- **Open:** none.
