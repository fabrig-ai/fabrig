Read-only review; no files changed.

1. **High — The title flag proves identity, not human authorship.**  
   [jig/docs/design/domain/people.md, §13.3](jig/docs/design/domain/people.md); [temper/docs/design/client/tools.md, §§5.3, 9](temper/docs/design/client/tools.md).

   The tools explicitly let an agent use a person’s credential. That agent can submit `Create`, then `ShowTitle`, and satisfy the proposed authorship check. It can also amend a shown title using the same credential without clearing the flag. Agent-written words then appear publicly.

   **Fix:** State that ordinary party credentials cannot establish human authorship. Keep titles absent until a separate human publication capability exists. Bind publication to an exact title and spec revision, clear it on every spec change or requester move, and persist provenance conservatively across restart and migration. Apply the same provenance rule to configurable public names; “configured” does not mean “human-written.”

2. **High — Public eligibility does not close over nested references and aggregates.**  
   [jig/docs/design/client/domain/contract.md, §12](jig/docs/design/client/domain/contract.md); [temper/docs/design/client/forge.md, §§11.1–11.2](temper/docs/design/client/forge.md).

   A goal tracked beneath a conversation can expose that excluded conversation through `PublicLine.parent`. A public change’s queue position and “tasks ahead” can expose changes outside goals: `temper/docs/design/domain/forge.md`, §9 explicitly permits such changes in the queue. `PublicLandings` and landing members lack an explicit equivalent of the individual change’s goal-tree restriction.

   **Fix:** Define one public eligibility predicate and apply it to every nested task, parent, goal, member, link, count, queue position and folded total. Stop ancestry at the public goal. Filter before counting or ordering, and omit values whose true meaning would disclose excluded work. Define explicitly how deployment-requested landings and pin-only members become eligible.

3. **High — Revocation during an outstanding read is underspecified.**  
   [jig/docs/design/domain/people.md, §§13.2, 13.6](jig/docs/design/domain/people.md); [jig/docs/design/domain/engine.md, §11](jig/docs/design/domain/engine.md).

   The public-specific path checks visibility at admission, then may load ended tasks or await a connector. A project can become private, enter deletion, or lose a task from its public tree before that work completes. The text does not explicitly invalidate that read, although the referee prohibits answering while private.

   **Fix:** Specify the authorization point for completed public reads. Recheck project visibility, deletion, task membership and title revision after asynchronous work, before constructing the final answer. Define invalidation of pending projections and held replies, without allowing public loads to delay revocation. Add races covering privacy changes, deletion, moves and title changes.

4. **High — `max-age` does not fully specify the promised revocation deadline.**  
   [jig/docs/design/protocol/client.md, §17.5](jig/docs/design/protocol/client.md); [temper/docs/design/protocol/sign-in.md, §2.4](temper/docs/design/protocol/sign-in.md); [temper/docs/design/client/browser.md, §8.2](temper/docs/design/client/browser.md).

   The design promises closure within `max-age`, but does not prohibit stale proxy delivery on upstream failure or during revalidation. A delayed pre-revocation response could also receive a fresh cache lifetime. The browser deliberately retains old content on repeated busy responses indefinitely.

   **Fix:** Require `must-revalidate`, forbid stale serving and background stale refresh, and preserve freshness age through every cache layer. Tie expiry to the authorized snapshot rather than delayed transmission. Make the honest browser expire displayed content when freshness ends. State that the guarantee bounds subsequent serving; it cannot withdraw copies already downloaded.

5. **High — A lower backlog watermark does not protect signed-in capacity.**  
   [jig/docs/design/domain/people.md, §13.5](jig/docs/design/domain/people.md); [jig/docs/design/domain/root.md, §§3, 10](jig/docs/design/domain/root.md).

   Public reads share the loop, store loads, memory and server connections. They can consume those resources while producing little journal backlog. Consequently, the lower watermark can remain open while signed-in reads or engine work wait. Separate exchange counters do not establish the promised priority.

   **Fix:** Specify protected capacity and scheduling for signed-in work and engine completions, including store loads, response memory and upstream connections. Give public processing a bounded share of loop work. Check configuration relationships at startup. The proxy must apply public admission before consuming protected upstream capacity; credential-less requests to other routes must likewise not occupy authenticated capacity unchecked.

6. **Medium — The byte budget admits work before accounting for its answer.**  
   [jig/docs/design/domain/people.md, §13.5](jig/docs/design/domain/people.md); [jig/docs/design/protocol/client.md, §19](jig/docs/design/protocol/client.md).

   With a small positive byte balance, several requests can pass admission together and each produce a maximum-sized answer. Charging only on completion leaves burst size, debt handling and memory consumption undefined.

   **Fix:** Define bucket capacities and burst semantics. Reserve the declared maximum answer bytes before starting projection or loading, then refund unused bytes. Release reservations on every terminal path. Apply a whole-answer size ceiling and reject inconsistent configurations. Cover simultaneous completions and cancellation in the component world.

7. **Medium — Output bounds do not establish a bound on projection work.**  
   [jig/docs/design/domain/engine.md, §§11, 11.2, 13](jig/docs/design/domain/engine.md); [jig/docs/design/protocol/configuration.md, §6](jig/docs/design/protocol/configuration.md).

   The public board inherits task-query machinery that scans live tasks across the deployment and filters ended index pages. A small board can therefore require substantial private-state scanning. The public contract gives no cursor or explicit stopping rule for searching through history to find enough ended goals. Nested goals, lines and forge members also multiply costs.

   **Fix:** Bound total rows examined, joins, loads, traversal depth, intermediate bytes and output bytes per public request. Define truncation when those limits are reached. Use indexes scoped to public data rather than repeatedly scanning private work. Price the complete nested document, not each constituent list independently.

8. **Medium — Internal project numbers disclose hidden allocations.**  
   [jig/docs/design/client/domain/contract.md, §12](jig/docs/design/client/domain/contract.md); [jig/docs/design/domain/core.md, §6.1](jig/docs/design/domain/core.md).

   Public records expose internal project numbers, while newly made projects take the deployment’s next number. In a deployment initially containing public project 1, subsequently observing public project 3 reveals an intervening allocation even though project 2 remains private or was deleted. Internal task numbers similarly expose allocation activity.

   **Fix:** Give public records opaque public identifiers that do not encode private allocations, and use them consistently in routes and references. If internal numbering is retained, record the resulting existence and activity disclosure as an explicit exception to the threat model.

9. **Medium — Identical `404` records do not establish indistinguishable failures.**  
   [jig/docs/design/domain/people.md, §13.2](jig/docs/design/domain/people.md); [jig/docs/design/protocol/client.md, §§17.5, 19](jig/docs/design/protocol/client.md).

   Looking up an existing retained task from another project can involve reading and decoding a row, while a nonexistent task produces no row. Both can return an empty `404` yet differ in latency and resource use. The public contract also does not explicitly map store truncation, malformed retained data and connector-specific failures into its four permitted answers.

   **Fix:** Reject inaccessible projects before task or connector work. Resolve task identifiers through an eligible public index so excluded objects follow the missing-object path. Specify fixed public error envelopes, headers and lengths, with no internal reasons, identifiers or bounds. Add paired-world checks comparing absent and inaccessible objects, including work performed before rejection.

10. **Medium — “Whole path” is insufficient as a shared proxy cache key.**  
    [temper/docs/design/protocol/sign-in.md, §2.4](temper/docs/design/protocol/sign-in.md).

    Two deployments behind one proxy can have the same `/jig/v1/public/...` path. A cache configured literally by path can serve one deployment’s document under another deployment’s origin. Cache hits can also bypass the engine’s Host check.

    **Fix:** Require cache isolation by deployment/upstream and validated origin, followed by the complete canonical path. Validate authority before cache lookup; define handling of forwarded headers and path normalization. Continue excluding cookies from the key because the public origin handler ignores credentials. Test identical paths on two origins and rejected Hosts against a populated cache.

11. **Medium — The golden verifies shape, not the allowlist’s meaning.**  
    [jig/docs/design/client/domain/contract.md, §12](jig/docs/design/client/domain/contract.md); [temper/docs/design/client/forge.md, §11.3](temper/docs/design/client/forge.md).

    A projection can populate an existing text field with a private spec or reason without changing the schema or golden. The specified field/type listing also does not explicitly cover enum payloads, aliases, defaults or wire tags.

    **Fix:** Keep the separate schemas, but require the golden to describe the complete reachable wire shape. Add a field-level provenance table naming each permitted source and transformation, including links and configured names. Test projections against independent expected records while filling prohibited source fields with distinct markers. Include mutation tests showing that nested additions and wrong-source assignments fail.

12. **Medium — The build sketch lacks a mandatory security-negative walk.**  
    [temper/docs/plan/public-reading/notes/next-cut.md, “The tasks”](temper/docs/plan/public-reading/notes/next-cut.md); [jig/docs/design/domain/people.md, §13.6](jig/docs/design/domain/people.md).

    The planned stories cover useful refusals, but neither the sketch nor those stories require the complete attack surface to be exercised through the server and proxy. Schema goldens and a successful anonymous browser walk will miss several failures above.

    **Fix:** Add explicit ownership and acceptance criteria for a negative walk and fuzz targets. Cover every owner/kind/version, wrong-project identifiers, nested excluded work, numeric overflow and malformed paths, all credential-less writes and streams, policy/configuration widening, migration defaults, revocation races, cancellation and load saturation. Exercise a caching proxy, including stale failures and key isolation. Verify that public exceptions do not relax authenticated guards, that cookies never affect public bodies, and that neither origin nor proxy adds CORS permission—including on errors and preflights.

Verdict: **Yes, safe to build after these fixes are incorporated into the design.**