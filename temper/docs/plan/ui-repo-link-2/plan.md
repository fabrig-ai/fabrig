# ui-repo-link-2: the plan

The request: the box's lifetime and its Forgejo sign-in settings, from a code review of `ui-repo-link` (`inputs/request.md`). Two tasks, as it asks.

## Parts and tasks

| Part | Task | Needs | Items | What it does |
|---|---|---|---|---|
| settings | `sign-in-settings` | | 2, 3, 4, 5 | a moved port reaches Forgejo's source; the pair refused; the session's hour; an empty name has no link |
| lifetime | `box-lifetime` | `sign-in-settings` | 1, 6 | `SIGHUP` as a stop; each step dies with the box on Linux; the walk of a default box |

## Critical path

`sign-in-settings`, then `box-lifetime`: about 230 implementer-minutes.

Item 1 alone builds on nothing. `box-lifetime` waits because it holds the walk, whose port and sign-in steps need `sign-in-settings`, and the request allows no third task for the walk. `sign-in-settings` goes first because it stays off the `skein-shell` files `debt-29` is changing, and both tasks write the `temper` crate. No shape crosses the tasks, so the plan has no `notes/contracts.md`.

## Decisions

The design holds them, in `temper/docs/design/protocol/sign-in.md`, 9.2, 9.9 and 10, `setup.md`, 6.2, 6.3, 6.4, 7.1, 7.2 and 13.1, `client/forge.md`, 5.4, and skein's `shell.md`, 6.3, `io.md`, 7 and `kernel.md`, 6.3. What the cut decided beyond the request:

- **A hangup is never a second signal.** A closing terminal's `SIGHUP` reaches the box from the kernel and again from the person's shell, and the box's second termination closes every step at once, which would break the request's "usual stop order". So `SIGHUP` has its own name, `ServiceSignal::Hangup`, rather than reading as `Terminate`. That costs six match arms and lets the box ignore a hangup once it stops.
- **`nohup temper box` keeps working.** A blocked signal reaches a signalfd whatever its disposition, so a box that simply watched `SIGHUP` would stop under `nohup`. Startup looks first, and a `SIGHUP` already ignored stays ignored.
- **`SIGHUP` is asked for, by the box alone.** A new `Watch::TerminationAndHangup`: `temper serve` and every other service keep the default. The request names only the box.
- **`SIGHUP` as a stop holds on macOS too;** only `forge-child` is Linux's. A macOS box killed outright still leaves its steps, as `setup.md`, 7.1 says.
- **The update runs last in a later preparation, under the set-up's deadline.** `state` is written by then, so a cut can't harm it. The deadline is there because `update-oauth` probably fetches the address it's given, which serve answers only once it's up: `sign-in-settings` checks this on the binary.
- **A failed update leaves the old address, and the entry still serves.** The one way to take the links down would be a marker file between the preparation and the `source` step. Both commands are the same command line that just prepared this Forgejo, so the failure is unlikely, and its one line in the preparation's log and Forgejo's password are enough. This is the one case where 9.9's "no address leads to a route Forgejo lacks" doesn't hold, and 9.9 says so.
- **The session's hour is on every owned Forgejo,** as the request places it, so a box with no `openssl`, whose links stay plain, asks for its password again after an hour. A condition on `sign_in` would save that and lose the reason's reach: cookies ignore ports whichever way a person signed in.
- **The refusal's words need a new `What::Because`** in `jig-protocol-config`: `What::Form` says only "invalid form", where the request asks for the configuration's words. `sign-in-settings` carries that one jig variant.
- **`temper/docs/design/domain/forge.md`, about 762, which the request cites,** says nothing about stopping: the stop's text is `setup.md`, 7.2, and that's what changes.
- **`sign-in.md`, 9.3 is unchanged:** nothing here touches the authorization's page.

A stored format and a public interface change, as the request allows, with no compatibility: `ServiceSignal`, `Watch`, `Settings`, `app.ini`, and a document with both `public` and `sign_in`, which no longer starts. `temper init --box` never wrote that pair.

No draft's status names this work, so no draft's entry closes. Slice 2 is `walk = true`, as `ui-repo-link`'s proof was; the task's own driver does the walk the request asks for.

## Overlaps with other drafts

- `box-lifetime` and `debt-29/shell-posix` (merged on its trunk, not on main): both edit `skein-shell`'s `linux/process.rs`, `linux/sync.rs` and `darwin/signals.rs`. The hunks are apart, and `open_termination_signals` moves there. The spec tells it to edit where that branch put the code, if its base has it. This is why `box-lifetime` goes second.
- `sign-in-settings` and `debt-29/link-readers-temper` (merged on its trunk): that one's `temper/crates/temper/**` and `temper-service/**` are wide globs, so an edit of `forge.rs` or `startup/problems.rs` may meet it. Each edit here is local.
- `temper/docs/help/box.md`: `model-check/trial-close` holds `temper/docs/help/**`. One sentence each here, away from any section it would change.
- `temper/docs/design/protocol/setup.status.md`, `sign-in.status.md`: this plan's entries are bullets of their own, so that merges with other drafts' long lines stay clean.
