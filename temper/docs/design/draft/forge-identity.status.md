# forge-identity: status

2026-11-07. Where the design lags or departs from [forge-identity.md](forge-identity.md).

- **Design lags:**
  - 6: decided by the user, 2026-11-07, "accept it for now": on an owned Forgejo the rule stops an agent's mistakes, not one trying, until agents and CI are contained.
  - 9: `../protocol/sign-in.md`, 10: its "person's forge identity" stays open for several members.
- **Design departs:**
  - 3: `../domain/forge.md`, section 4 and `../protocol/store.md`, section 2 use `user_key` and `key_user` in one forge module, rather than one function knowing the subject's form. The `ui-forge-reviewer` cut chose the pair because the inventory and eligibility start from a party and must read its forge and user back; the shared person lookup remains one function.
  - 8 and 10, decided by the `ui-forge-identity` cut's `approval` part: a filter by kind takes no approval, since no kind of the inbox's is one, so the inbox's address and its kinds stay as they are (`../client/README.md`, 3.5, *An approval*); a watcher already counted at the head has no row, and an owner whose role isn't the approval's sees the wait with no review link, its words saying whose review it waits on (`../domain/forge.md`, 8.12; `../client/forge.md`, 4.1); and 10's "the card and its link in the browser" is a step of the joint world's change story (`../domain/testing.md`, 6.3), because no browser story reaches a change waiting on a forge with an address, the box's demo forge refusing such a rule, and that tier is at its cap of 12.
- **Open:**
  - Whether v0.1.0's ladder walks this: `v0.1.0.md` names no approvals, by the user's choice of 11-06.
