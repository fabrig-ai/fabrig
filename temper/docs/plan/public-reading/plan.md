# public-reading: plan

v0.1.0's public reading (`temper/docs/design/draft/v0.1.0.md`, section 5): the read-only,
anonymous view of a temper deployment, from `temper/docs/design/draft/first-release.md`, 5.3.
This plan writes the design only (`inputs/request.md`): it exposes a deployment to anyone, so
the design is reviewed for security before any code, and a second plan builds it after the
review. Written by the cut as it releases; each batch brings it up to date.

## Parts

None. The code batch is sketched in `notes/next-cut.md` for the plan that follows the review.

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

Written next: the public projections (temper's forge documents and landings), their routes
and `max-age`, the web client's anonymous mode; then the draft's status closed.

## Critical path

No tasks; the review comes first. The code batch's path, as sketched, is in
`notes/next-cut.md`.

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

## Overlaps with other plans

- configuring-p2-build's merged tasks (on its trunk, not yet on main) edit jig's
  `domain/engine.status.md` (`restore-story`) and `client/domain/contract.status.md`
  (`settings-wire`). This cut adds one entry to each, at 11 and at 3.2, 5.1 and 12; whichever
  lands second merges a line.
- No code file is changed by this plan.
