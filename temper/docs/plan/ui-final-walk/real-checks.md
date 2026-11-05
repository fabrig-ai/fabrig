# The two real checks

Run on 2026-11-05, at commit `a04030242092c9593f3137d6ce949f7f33666d99`.

**Fetch: the real pinned download and launch passed; the displayed command needs the deployment's directory variables to work (R1).**

**Sign out: passed seven times, at 1440 and 420 pixels; reload, a typed address and the server's refusal agreed. No HTTP 503 observed.**

## Evidence and conditions

Builder: `debian-32gb-hel1-11`, Linux x86_64, glibc 2.41. Evidence is outside the checkout, under `~/src/fabrig/ui-final-walk/real-checks/` on that builder. `index.html` links the screenshots, terminal captures and logs. Chromium was 147.0.7727.101, headless, controlled through CDP; phone captures used `Emulation.setDeviceMetricsOverride`, width 420, height 1000, scale 1. Initial desktop captures were 1440 by 861; desktop repetitions used CDP metrics of 1440 by 1000. The driver accepted JavaScript departure dialogs.

Built with `TEMPER_BUNDLE=embed heavy cargo build --locked -p temper`: exit 0, 32.40 seconds. The embedded bundle's digest was `5c5e9c35a6e06033`; no external bundle or fake backend was used. The binary was copied to the evidence folder's `build/` before either check.

The machine's existing `claude` was excluded: both the box and the terminal command used `PATH=BUILD:/usr/local/bin:/usr/bin:/bin`, where `BUILD` held only the copied `temper`. Both used the same fresh, initially empty home directory, not the builder user's home. Box and deployment directories were absolute, private (0700); their temporary stdout files were 0600. The driver consumed printed temper sign-in links without displaying or retaining them. Browser background networking was disabled and non-loopback name resolution blocked. The terminal environment set `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1`; the second login attempt also set `BROWSER=/bin/false`. No provider sign-in address was followed, no sign-in method was selected and no credentials were entered or read.

## Claude Code fetch

| Step done | What it showed | Verdict / evidence |
|---|---|---|
| Start a fresh real `temper box` with explicit config, data and cache directories | Init completed; `CONFIG/agent.json`'s `programs.claude_code.path` was `"fetched"`. The real box printed its owner link. | Pass; `fetch-box.log`. |
| Open the printed link; press Set up temper | Welcome opened, then Connect a forge. | Pass; `fetch-01-welcome-1440.png`, `fetch-02-forge-1440.png`. |
| Choose Forgejo's Run on this machine | The sheet offered an empty repository with Organization `work`, Name `hello`; those values were retained. | Pass; `fetch-03-empty-repository-sheet-1440.png`. |
| Submit the sheet | Model offered both ChatGPT and Claude while Forgejo prepared. Forgejo subsequently said connected as temper, `work/hello made`. | Pass; `fetch-04-model-both-offered-1440.png`, `fetch-05-claude-command-1440.png`. |
| Choose Use your Claude subscription | The page showed `temper program-login claude-owner` and explained its separate configuration. | Offer passed; command fault R1; `fetch-05-claude-command-1440.png`. |
| Run that exact command with `script -qfc`, the clean PATH and fresh home | It looked for the default configuration under the fresh home instead of the running box's explicit config directory, then refused: `configuration HOME/.config/temper/temper.json: absent; temper init writes one`. | Fail R1; `fetch-command-verbatim.terminal` contains the full local error. The capture wrapper did not propagate the child's failure; the diagnostic and capture's exit status record the refusal. |
| Supply `TEMPER_CONFIG`, `TEMPER_DATA`, `TEMPER_CACHE` for this box and repeat the unchanged displayed command with `script -qefc` | `fetching Claude Code`, then `fetched claude-code 0 of 293776752 bytes` through `293776752 of 293776752 bytes`; then `Opening browser to sign in…` and `Paste code here if prompted >`. | Fetch and launch passed; `fetch-first.terminal`. Stopped with Ctrl-C at this first sign-in prompt, exit 130. No theme question preceded it. Provider authorization URLs are omitted from retained captures. |
| Measure the fetched file, compare the source pin | 293,776,752 bytes; SHA-256 `86dc1fcc69498159eaf15f446613e847010f9036bfbbdffd4c49350be8489ada`. Both equal the `linux-x64` build in `temper/crates/temper-forge-setup/src/claude_code.rs`. Mode 0755. | Pass; `stat` and `sha256sum`, with size retained in `fetch-evidence.json`. |
| Check launcher and version | `DATA/claude-code/claude` linked to the copied `temper`; its sibling `cache` linked to this cache. The compiled launcher's source, `temper/crates/temper/src/claude_code.rs`, sets `DISABLE_AUTOUPDATER=1` immediately before exec. Running the launcher with `--version` printed `2.1.311 (Claude Code)`. | Pass. The updater setting was verified in source, without reading a process environment. |
| Run the command a second time with the same directory variables | It went straight to the same sign-in prompt; zero `fetching Claude Code` or `fetched claude-code` lines. Ctrl-C, exit 130. | Pass; `fetch-second.terminal`, `fetch-evidence.json`. |
| List names under this home and the program directory | Home was empty. Program names were under `DATA/programs/claude-owner/`: `.claude.json`, its lock and backups. No file contents were read. | Pass for isolation; `fetch-evidence.json`. |
| Back on the page, press I’ve signed in once | `Claude Code isn’t signed in here yet. Run the command above, then press I’ve signed in.` Forgejo remained connected with `work/hello made`. | Pass; `fetch-06-not-signed-in-1440.png`. |
| Stop and remove the box | SIGTERM; `23:05:46Z Stopping`, `23:05:47Z Stopped`; exit 0. Removed its configuration, data, cache, home, stdout and downloaded binary. | Pass; `fetch-box.log`. |

Download timing: the command's UTC start marker was `23:04:49Z`; the fetched file's last-write timestamp was `23:04:55.266267731Z`. Thus command start to the final file write was between 5.266 and 6.267 seconds, given the marker's one-second precision. Network-only duration and the later digest/rename completion time were not separately measured. The first terminal capture records the complete fetch progress and subsequent prompt.

This exercises D12's first fetch and size-only reuse, without changing that decision. The Model-step forge pane, `claude-owner` name and unsigned-in alert follow the listed First run decisions.

## Browser Sign out

Two fresh bare deployments were initialized with `temper init`, then run with real `temper serve`, without `--box`, fake backends, a forge connection or a model account. Init's default engine profile was `standard`, with twelve HTTP client calls and eight streams. The first deployment supplied the full persistence checks at both widths.

| Step done | What it showed | Verdict / evidence |
|---|---|---|
| Open serve's printed link at 1440 | Signed in as Owner; Needs you opened directly, with No project yet. No welcome or playground. | Pass; `signout-01-inbox-1440.png`. |
| Press Accounts, then Sign out | Accounts showed zero accounts. Sign out changed the page to `Signed out. To sign in, open a sign-in link temper serve printed.` | Pass; `signout-02-accounts-1440.png`, `signout-03-signed-out-1440.png`. |
| Reload, then type `/accounts` as a new navigation | Both loads still said signed out. | Pass; `signout-04-reload-1440.png`, `signout-05-typed-accounts-1440.png`. |
| From that page's context, fetch `/jig/v1/stream` with `credentials: 'include'` | HTTP 401, empty body, using whatever credentials the browser still held; no cookie or stored token was read. | Pass; `network-standard.json`. An earlier diagnostic POST to `/jig/v1/read` returned HTTP 200 with bytes `02 04`; it was not used as evidence of authentication refusal. |
| Reopen the spent printed link, then consume the fresh link serve printed | The spent link left the browser signed out; the new link signed it in. | Pass; no link retained. |
| Set 420-wide metrics; open Menu, Accounts; reopen Menu and press Sign out | The inbox and Accounts remained usable. Sign out again showed the signed-out message. | Pass; `signout-06-inbox-420.png` through `signout-10-signed-out-420.png`. |
| At 420, reload, type `/accounts`, then repeat the credentialed stream request | Both pages stayed signed out; the server again answered HTTP 401. | Pass; `signout-11-reload-420.png`, `signout-12-typed-accounts-420.png`, `network-standard.json`. |
| Five additional sign-ins and Sign outs on the second deployment | Every click took, including three sent immediately after navigating to Accounts, before yielding to its asynchronous reads, and two after a 1.2-second settle. | 5/5 passed; table below, `repetitions.jsonl`, `signout-repeat-*.png`. |
| Watch network responses and browser exceptions | First browser: 177 responses, two sign-out responses, zero HTTP 503, zero JavaScript exceptions. Repetition browser: 226 responses, five sign-out responses, zero HTTP 503, zero JavaScript exceptions. | Pass; `network-standard.json`, `network-small.json`. The first log also includes the preceding fetch walkthrough. |
| Stop both serves and Chromium; remove runtime files | Both serves terminated on SIGTERM with exit 0; Chromium and driver processes ended. No process whose arguments referenced this task's artifact directory remained. Removed `boxes/` and `build/`, including all private stdout, browser profiles, secrets and downloaded binaries; removed the throwaway driver from the checkout. | Pass; `serve-standard.log`, `serve-small-calls.log`. Screenshots, sanitized logs, captures and `index.html` remain. |

The repetition deployment used the working standard engine profile with the small profile's client limits: `clients.exchanges=8`, `clients_http.calls=6`, `clients.streams=2`, `clients_http.streams=2` (four ordinary exchanges beside two streams). Other init settings stayed at their generated values. An attempt to change the whole engine profile to small first refused startup: `engine.limits.overrides.tasks.charters: tasks.charters is 4; the stock offers 6 charters; raise it in engine.limits.overrides` (`small-profile-start-refusal.log`). This was an intentionally reduced configuration's stock-size guard, not a failure of the generated standard deployment. The full small profile remains unproved; its client-call limits were exercised without changing unrelated capacities.

| Sign out | Width | Timing on Accounts | Result | HTTP 503 |
|---|---|---|---|---|
| 1 | 1440 | Settled; init defaults | Signed out | 0 |
| 2 | 420 | Settled; init defaults | Signed out | 0 |
| 3 | 1440 | Immediate; small client limits | Signed out | 0 |
| 4 | 420 | Immediate; small client limits | Signed out | 0 |
| 5 | 1440 | Settled; small client limits | Signed out | 0 |
| 6 | 420 | Settled; small client limits | Signed out | 0 |
| 7 | 1440 | Immediate; small client limits | Signed out | 0 |

## Findings

| id | where (step, page, state, width) | seen | the mock or rule says | kind | size | owner | state |
|---|---|---|---|---|---|---|---|
| R1 | Real box; Model / Your Claude subscription; explicit deployment directories; 1440 | Displayed `temper program-login claude-owner` uses the fresh home's default directories and refuses because their configuration is absent. It only reaches the pinned download after the running box's three directory variables are supplied. | The page's terminal command should operate on the box being set up (`temper/docs/help/box.md`, 5, shows the directory flags). | other | small | fix-flow | |

Reproduce R1: start a real box with explicit empty config/data/cache directories and a fresh home, with no `TEMPER_CONFIG`, `TEMPER_DATA` or `TEMPER_CACHE` exported; open its owner link, choose Set up temper, Forgejo on this machine and the empty `work/hello`, then Claude. Run the shown command in the same home and PATH. It refuses at the default config path before downloading. Export the three directory variables and repeat: the pinned download and sign-in prompt become reachable. No implementation fix was made by this task.

## What stays unproved

- Provider authentication, paid model calls and a signed-in Claude session: both terminal attempts stopped at the first sign-in prompt. No ChatGPT or Google sign-in.
- TLS termination, a proxy, remote access, or a deployment matching the person's own. These checks used fresh local loopback deployments and the printed owner links.
- A project-bearing Accounts page, populated account-list redraws and every possible sign-out race. The real bare deployment has no project or model account; seven successful clicks and no observed 503 do not prove all schedules.
- The full small engine profile; only its client limits ran, as described above. There was no synthetic saturation traffic to force all four ordinary exchanges to stay occupied.
- Other platforms, a failed download, digest corruption and concurrent fetches. Only the real Linux x64 pin and one subsequent reuse were exercised here.
- Copying from a public host. The real Forgejo made the requested empty repository; no external repository was contacted.
