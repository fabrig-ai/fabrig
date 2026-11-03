# sign-in: status

2026-11-03. Where temper's code lags or departs from [sign-in.md](sign-in.md).

- **Not yet:** 2.4: public reading's deployment: no proxy configuration caches `/jig/v1/public/` by deployment, `Host` and path, keeps no stale copy, or limits an address, and no deployment sets jig's public limits.
  2.5: the negative walk's real-service reads through the proxy: owners,
  versions and kinds, paired unknown answers, malformed paths,
  credential isolation, a member's visibility refusal, an older store's
  private projects, revocation during reads and cached service, load
  alongside signed-in work, abandoned reads' refunds and no CORS on
  every answer. The proxy's own rules are tested against a scripted
  upstream (`temper/tests/one-process/src/public/checks.rs`); the
  startup-relations test fails on a declared kind's answer-ceiling
  diagnostic.
  9.2, 9.7: a `sign_in.forges` entry checked once its forge's records load, and the page's metas made from the kept forges: the start checks the client alone (`temper-service/src/startup/application/sign_in/forges.rs`), and the metas are made at start against no forge (`temper/src/lib.rs`, `fill_forge_sign_in` given none; `temper-service/src/clients/bundle.rs`, `forge_metas`), so a document with an entry and a bundle refuses its start, naming `application.clients.bundle`.

- **Departs:** 2.5: a declared public kind above
  `clients.public.answer_bytes` refuses the start at `engine.clients`,
  not the limit's path (`temper/crates/temper-service/src/clients.rs`,
  `Clients::new` maps the component's `Unusable::Limits` through
  `invalid`); the service maps every component construction error
  through that one diagnostic.
