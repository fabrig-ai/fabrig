R1 passed again, matching `walk-2/real-check-login.md`: the unchanged shown command fetched the pin and reached sign-in. The run's external-service boundary failed separately (R2: the required Forgejo step also fetched).

# The program-login command on this trunk

2026-11-06, base `fa5b518db6093a96a90d3507a4e18ee1ed27eea6`.
Builder: `debian-16gb-hel1-8`, Linux x86_64, glibc 2.41;
Chromium 147.0.7727.101. Evidence is outside the checkout at
`~/src/fabrig/ui-walk-3/real-login/` on that builder; `index.html` links it.

## Conditions

Built with `TEMPER_BUNDLE=embed heavy cargo build --locked -p temper`, exit 0,
Cargo's reported build time 60 seconds. Copied `target/debug/temper` before
starting any box. Its SHA-256 was
`276e111ecf2b5006215e883eaea09b5465000ca2dfb6cc9a4e4efd9a3e2e055b`.
The box served its embedded browser bundle, with no fake backend or external
bundle. `BUILD/` held only `temper`, with no `claude`.

The custom box and both terminal invocations shared a fresh, initially empty
`HOME` and `PATH=BUILD:/usr/local/bin:/usr/bin:/bin`. The explicit environment
exported no `TEMPER_*` or XDG variable; it set `BROWSER=/bin/false` and
`CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1`. It also set `TERM=xterm-256color`,
`SHELL=/bin/bash` and `LANG=C.UTF-8`. `D` below abbreviates the absolute
`~/src/fabrig/ui-walk-3/real-login/boxes/1` directory. `D/config space`,
`D/data's`, `D/cache` and `D/home` were initially empty, private (0700).
The box ran with `--config 'D/config space' --data "D/data's" --cache D/cache`.

Chromium was headless with background networking, component updates and sync
disabled; name resolution for non-loopback hosts was blocked. This does not
restrict the box's own downloads: see R2. No provider sign-in method was
selected, no provider sign-in address was opened, no credentials were entered
or read, and no model call was made.

## Conditions and steps observed

| Step done | What it showed | Verdict / evidence |
|---|---|---|
| Start a real custom-directory box and open its printed owner link at 1440 | Welcome offered Set up temper. | Pass; `custom-01-welcome.png`. |
| Set up temper, then Forgejo's Run on this machine | The sheet offered the empty repository `work/hello`; its defaults were retained. | Pass; `custom-02-forge.png`, `custom-03-repository-sheet.png`. |
| Press the sheet's Run on this machine | Model offered ChatGPT and Claude; the Forgejo pane began fetching Forgejo. | Page transition passed; external-service restriction failed R2; `custom-04-model.png`, `custom-model-1440.png`. |
| Use your Claude subscription | The live code element contained all three directory assignments and `temper program-login claude-owner`, including POSIX quoting of the space and apostrophe. | Pass; `custom-model-{1440,420}.png`. |
| Feed that element's `textContent`, without edits, to `script -qfc COMMAND /dev/null`, with the box still running and the same clean home and PATH | `fetching Claude Code`, 101 progress messages ending at `fetched claude-code 293776752 of 293776752 bytes`, then `Opening browser to sign in…` and `Paste code here if prompted`. No theme question came first. Sent Ctrl-C immediately at the sign-in prompt. | Pass; `command-1.terminal`, `evidence.json`. Wrapper exit 0 is not sign-in success. |
| Check the fetched file by `stat` and `sha256sum` alone | 293,776,752 bytes, mode 0755, SHA-256 `86dc1fcc69498159eaf15f446613e847010f9036bfbbdffd4c49350be8489ada`; both size and digest equal the `linux-x64` pin for Claude Code 2.1.311 in `temper/crates/temper-forge-setup/src/claude_code.rs`. | Pass; `evidence.json`. |
| List names alone under home and the program directory | Home stayed empty. `D/data's/programs/claude-owner` held `.claude.json` and `backups`; their contents were not read. | Pass; `evidence.json`. |
| Run the identical command a second time | Reached the same sign-in prompt; zero `fetching Claude Code` and zero `fetched claude-code` messages. Ctrl-C there; wrapper exit 0. Home remained empty. | Pass for reuse; `command-2.terminal`, `evidence.json`. |
| Open Connect a model, choose Use your Claude subscription | Its command was byte-for-byte equal to Model's, all 260 characters. No account submission was pressed. | Pass; `custom-connect-{1440,420}.png`, `connect_equals_model` in `evidence.json`. |
| SIGTERM the custom box and wait | Exit 0; its lifecycle log says `09:02:30Z Stopping`, `09:02:31Z Stopped`. | Pass; `box-1.log`. |
| Start a plain real box, with no directory flags, in a different fresh home; repeat the same welcome, Forgejo and Claude choices | Exactly `temper program-login claude-owner`, 33 characters, at both widths. This short command was rendered, not executed. | Pass; `plain-01-welcome.png` through `plain-04-model.png`, `plain-model-{1440,420}.png`. |
| SIGTERM the plain box and wait | Exit 0; `09:02:35Z Stopping`, `09:02:35Z Stopped`. | Pass; `box-2.log`. |
| Stop Chromium and remove temporary directories | Chromium exited 0. Removed `boxes/` and `build/`, including the fetched binary, fresh homes, configurations and browser profile. | Pass; `cleanup` in `evidence.json`. |

The command below generalises only the evidence directory. The command that
ran was the live element's text, with its absolute paths, not this excerpt:

```sh
TEMPER_CONFIG='D/config space' TEMPER_DATA='D/data'\''s' TEMPER_CACHE='D/cache' temper program-login claude-owner
```

First command: `09:02:22.262323Z` to the prompt at `09:02:27.939639Z`,
5.677 seconds. Second: `09:02:28.194680Z` to `09:02:28.417670Z`,
0.223 seconds. These measure the whole command to its first prompt;
network-only download time was not measured.

## Render proof

All widths were set through CDP `Emulation.setDeviceMetricsOverride`,
height 1000, scale 1. Every character's range rectangles lay inside its
command panel. The panel's scroll and client widths matched; document width
equalled window width. All six screenshots were also inspected visually.
The long command has 260 rather than the second walk's 314 characters because
this evidence directory's absolute path is shorter.

| Page | Viewport | Characters | Panel client / scroll width | Panel height | Characters outside | Document / window width |
|---|---:|---:|---:|---:|---:|---:|
| Custom Model | 1440 | 260 | 588 / 588 | 114 | 0 | 1440 / 1440 |
| Custom Model | 420 | 260 | 344 / 344 | 168 | 0 | 420 / 420 |
| Custom Connect a model | 1440 | 260 | 1307 / 1307 | 66.875 | 0 | 1440 / 1440 |
| Custom Connect a model | 420 | 260 | 341 / 341 | 159.0625 | 0 | 420 / 420 |
| Plain Model | 1440 | 33 | 588 / 588 | 42 | 0 | 1440 / 1440 |
| Plain Model | 420 | 33 | 344 / 344 | 42 | 0 | 420 / 420 |

## Findings

| id | where (step, page, state, width) | seen | the mock or rule says | kind | size | owner | state |
|---|---|---|---|---|---|---|---|
| R2 | Real check; sheet's Run on this machine; fresh custom or plain cache; 1440 and 420 | The required UI action starts fetching Forgejo as well as the later Claude Code fetch. The screenshot's Forgejo progress advances; the custom box subsequently logs its Forgejo group started. | The real-login spec permits the pinned Claude Code download as its one real service. An empty cache plus the required Forgejo setup does not keep that boundary. | other | large | fabrig/ui | |

### R2: the check's service boundary conflicts with its required route

Start a real box with empty custom directories, open its owner link at 1440,
press Set up temper, Forgejo's Run on this machine, then the sheet's
Run on this machine. Before running the Claude command, Model already says
`Setting up: fetching Forgejo`; `custom-model-1440.png` shows 30%, and
`custom-model-420.png` shows 42%. The plain box repeats this in
`plain-model-{1440,420}.png`. `box-1.log` records Forgejo's group started at
09:02:27Z. No extra service probe was directed by the driver; the extra fetch
followed the required page action. This is a conflict in the check's scope,
not evidence that the product's Forgejo setup is wrong. Resolving which
service downloads this real check permits, or changing its required route,
needs the plan owner's decision; this observation-only task changes neither.

## Evidence limits

R1's missing-directory failure in `walk-2/real-checks.md` did not recur;
this repeats the successful fix verification in `walk-2/real-check-login.md`.
No product fault was found in the command, fetch, reuse or wrapping. R2 means
this run cannot claim the Claude download was the only real service reached.

Provider authorization, successful authentication, returning after sign-in,
account checking, a funded trial/model call, default-directory command
execution, Forgejo's repository creation, and other platforms remain
unproved. The fake-box supplement from the second walk was not repeated.

The first driver attempt incorrectly expected `Stopped` on stdout. Its box
exited 0, but the driver checked the wrong destination and removed its data
before collecting the lifecycle log. Reading the existing log-routing test
showed that notices belong in `DATA/box/logs/box.log`. The corrected complete
run above collected that log before removal; no product code changed.
`initial-attempt/` retains that attempt's sanitised evidence: the same digest,
10.168 seconds to the first prompt, 0.278 seconds on reuse, and four passing
custom-command render measurements. Its boxes, Chromium and binaries were
also stopped and removed.

Owner and provider sign-in addresses were stripped from every retained log
and terminal capture; no raw capture remains. Screenshots contain only the
application pages, with no sign-in address. The evidence index, screenshots,
sanitised terminal and box captures, Chromium diagnostics and measurement JSON
remain outside the checkout. The throwaway driver was removed before push.
