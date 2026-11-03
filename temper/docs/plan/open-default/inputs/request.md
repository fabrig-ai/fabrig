# open-default: temper serve stops answering after the welcome's "Open Default"

The user found this walking the welcome on their laptop (2026-11-03). Reproduce it in a box-tier test, find the fault, fix it in the lowest tier that shows it (jig or temper serve; `AGENTS.md` 6), and land it.

## The trunk

This trunk starts from `3a2ef76a56`, the tip of `ready/look-fixes/welcome-model`. That branch stacks `ready/look-fixes/welcome-forge` and `ready/look-fixes/welcome-forge-at-once`. All three are in main's queue and not landed yet; the bug was seen on them, and they will reach this trunk through main by catch-up.

## The report, as walked

On a fresh `temper box --fake-backends`:
1. Welcome → Set up temper → "Run on this machine" (sends `SetUpOwnedAtOnce`, work/hello).
2. Model → "Use an OpenAI API key" with the box's dummy key from `DATA/box/fake-backends/data/llm.key` → Add key → Continue (to `/welcome?at=ready`).
3. Wait for Forge and Model to read done, then press "Open Default".

What happened:
- **The page stays at "Making Default…".** `pages.rs` `make_default` sent jig's `MakeProject { name: "Default", policy: 1 }` under the named key `TOOLS/"welcome-default-1"`. `default_told` clears that on any answer, so no answer came.
- **The store looks to have made the project.** Its log was last written at 17:53:35Z, seconds after the press, and its strings show "Default" there. Nothing was written afterwards.
- **serve stayed up but went quiet:** one process with no restart, idle in `io_cqring_wait`, 2 s of CPU over 15 minutes.
- **After a reload,** `/welcome?at=ready` shows "Open Default" disabled, and the Forges and Accounts pages are empty, although the store still holds the forge and the key. `/chats` renders. So serve answers HTTP but no longer delivers watches or answers.
- **serve.log, whole:** listening on clients `127.0.0.1:8367` and webhooks `:8371` at 17:49:10Z. Then, at 17:50:19Z: `temper serve: forge 1: Not connected: prepare it with temper forge prepare`. That came while the owned set-up ran; the UI later showed the forge connected, so it may be unrelated.
- **The box's stderr:** Forgejo started 17:50:19 to 17:50:22Z, then nothing more.
- **Not the dummy key:** `MakeProject` does not touch the model, and the key was kept at 17:53:07Z.

## What is missing

No test walks "Open Default" against a real temper box. `tests/client`'s welcome suites drive the fake engine only, which is why everything was green.

## The task

1. **A box-tier test:** welcome Ready → Open Default → the Default project's chats page, with watches still delivered after a reload. It must fail before the fix.
2. **The fault:** found and fixed in the lowest tier that shows it. If it's jig's (the engine's or the clients' component's answer to `MakeProject` under a named key, or a watch stalling after it), reproduce it in jig's world as well.
3. **The narrowest tests** after each change, then the box test green.

No design, unless the fault shows a design section wrong; then the section and its status say what changed. A budget or timeout is the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## After it lands

Nothing more here. The user walks the welcome-set-up journey anew in a browser once this and the three look-fixes branches are on main.
