# forge-setup: status

2026-11-02. Where skein's code lags or departs from [forge-setup.md](forge-setup.md).

- **Departs:** 6.3: the host component takes a forge and drops it while it runs (`Request::Add`, `Request::Remove`; a removal forgets the forge's runs and callbacks), where the design names every forge in its settings at construction. Because forges are store records and the playground is reset (`temper/docs/design/draft/playground-reset.md`, 3, 4, 6).
