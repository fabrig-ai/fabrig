# forge-reads-targeted: status

2026-11-10. Where the design lags or departs from [forge-reads-targeted.md](forge-reads-targeted.md).

Items 1 and 5 are designed, with item 6's text for them: every containment reader reads the ancestry range (`../domain/forge/keeping-up.md`, 3, and `../domain/forge/branches.md`, 2; skein's `forge/client.md`, sections 4 and 7), a comparison reads names and counts alone (skein's `forge/README.md`, 3.2 and 3.3), and item 3's narrow lists are designed (`PullFor` head/base filters, `Item` since, and branch names bounded by bytes alone; skein's `forge/README.md`, 3.7, and `forge/forgejo.md`, 1.2; `skein/docs/design/forge/github.md`, 4.1), runs are resolved once per call and a short first status page supplies `Pull`'s combined state (skein's `forge/forgejo.md`, 1.1 and 1.2), and a `TooLarge` names its bound and size (skein's `forge/README.md`, 3.4; `../domain/forge/change/repairs.md`; `../domain/agent/tools.md`).

- **Design lags:**
  - Item 1: goal overlap reads a pull request's files, both sides, for their paths alone (`../domain/forge/README.md`, 5.3); a pull request's paths alone wait for item 4's pages (skein's `forge/README.md`, section 10, contents on demand).
  - Item 2: `../domain/forge/keeping-up.md`, 3 folds every status row of every page, and skein's `forge/forgejo.md`, 1.2 reads a pull request's statuses with each pull read and each status link's run whole; no read is once per gather.
  - Item 4: skein's `forge/forgejo.md`, 1.1 sizes a page for the largest row a listing may hold.
  - Item 6: the design for items 2 to 4.
