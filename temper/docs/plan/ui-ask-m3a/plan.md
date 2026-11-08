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

## Parts and tasks (first batch)

- **paths:** `ways-surface`, high, about 90 minutes, no needs: the crate
  `temper-engine-domain-ways` with every shape of `notes/contracts.md`,
  section 1, its small functions whole and `derive` a stub, and the
  root's boundary (contracts, section 4).
- **wire:** `ways-wire`, high, about 110 minutes, no needs: the records
  of contracts, section 3, regenerated, with their goldens.
- **instructions:** `ask-words`, medium, about 50 minutes, needs
  `ways-surface` (one writer per crate): `ask.txt`'s M3a Showing part and
  `ways` on its `see` line, `see ways` answering not yet.
- **drawing:** `path-draw`, high, about 130 minutes, no needs: temper's
  own drawing kind for a path, its fallback and its look.

Later batches: the derivation (`ways-derive`), the stock's ways
(`stock-ways`), the root's `ways` document, subject and `see ways`
(`ways-read`), the root's `View` read (`ways-view`), the family's
encoding (`ways-family`), the client's view blocks (`client-views`), their
drawing in a chat (`views-draw`), Ask temper's page's way acts
(`client-ways`), the policy's delegation item (`ways-delegation`), and
the joined stories (`ways-stories`).

## Critical path, as it stands

`ways-surface` (90), then the root's `ways-read` and `ways-view`, then the
joined stories.

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
`ways-surface`); its later tasks add a `route/ways` module. No change to
`forge_route.rs` is planned.

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

## Overlaps with drafts beside this one

- `look-round-20/tabs-and-switch` and `night-shadows` change
  `temper.css`: `path-draw`'s rules go under the chats marker.
- `tui-launchpad` changed `temper-client-view`'s chat page, shared parts
  and `pages/ask.rs`, and the gallery: this plan's `views-draw` changes
  the chat page's text drawing and adds gallery samples late in the plan,
  regenerating `names.txt` and goldens on a catch-up.
