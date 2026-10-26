# temper's client, in mockups

Revised 2026-10-24, with the user. The mockups are the design of
temper's client's pages, as design documents are of its code: what
each page shows, in what order and words, and where each act leads, in
its intended form. `STATUS.md` says how far the client is from them,
and which pages are under the rules below yet.

## 1. In one page

- **Binding for what is not built yet.** temper is the source of truth
  for what it draws (the user, 10-26): where a mockup differs from what
  temper draws, the mockup is reconciled to temper, gradually, and a
  built page's look is tuned on its journey's print
  (`../journeys.md`, section 7). For what is not built yet, a page's
  look, its parts and their order, its words and where each act leads
  are its mockup's. The client's documents (`../README.md`,
  `../forge.md`, `../browser.md`) hold what a page reads, watches and
  does, and cite the mockup for the rest. The two agree: a change to
  one changes the other in the same landing, and a disagreement found
  is settled, never recorded.
- **Realistic.** A page's main drawing shows only what the engine has
  today: the data the client can read and the operations it can call,
  so that building the page is the client's work alone. Each value
  cites where it comes from (section 3).
- **The future, beside it.** What would need work below the client (the
  engine, jig, a protocol, temper box) is drawn only under View →
  Future, each piece marked with what it needs. Once that lands, the
  mark goes and the piece joins the main drawing.
- **Page by page, state by state.** Every state of a page has an
  address of its own, and the contents page lists them page by page.
  Worlds hold the data that keeps the pages consistent; a journey is a
  world whose steps walk one path (section 4).
- **`STATUS.md` keeps the distance:** what a main drawing shows that
  the client doesn't draw yet, where the client does otherwise, and the
  pages not yet under these rules.
- **Brought under the rules a group at a time:** an audit, the user's
  review, then the drawing, the documents and `STATUS.md` changed
  together (section 5). The bar above each page says whether it is.

## 2. Opening them

```sh
python3 -m http.server 8765 --directory docs/design/client/mockups
# http://localhost:8765/index.html
```

Each file also opens directly in a browser, with no build step; serving
them keeps the chosen world, step and switches from page to page. They
fetch nothing from the network: their fonts are the machine's, the
monospaced stack and its sans and serif.

- **The bar** above each page is the mockups' own: ‹ and › move the
  step, **Steps** lists the world's steps, the step's title says what
  it shows and where to look, and **View** switches the world and the
  marks: **Annotations**, **Sources** and **Future** (section 3).
- **An address is a state:** the page, `?world=`, `?step=` and the
  page's own query (`welcome.html?world=first-run-real&step=2&at=model`).
- **For a capture,** `?bare` draws the page alone, without the bar and
  the marks, and every link keeps it; `?bare&future` draws it with its
  future pieces, unmarked. View's **Hide this bar** does the same from
  page to page, keeping Future as it was set; Esc, or the bottom-right
  corner's handle, brings the bar back.
- **Acts** that lead to no drawn state show a note instead of acting.
  Nothing is sent anywhere.

## 3. Sources and marks

temper is a generic engine, so a page assumes no workflow. Every value
it shows comes from a named source, drawn through one helper each in
`pages.js`, and underlined in its colour while **Sources** is on;
hovering one shows its citation.

| Source | What it is | Its citation |
|---|---|---|
| Engine fact | what jig or temper keeps or computes, and the client can read | the design section that defines it, and its record and field: jig's `client/domain/contract.md`, 7.1, `Account.display` |
| Configuration | what the deployment's configuration gives | its key, and `../../protocol/configuration.md`'s section |
| As written | words a person or an agent wrote, with the writer | the writer, and when |

- **The UI's own words** are unmarked, and name no workflow: a
  workflow's words (the director, an owner, a check) appear only in
  what was configured or written. A page that reads wrong in one world
  assumed another's workflow.
- **Future** (`future · NEED`, slate, dashed): a piece the engine
  can't supply yet. NEED names what would: implementation work under way,
  a draft, or "not designed". With **Future** off the page shows what
  it shows today in its place, if anything.
- **The mockups' notes** are slate, never the product's palette: a
  note on an act that leads nowhere drawn, the address a GitHub page is
  at, a world's case names (`B`, `f1`, under **Annotations**).
- **Not under the rules yet:** a page the bar marks so may still draw
  what the engine lacks with older marks, `ahead · LABEL` and
  `gap 6.N`, and an ops application's values (`STATUS.md`, section 2).

## 4. The structure

- **Pages:** one HTML shell for each of the client's pages, named after
  its address in `../README.md`'s 3.5 (`connect-forge.html` is
  `ConnectForge`), all drawn by `pages.js`. Two draw pages a journey
  meets that are not the client's: `github.html`, GitHub's own, each
  under a slate strip with its address; and `sign-in.html`, the
  signed-out front page and the return route's answers.
  `first-run.html` opens the first run at its first step, the link
  `temper box` prints.
- **Worlds:** `world-*.js`, each a deployment's data, moved on step by
  step, so that every page agrees at each step. A step names the
  addresses where it shows (its `look`), and the contents page gathers
  them page by page.
- **Journeys:** a world whose steps walk one path through the pages,
  its happy path: the welcome's setup on a Forgejo here
  (`world-first-run-real.js`), or on GitHub
  (`world-first-run-github.js`). A branch or an ending off that path is
  a state: an address of its own at the step where it can happen,
  listed in that step's `look`. A page draws no gallery of its other
  states.
- **The look:** `style.css`, `terminal-chat.css` and
  `terminal-pages.css` sketch the look of what is not built yet; the
  product's is `temper.css`, tuned on the journeys' prints
  (`../browser.md`, 2.5; `../journeys.md`), to which these sheets are
  reconciled. `mockup.css` is the mockups' own bar and marks, never the
  product.

## 5. Keeping them

- **The user draws them,** in a session with Claude. A change to a drawing lands with its
  documents' change and its `STATUS.md` rows.
- **The UI is built from them**. A landing that draws
  what a mockup shows removes its `STATUS.md` row, and one that leaves
  part of it adds one. A future piece is built once its need has
  landed: the build moves it into the main drawing and drops its mark.
- **A group of pages comes under the rules** in three moves, one group
  at a time:
  1. **An audit,** by Codex, from `audit.md`'s brief: every state of
     the group's pages, each value, word and act in it built, lagging
     or future, with its evidence in the code; where the mockup, the
     documents and the code disagree; stale marks; and what the client
     draws that no state shows.
  2. **The user's review,** in a session with Claude: what stays in the
     main drawing, what goes under Future and what goes; each
     disagreement's side; the states to add or drop.
  3. **One landing:** the drawing (future pieces marked, citations,
     states as addresses, galleries gone), the documents (a passage
     restating the mockup's words or order cut to a citation of it, a
     disagreement settled) and `STATUS.md`'s section for the group,
     with the audit's date. `pages.js`'s `REVIEWED` names the group's
     pages, so the bar says so.
- **An audit is redone** when a group's pages or the engine under them
  have moved much, at the user's word.
