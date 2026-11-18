# Whoever widens a task funds it

Draft, 2026-11-17, on the user's word. On 11-17 a project's owner amended another party's task in the project, so that the owner's own pool would fund it. The amendment didn't apply. It became a proposal addressed to role 0, which only the owner holds. Nothing recorded which field took the amendment past the owner's direct bounds.

## 1. What happens

- **The design:** whoever widens a task funds it. When the widening is within the amender's own authority, the amender funds it; when it was proposed, its accepter does. From that commit the giver is the task's funder (`jig/docs/design/domain/authority.md`, 7).
- **The code:** only an accepted proposal is funded by its giver, the accepter (`jig/crates/jig-domain-core/src/routing/party/decisions.rs`; `jig/crates/jig-domain-tasks/src/funders/refunding.rs`). A direct amendment sends no funder, so the task's existing funder is charged (`jig/crates/jig-domain-core/src/routing/party/controls.rs`; `jig/crates/jig-domain-tasks/src/control.rs`). `jig/docs/design/domain/authority.status.md` records 7's rule as not yet built.
- **The offer:** the client compares the whole restated authority with the bounds the engine offers, and asks for a proposal when the amendment doesn't fit them (`temper/crates/temper-client-tools/src/amend.rs`; `jig/crates/jig-domain-core/src/acts/bounds.rs`). A direct amendment and the acceptance of the same amendment are therefore judged against different funders. As a result, an amendment its maker could accept, and fund, can still be proposed to the maker's own role.

## 2. Wanted

- **A direct widening within its amender's authority applies at once,** and makes the amender the task's funder, as `authority.md` 7 says and as an accepted proposal already does. The task's requester, its tree and its delegates' funding stay as they are.
- **The offer and the act agree:** the bounds a client is offered are the bounds the direct act is judged by, so a client asks for a proposal only when the amendment is beyond its maker's own authority or funding.
- **The case is explained:** the goal finds which field took 11-17's amendment past its maker's direct bounds, and records it in `authority.status.md`.
- Revise `authority.status.md`, 7.

## 3. Out

- Reading, deciding and withdrawing proposals (proposal-reach).
- Who may decide a proposal.

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- The acts tests (`jig/crates/jig-domain-core/src/tests/acts.rs`) and the core stories (`jig/tests/core/tests/suite/stories.rs`):
  - an owner widens another party's task within its own authority, and the widening applies, funded by the owner;
  - the old funder gets back what the task had left, and keeps what it spent;
  - a widening beyond the owner's pool is still proposed;
  - a narrowing keeps the task's funder;
  - the offered bounds match the act's judgement.

Run only the new and affected tests, by name.
