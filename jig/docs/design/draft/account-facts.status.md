# account-facts: status

2026-11-19. Where the design lags or departs from [account-facts.md](account-facts.md).

- **Design lags:** 3, 6: section 6's third step is designed for usage windows alone (temper's `draft/usage-windows.md`): no document says where a plan, a workspace, a renewal or an end comes from, so `../domain/accounts-lists.md`, 1 registers kinds 3 to 6 with nothing to say them, and keeps them under its Open questions. The user is still deciding (the request `account-email`, 2026-11-06).
- **Design departs:**
  - 3: `../domain/accounts-lists.md`, 1 also takes kind 1 from a program's status, as origin 5, where the table names an ID token alone. Claude Code's `claude auth status` names an `email` (read on 2026-11-07, version 2.1.312; `../domain/providers.md`, 3.3), and the request `account-email` asked that a Claude subscription's email be kept so.
  - 4: `seen` is when the provider first said a fact's value through the same source with the same origin, not each time it says it again, so that an unchanged refresh rewrites no record. Changing the source or origin gives a new `seen`, even with the same value (`../domain/accounts-lists.md`, 1). The cut of `account-email` decided the unchanged-report rule, the user being asleep and taking its judgement.
- **Open:** none.
