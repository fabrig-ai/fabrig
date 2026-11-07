# account-facts: status

2026-11-07. Where the design lags or departs from [account-facts.md](account-facts.md).

- **Design lags:** 3, 6: section 6's third step is not designed: no document says where a plan, a workspace, a renewal, an end or a usage window comes from, so `../domain/accounts.md`, 3.7 registers kinds 3 to 7 with nothing to say them, and keeps them under its Open questions. The user is still deciding (the request `account-email`, 2026-11-06).
- **Design departs:**
  - 3: `../domain/accounts.md`, 3.7 also takes kind 1 from a program's status, as origin 5, where the table names an ID token alone. Claude Code's `claude auth status` names an `email` (read on 2026-11-07, version 2.1.312; `../domain/providers.md`, 3.3), and the request `account-email` asked that a Claude subscription's email be kept so.
  - 4: `seen` is when the provider first said a fact's value, not each time it says it again, so that a refresh that says the same rewrites no record. The cut of `account-email` decided it, the user being asleep and taking its judgement.
- **Open:** none.
