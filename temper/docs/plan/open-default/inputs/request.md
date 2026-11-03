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

## Addendum (the laptop session, 18:15Z): making a project is the fault, and it's the priority

The same stall happens on "Try the playground". On a fresh fake box built from 3a2ef76a56, whose changes on top of `3803f68c40` are view-only:
- the box's stderr reads "18:13:59Z Playground: starting / started";
- the store's log was last written at 18:13:59Z, with "Demo" in it twice, as "Default" was;
- then nothing: temper serve is up and idle (`io_cqring_wait`);
- `/chats` shows "No project yet", with no "Set up temper →" in the banner and no playground note in the side pane.

So the fault is in making a project, not in the welcome: serve stops answering once a project is made, by the welcome's `MakeProject` or by the playground's start. It also blocks the user's playground look work, so it is this plan's priority.

**It looks like a regression since this morning:** `again welcome-playground` recorded Demo as project 1 successfully at `f6a5d32b60`. Between `f6a5d32b60` and `3803f68c40` there are about 360 commits. The likeliest are the jig store commits that touch project policies:
- `6ed91f5429`, replace project policies through settings decisions;
- `c743971fbe`, store size expectations for retained project policy values;
- `239f6e6bf8` and `655de5b63c`, store codecs for replacements and restored settings.

**Bisect between `f6a5d32b60` and `3803f68c40`** with `cargo run -p temper-client-journeys -- again welcome-playground`, which walks the playground on a fresh fake box in headless Chromium. Through `heavy`, one command at a time. Then reproduce the fault in the lowest tier: a jig world or engine test if it is jig's, which these commits suggest. The box test above still follows.

## Second addendum (the laptop session, about 18:25Z): the browser's link drops, so check this first

After the playground's start, the page's banner reads "Offline" and "Behind" (jig's "Behind: waiting for current facts"). So the browser's link to serve drops right after a project is made. serve's idleness is just no client being connected, and the earlier "serve stops answering" is likely this.

**The hypothesis to check first:** the new project's records, its retained policy values, exceed a limit of the **wasm32 client**, which drops the link. The candidate commits:
- the records: `6ed91f5429`, `c743971fbe`, `3d524c8f4b` and `655de5b63c`;
- several of today's jig commits scoped size expectations to non-wasm targets: `b4e12f9304`, `f349e1fb26`, `a9822f8403` and `6bf9fadd50`.

The native client tests would not see that. The browser tier (`tests/client-browser`) or `again welcome-playground` should. So:
- **The failing test** is in the browser tier, or a wasm32-built client against a real serve: make a project, and the link stays up, watches delivered.
- **The fix** is in the lowest tier that holds the limit. If it is a bound that differs between wasm32 and native, the record's worst-case size must hold on both targets.
- **Bisect** only if this hypothesis fails.
