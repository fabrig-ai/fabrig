# forge-identity: status

2026-11-07. Where the design lags or departs from [forge-identity.md](forge-identity.md).

- **Design lags:**
  - 3: `../domain/forge.md`, section 4: no link in the one lookup. `../protocol/forge.md`, 2.1: no link on the forge's record.
  - 3 and 9: `../domain/agent.md`, 8.1: "an absent forge identity" and its instructions on it, and its open question on a person's forge identity, which this closes.
  - 4.1: `../protocol/setup.md`, 5.2, and skein's `forge-setup.md`, 4.1 and 4.3: `Reconcile` is given no login and answers no id.
  - 4.2: `../protocol/setup.md`, 17.2, and skein's `forge-setup.md`, 5.2, 5.6 and 6.4 and `forge-github.md`, and `../protocol/credentials.md`: no `owner` from `GET /app`, no `AppMade`.
  - 4.4: `../domain/forge.md`, section 4: adoption seeds every human collaborator.
  - 6: `../client/README.md`, 5.13: the proposal doesn't say what a rule holds against on a box. `../protocol/setup.md`, section 15: CI's isolation doesn't name approvals.
  - 7: `../domain/forge.md`, 8.9 and `../protocol/configuration.md`, section 13: eligibility counts an account on any forge of the project, not on the forge of each repository the rule covers, and the refusal's words.
  - 8: `../client/README.md`, 3.5 and 4.6, and `../client/forge.md`, section 4: no subject, card or merge with the inbox for an approval on the forge. `../domain/forge.md`, 5.5 and 8.7: no read, and review facts kept past the merge's admission.
  - 9: `../protocol/sign-in.md`, 9.9: no refusal of `machine.owned.sign_in` beside `application.sign_in`. jig's `domain/people.md`: no accessor for the deployment's owners. `../protocol/sign-in.md`, 10: its "person's forge identity" stays open for several members. The help's `landing-rule.md`, `configure.md` and `party.md` speak for every deployment.
  - 10: `../domain/testing.md`: its story of a box's owner asserts today's refusal.
- **Open:**
  - 6: on an owned Forgejo, an agent or CI job trying can approve as the person, until agents and CI are contained. The user decides whether that's enough for now, beside the box's loopback trust.
  - Whether v0.1.0's ladder walks this: `v0.1.0.md` names no approvals, by the user's choice of 11-06.
