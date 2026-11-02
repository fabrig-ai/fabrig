# sign-in: status

2026-11-02. Where temper's code lags or departs from [sign-in.md](sign-in.md).

- **Departs:** 9.7: the start checks a `sign_in.forges` entry's client only against the provider's clients, not its forge's kind or its `redirect_uri` against the forge's browser origin (`temper-service/src/startup/application/sign_in/forges.rs`), because forges are store records the configuration no longer holds and the playground is reset (`../draft/playground-reset.md`, 3, 4, 6).
