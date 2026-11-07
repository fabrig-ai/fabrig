# ui-forge-identity: nothing lands without a box's person's approval, if they say so

The rest of `temper/docs/design/draft/forge-identity.md`, sections 4 to 9, temper's side, as `inputs/request.md` asks. Section 3's hole is closed (`ui-forge-reviewer`) and section 4's generic part is in skein (`ui-forge-setup`).

## Parts and tasks

| Part | Task | Needs | Tree | What it makes true |
|---|---|---|---|---|
| `scope` | `sole-owner` | | jig | `Domain::sole_owner` answers the deployment's one owner, else none |
| `scope` | `box-sign-in` | | temper | the start refuses `machine.owned.sign_in` beside `application.sign_in` |
| `link` | `link-record` | | temper | the forge's record holds `person: Option<Link>`, stored in place; the connector's `Owner` and `PersonOf` inputs and its `LinkNoted` note exist, doing nothing |
| `link` | `link-lookup` | `link-record` | temper | `forge_person` answers the link first; `forge_human` and `eligible_people` count per forge; the refusal names the forge; the playground's forge has its own words |
| `link` | `link-decide` | `link-record` | temper | the connector keeps, replaces and clears the link (4.6); adoption skips the link's user |
| `link` | `link-setup` | `link-record` | temper | an owned Forgejo's `Reconcile` is given the person's login; each setup answer becomes `PersonOf` |
| `link` | `link-scope` | `link-decide`, `sole-owner` | temper | the root hands the forge the sole owner at each start |
| `link` | `link-words` | | temper | the help and the configuring agent's instructions say a box's person needs nothing added, with section 6's sentence |
| `link` | `box-story` | `link-lookup`, `link-scope`, `link-words` | temper | the engine's world: a box's person linked, their rule accepted (section 2, steps 1 and 2) |

| `approval` | `approval-wire` | | temper | the client wire's `approvals` document and watch, and `ProposalItemsAnswer`'s `owned_approval` |
| `approval` | `approval-waits` | `link-decide` | temper | the forge's `APPROVALS_SUBJECT`, marked with `changes`; review facts dropped at the merge's admission; `Domain::approval_waits` |
| `approval` | `approval-client` | `approval-wire` | temper | the client watches `approvals` and merges its rows into Needs you, its filters, Home's and the rail's counts, and Later; reads a proposal's `owned_approval` |
| `approval` | `approval-read` | `link-lookup`, `link-scope`, `approval-wire`, `approval-waits` | temper | the root answers each watcher's approvals, whether their review would count; the service carries them with each row's address; a proposal's `owned_approval` |
| `approval` | `approval-card` | `approval-client` | temper | the card on Needs you, Home and a goal's band, its review link, the counts; the proposal's sentence on a box; the gallery's samples |
| `approval` | `approval-story` | `box-story`, `approval-read` | temper | the engine's world: the box's person's change waits, their `approvals` row, another project's party reads none, their review lands it (section 2, steps 3 to 5) |
| `approval` | `approval-joined` | `approval-read`, `approval-card` | temper | the joint world: the card and its "Review on Forgejo ↗" link on Needs you, gone once the review lands the change (in place of a browser step) |

The `approval` part was written by a designer, built to `notes/contracts.md`, *The approval read*.

**Critical path:** `link-record`, `link-decide`, `approval-waits`, `approval-read`, `approval-story` (with `link-lookup` and `link-scope` beside it into `approval-read`); on the client's side, `approval-wire`, `approval-client`, `approval-card`, `approval-joined`.

## What this cut decided

- **Section 6 is decided** (the user, 2026-11-07, "accept it for now"), marked in `forge-identity.status.md`, and said in `protocol/setup.md`, section 15 and the help.
- **jig's read is `sole_owner`,** gated as `project_members` is, so the root can read it during a start's restore (`jig/docs/design/domain/people.md`, section 4).
- **The link's rules are one new section,** `temper/docs/design/domain/forge.md`, 4.6: the connector keeps the sole owner the root hands it, links only under a box, only an owned Forgejo or a GitHub forge, never temper's writer; a Forgejo account gone leaves the link, an App not the person's clears it.
- **The setup host decides GitHub's answer** from the forge's `AppMade` and `AppId` items and the account's type, and sends one `PersonAnswer` (`Is`, `Gone`, `NotTheirs`); the connector decides what it does. So no secret item reaches the forge domain.
- **The log lines go through the root's facts** (`Fact::ForgeLink`), as the other connector observations do.
- **The fake forge's refusal is whatever the count:** it has no accounts, so a rule covering it can't be met.
- **The inventory's configured owned-forge account row stays as it is:** with the link, the person's own line names their account, and the row stays a separate fact.
- **Stored format, protocol and public interface changed in place,** with no compatibility or version bump, as the request allows: the forge record's `person` (store v11), the client wire's `Approvals` document, its `Watch::Approvals` and `ProposalItemsAnswer`'s `owned_approval` (client wire v1).
- **The approval is a read, not a task or a hold** (draft, section 8): `APPROVALS_SUBJECT`, of the party's own scope, marked wherever `changes` is; the forge answers the waits, the root joins the people per watcher (`domain/forge.md`, 8.12). Review facts now go when the merge is admitted (8.7).
- **Who sees a row:** a watcher with a role in the project; one already counted at the head has none; an owner whose role isn't the approval's sees the wait with no review link. The card says why a review wouldn't count, in this order: too few people eligible on that forge, no account there, another role.
- **No approval kind filter:** project and goal filters take approvals, a filter by kind takes none, so the inbox's address codec, the browser's kind list and the terminal stay as they are (a **Design departs** entry in the draft's status).
- **Section 10's browser step goes lower,** to the client-and-engine joint world (`approval-joined`): no browser story reaches a change waiting on a forge with an address (the box's demo forge refuses such a rule; the GitHub stories run `temper serve`, with no person linked), and that tier is at its cap of 12. The user's rule allows it: "a step of an existing one or goes lower".
- **The engine's story of steps 3 to 5** defaults to `approval_identity.rs`'s box world, which has the helpers to make and merge a change; `box-story`'s `configuring.rs` world says steps 1 and 2.

## Left out

- **Section 10's live story, "on GitHub, a real review counted":** it needs a person's own GitHub account and token to post the review, a sign-in and a credential that are the user's, which no source gives. Left for QA after landing; the draft's own step 5 needs nothing new to land.

## Overlaps with the drafts beside this one

- **`forge_route.rs`** (temper/migration's `goal-delivery` and `call-room`, by the request): `link-record` adds one `outputs` arm, `link-lookup` changes the four lookup functions. Whichever lands second catches up.
- **`limits-fuzzy/under-causes`** globs all of `temper/crates/temper-service/src/**`: `box-sign-in` (startup), `link-record` (`engine.rs`, `lib.rs`), `link-setup` (`setup/lifecycle.rs`) and `link-words` (`stock/configuring.txt`) touch it. Small and unavoidable.
- **`model-catalogue-3/models-read`** writes `temper-client-view`'s `words.rs` and `root.rs`, `temper.css` and the gallery: `approval-card` adds its words, one argument in `root.rs`, one rule under `/* ==== ui: needs ==== */` and two gallery samples; whichever lands second regenerates the gallery.
- **`model-catalogue-3/list-story`** globs `temper/tests/client/src/**`: `approval-joined` touches `fixtures.rs` and `front.rs` there.
- **`model-catalogue-3/list-close`** globs `**/*.status.md`; ours are single entries.
- **No heads-up to hearth/core:** hearth names skein alone, and `jig-domain-people` is no crate hearth builds on.
