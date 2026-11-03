# Request: measure the design's other wasm32 crates; add the green ones to the gate list

From agents/loop, 2026-11-03 ~17:45Z (unattended; the recommended option of plan wasm32-list's
question `wasm32-more`). fab's gate step `clippy-wasm32` reads `.config/gate/wasm32.txt` (six
packages today). The designs name more crates for the wasm32 gate: skein's other portable crates
(lib, io, the simulator, the shell, skein-sim-world, the fake machine, the fakes, kv, http, json,
channel, codec, tls; skein's testing.md 6 and 10, lib.md 2, shell.md 14, world.md 8),
jig-client-test-shell-web, jig's client crates and jig-protocol-transcript (jig's
protocol/browser.md 15 and 18, protocol/README.md 11), temper-client-domain, -view,
-protocol-wire and -shell-web (temper's client/browser.md 4.3), and smith-agent-shell with its
smith dependencies (smith's shell.md 2.5).

One task:
- run `heavy cargo clippy --target wasm32-unknown-unknown -p <crate> -- -D warnings` at main for
  each crate the designs name (resolve the names from `cargo metadata`; a name that is no package
  is reported, not guessed), one at a time;
- add every green one to `.config/gate/wasm32.txt`, keeping its comment line; then run the
  whole list's combined command once; it must pass;
- for each red one, change nothing in it and record it in its tree's status file as Not yet with
  the first error's file, line and lint (one line each), so its tree repairs it;
- write `answer = "..."` for question `wasm32-more` in `temper/docs/plan/wasm32-list/plan.toml`:
  "Measured by plan wasm32-more: <n> added, <m> red and recorded as Not yet in their trees."
- report the per-crate results and the combined step's wall time (the gate pays it on each
  touching batch).
