# public-reading: plan

v0.1.0's public reading (`temper/docs/design/draft/v0.1.0.md`, section 5): the read-only,
anonymous view of a temper deployment, from `temper/docs/design/draft/first-release.md`, 5.3.
This plan writes the design only (`inputs/request.md`): it exposes a deployment to anyone, so
the design is reviewed for security before any code, and a second plan builds it after the
review. Written by the cut as it releases; each batch brings it up to date.

## Parts

None: the request asks for no tasks. The code batch is sketched in `notes/next-cut.md` for the
plan that follows the review.

## The design, batch 1: jig's generic part

Released with the first push, by the lead:

- jig's `domain/people.md`, section 13 (new), "Public reading": 13.1 the public party, fixed in
  code, its role `public` granting public reads alone and refused in any policy; 13.2 a
  project's visibility, private by default, `SetVisibility` by its owners; 13.3 what it reads,
  jig's public documents (projects, a board, a card, the public line) and a goal's title shown
  by its author (`ShowTitle`); 13.4 who sees what and guests, answering section 12's two open
  questions; 13.5 its admission (a room, a rate of reads and bytes, behind every signed-in
  request, busy first, off unless figured); 13.6 its world. Sections 1, 4, 5.1 and 12 point to
  it.
- jig's `protocol/client.md`: section 19 (new), `PublicRead`; 17.5 (new), its `GET`, its
  answers' statuses and `Cache-Control`, no credential read, its room in the engine end;
  17.1's table and sections 1 and 13.
- jig's `client/domain/contract.md`: section 12 (new), the public documents' records; 3.2's
  `SetVisibility` and `ShowTitle`; 5.1's `me` saying a project is public.
- jig's `domain/engine.md`, 11 (public reads admitted on visibility); `domain/root.md`, 3 and 10
  (the public watermark); `domain/authority.md`, 6 (the role `public`); `domain/tasks.md`, 3 (a
  goal's title shown); `protocol/configuration.md`, 6 (the limits `clients.public.*`,
  `clients_http.public_calls`, the reserved role); `protocol/store.md`, 4.4 (the two flags).
- Statuses: each of those documents' Not yet entries.

## The design, batch 2: temper's part

Released with the second push, by the lead:

- temper's `client/forge.md`, section 11 (new), "Public documents": 11.1 the forge's public
  kinds, *change*, *landings* and *landing*; 11.2 what crosses (structure and mechanics),
  what is linked (pulls, commits and CI runs, as forge addresses) and what never does
  (questions, reasons, tiers' text, branch names, reviews, results); 11.3 their own
  `public-v1.schema`, naming no panel's record, and the text golden of public fields; 11.4
  their world. 9's table gains their row.
- temper's `client/browser.md`, section 8 (new), "Reading without signing in": the public
  pages and their `/public` addresses (8.1), reading by `GET` with no sign-in, stream or small
  store, again every `max-age` while visible (8.2), what each page shows (8.3), its tests
  (8.4); 2.3 and 2.4 point to it; 5 gains a shared stream per document as an open question.
- temper's `protocol/sign-in.md`, 2.4 (new): a deployment opening, the proxy caching the
  public `GET`s alone and limiting each address.
- temper's `client/tools.md`, 5.2 and 5.3: `project amend PROJECT --visibility
  public|private`, `goal amend TASK --title shown|hidden` (the closed vocabulary's `amend`,
  no new verb), `project show`'s visibility.
- jig's `client/domain/contract.md`, 12 and `protocol/client.md`, 19, compatible detail: the
  public records are a schema of their own, versioned apart, naming nothing outside it.
- Statuses: forge, browser, sign-in and tools Not yet; the draft's `first-release.status.md`
  5.3 entry closed, with the design that carries it, one Design departs and three Open
  entries; `v0.1.0.status.md`'s 5 entry narrowed to the site.

## Critical path

No tasks; the security review comes first (the report's Question). The code batch as
sketched (`notes/next-cut.md`): `public-surface` (90) -> `visibility-core` (120) ->
`public-reads-core` (180) -> `forge-public` (150) -> `public-walk` (120), about 660
implementer-minutes, with `public-wire` -> `public-component` -> `public-http` ->
`public-web` (570) beside it; 13 tasks in two parts, `jig-public` and `temper-public`.

## Decided along the way

- **The board's tasks are jig's.** A project's board, a task and its tree are jig's documents
  (`client/domain/contract.md`, 5.1), so their public forms are jig's too, generic: a public
  line holds structure and mechanics alone. temper's projections are its forge's documents and
  its landings, as 5.3 says ("temper holds its projections (its forge's documents, its
  landings)").
- **A goal's title flag is jig's,** since the title is a jig task's spec's first line; the
  flag is refused on a goal its requester did not write (one accepted from a proposal) and
  cleared by another's amendment or a move.
- **Only goals' trees are public:** a conversation and its delegates, and tasks in no goal's
  tree, have no public line.
- **Public reading is off unless figured:** every profile's public rate is zero; dev.fabrig.ai
  sets its figures, which are the user's (the draft's status, Open).
- **One `max-age` per deployment,** at most 60 seconds, for every public answer, jig's and the
  application's, and the bound on how long a project made private stays readable at a proxy.
- **Public reads alone go without a credential:** every other credential-less exchange stays
  signed out, so the public party's surface is one message (the draft's status, Design
  departs: the review may say otherwise).
- **The public records are schemas of their own** in jig's and temper's wires, naming no
  record of the private documents, so a field added to a panel or a task never reaches a
  visitor; a text golden of their fields is what review sees change.
- **Visibility and the title flag through ctl** use the closed vocabulary's `amend`
  (`agent-docs.md`, 7.2): a new verb would change a draft, which a cut never edits.
- **Names hidden, the mirror Later, the figures the user's:** the request's conservative
  defaults, recorded as Open in `first-release.status.md`, with proposed figures for
  dev.fabrig.ai.

## Overlaps with other plans

- configuring-p2-build's merged tasks (on its trunk, not yet on main) edit jig's
  `domain/engine.status.md` (`restore-story`), `client/domain/contract.status.md`
  (`settings-wire`) and temper's `client/tools.status.md` (`policy-ctl`, `restore-ctl`,
  `restore-story`). This cut adds one entry to each (engine's 11; contract's 3.2, 5.1 and 12;
  tools' 5.2 and 5.3); whichever lands second merges a line.
- No code file is changed by this plan.
