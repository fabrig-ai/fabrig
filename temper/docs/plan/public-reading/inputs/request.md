# public-reading: reading temper without signing in (design first)

At the user's request (11-03), carry out v0.1.0's public reading: the read-only, anonymous view of a temper deployment (`temper/docs/design/draft/v0.1.0.md`, section 5, "Public reading, the read-only public mode"). It has no draft of its own. Its direction is `temper/docs/design/draft/first-release.md`, 5.3, "Reading without signing in". Its status says it has no design: jig has no anonymous party, no public routes and no per-project visibility (jig's `domain/people.md` leaves guests open, its section 12), and temper has no public projections.

## This plan writes the design only

This exposes a deployment to anyone, so its design is reviewed for security before any code (the user's practice: design reviews by Codex). So:
- **Release the design**, in jig's and temper's design documents, with their status files' Not yet entries and the draft's status closed, but **no tasks**.
- **End the cut with a `Question:`** asking for the security review. Sketch the code batch in `notes/next-cut.md` (tasks, owners, order), as configuring-p2 did. A second plan builds it after the review.

## What to design, from 5.3

- **jig, the generic part:**
  - one anonymous party, fixed in code, whose role grants public reads alone, with a start refusing any policy that widens it;
  - a request with no credential served as that party;
  - its admission within a budget of its own, behind every signed-in request, answering busy first;
  - a few public `GET` routes, with no streams for it;
  - per-project visibility, which answers `domain/people.md`'s open questions on guests and who sees what.
- **temper, its part:**
  - the public projections, each a typed document built by a projection, as an allowlist: private by default, and a golden of the public types so that every addition shows in review;
  - their routes for a project's board, a task's public card and the landings;
  - the answers' `max-age` for a caching proxy;
  - the web client's anonymous mode, reading those routes, refreshing itself, never streaming.
- **What is public:** 5.3's lists hold.
  - *Public:* structure and mechanics.
  - *Private unless marked:* what people wrote; a goal's title only by its author's flag.
  - *Never:* what agents wrote.
  - *Linked, not shown:* commits and pull requests.
- **Open in 5.3, decided conservatively while the user is away; record each as Open in `first-release.status.md`:**
  - people's names do not appear;
  - the mirror fallback stays Later unless the review asks for it.

`AGENTS.md` 1 holds: jig never names temper. A budget or timeout is the user's (skein's `docs/foundation/testing-strategy.md`, 8.1); propose the anonymous budget's figures as Open for the user.

## Size

One cut is at most 90 minutes. Release the design in two batches, jig's then temper's.
