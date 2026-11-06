# box-small: three small things the user met walking a real box

The user asked for these, 2026-11-06, while walking a real box. **Quick: three tasks, in this order, each landing on its own if it can.** Where a choice is left open, take the one written here.

## 1. The command on the page names the binary that's running

The user: "it should print the path of the actual binary the process was started from (or just "temper" if that binary/process is in $PATH)", "with $HOME converted to ~/ (and slashes deduped if needed)".

The welcome's Claude page printed `TEMPER_CONFIG='...' TEMPER_DATA='...' TEMPER_CACHE='...' temper program-login claude-owner`. The box ran from a checkout's `target/debug`, and the `temper` first on the user's `PATH` was a months-old build, which answered "unknown command `program-login`".

- **The rule:** take the running process's executable. Find the first executable named `temper` on its `PATH`. If that's the same file, by canonical paths, the command says `temper`. Otherwise it says the executable's path, a leading `$HOME` as `~/`, and repeated slashes collapsed. Only the first on `PATH` counts, as that's what a shell runs.
- **A binary rebuilt while the box runs:** Linux's `current_exe` carries a ` (deleted)` suffix. Drop it.
- **Quoting:** the word stays unquoted where it can, so that the shell expands `~/`; a path needing quotes quotes only what follows `~/`.
- **The same `~/` and slash treatment for the three `TEMPER_` values,** unquoted when safe, as a tilde expands right after `=` in an assignment prefix. They're what makes the line long.
- **Where:** the welcome's Model step and ConnectModel (`temper/crates/temper-client-domain/src/pages/connect_model.rs`, `temper/crates/temper-client-view/src/pages/welcome_model.rs`, `connect_model.rs`; the answer's text is made in `temper/crates/temper/src/directories.rs`, `program_environment`), and any other command handed to the person for this machine. `On each host, run: temper program-login` (`temper-client-view/src/accounts/words.rs`) names other machines, so it stays plain `temper`.
- `first-usable-model`, in the queue, also touches what the page prints: take it by catch-up.

## 2. A copy button on the command

No look request: this is the whole of it. On every `temper-welcome-cmd` block, browser only: the terminal has no button. A quiet text button "Copy" in the block's top right corner, in the style of `temper-flow-quiet` at 12px. The block's right padding keeps the text from running under it, at every width. Pressed, it writes the command's exact text as one line, and reads "Copied" for 2 s. It's an effect asked in the gesture's turn (skein's `skein/docs/design/ui.md`, section 13; `skein/docs/design/ui-web.md`'s effects). The clipboard's only there in a secure context, so on plain http, or on any refusal or failure, it reads "Couldn't copy" for 2 s, and the text stays selectable. A generic mechanism goes in skein first, and skein names no temper. fabrig/ui has priority here: keep hunks small and name overlaps.

## 3. The sign-in link lives 2 hours, and the box's message says so

The user: "can we raise the expiration of the signin link to 2 hours? And update the message printed at box startup as well."

- `link_span` in `jig/crates/jig-protocol-config/src/profile/small.rs`: 300 s to 7200 s. It's one value, so a server's printed link takes it too; fixtures keep their own. Say in the design, in jig's `jig/docs/design/protocol/client.md`, section 8, what it now is, and why: a link may be opened late on a person's own machine. **Say what it means for a server's printed link, as it's a secret live for longer:** a one-time link, on the machine that printed it.
- The footer in `temper/crates/temper-box/src/terminal.rs` and its copies in tests:

  ```
  It signs you in as this temper's owner, once, within 2 hours.
  Opened after that, it prints a fresh one here.
  ```

  "Opened after that", because an expired link also draws a fresh one (`jig/docs/design/protocol/client.md`, section 8).
- The words in `temper/docs/help/box.md`, `temper/docs/design/protocol/setup.md`'s block and `temper/docs/help/party.md` ("once, within 2 hours").
- If it's cheap, one test that holds the footer's hours to the profile's `link_span`.

## Tests

Failing tests first, in the lowest tier: each rule of item 1 as a pure function, with a fake `PATH`, `HOME` and a deleted suffix; the button's states in the view; the footer's exact bytes. No test reaches a real host. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). `walk = false`: the user's walking a box now.
