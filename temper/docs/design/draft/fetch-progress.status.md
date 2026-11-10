# fetch-progress: status

2026-11-05. Where the design lags or departs from [fetch-progress.md](fetch-progress.md).

- **Design lags:** none.
- **Design departs:**
  - 2: a whole percent's line falls at most once for a chunk, so a chunk past several percents says one line, not one for each (`../protocol/setup/owned.md`, 6). The cut's call, 2026-11-04.
  - 3: not every fetch line becomes a pipe line: the box holds the latest alone and writes it once its replies are written, because its reply queue is short and drops on no room, so a slow serve loses a figure, never a `failed`; and a line left in the step's pipe after its exit stays in the log, so none follows the next `step` line (`../protocol/setup/box.md`, 2). The cut's call, 2026-11-04.
  - 4: the forge domain keeps the latest `fetched` line, beside the forge's connection, not the box's link: the link decides nothing and keeps no state a row reads (`../protocol/forge/engine.md`, 8), and the row is the domain's. In memory alone all the same, and gone as the draft says. The cut's call, 2026-11-04.
  - 5: in the browser the muted detail names the binary alone, "Setting up: fetching Forgejo", and the figure stands at the bar's end, as the look request's pages draw it; the paragraph's name, the terminal and every other page say the whole line (`../client/pages/README.md`). The cut's call, 2026-11-04. The step list's first step names the versions where the row has them, "Fetching Forgejo 17.0.0 and its runner 13.2.0 · 42%", as it did without a figure (`../client/forge/deployment.md`).
  - 7: a body past its size is refused at the fetch's machine, and no test shows it at the host: skein-http delivers no byte past `Content-Length`, so the host never meets one (`../protocol/setup/owned.md`, 6).
  - 8: a walk does see the fake box's fetch: `welcome-set-up`'s recording holds its rows, about 190 changes across both binaries, each with its size. They keep the page moving, and `again` takes no page while it moves, so its pages 07 and 08 fall at preparing, the set up's words in place of the look request's 42%. The answer call's, 2026-11-04. The percentage's look stands on two gallery states (`../client/web/gallery.md`), the fake fetch stays Later for that reason (`../protocol/setup/README.md`, section 9), and a replay reads those rows back with no box, so no test runs a fetch through a box to a client.
- **Open:** none.
