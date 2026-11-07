# forge-identity: status

2026-11-07. Where the design lags or departs from [forge-identity.md](forge-identity.md).

- **Design lags:**
  - 6: decided by the user, 2026-11-07, "accept it for now": on an owned Forgejo the rule stops an agent's mistakes, not one trying, until agents and CI are contained. `../client/README.md`, 5.13: the proposal doesn't say what a rule holds against on a box.
  - 8: `../client/README.md`, 3.5 and 4.6, and `../client/forge.md`, section 4: no subject, card or merge with the inbox for an approval on the forge. `../domain/forge.md`, 5.5 and 8.7: no read, and review facts kept past the merge's admission.
  - 9: `../protocol/sign-in.md`, 10: its "person's forge identity" stays open for several members.
- **Design departs:**
  - 3: `../domain/forge.md`, section 4 and `../protocol/store.md`, section 2 use `user_key` and `key_user` in one forge module, rather than one function knowing the subject's form. The `ui-forge-reviewer` cut chose the pair because the inventory and eligibility start from a party and must read its forge and user back; the shared person lookup remains one function.
- **Open:**
  - Whether v0.1.0's ladder walks this: `v0.1.0.md` names no approvals, by the user's choice of 11-06.
