# credentials-git-identities: status

2026-11-22. Where the tree's code lags or departs from [credentials-git-identities.md](credentials-git-identities.md).

- **Not yet:** 1: a removed forge's grant is not withdrawn from jig's accounts: `temper-service/src/forge/gone.rs` takes its values out of temper's identities' table alone, so a Forgejo identity's unbounded grant holds its room in jig's table until a restart, and a forge made later may find none.
