# ui-ask-m3a: plan

Step 3 of `temper/docs/design/draft/stock-ways.md`, section 7, and Ask
temper's M3a (`temper/docs/design/draft/ask-temper.execution.md`): the
stock ways as data, one derivation of a branch's path, the forge's `ways`
read and its acts, `see ways`, and the views `way`, `path`, `proposal`
and `charter` drawn by the browser in a reply (`inputs/request.md`).

The cut wrote the design first: `temper/docs/design/domain/forge.md`,
8.13 (the path); `temper/docs/design/protocol/configuration.md`, 13.10
(the stock ways and their items); `temper/docs/design/client/README.md`,
4.6 (the records), 5.13 (*A way's acts*) and 5.16 (views in a reply);
`temper/docs/design/client/forge.md`, section 9's row;
`temper/docs/design/domain/agent.md`, 8.4 and 8.4.1 (`see ways`, the
instructions); `temper/docs/design/domain/landing.md`, 13.1; each with
its status, and the drafts' statuses.

## Parts and tasks

- **paths:** `ways-surface`, high, 90 minutes, no needs: the crate
  `temper-engine-domain-ways` with every shape of `notes/contracts.md`,
  section 1, its small functions whole and `derive` a stub, and the root's
  boundary (contracts, section 4). `ways-derive`, high, 180, needs
  `ways-surface`: the derivation of `forge.md`, 8.13, its focused table of
  the look request's three paths (with CI and both tiers, with neither, by
  turn) and its fuzzy test (never an agent's update, merge or turn).
- **wire:** `ways-wire`, high, 110, no needs: the records of contracts,
  section 3, regenerated, with their goldens.
- **instructions:** `ask-words`, medium, 50, needs `ways-surface` (one
  writer per crate): `ask.txt`'s M3a Showing part and `ways` on its `see`
  line, `see ways` answering not yet.
- **drawing:** `path-draw`, high, 130, no needs: temper's own drawing kind
  for a path, its fallback a stop a line, and its look.
- **stock:** `stock-ways`, medium, 70, needs `ways-surface` and
  `ask-words` (both write `temper-service`'s stock tests): the three ways
  as data, given to the root at each start.
- **read:** `ways-read`, high, 260, needs `ways-surface` and `ask-words`:
  the root gathers a path's inputs, makes the `ways` document for its
  subject, marks it, and answers `see ways`. `ways-view`, high, 180, needs
  `ways-read` and `ways-derive`: the root's `View` read, way, path,
  proposal and charter. `ways-family`, high, 100, needs `ways-surface`,
  `ways-wire` and `stock-ways`: the service's mapping and encoding of both.
- **views:** `client-views`, high, 150, needs `ways-wire`: the view blocks'
  splitter and the chat page's view reads. `views-draw`, high, 180, needs
  `client-views` and `path-draw`: their drawing in a turn and the chat's
  document, and three gallery samples.
- **acts:** `client-ways`, high, 180, needs `ways-wire` and `client-views`
  (one writer of `temper-client-domain`): the Ask page's `ways` watch,
  `tabs`, "Use this", "Keep it" and "Yours". `ways-delegation`, medium,
  90, needs `client-ways`: the project policy's item where a way's
  delegation is lacking.
- **stories:** `ways-stories`, high, 150, needs `ways-derive`,
  `stock-ways`, `ways-view`, `ways-family`, `client-ways` and
  `ways-delegation`: the paths with CI and both tiers and with neither,
  current and "Yours", "Use this" refused for too few approvers, "Keep it"
  marking To begin's step done; the last **Not yet** entries closed.

Slice 1 holds paths, wire, instructions and drawing; slice 2 stock and
read; slice 3 views, acts and stories. No walk: the look request's plan,
`look-ask-ways`, draws Ask temper's opening from the `ways` read and its
acts, and its walk follows.

## Critical path

`ways-surface` (90), `ask-words` (50), `ways-read` (260), `ways-view`
(180), `ways-stories` (150): about 730 implementer-minutes. The client's
chain, `ways-wire` (110), `client-views` (150), `client-ways` (180),
`ways-delegation` (90), meets it at `ways-stories` with time to spare.

## Who hears first

For the TUI's owner, who draws views in the TUI (M3b):

- **The `ways` read:** the subject `ways { project }`, its document
  `WaysPanel`: each stock way's name, line, path over the project and
  items, the way the project runs or none, the project's own path, and
  whether the reader may change settings (`client/README.md`, 4.6).
- **A path:** a list of at most 32 stops, each its actor (person, agent,
  code or a branch's bar), its step, its words, what it opens and its
  card (`who`, `name`, at most four facts).
- **The views' blocks:** a fenced block at a reply's top level, info
  string `view`, its one line an address: `way NAME`, `path OWNER/NAME
  [BRANCH]`, `proposal tN/K`, `charter N`, optionally `@revision N`. The
  client splits a reply at them and reads each through the forge's read
  `View`, answered as `WayView`, `Path`, `ProposalView` (before and after),
  `CharterView` or `NotDrawn` (`client/README.md`, 5.16). Its text fallback
  is a stop a line, its words and its actor in words.

For the owner of coding runs: `temper-engine-domain` gains the crate
`temper-engine-domain-ways` as a dependency, `Config::ways_stock`,
`ForgeRead::View`, `Delivery::Ways` and `SeeAnswer::Ways` (task
`ways-surface`); `ways-read` and `ways-view` add a `route/ways` module and
mark the `ways` subject where the root commits temper's kinds
(`route/kinds.rs`). `ways-read` may add one line marking it where the root
routes an adoption's or a removal's outcome, if that is
`route/forge_route.rs`, and a public read of the forge's given
every-project rules (`temper-engine-domain-forge/src/kinds.rs`); its report
names both.

## What was decided

- **One new crate below the root,** `temper-engine-domain-ways`, holding
  the shapes, the stock ways' type, the items laid over a project and the
  derivation, pure and with nothing of jig's: the root gathers the inputs
  (the forge's held rules, charters, ways and landing; the core's
  templates, roles, charters and offer) and calls it, so that the
  client's read, `see ways` and the views share one derivation.
- **Paths over settings with items laid over them:** a way's path, and a
  proposal's "after", are the derivation over the project's settings with
  items laid over them, as the decision would leave them.
- **You approve and Agents review set kind 2 alone,** as the draft's table
  has them: a project that ran Draft first keeps its goals' way, and their
  paths over it start with a goal (`configuration.md`, 13.10).
- **"verify" on main for every way:** main's tier follows main whatever
  the way, so over a project with CI and both tiers You approve and Agents
  review end "merge, verify, main", where the look request's two pages
  draw no tier on main (`stock-ways.status.md`, *Design departs*).
- **`@revision N`** draws only the newest revision; an older one says it
  is not drawn yet. **`proposal tN/K`** draws a waiting proposal alone
  (`ask-temper.status.md`, *Design departs*).
- **A view block is split out by temper,** not by skein's Markdown reader:
  the address is temper's and is read through temper's own read, and
  skein keeps an unknown fence as code.
- **No version bump:** the wire's records change in place, the version
  moving at a release (`client/README.md`, 4.6).
- **Charters have no names:** a card names an agent by its place on the
  path, "the coding agent", "the reviewer", "the coordinator".

## Overlaps

Within the plan, two first-wave tasks write compiler-forced arms in one
file: `ways-surface` (`forge_read`'s `View` answer) and `ways-wire`
(`forge_settings_query`'s `View` read) in `temper-service/src/family.rs`,
in different functions; every other crate has one writer a wave.

With drafts beside this one:

- `look-round-20/tabs-and-switch` and `night-shadows` change
  `temper.css`: `path-draw`'s rules go under the chats marker.
- `tui-launchpad` changed `temper-client-view`'s chat page, shared parts
  and `pages/ask.rs`, and the gallery: this plan's `views-draw` changes
  the chat page's text drawing and adds gallery samples late in the plan,
  regenerating `names.txt` and goldens on a catch-up.
