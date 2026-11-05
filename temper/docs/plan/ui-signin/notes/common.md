# ui-signin: what every task holds to

The user is asleep and takes fabrig/ui's judgement.

- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **A branch stays in its tree.** `peer-rejected` is skein's, `issuer-connections` is temper's.
- **Throwaway code** (a walk driver, a render script) stays untracked, outside the commit, and is gone before the push.
- **A CDP driver handles `Page.javascriptDialogOpening`** with `Page.handleJavaScriptDialog`: `closing_loses` in `temper/crates/temper-client-shell-web/src/lib.rs` arms `onbeforeunload` while a request is pending, and a dialog nothing answers hangs the renderer.
- **Chromium 147's new headless** won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`.
- **`temper.css`, goldens, prints and the gallery's `names.txt`/`tests.rs`:** this plan changes none of them. If a catch-up meets them, regenerate rather than hand-merge.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`.
