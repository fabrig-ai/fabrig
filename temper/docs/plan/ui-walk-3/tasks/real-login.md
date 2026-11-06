+++
id = "real-login"
part = "real-login"
kind = "docs"
needs = []
files = [
  "temper/docs/plan/ui-walk-3/real-check.md",
]
effort = "medium"
minutes = 40
+++

# R1 on this trunk: the shown command, on a box with custom directories, up to the sign-in prompt

## Goal

Run again what `temper/docs/plan/ui-walk-3/walk-2/real-check-login.md` ran, on this trunk: a real box with custom directories, the command copied from the page, up to the sign-in prompt, and no further. This task fixes nothing: it writes down what happens.

**A stated departure from `AGENTS.md`, 3:** the pinned Claude Code download is the one real service this task reaches, as the request names it. Nothing else: no provider sign-in, no sign-in method chosen, no sign-in address opened, no model call, no credentials entered or read. Check a fetched file by `stat` and `sha256sum` alone, and list names alone under the program's directory.

Acceptance:

- `real-check.md` says each step done, what it showed and its verdict, in the form of `walk-2/real-check-login.md`, and what stays unproved. R1's verdict is on its first line, compared with the second walk's.
- A real `temper box`, non-fake, with the embedded bundle, `--config 'D/config space' --data "D/data's" --cache D/cache`, all empty and private, under a fresh empty `HOME` and `PATH=BUILD:/usr/local/bin:/usr/bin:/bin`, where `BUILD` holds the copied `temper` and no `claude`, and no `TEMPER_*` or XDG variable exported. `BROWSER=/bin/false`, `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1`.
- By the page, at 1440: Set up temper, Forgejo's Run on this machine, the sheet's Run on this machine, Use your Claude subscription. The live command element's `textContent`, unedited, run by `script -qfc` with that home and PATH: fetches the pinned build; reaches its first sign-in prompt; stopped there with Ctrl-C.
- The fetched file's size and SHA-256 equal this platform's pin in `temper/crates/temper-forge-setup/src/claude_code.rs`. `HOME` stays empty. A second run fetches nothing.
- The same command at Connect a model is byte-for-byte equal. A plain box, with no directory flags, shows `temper program-login claude-owner`. Both the long and the short command, at 1440 and 420 by `Emulation.setDeviceMetricsOverride`: every character within its panel, the document's width equal to the window's.
- Screenshots and terminal captures under `~/src/fabrig/ui-walk-3/real-login/`. No sign-in address in any of them or in `real-check.md`: strip them from what's retained. Every box and Chromium stopped; `boxes/` and `build/` removed, the fetched binary with them.
- A fault is a row `R2`, `R3`, ... in the findings table's form (`notes/common.md`), owner `fix-flow`, or `fabrig/ui` if `large`.

## Inputs

- `temper/docs/plan/ui-walk-3/inputs/request.md`, *The walk*, *R1*.
- `temper/docs/plan/ui-walk-3/notes/common.md`.
- `temper/docs/plan/ui-walk-3/walk-2/real-check-login.md`, all of it: the method; `walk-2/real-checks.md`, *Claude Code fetch*, what it looked like before the fix.
- `temper/docs/design/client/README.md`, 4.6.
- `temper/docs/plan/ui-walk-3/driver/cdp.py`, to drive the page. A throwaway script, untracked and gone before the push.

## Owns and may change

Owns `real-check.md`. Nothing else.

Options, each pre-approved: **Build:** `TEMPER_BUNDLE=embed heavy cargo build --locked -p temper`, the one departure from the machines' `TEMPER_BUNDLE=none`, as the check needs a served client; copy the binary before any box starts. **The download fails on the network:** retry twice, spaced; then the row is `unobserved` with the error, not a fault in temper, and you end. **A prompt other than sign-in comes first** (a theme question): stop at the first prompt, and say which.

## Steps

1. Build, copy, make the directories.
2. The custom box and its command; the file's check; the second run; Connect a model.
3. The plain box.
4. The render proof at both widths.
5. `real-check.md`. Commit and push.

## Report

As `notes/common.md`, Your report, and: the digest and size against the pin, the command's seconds, and each fault's id.
