# debug-secrets: Debug prints no credential

The questions bugs-security left, decided by agents/loop as that plan's owner, 11-05, each on its recommended option. All four still hold on main. No user-facing behaviour changes; a trace shows a length where it showed bytes.

1. **A carrier's rule.** `skein/docs/design/lib.md` 6 gains: a record carrying a peer's bytes prints them as their length, and a header prints its name and its value's length. Carry it out in `skein-http`'s `Header`, server `Call` and the client's written head (Authorization, Cookie and Set-Cookie values reach jig-protocol-engine-clients-http's `Call` and temper-service's sign-in queues), and `skein-channel`'s `Frame` and `Event::Body` (smith's grant refresh, a bearer or key, travels in one). Other carriers (skein-io's bytes, skein-kv's values, skein-oauth's Json, jig's bodies) have no known credential path: name them in the status file as Not yet, untouched.
2. **`skein-oauth-issuer`'s `Authorization`:** a hand-written Debug printing `state_len` and `nonce_len`, as `skein/docs/design/oauth.md` 4 says; `oauth-issuer.md` 4's sketch changes to match.
3. **`LinkSignIn { secret: bytes 64 }`** in jig's client wire 19 (`client-v19.schema`, generated `v19.rs`): the field takes the generator's secret type, regenerated, with `LinkSignInParts` taking a `Secret` at its call sites across jig and temper. No compatibility is needed before v0.1.0.
4. **`smith-protocol-machine`'s `Variable`, `Environment` and `Spawn`:** print names only, as skein-io's `Spawn` does.

Rules: small tasks, no reshaping. Each carries a test that a value's bytes aren't in `format!("{:?}")`, and fails on the old code. A test that compares Debug output with a value changes with it; anything else asserting on printed bytes is a question. Keep every part in the plan. Wait for a designer before reporting `Done:`.
