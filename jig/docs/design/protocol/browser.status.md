# browser: status

2026-11-03. Where jig's code lags or departs from [browser.md](browser.md).

- **Not yet:** 15: of jig's wasm32 builds, fab's gate list (`.config/gate/wasm32.txt`)
  holds `jig-client-shell-web` alone: `jig-client-test-shell-web` and the client crates
  are not in the gate's wasm32 clippy. 18: nor is `jig-protocol-transcript`, so no gate
  checks `smith-transcript`'s build for wasm32.
