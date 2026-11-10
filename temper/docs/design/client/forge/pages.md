# The forge's pages

- **The forge's parts** of a page (a goal's issue and changes on its
  page; changes in flight and the project's health on `Changes`) are
  `README.md`'s, and "Shown on the forge." while the forge's side does
  not read them (`family.md`, 2).
- **The forges' page** is the forge's part of the deployment's
  connectors (jig's `client/domain/pages.md`, section 7), on a page of
  temper's own (`deployment.md`). It is read for a person whose role
  on project 0 is `Owner`, in standalone the deployment's owner; any
  other's read is refused, and the page says so in jig's words, and a
  request the root refuses for the role says "Your role does not hold
  the deployment's forges." Its acts are the FORGE
  requests (`../engine.md`), each sent as `../README.md`, 3.6
  says; a pasted key or token is a secret field (`../README.md`, 3.6). A connect's or
  an install's link is where the page's own tab goes, once (`Request::Leave`,
  from the answer), the request naming
  the page's address as its `back` (`../engine.md`); GitHub's last step
  sends the tab back there (`served.md`, 1), and the page shows a
  whole address's link as an external link meanwhile (`../line.md`, 5); a
  connect's path (`../engine.md`) leaves at once, and none is drawn. How a
  connect ends is read on the page, never assumed from its answer. Its
  rows are the forges the store keeps, made by a set up or a connect
  (`../../protocol/forge/engine.md`, 1 and 2); "Remove" on a row, confirmed,
  sends `RemoveForge` (`../engine.md`; `deployment.md`). While the
  playground runs, it lists the playground's forge and offers nothing
  to add (`family.md`, 7).
- **Changes** (the user's greenfield mock's `changes.html`; `README.md`,
  section 3): `Changes { project }`, at `/projects/<project>/changes`, Work
  under its Changes filter (`../pages/work.md`, 1, *Work*), a
  project page in the rail's frame, a column as Home's, at most 720 px
  wide and centred, its parts 60 px apart, 32 on a phone. BRANCH below is
  the name of the one branch the queues go into; with none queued, the
  document's `landing` (`records.md`); with more than one, or no name to say, the
  heading says "landing" and the lede "its branch". In order:
  - Work's head and filters (`../pages/work.md`, 1, *Work*), "Changes" current, in the
    kicker's place;
  - the heading, at level 2: "Nothing is on the way to BRANCH." with none
    queued; "One change is on the way to BRANCH."; from two to nine the
    number in words, "Three changes are on the way to BRANCH."; from ten
    in digits, "12 changes are on the way to BRANCH.";
  - the lede: "Each change is checked on top of BRANCH before it lands.";
    the landing's line is Work's head's. **The landing's line,** one function's,
    its parts joined by " · ": with something queued, "3 queued" and, once
    anything has landed, "last landed 6 min ago"; with nothing queued and
    something landed, "main last moved 1 h ago" and "nothing queued"; with
    neither, "nothing has landed yet"; then, for each branch `red`, "main
    red", in the signal's colour. The times are `../view.md`, 6's *ago*. No green
    time is kept, so none is said;
  - "Queue", a section, with its rows: at the left the change's title, a
    space and, in the mono face, where it came from; at the right, on the
    first, "checking" in the good colour and " · checks started 6 min
    ago", by `../view.md`, 6's *ago*, and on the others "queued 2nd", "queued 3rd",
    "queued 4th". Past the document's rows, a last row, not a link, "and
    N more". With no rows, "Empty.". With several branches holding rows, a
    section each, "Queue · owner/name BRANCH", by the document's order;
  - "Landed recently", the same section: the title, where it came from,
    and at the right its time by `../view.md`, 6's *clock*, newest first. With no
    rows, its rule alone.

  *Where a change came from:* the title of its task's `goal`, the chip
  in its `lineage_chips` naming that task; else the root of its
  `lineage_chips`, "chat T6", where that root is on the charter of the
  project's chat template; else nothing. The page reads each row's task, as a task's
  page reads a chip, those of the rows shown alone; until one arrives,
  its row has no mono part. Every row but "and N more" is a link to its
  change's `Task { task }`. Until its first document, the page is Work's
  head; with its watch refused or ended, Work's head and "Shown on the
  forge." (`family.md`, 2). The terminal prints the page; the view is shared.
