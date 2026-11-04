# json: status

2026-11-04. Where skein's code lags or departs from [json.md](json.md).

- **Not yet:** 6: the fuzz target.
- **Departs:** 5.3: one whole buffer kept as a document is `parse`, but
  `skein-forge-conformance/src/json.rs`, `Json::parse`, and
  `skein-throwaway-forgejo/src/instance.rs`, `value`, each append it to
  an `Intake` and pump by hand a `Collector` that keeps the whole value.
  Both are test kits; no debt item holds them.
