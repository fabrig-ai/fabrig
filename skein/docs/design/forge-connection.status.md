# forge-connection: status

2026-11-02. Where skein's code lags or departs from [forge-connection.md](forge-connection.md).

- **Departs:** 2.1, 3.1: `Request::Add` and `Request::Remove` take a forge and drop it while the component runs, where the design hands it every forge at start. Because forges are store records and the playground is reset (`temper/docs/design/draft/playground-reset.md`, 3, 4, 6).
