# sign-in: status

2026-11-03. Where temper's code lags or departs from [sign-in.md](sign-in.md).

- **Not yet:** 2.4: public reading's deployment: no proxy configuration caches `/jig/v1/public/` by deployment, `Host` and path, keeps no stale copy, or limits an address, and no deployment sets jig's public limits. 2.5: the negative walk and its caching proxy of the world. 9.2, 9.7: a `sign_in.forges` entry checked once its forge's records load, and the page's metas made from the kept forges: the start checks the client alone (`temper-service/src/startup/application/sign_in/forges.rs`), and the metas are made at start against no forge (`temper/src/lib.rs`, `fill_forge_sign_in` given none; `temper-service/src/clients/bundle.rs`, `forge_metas`), so a document with an entry and a bundle refuses its start, naming `application.clients.bundle`.
