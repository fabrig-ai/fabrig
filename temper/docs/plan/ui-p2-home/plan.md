# ui-p2-home: plan

Milestone P2 of `temper/docs/design/draft/whole-story.execution.md`:
Home and Needs you across projects, and the brief
(`temper/docs/design/draft/surfaces.md`, sections 3, 4 and 12). The
design this cut wrote: `temper/docs/design/client/README.md`, 3.5,
*Home* (*The cards*, *The Ask temper line*), *Home on All projects*,
*Needs you*, *A page's parts*; `temper/docs/design/client/browser.md`,
2.3's `AllHome` row and 7.2's new states; their status files, and the
lags left in `temper/docs/design/draft/surfaces.status.md`.

## Parts

| Part | What it makes run |
|---|---|
| `across` | Home on All projects: `AllHome` at `/home`, every project's `status` and `decided` read, the briefs joined, Projects, the Ask temper line |
| `needs` | Needs you across projects as a list on a wide page, *Tell it something else*, Decided for you on Needs you; Home's needs as at most three cards |
| `density` | `pages::parts`'s rules (section 12) held by a fuzz test over each page's model |
| `proof` | the gallery's states, the joint world's stories across two projects |

## Tasks so far

| Task | Part | Needs | Crates |
|---|---|---|---|
| `all-home-surface` | across | | `temper-client-domain` (address, `all_home`, `parts`), forced arms elsewhere |
| `needs-list` | needs | | `temper-client-view` (Needs you), `temper.css`, the gallery's needs states |

The rest, written next: `all-home-reads` (domain: the reads across
projects, Needs you's decisions, `Act::AskWith`, Overturn off a project's
Home), `all-home-page` (view: the page from its facts), `home-cards`
(view: Home's cards on both Homes, the Ask line, Needs you's Decided for
you), then `gallery-across`, `joint-across` and `density-fuzz`.

## Critical path

`all-home-surface` → `all-home-reads` → `home-cards` → the proof tasks;
`needs-list` → `all-home-page` → `home-cards` beside it. One writer per
crate: the domain's tasks run in that order, the view's in theirs.

## Decided

- **No engine change.** Each project's `status` record already holds
  what the brief across projects needs (`landed`, `seen`, `previous`, its
  brief, `red`, `branch`); jig's inbox and `approvals` already span
  projects. A part a read lacks (owners' Nows, tiers, a follow's parity,
  "owned", why a need reached the person) is a lag in the design's
  status, not a placeholder.
- **`AllHome` at `/home`,** a new address, not `Home { project: Option }`,
  so that every reader of a project's Home stays as it is; `/` stays
  Needs you across projects. It draws in the open project's frame, the
  rail untouched (P1's), "Home" current. Reached for now from Needs you's
  Done "Home" with two projects or more; P1's switcher row "All projects"
  leads to it.
- **At most 16 projects read** (`all_home::ACROSS`), the rest left out,
  so that the page's watches stay bounded.
- **The joined lede names each project** ("vsr: …"), since each brief is
  its own project's words (`surfaces.md`, 15's default).
- **"Later" leaves the browser's Needs you** and Home's cards, as the
  mockups draw them; the domain keeps it and the terminal's inbox shows it.
- **Home's "Work carries on…" line goes:** no part is drawn empty
  (`surfaces.md`, 14); the headline still says "Nothing needs you.".
- **A project's Home keeps its Away form of Decided for you** as it is
  today; widening Decided for you to any task is N1's (`what-reaches-you.md`,
  6), so this plan stays off its rule.
- **Stored formats, protocols, public interfaces:** none changes.

## Who hears first

- **temper/tui (T1 reads Home's brief):** `temper-client-domain` gains
  `Address::AllHome` (`/home`; every `match` on `Address` gains an arm,
  `temper-client-tui`'s and `temper-client-shell-tui`'s included),
  `pages::all_home` (`Facts`, `intro`, `brief`, `first_sentence`, the
  reads on `AllHome`), `pages::parts` (`home`, `needs`, `needs_facts`),
  `home::Facts::parts_facts`, and `Act::AskWith`. T1's framed brief is
  `all_home::brief`, pure over `all_home::Facts`.
- **The owner of coding runs:** nothing in `temper-engine-domain` or
  `forge_route.rs` changes.

## Overlaps not avoided

- **ui-reaches (N1, cut in flight):** its why, Stop asking and Decided
  for any task meet this plan's Needs you card
  (`temper-client-view/src/pages/needs.rs`), Home's decisions
  (`pages/home.rs`) and Overturn (`pages/away.rs`). This plan draws no why
  and keeps the decisions' rule, so the overlap is the files, not the
  meaning.
- **look-round-24's `role-pages` (integration):** it changes
  `temper-client-domain/src/address.rs` and `pages.rs`; `all-home-surface`
  adds an arm beside its own.
- **tui-box-front (integration):** `AllHome`'s forced arms reach
  `temper-client-tui/src/view*.rs` and `temper-client-shell-tui/src/host.rs`.
- **Every UI plan:** `look.rs`'s style tokens, `words.rs`'s `Word` and the
  gallery's `samples.rs` and `names.txt` are appended by all of them.
