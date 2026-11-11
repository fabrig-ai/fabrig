# forge-identity: status

2026-11-11. Where the design lags or departs from [forge-identity.md](forge-identity.md).

Section 6 is decided by the user, 2026-11-07, "accept it for now": on an
owned Forgejo the rule stops an agent's mistakes, not one trying, until
agents and CI are contained. The proposal sentence and help carry that
limit. Several members, pasted Apps, second accounts, containment and
approval in the client remain section 12's later work, rather than gaps
in this round's one-person design.

- **Design lags:**
  - 10: the real human GitHub review is deferred in `../testing/README.md`,
    section 7, rather than supplied as a written live story: it needs
    the user's authorized review credential. The full fake-GitHub box
    story is designed in that document's 6.4 but remains unbuilt, as its
    status records.
- **Design departs:**
  - 3: `../domain/forge/adoption.md` and `../protocol/store.md`, section 2 use `user_key` and `key_user` in one forge module, rather than one function knowing the subject's form. The `ui-forge-reviewer` cut chose the pair because the inventory and eligibility start from a party and must read its forge and user back; the shared person lookup remains one function.
  - 8 and 10, decided by the `ui-forge-identity` cut's `approval` part: a filter by kind takes no approval, since no kind of the inbox's is one, so the inbox's address and its kinds stay as they are (`../client/pages/README.md`, *An approval*); a watcher already counted at the head has no row, and an owner whose role isn't the approval's sees the wait with no review link, its words saying whose review it waits on (`../domain/forge/change/questions.md`, 2; `../client/forge/README.md`, 4.1); and 10's "the card and its link in the browser" is a step of the joint world's change story (`../testing/checks.md`, 3), because no browser story reaches a change waiting on a forge with an address, the box's demo forge refusing such a rule, and that tier is at its cap of 12.
  - 1, 2: `../domain/agent/agents.md`, 1, and `../domain/agent/ask-temper.md`, 2 have the person tell Ask temper, whose inventory shows their forge account and whose `propose_settings` judges the rule's approvals at the call, where this draft has the configuring agent, whose instructions keep forge identity back unless asked; Ask temper's instructions carry no such sentence: `ask-temper.md`, sections 5.2, 8 and 9, the user, 2026-11-07.
  - 8: `../domain/forge/change/gates.md`, 1 and `../client/forge/README.md`, 4.1 have a person who won't approve ask for changes on the forge, and the change repairs with their remarks rather than taking the refused-requirement hold this draft still describes. The user chose this review-and-repair loop on 2026-11-08.
- **Open:**
  - Whether v0.1.0's ladder walks this: `v0.1.0.md` names no approvals, by the user's choice of 11-06.
