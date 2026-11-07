# per-wake-temper: temper's settings, reference and template take charter parts and a later choice

`temper/docs/design/draft/per-wake.md` gives a charter `parts`, each for some reasons a run runs, and a task's LLM choice a `later`. Its jig half is on main, from the plan `per-wake`: configuration, the store, the client wire (19) and the client contract carry both, a run reads the shared instructions and the parts its reasons meet, and only the kinds its parts cover are offered. Its status file records the departures that plan decided: jig's limit is `tasks.charter_parts`, a wire refusal names the charter rather than the part, and only a party's request or a template gives a `later`. This plan builds the draft's temper half: its section 9, item 2, and the temper sections its status file lists as lagging, 3, 5, 6, 7 and 8. The draft wins where other designs differ.

## What's wanted, ranked. Drop from the bottom if it passes seven tasks, and say what you dropped

1. **The story first, at the lowest tier that holds temper's settings and a goal:** the reference's settings applied as a settings file; a goal started from template 4; its first run reads charter 5's shared text and its cut part at Opus xhigh; a question to it is kept until the run after, which reads the shared text and the questions part at Opus high, with no amendment. A scripted model, as the real loop's stories have, is enough; if a lower world holds it, go lower.

2. **temper's settings form and `temper ctl settings show` and `apply` carry `parts` and `later`** (`../client/tools.md`, 5.3): a settings file with them round-trips, and `show` prints a charter's parts with their `on` and sizes. A file refused for a part says which charter and why, in the words jig's refusal allows.

3. **Template 4 takes a `later`** (`../protocol/configuration.md`, 13.9): Opus 5.5 at xhigh first and Opus 5.5 at high later, each with the first choice's sub-agents. `../domain/agent.md`, 4.3.1 says a change's `llm` carries no `later`.

4. **The reference's charter 5 in four parts** (draft, section 6): shared, the cut `on` `first`, questions, results and holds, and news, with the kinds the draft's table gives each. Write each from the reference's own charter 5 text, moving each rule to the part that acts on it, and each rule once. Add the draft's rule for a request that needs no design: the cut writes none, says so in its report note, and makes its changes. `docs/examples/reference/settings.json` is generated, so it follows its sources and their test.

5. **The reference's charters are focused** (section 7): each holds the rules its role acts on. The goal's holds what a spec must carry of building and testing, and not the rest. Say what moved and where, with sizes before and after.

6. **A charter too large to store says how large** (section 8; `../domain/root.md`, section 10): a change of settings refused because a charter's stored policy passes `root.journal.run_bytes` names the charter, its size and the bound, as a refused start already does. A row in an existing refusal test.

7. **The reference's settings test holds its charters within the stock worker's budget and the root's bound:** a charter's `turns`, `time` and the run spend within a stock worker's advertised bounds (`jig-domain-fleet/src/needs.rs`), and each stored policy within `root.journal.run_bytes`, so a reference that no host would take, or that a box can't store, fails here and not on a box.

## Not in this plan

- An LLM choice per kind of wake, and documents per part: the draft's Later.
- `reference-code-production.md` stays unedited: tests and a judge read it. Its lag stays in the status file.
- The chat's and the configuring agent's charters, which have no parts.

**Tests stay cheap:** the gate rejects a plan that adds cost. A new case of one behaviour is a row of an existing table test, and a new story goes in a world below the real loop. Failing tests first; no test reaches a real host. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** The stored formats and client records this needs are jig's, landed; any further stored format or protocol change is a question first, as both main areas write `temper-engine-domain` and `forge_route.rs`. Formats change in place before v0.1.0. `walk = false`.
