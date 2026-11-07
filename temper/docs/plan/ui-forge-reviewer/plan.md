# ui-forge-reviewer

A review counts only from the forge it was made on: the first piece of `temper/docs/design/draft/forge-identity.md`, section 3.

## Parts and tasks

One part, `reviewer`, in one slice.

| Task | Needs | What it makes true |
|---|---|---|
| `forge-subject` | | The hole reproduced, failing, then closed: provider 0's subject names the forge; adoption seeds it; `forge_person` is the one lookup, read by `landing_reviewers`, `human_forge_identity` and `settings_people`; every world's provider-0 key moves with it |
| `inventory-forges` | `forge-subject` | The inventory's line names each forge of the project, through `person_forges` |

**Critical path:** `forge-subject`, about 90 implementer-minutes, then `inventory-forges`, about 30.

They're two so that the fix doesn't wait on, or conflict in, `temper-engine-smith/src/settings.rs`, which `ui-box-slots/box-words` and `model-catalogue-2/configuring-learns` are writing. `forge-subject` keeps `ProjectPerson`'s shape, fed through the lookup, and `inventory-forges` changes it.

## Decided

- **A stored format and a public type change in place,** with no compatibility and no `Question:`, by the request. Provider 0's subject becomes 10 bytes in jig's party rows, which hold it opaque, so no schema or `STORE_VERSION` moves. A row of the old form names no forge's user: there's no supported deployment. No production sign-in uses provider 0 (the templates' owners are on providers 1 and 3), so only adoption's seeds and the test worlds hold it.
- **`user_key` and `key_user` are a pair,** where the request and the draft say "one function". The inventory and eligibility start from a party, so they must read the user back. Both sit in one small module, and nothing else knows the form.
- **`forge_person` takes the project,** as well as the forge and user the request names, because jig's people answer a key's party only with its role in a project (`role_for_identity`), and this branch stays in temper's tree.
- **Eligibility is per project still,** which leaves the draft's section 7 to a later plan: the request lists neither it nor `../domain/forge.md`, 8.9. But what today's loop over the project's repositories means has to change with the subject: a party counts only if known on a forge of the project. So 8.9 gains that clause, and `../draft/forge-identity.status.md` keeps its entry for the rest.
- **Design lines beyond the request's list:** `temper/docs/design/domain/agent.md`, 8.1, for the inventory's line, and `temper/docs/design/domain/forge.md`, 8.9 and section 20, as above. The design is the contract, and the code changes these.
- **The reproduction is a table in `route/forge_route/tests.rs`,** the one new table of this plan beside `human_forge_identity`'s and `person_forges`'. No table reaches `landing_reviewers`, the domain world's fake holds one forge, and `two_forges()` already stands there, at milliseconds a row.
- **No slow-tier story reads the subject,** by the survey: none is changed.
- **Walks and render checks:** none; the draft's section 10 stories for its later sections go with those plans.

## Overlaps that couldn't be avoided

- `goal-delivery`: `temper-engine-domain/src/route/forge_route.rs` and `route/forge_route/tests.rs`, and `domain/forge.status.md`. Whichever lands second catches up, as the request says.
- `model-catalogue-2/catalogue-clients`: `temper-engine-domain/src/route/**`, a wide glob.
- `limits-fixtures` (`forge-routes-steps`, `client-world`, `service-fixture`) and `ui-small-1`, `limits-worlds`, `browser-cap`: the sweep edits one expression in world fixtures they also write, which is why it's kept to the expression.
- `ui-box-slots/box-words`, `model-catalogue-2/configuring-learns`: `temper-engine-smith/src/settings.rs` and its tests, in `inventory-forges` alone.

## Not in this plan

The link and its sources, `Reconcile`'s login, `AppMade`, the approval's card and per-forge eligibility: `temper/docs/design/draft/forge-identity.md`, sections 4, 7, 8 and 9, whose entries stay in its status.
