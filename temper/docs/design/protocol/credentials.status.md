# credentials: status

2026-11-02. Where temper's code lags or departs from [credentials.md](credentials.md).

- **Not yet:** 7: the worlds' search for issued secrets covers LLM access tokens, git tokens and a forge's API token (`tests/one-process/src/referee/secrets.rs`), Apps' keys and installations' tokens in one story (`tests/one-process/src/m15/observer.rs`), and the forge's protocol world one API token (`tests/engine/protocol-forge/src/fixtures.rs`): no world searches LLM keys, admins' tokens or webhooks' secrets.
- **Departs:** 2.1, 4, 5: a forge's number is given at runtime and its mode is its store record's, not configuration's `forges` and `setup`, because forges are store records and the playground is reset (`../draft/playground-reset.md`, 3, 4, 6). 2.1, 5: `temper-service` keeps a Forgejo file-credential path (`settings::Forge.api_token` and `webhook_secret`, `file_access`, `file_api`) that only in-process test fixtures fill, its refusals naming `application.forges[N].api_token`, `app_key`, `webhook_secret` and `user`, members the document no longer has, because those fixtures still connect forges by these fields; a code task moves them to records.
