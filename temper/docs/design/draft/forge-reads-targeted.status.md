# forge-reads-targeted: status

2026-11-10. Where the design lags or departs from [forge-reads-targeted.md](forge-reads-targeted.md).

Items 1 and 5 are designed, with item 6's text for them: every containment reader reads the ancestry range (`../domain/forge.md`, 5.3 and 11.2; skein's `forge-client.md`, sections 4 and 7), a comparison reads names and counts alone (skein's `forge.md`, 3.2 and 3.3), and a `TooLarge` names its bound and size (skein's `forge.md`, 3.4; `../domain/forge.md`, 8.4; `../domain/agent.md`, 4.3).

- **Design lags:**
  - Item 1: goal overlap reads a pull request's files, both sides, for their paths alone (`../domain/forge.md`, 7.3); a pull request's paths alone wait for item 4's pages (skein's `forge.md`, section 12, contents on demand).
  - Item 2: `../domain/forge.md`, 5.3 folds every status row of every page, and skein's `forge.md`, 5.2 reads a pull request's statuses with each pull read and each status link's run whole; no read is once per gather.
  - Item 3: skein's `forge.md`, 5.2 finds a pull request by paging every open one, reads an issue's comments whole, and bounds a branch list at `rows`.
  - Item 4: skein's `forge.md`, 5.1 sizes a page for the largest row a listing may hold.
  - Item 6: the design for items 2 to 4.
