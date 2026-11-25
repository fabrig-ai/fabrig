# look-loop: status

2026-11-25. Where the design lags or departs from [look-loop.md](look-loop.md).

- **Design lags:** none.
- **Design departs:**
  - 4.2, 5: `../../client/journey-writer.md`, 6.1, gives a page whose edits all landed one row, `landed`, and each hunk where they didn't a row, rather than a row per edit: git's merge says whether a page's edits landed, not which hunk is which edit. The comparison's design, 2026-11-25.
  - 4.2, 6: `../../client/journey-writer.md`, 6.1, exits 3 where another git child, the print or a file fails, which the draft's three exits leave out, so that a comparison that couldn't finish never reads as one with rows to account for. The comparison's design, 2026-11-25.
  - 5.2: `../../client/journey-writer.md`, 6.2, seeks the pages a request's rules select in every journey under the overlay's root, not only in the journeys whose pages the request changes, so that a request of rules alone is rendered too. The render check's design, 2026-11-25.
  - 5.2: `../../client/journey-writer.md`, 6.2, lists a page's first 50 differences and counts the rest, and compares no custom property (`--*`), whose effect shows in the properties it feeds, so that one inherited change doesn't bury a verdict in thousands of lines. The render check's design, 2026-11-25.
  - 5.3: `../../client/journey-writer.md`, 6.3, gives a request whose `docs/look/NAME` is missing, or isn't a request, an entry `not checked`, saying why, where the draft checks only requests whose branch holds them, so that the person's branch can drop it (5.4). The render check's design, 2026-11-25.
- **Open:** none.
