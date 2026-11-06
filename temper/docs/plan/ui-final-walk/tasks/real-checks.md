+++
id = "real-checks"
part = "real-checks"
kind = "docs"
needs = []
files = [
  "temper/docs/plan/ui-final-walk/real-checks.md",
]
effort = "high"
minutes = 75
+++

# The two real checks: Claude Code's fetch, and a browser's Sign out

## Goal

The request's *Two real checks*: what no fake has run. This task fixes nothing: it writes down what happens, and `fix-flow` takes what's small.

Acceptance:

- `real-checks.md` says, for each check, each step done, what it showed and its verdict, with a finding `R1`, `R2`, ... for each fault (`notes/common.md`), and what stays unproved.
- **The fetch:** the real box's Model step offered Claude, with no `claude` on `PATH`; the command the page shows really downloaded the pinned 2.1.311; its size and SHA-256 equal `temper/crates/temper-forge-setup/src/claude_code.rs`' pin for this platform; it ran as far as asking to sign in, and was stopped there.
- **Sign out:** on a non-fake `temper serve`, headless Chromium signed in by the printed link and signed out; the page says so; it stays signed out through a reload, through an address typed, and to the server.
- Screenshots and the terminal's capture are on the builder under `~/src/fabrig/ui-final-walk/real-checks/`, and every process you started is stopped.

## Inputs

- `temper/docs/plan/ui-final-walk/inputs/request.md`, *Two real checks*, *For every task of this plan*.
- `temper/docs/plan/ui-final-walk/notes/common.md`; `notes/decisions.md`, D12 and *First run*.
- `temper/docs/design/draft/claude-provider.md`, 2 and 3; `temper/docs/design/protocol/setup.md`, 6.6.
- `temper/docs/help/box.md`, 1, 2, and the `program-login` lines in 5.
- `temper/docs/design/draft/serve-without-box.md`, 2 and 3.
- `temper/tests/client-browser/tests/browser_all/browser_sign_in.rs`: the same Sign out on a fake, its steps worth copying.

## Departures from the implementer's text and `AGENTS.md`

- **A real download:** `AGENTS.md`, 3's "no probes against real services" gives way, for this task alone, to what the request names: the pinned file from `downloads.claude.ai`, and the pinned Forgejo and runner that a real box's own set up fetches. Nothing else leaves the machine.
- **No credentials, no sign-in:** you never choose a way to sign in, open a sign-in address or type anything to one. No ChatGPT sign-in. If the program asks only a first-run question ahead of signing in (a theme), answer it to go on; stop at the first question about signing in.
- **A real box's sign-in link is a secret:** it goes from the box's output to the browser, and never into a file you keep, a report or a screenshot's name. Its stdout goes to a file 0600 in a 0700 directory, removed at your end.

## Owns and may change

Owns `real-checks.md`. No other tracked file; a script you need is throwaway, untracked and gone before the push.

Options, each pre-approved:

- **The machine:** if the builder has a `claude` on `PATH`, run with a `PATH` that lacks its directory, for `temper box` and for the command alike, and say so. A `claude` named at init would stop this being the check: read `CONFIG/agent.json`'s `path`, which must be `"fetched"`.
- **`HOME`:** a fresh empty directory of your own for both, so that no login of anyone's can be read. Say so.
- **The build:** a real box takes no `--bundle`: `TEMPER_BUNDLE=embed heavy cargo build --locked -p temper`, as `temper/scripts/install.sh` builds with `--release`, then copied (`notes/common.md`, Artefacts). If the machines' cargo configuration holds it at `none`, the bundle isn't in the binary: then `heavy temper/scripts/bundle.sh BUNDLE` and the deployment's configured bundle directory, and say so.
- **A download that fails** (the network, an address gone): its exact error is the finding. Try it twice, a few minutes apart. Don't look for the file elsewhere. A failed fetch doesn't stop the Sign out check.
- **A stuck page, a refusal, a crash:** the finding, with its log's lines; then go on to what you can still reach.

## Steps

1. Build and copy.
2. **The fetch.** Fresh directories; the clean `PATH` and `HOME`; `temper box --config CONFIG --data DATA --cache CACHE`, real backends. In headless Chromium at 1440, through CDP, handling `Page.javascriptDialogOpening`: the link, the welcome, "Set up temper", Forgejo's "Run on this machine" on the Forge step and its sheet, the empty `work/hello`. Screenshot each. On Model: are ChatGPT and Claude both offered? Choose Claude's way; screenshot. Note the command it shows.
3. Run that command in a pty, by `script -qfc`, under the same `PATH` and `HOME`, capturing the screen to your folder, and watch the fetch's words. Stop at the first question about signing in; capture; stop it with Ctrl-C.
4. Check: `stat` and `sha256sum` of the fetched file, against the pin; the launcher at `DATA/claude-code/claude` and `DISABLE_AUTOUPDATER=1` in it; its version by the launcher's `--version`; that a second run of the command fetches nothing (stop it at the same place); that the program's own directory sits under `DATA`, and `HOME` stayed empty of a login, by listing names alone. Back on the page, press "I've signed in" once, and screenshot what it says.
5. Stop the box. Remove its directories, the fetched file with them.
6. **Sign out.** Fresh directories; `temper init --config CONFIG --data DATA --cache CACHE`, the bare deployment; `temper serve` on them. In headless Chromium, at 1440, then again at 420 by `Emulation.setDeviceMetricsOverride`, by what a person presses: the link, the page signing in opens, Accounts, "Sign out". Screenshot each.
7. Check: the signed-out page; a reload; a page's address typed; then, from the page's own context through CDP, one request with what the browser still holds, which the server refuses. Sign in again by a fresh link (`temper/docs/help/server.md`, 6, on a spent link).
8. Sign out five more times, each a new sign-in, some at once on the page's load and some once settled: the earlier faults were a lost click while the accounts list redrew, and a bare 503 once a page used all four of the client's calls on the `small` profile. Count how many took, and watch the network for any 503.
9. Name in `real-checks.md` what stays unproved: no TLS proxy, no Google sign-in, no deployment like the person's own.
10. `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

## Report

As `notes/common.md`, Your report, and: each check's verdict in a line, the download's time, and the finding rows.
