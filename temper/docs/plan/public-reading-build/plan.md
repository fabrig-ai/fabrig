# public-reading-build: plan

v0.1.0's public reading (`temper/docs/design/draft/first-release.md`, 5.3), built after its
security review (`inputs/request.md`, `inputs/review.md`). The cut first settles the review's
twelve findings in the design, released with the first batch, then cuts the build from
`temper/docs/plan/public-reading/notes/next-cut.md` as the revision amends it. Written by the
cut as it releases; each batch brings it up to date.

## The revision (released with batch 1)

How each finding is settled, and where:

1. **The title flag:** gone. No public record holds text or a configured name (a goal's
   title, a project's, charter's, procedure's, gate's or landing branch's name): a credential
   proves who sent a request, not who wrote it. A human publication capability is Later
   (jig's `domain/people.md`, 12 and 13.3; `temper/docs/design/client/forge.md`, 11.3;
   `first-release.status.md`, Design departs). `ShowTitle`, `goal amend --title` and the tasks
   family's flag are removed from the design.
2. **One rule for what is public:** people.md 13.7 (new): a public goal is a goal of a public
   project whose parent is in no goal's tree; a public task is one in its tree that the public
   index holds; ancestry stops at the public goal; every reference, count, position and total
   passes it, filtered first, and a value that would disclose what is not public is not given.
   temper's: forge.md 11.2 (new): a public change's task is public; a landing is public only if
   every member is; a pin-only member or a deployment's landing task is never named; a queue
   lists public changes without places or tasks ahead.
3. **A read under way:** people.md 13.10 (new): rechecked in the step that completes it, after
   its connector and its wait to be durable; revocation never waits for reads; races in 13.6.
4. **Caching:** client.md 17.5: `must-revalidate`, no stale directives, the `max-age` shortened
   by the time since the answer's decision, no `Age`/`ETag` from the origin; sign-in.md 2.4: the
   proxy serves nothing stale and keeps `Age`; browser.md 8.2: the page drops a document when
   its freshness ends; the bound is on later serving only.
5. **Signed-in capacity:** people.md 13.5 and 13.9: public reads load nothing from the store
   (the public index), their work per read is bounded, the core holds at most `public_reads`
   of them (root.md 3, 10), their HTTP calls have a room of their own; sign-in.md 2.4: the
   proxy admits public reads before its connections to the engine, in a pool apart; the
   relations are checked at start (configuration.md 6).
6. **The byte budget:** people.md 13.5: bursts, the kind's largest answer reserved before work
   and refunded after on every end, an answer ceiling, inconsistent figures refusing the start.
7. **Bounded work:** people.md 13.9 (new): the public index, bounded per project, read alone;
   bounds per read; truncation by rule; the whole document priced.
8. **Opaque identifiers:** people.md 13.8 (new): a keyed 64-bit permutation of the number,
   apart for projects and tasks, under the clients' component's key in the secrets' store
   (store.md 9); routes and records carry identifiers alone; `me` gives a public project's.
9. **Indistinguishable failures:** people.md 13.2, 13.8, 13.9: every well-formed read mapped
   and handed on alike, the index's miss the one path for absent and excluded; client.md 17.5:
   fixed answers and headers; paired checks in the worlds and the negative walk.
10. **The proxy's key:** sign-in.md 2.4: deployment and validated `Host`, then the path as
    received; `Host` checked before the cache; cookies, `Authorization` and client forwarding
    headers dropped.
11. **The golden:** contract.md 12 and forge.md 11.3-11.4: the golden of the whole reachable
    wire shape, a field-level source table, projections tested against independent records
    with forbidden sources holding markers, and mutation tests.
12. **The negative walk and fuzzy tests:** sign-in.md 2.5 (new), owned by `public-negative`;
    fuzzy tests in client.md 19 and browser.md 8.4, owned by `public-fuzzy-jig` and
    `public-fuzzy-temper`.

Also settled on the way: `public` has no role number, so no policy can name it (no start
check is needed); owner numbers on the route (0 jig, family + 1), the root mapping a family to
its connector; a store written before public reading holds no public row and reads every
project private.

## Revisions after batch 1

Compatible detail, released with batch 2b; no released spec relied on what changed:

- **people.md 13.9, engine.md 11:** the public index's ended goals are those ended since start
  or publication; the store keeps no index of a project's ended goals to load from
  (store.md 7.4), so nothing is loaded. A store index for them is open (people.md 12).
- **skein's ui-web.md 10:** an exchange's cache (`no-store`) and an answer's kept headers, which
  the web client's public mode needs for its freshness and `Retry-After`; built by
  `fetch-headers` (part `skein-fetch`), found by the temper designer.

## Parts

- `jig-public`: jig's generic part, people.md 13, client.md 17.5 and 19, contract.md 12.
- `temper-public`: the forge's public documents, ctl's visibility, the web client's public
  mode and the walk.
- `public-checks`: the negative walk, the fuzzy tests and the statuses.
- `skein-fetch`: skein's fetch reporting an answer's kept headers, with no browser cache.

## Tasks

| Task | Part | Kind | Needs | Minutes | Batch |
|---|---|---|---|---|---|
| `public-limits` | jig-public | interface | | 90 | 1 |
| `visibility-surface` | jig-public | interface | | 150 | 1 |
| `public-wire` | jig-public | interface | | 120 | 1 |
| `public-vocabulary` | jig-public | interface | | 100 | 1 |
| `forge-public-wire` | temper-public | interface | | 90 | 1 |
| `fetch-headers` | skein-fetch | interface | | 75 | 2b |
| `visibility-wire` | jig-public | interface | visibility-surface | 130 | 2a |
| `visibility-core` | jig-public | feature | visibility-surface, visibility-wire | 120 | 2a |
| `public-index` | jig-public | feature | public-limits, public-vocabulary, visibility-core | 150 | 2b |
| `public-reads-core` | jig-public | feature | public-index | 150 | 2b |
| `public-component` | jig-public | feature | public-limits, public-vocabulary, public-wire, visibility-wire | 150 | 2b |
| `public-http` | jig-public | feature | public-component | 120 | 2 |
| `public-story` | jig-public | feature | public-reads-core, public-http | 120 | 2 |
| `public-fuzzy-jig` | public-checks | feature | public-http, public-wire | 60 | 2 |
| `forge-public` | temper-public | feature | forge-public-wire, public-reads-core, public-component | 150 | 2b |
| `public-ctl` | temper-public | feature | visibility-wire | 90 | 2a |
| `public-web` | temper-public | feature | forge-public-wire, public-wire, public-http, fetch-headers | 180 | 3 |
| `public-walk` | temper-public | feature | forge-public, public-web, public-ctl, visibility-core | 120 | 3 |
| `public-fuzzy-temper` | public-checks | feature | forge-public-wire, public-web | 60 | 3 |
| `public-negative` | public-checks | feature | forge-public, public-http, public-ctl | 210 | 3 |
| `public-status` | public-checks | docs | every other task | 30 | 3 |

## Critical path

`visibility-surface` (150) -> `visibility-wire` (130) -> `visibility-core` (120) ->
`public-index` (150) -> `public-reads-core` (150) -> `forge-public` (150) ->
`public-negative` (210) -> `public-status` (30): about 1090 implementer-minutes. Beside it,
`public-wire`/`public-vocabulary`/`public-limits` -> `public-component` -> `public-http` ->
`public-web` -> `public-walk`. Five tasks start at once.

## Overlaps with other plans

- configuring-p2-build (on its trunk, not yet on main) edits files this plan's early tasks
  also edit, unavoidably, since visibility is a new request and `me` gains a field:
  `jig-client-protocol-wire`'s `client-v19.schema`, `src/generated/v19.rs` and `golden/**`
  (`visibility-wire`); `jig-client-domain/src/contract/asks.rs` (`visibility-wire`);
  the engine-clients' and link's `translate/asks.rs` (`visibility-wire`); the store's people
  family, `people-v14.schema` in place, its generated code, codec and goldens
  (`visibility-surface`); and jig's `domain/engine.status.md` and `client/domain/contract.status.md`
  (the revision). Every variant is appended last and every hunk kept local, so whichever lands
  second merges lines; the generated code and goldens are regenerated, not merged by hand.
- playground-temper (integration) edits `temper/tests/one-process/**`; `public-negative`
  adds files there and a module line in its `lib.rs` and `tests/suite/main.rs`.
- No other draft beside this one touches the files of batch 1.
