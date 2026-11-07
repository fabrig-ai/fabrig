# ui-forge-identity: nothing lands without a box's person's approval, if they say so

The rest of `temper/docs/design/draft/forge-identity.md`, sections 4 to 9, temper's side, as `inputs/request.md` asks. Section 3's hole is closed (`ui-forge-reviewer`) and section 4's generic part is in skein (`ui-forge-setup`).

## Parts and tasks

| Part | Task | Tree | What it makes true |
|---|---|---|---|
| `scope` | `sole-owner` | jig | `Domain::sole_owner` answers the deployment's one owner, else none |
| `scope` | `box-sign-in` | temper | the start refuses `machine.owned.sign_in` beside `application.sign_in` |

The `link` part (sections 3, 4, 7: the forge record's link, the lookup, eligibility per forge, the refusal) and the `approval` part (section 8: the read and the card under Needs you) follow in later batches.

**Critical path, as it stands:** `sole-owner` and `box-sign-in` run at once; nothing waits on them but the link's scope task, later.

## What this cut decided

- **Section 6 is decided** (the user, 2026-11-07, "accept it for now"), marked in `forge-identity.status.md`.
- **jig's read is `sole_owner`,** gated as `project_members` is, so the root can read it during a start's restore; written in `jig/docs/design/domain/people.md`, section 4.
- **No heads-up to hearth/core:** hearth names skein alone, and `jig-domain-people` is no crate hearth builds on.

## Overlaps with the drafts beside this one

- `limits-fuzzy/under-causes` globs all of `temper/crates/temper-service/src/**`; `box-sign-in` touches `startup/application.rs`, `startup/problems.rs` and two test files there. Small and unavoidable.
- `model-catalogue-3/list-close` globs `**/*.status.md`; ours are single entries.
