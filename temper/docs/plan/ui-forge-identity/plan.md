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

The rest of the `approval` part (section 8: the forge's waits, the root's read, the client's merge and card; section 6's sentence on the proposal's card; section 2's steps 3 to 5 in the world) follows from its designer, built to `notes/contracts.md`, *The approval read*.

**Critical path, as it stands:** `link-record`, `link-decide`, `link-scope`, `box-story`, then the `approval` part's story. `link-lookup` feeds the `approval` part's read.

## What this cut decided

- **Section 6 is decided** (the user, 2026-11-07, "accept it for now"), marked in `forge-identity.status.md`, and said in `protocol/setup.md`, section 15 and the help.
- **jig's read is `sole_owner`,** gated as `project_members` is, so the root can read it during a start's restore (`jig/docs/design/domain/people.md`, section 4).
- **The link's rules are one new section,** `temper/docs/design/domain/forge.md`, 4.6: the connector keeps the sole owner the root hands it, links only under a box, only an owned Forgejo or a GitHub forge, never temper's writer; a Forgejo account gone leaves the link, an App not the person's clears it.
- **The setup host decides GitHub's answer** from the forge's `AppMade` and `AppId` items and the account's type, and sends one `PersonAnswer` (`Is`, `Gone`, `NotTheirs`); the connector decides what it does. So no secret item reaches the forge domain.
- **The log lines go through the root's facts** (`Fact::ForgeLink`), as the other connector observations do.
- **The fake forge's refusal is whatever the count:** it has no accounts, so a rule covering it can't be met.
- **The inventory's configured owned-forge account row stays as it is:** with the link, the person's own line names their account, and the row stays a separate fact.
- **Stored format changed in place** (the forge record's `person`), with no version bump, as the request allows.

## Left out

- **Section 10's live story, "on GitHub, a real review counted":** it needs a person's own GitHub account and token to post the review, a sign-in and a credential that are the user's, which no source gives. Left for QA after landing; the draft's own step 5 needs nothing new to land.

## Overlaps with the drafts beside this one

- **`forge_route.rs`** (temper/migration's `goal-delivery` and `call-room`, by the request): `link-record` adds one `outputs` arm, `link-lookup` changes the four lookup functions. Whichever lands second catches up.
- **`limits-fuzzy/under-causes`** globs all of `temper/crates/temper-service/src/**`: `box-sign-in` (startup), `link-record` (`engine.rs`, `lib.rs`), `link-setup` (`setup/lifecycle.rs`) and `link-words` (`stock/configuring.txt`) touch it. Small and unavoidable.
- **`model-catalogue-3/list-close`** globs `**/*.status.md`; ours are single entries.
- **No heads-up to hearth/core:** hearth names skein alone, and `jig-domain-people` is no crate hearth builds on.
