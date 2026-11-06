# The program-login command on its box

2026-11-06. R1 passed: the command copied from the real Model page, without edits, fetched the pinned Claude Code and reached its first sign-in prompt. No provider sign-in method was chosen, no sign-in address was opened, and no model call was made.

## Conditions and steps

Built with `TEMPER_BUNDLE=embed heavy cargo build --locked -p temper`; the final build took 20.97 seconds. The embedded browser bundle was used, with no fake backend or external bundle. The copied binary's SHA-256 was `382fa216ac79ee856bc989bfe6c186d2611d33d4f5b0688f239aba3c5fc76ef4`.

The box and command shared a fresh, initially empty `HOME/` and `PATH=BUILD:/usr/local/bin:/usr/bin:/bin`, where `BUILD` held the copied `temper` and had no `claude`. No `TEMPER_*` or XDG variable was exported. `BROWSER=/bin/false` and `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1` prevented opening a provider sign-in page and disabled nonessential traffic. Chromium's background networking was disabled and its non-loopback name resolution blocked. Its version was 147.0.7727.101.

Started a real `temper box --config 'D/config space' --data "D/data's" --cache D/cache`, with all three directories empty and private. The owner link opened Welcome. Pressed Set up temper, Forgejo's Run on this machine, and the sheet's Run on this machine, retaining the empty repository `work/hello`. On Model, chose Use your Claude subscription. The public agent configuration named the program path `"fetched"`.

The page showed this line, with only the evidence directory generalised to `D/`:

```sh
TEMPER_CONFIG='D/config space' TEMPER_DATA='D/data'\''s' TEMPER_CACHE='D/cache' temper program-login claude-owner
```

Copied the live command element's `textContent` directly to `script -qfc`, under the same home and PATH, while the box was running. Nothing was added to or changed in the command. Between 02:26:17.297371Z and 02:26:51.079170Z the terminal printed `fetching Claude Code`, fetch progress ending at `fetched claude-code 293776752 of 293776752 bytes`, `Opening browser to sign in`, and `Paste code here if prompted`. Sent Ctrl-C at that prompt. The terminal wrapper exited 0; that wrapper status is not a sign-in success.

The fetched file was 293,776,752 bytes, SHA-256 `86dc1fcc69498159eaf15f446613e847010f9036bfbbdffd4c49350be8489ada`, matching the Linux x64 pin in `temper/crates/temper-forge-setup/src/claude_code.rs`. `HOME/` remained empty. Listing names alone found `.claude.json` and `backups` under `D/data's/programs/claude-owner`; no program login file was read.

Opened Connect a model at the same program's way. Its command was byte-for-byte equal to Model's. Stopped the custom box: `02:26:54Z Stopping`, `02:26:55Z Stopped`.

Started another real, plain `temper box`, with no directory flags, under a different fresh home and the same clean PATH. Repeated the welcome's Forgejo and Claude choices. At both widths the command was exactly `temper program-login claude-owner`, on one line. Stopped that box: `02:28:03Z Stopping`, `02:28:03Z Stopped`. This exercises the child's explicit directory flags and inherited HOME together.

## Render proof

CDP set viewport widths 1440 and 420, height 1000, scale 1, using `Emulation.setDeviceMetricsOverride`; the driver handled departure dialogs. Both custom-directory pages retained all 314 command characters. Every character rectangle stayed within its panel, each panel's scroll width equalled its client width, and the document's width equalled the viewport's.

| Page | Viewport | Panel client / scroll width | Command height | Characters outside |
|---|---:|---:|---:|---:|
| Model | 1440 | 588 / 588 | 132 | 0 |
| Model | 420 | 344 / 344 | 222 | 0 |
| Connect a model | 1440 | 1307 / 1307 | 66.875 | 0 |
| Connect a model | 420 | 341 / 341 | 177.5 | 0 |

The default command retained all 33 characters and a 42-pixel panel height at both widths. Screenshots were inspected at 420 as well as measured. Evidence remains outside the checkout in the task's `ui-final-walk/login-command` directory: `real-login.json`, `real-command-generalised.txt`, `custom-welcome-command-{1440,420}.png`, `custom-connect-command-{1440,420}.png`, `defaults-final-command-{1440,420}.png`, their measurement JSON and the shutdown observations. Raw terminal captures and printed owner links were removed; no provider URL or token is retained.

A separate fake-box walk used the loopback bundle built by `heavy temper/scripts/bundle.sh --loopback OUT`. Its Model page correctly showed all three effective sandbox directories in the long form, including `D/data's/box/fake-backends/config` and `D/data's/box/fake-backends/data`. Both viewport proofs passed (`fake-command-{1440,420}.png` and `fake-command.json`). That box printed `02:36:18Z Stopping`, `02:36:19Z Stopped`. No fake-only product path was added. All boxes, browsers, copied builds and fresh homes were removed after collecting the evidence.
