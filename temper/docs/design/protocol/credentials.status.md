# credentials: status

2026-11-05. Where temper's code lags or departs from [credentials.md](credentials.md).

- **Not yet:** 2.1, 8: rows of a forge with no record are not named in the log (`temper-service/src/forge/records.rs`). 4: a removed forge's grant is not withdrawn from jig's accounts: `temper-service/src/forge/gone.rs` takes its values out of temper's identities' table alone, so a Forgejo identity's unbounded grant holds its room in jig's table until a restart, and a forge made later may find none. 7: the worlds' search for issued secrets covers LLM access tokens, git tokens and a forge's API token (`tests/one-process/src/referee/secrets.rs`), Apps' keys and installations' tokens in one story (`tests/one-process/src/m15/observer.rs`), and the forge's protocol world one API token (`tests/engine/protocol-forge/src/fixtures.rs`): no world searches LLM keys, admins' tokens or webhooks' secrets.
