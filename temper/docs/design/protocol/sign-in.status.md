# sign-in: status

2026-11-05. Where temper's code lags or departs from [sign-in.md](sign-in.md).

- **Not yet:** 2.4: public reading's deployment: no proxy configuration caches `/jig/v1/public/` by deployment, `Host` and path, keeps no stale copy, or limits an address, and no deployment sets jig's public limits.
  2.5: the proxy's own rules are played against scripted upstreams, not
  temper's service (`temper/tests/one-process/src/public/checks.rs`):
  the `Age` it keeps and the stale copy it never serves, what it keeps,
  its upstream stopped, two deployments behind it with one path, and a
  refused `Host` with a copy cached.
  7, 9.1, 9.2, 9.7: a `sign_in.forges` entry checked once its forge's records load, the page's metas made from the kept forges, and the browser origin by a record's `web`: the start checks the client alone (`temper-service/src/startup/application/sign_in/forges.rs`), and the metas are made at start against no forge (`temper/src/lib.rs`, `fill_forge_sign_in` given none; `temper-service/src/clients/bundle.rs`, `forge_metas`), so a document with an entry and a bundle refuses its start, naming `application.clients.bundle`.
