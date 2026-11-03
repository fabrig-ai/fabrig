# public-reading-build: settle the security review, then build public reading

At the user's request (11-03), build v0.1.0's public reading. Plan `public-reading` released its design and cut no tasks:
- jig: `domain/people.md` 13, `protocol/client.md` 17.5 and 19, `client/domain/contract.md` 12, `domain/root.md` 3 and 10, `domain/authority.md` 6, `protocol/configuration.md` 6;
- temper: `client/forge.md` 11, `client/browser.md` 8, `protocol/sign-in.md` 2.4, `client/tools.md` 5.3.

Its plan's `plan.md` and `notes/next-cut.md` sketch the build (13 tasks in two parts, `jig-public` and `temper-public`). This trunk starts from that plan's trunk, so all of that is here.

A Codex security review of that design (xhigh, read-only) found 12 issues and judged it safe to build once they are settled in the design. The review is `inputs/review.md` beside this request.

## First: settle the review in the design

Settle every finding of `inputs/review.md` in the design text before cutting tasks that depend on it. Release it as the first batch. In brief:
1. **The title flag:** a party's credential cannot prove a person wrote the title, since agents use people's credentials (`client/tools.md` 5.3). Keep goal titles out of the public records for now (Later, with a human publication capability). Apply the same rule to configured public names.
2. **Eligibility closes over everything nested:** one predicate for every parent, member, link, count, queue position and total. Ancestry stops at the public goal. Filter before counting.
3. **Revocation during a read:** recheck visibility, deletion, membership and revision after asynchronous work, before the answer, with races in the world.
4. **Caching:**
   - `must-revalidate`, no stale serving, and the age kept through caches;
   - the browser drops content when its freshness ends;
   - the bound is on later serving only.
5. **Signed-in capacity is protected:**
   - loop share, store loads, memory and connections, not only the backlog watermark;
   - the proxy admits public reads before upstream capacity;
   - configuration relations are checked at start.
6. **The byte budget** reserves an answer's maximum before work and refunds after, with burst sizes and a whole-answer ceiling.
7. **The work per request is bounded:** rows examined, loads, depth and bytes, with defined truncation; prefer indexes of public data.
8. **Public identifiers are opaque,** not internal project or task numbers.
9. **Indistinguishable failures:**
   - inaccessible projects are refused before task work;
   - lookups go through a public index;
   - error answers are fixed;
   - paired-world checks compare absent and inaccessible.
10. **The proxy's cache key** is deployment and validated origin plus the canonical path; the authority is checked before lookup.
11. **The golden covers the whole reachable wire shape,** with a field-level provenance table and tests that fill forbidden source fields with markers.
12. **A security-negative walk and fuzz targets** have owners and acceptance in the build (see the review's list), including CORS on errors and preflights.

Where a fix needs a figure (a budget, a bound, a timeout), propose it as Open in `temper/docs/design/draft/first-release.status.md`: figures are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). Public reading stays off by default: zero rates in every profile, every project private.

## Then: the build

Cut the tasks from `notes/next-cut.md`, amended by the revision. Add the review's negative walk and fuzz targets as tasks of their own. Mark as `walk = true` the slice where an anonymous visitor reads a public project's board, a goal's tree and the landings in the web client, and sees nothing private.

## Context

- `AGENTS.md` 1 holds: jig never names temper.
- The user's open calls, recorded in `first-release.status.md`, stay Open: the figures, names hidden, the mirror Later, and only public reads served without a credential.
- The design was released, not landed; it lands with this plan.

## Size

One cut is at most 90 minutes. Release the revision first, then the jig part's tasks, then temper's.
