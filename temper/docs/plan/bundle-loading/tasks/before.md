+++
id = "before"
part = "figures-before"
kind = "docs"
needs = []
files = ["temper/docs/plan/bundle-loading/figures.md", "temper/docs/plan/bundle-loading/driver/**"]
effort = "medium"
minutes = 45
+++

## Goal

Say what a person waits for today, on a **release** bundle, before anything of this plan lands: so far only a debug build's sizes are known. Your slot's base is the plan's "before". This task changes no code.

Acceptance:

- `temper/docs/plan/bundle-loading/figures.md` holds, under `## Before`, with how each was measured:
  - each of the bundle's files, raw and by `gzip -9`, with a total, for a release module (the `web` profile);
  - at 2 Mb/s, under temper's exact policy, served by `temper serve`: first paint and what it looked like, the bytes sent by then, something to read, and the client's first node;
  - `temper serve`'s start with that bundle: time to its listening line and its peak memory (`VmHWM`);
  - one run of the browser tier: CPU seconds and wall against its budget (85 CPU s), beside the baseline `browser-trim` left, 58 to 79 CPU s (machine `debian-32gb-hel1-14` under load, both figures quoted from its report);
- `temper/docs/plan/bundle-loading/driver/load.py` is committed, for `page-temper` to measure by the same script.

## Inputs

- `temper/docs/plan/bundle-loading/notes/common.md`.
- `temper/docs/design/draft/bundle-loading.md`, sections 1 and 4: the table your figures sit beside.
- `temper/docs/design/client/browser.md`, 2.2 and 2.6 (the module's profile, `web`), 4.3.
- `temper/tests/client-browser/src/lib.rs`: how the tier writes `temper serve`'s configuration, naming a bundle's directory in `application.clients.bundle`.

## Owns and may change

- Owns: the two paths in `files`. A departure from the implementer's text: `driver/load.py` is committed in the plan's directory, not thrown away, because a later task on another builder must measure identically; it leaves `main` with the plan's files.
- Nothing else. If the bundle can't be built or served at the base, that is `Failed:`.
- Options: if `temper serve`'s configuration is awkward to write by hand, take the tier's by reading its code and write a file of your own under `~/src/fabrig/bundle-loading/before/`. If a figure can't be had, say `unknown` and why, and go on.

## Steps

1. Build, each through `heavy`, one at a time:
   `heavy cargo build --target wasm32-unknown-unknown --profile web -p temper-client-shell-web`,
   `heavy cargo build -p skein-ui-bundle`, `heavy cargo build -p temper`.
2. Make the bundle with the arguments of `temper/crates/temper-client-bundle/build/steps.rs` (`commands`), with the module from `target/wasm32-unknown-unknown/web/`, into `~/src/fabrig/bundle-loading/before/bundle/`. Keep it there.
3. Sizes: `wc -c` and `gzip -9 -c FILE | wc -c` for each file.
4. Write `driver/load.py`, standard library only, `python3 -I -B`, over Chromium's `--remote-debugging-pipe`, with `SKEIN_TEST_CHROMIUM`'s binary, new headless, a fresh profile each run. Its arguments: the URL, a width (it uses `Emulation.setDeviceMetricsOverride`), an out directory, and a repeated `--mark` to take a screenshot at a named moment. It:
   - sets `Network.emulateNetworkConditions` to `downloadThroughput` and `uploadThroughput` 250000 bytes/s, `latency` 0, offline false, and `Network.setCacheDisabled` false;
   - answers `Page.javascriptDialogOpening`;
   - logs, each with its time from navigation: `Network.responseReceived`'s URL, status, `Content-Encoding`, and `encodedDataLength` at `Network.loadingFinished`; the page's `first-paint` and `first-contentful-paint`; the moment of the first text a person can read (a poll of `document.body.innerText` by `Runtime.evaluate` every 50 ms); the first child of `#app`; bytes received at each of these; every `Runtime.exceptionThrown` and `Log.entryAdded` of source `security`, which a violation of the policy shows;
   - can run a second time on the same profile, for a cached load;
   - prints the log as one JSON document and writes it to the out directory.
5. Start `temper serve`, as the tier does, naming that bundle, and note its time to the listening line and `VmHWM` from `/proc/<its pid>/status`. Run the driver at 1440 twice from fresh, and once cached. Keep the outputs under `~/src/fabrig/bundle-loading/before/`. Stop your server.
6. The tier: `python3 -I -B tools/tiers/run temper browser`, once. Note CPU, wall and the machine's load.
7. Write `figures.md`. `## Method` says how it was measured, in words that name no machine; `## Before` has the tables. The builder goes in your report.
8. `python3 -I -B tools/citations/citations.py --base origin/main`, commit, push.

There is no test to bite; there's nothing to fmt or clippy but your Python, which you check by having run it.

## Report

Which builder; the figures; anything that surprised you (a violation of the policy, say, is a finding, not a fix).
