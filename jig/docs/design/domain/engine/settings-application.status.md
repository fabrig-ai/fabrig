# settings-application: status

2026-11-23. Where the tree's code lags or departs from [settings-application.md](settings-application.md).

- **Not yet:** 1: validation of declared application kinds against
  configured limits before construction is missing, because
  `jig-domain-core/src/settings.rs`, `Current::new`, expects the capacity to
  be representable instead of returning a named configuration refusal; the
  bounds check takes limits without declared kinds. No reason is recorded.
