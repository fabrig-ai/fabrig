+++
id = "catch-and-fix"
part = "browser-red"
kind = "feature"
needs = ["say-what-failed"]
files = [
  "temper/tests/client-browser/tests/browser_all/browser_sign_in.rs",
  "skein/testing/skein-browser/src/browser.rs",
  "skein/testing/skein-fake-person/src/lib.rs",
  "skein/testing/skein-fake-person-browser/src/face.rs",
  "skein/tests/browser/**",
]
effort = "high"
minutes = 150
+++

# Catch the cookie and Sign out test's two failures, and fix each at its lowest tier

## Goal

`browser_sign_in::signing_in_reloading_reconnecting_and_signing_out_keep_the_cookie_contract` fails now and then with an unexpected 503, or with `browser refused act: Protocol` pressing Sign out. Nobody knows why, and the user had trouble signing out on a real deployment today. Catch each failure with the words `say-what-failed` gave it, find its cause, fix it where it lives, and say plainly whether a person can hit it.

Acceptance:

1. Each failure's cause is named from evidence: the 503's URL and what answered it; the `Protocol`'s CDP method and error and what the page was doing. A cause you read in the code but never caught is said to be that, with the reproduction below standing in for the catch.
2. Each fix is at the lowest tier that shows the fault, with a test there that fails without the fix. A product fault gets a product fix. A harness fault gets a harness fix and a statement of why the product is sound.
3. After the fixes, the test passes 40 runs alone and 8 runs of the whole tier in a row, with the tier within its budget.
4. Every cookie, reload, reconnect and sign-out assertion stands (`notes/common.md`).
5. The report's first section says plainly, in these words: whether a person can hit each failure, what they would see, and whether signing out works on a real browser.

## Inputs

- `temper/docs/plan/ui-browser-red/notes/common.md`, and `temper/docs/plan/ui-browser-red/inputs/request.md`.
- `temper/docs/design/client/browser.md`, 4.6 (the test's story).
- `jig/docs/design/protocol/client.md`, 17.4 (the 503: a call under the prefix with no room; the client "sends it again after its backoffs").
- `jig/docs/design/protocol/browser.md`, 11 (signing out).
- `skein/docs/design/browser.md`, 1 ("the test can wait and try again"), 5 and 6 (press).

## What's known, and what's only a lead

Known:

- After-landing's failure took 1.658 s, far short of a whole run (3 to 5 s), on the page titled "Hello · temper". So the 503 comes around the first chat's start or the first reload, before the five rapid reloads. Its "step 0" was the harness's placeholder, which `say-what-failed` replaces. The log, if your builder reaches it: `~/src/fabrig/fab/after-landing/47457baa780f-20261105T155451083307Z/browser-temper.log` on hetzner8.
- `ui-welcome/red-36-1` and `red-36-2` saw `Protocol` alone, at Sign out, with the button visible in the snapshot; `red-36-3` saw none in its runs. Their reports are under `~/src/fabrig/fab/runs/ui-welcome/`.

Leads from reading the code, none proven. Test each against what you catch, and drop any the evidence doesn't bear out:

- **The 503 may be a full room.** `temper/crates/temper-service/src/clients.rs`, `server_requests`: a call the mapping claims with `room_for` 0 goes to `serve_call(call, &head, claimed)`, whose `Mount::refusal` answers a bare 503. The browser tier's `limits` name the `small` profile (`temper/tests/client-browser/src/configuration.rs`), whose `clients_http.calls` is 4, where `temper init`'s template names `standard`, 16. A call holds its room until its connection is let go, so a reloading page's calls may still count against the new page's. Check which limit governs the tier and what the peak is. Other 503s to rule out by the URL: `temper-service/src/setup/routes.rs`, `clients/provider.rs`, `clients/google.rs`, `clients/github.rs`.
- **What a busy answer does to a person.** Chromium logs any failed load, even one the client handles. The client is designed to send it again, but `jig/crates/jig-client/src/client.rs` has `sign_out_failed` and `sign_out_lost`: find out whether a busy answer to `/jig/v1/sign-out` is resent, or lost in front of the person. That's the likeliest link to the user's trouble, and worth settling even if the test never shows it.
- **`Protocol` may be a node a render removed.** `skein/testing/skein-browser/src/browser.rs`, `command_error`: a step's error containing `No node found` is `Gone`, and any other is `Protocol`. `node_missing`, in the same file, knows more phrases, but this path doesn't use it. A press's steps can meet a node gone mid-press.
- **Reclassifying won't turn it green by itself.** `skein-fake-person/src/lib.rs`, `Step::Press`, finds and acts once, and `face.rs` makes any refused act the step's failure, though `skein/docs/design/browser.md`, 1, says the test can wait and try again.
- **A click racing a render may be a product fault.** `skein/crates/skein-ui-web/src/page/web.rs` has a `replace_child` for a group. If a render replaces the Sign out button's element between a person's mouse down and up, the browser sends no click on it and the person's press is lost with nothing shown. Find out whether the Accounts page does this, and when.

## Owns and may change

Owns the `files` above. Beyond them, follow the cause, within these defaults:

- **The fix's own crates:** `jig/crates/jig-client/**`, `jig/crates/jig-protocol-engine-clients-http/**`, `temper/crates/temper-service/src/clients.rs`, `temper/crates/temper-service/src/clients/**`, `temper/crates/temper-client-view/src/pages/settings.rs`, `skein/crates/skein-ui-web/**`, their tests and their worlds: change them if the cause lies there, and name it in the report. `temper-client-view/src/pages/settings.rs` and `skein-ui-web/src/page.rs` are in other plans' hands: keep edits there small, and regenerate goldens on a catch-up.
- **A design line** the fix makes untrue: edit it, in `jig/docs/design/protocol/client.md`, `jig/docs/design/protocol/browser.md`, `skein/docs/design/browser.md` or `temper/docs/design/client/browser.md`, with its status, and name it in the report. This departs from the implementer's text, which closes `docs/design/`.

Options, each decided:

- **If the 503 is a designed busy answer the client resends and a person can't see,** first ask whether a page on its own should fill the room. If one person's ordinary page or reload does, in a profile a deployment can run, that's a product fault: fix it (a dead page's calls let go sooner, or fewer calls at once). Only if it isn't: add the narrowest words to this test's `expected`, with a comment naming the URL and the reason, and pin the client's resend at the lowest tier if nothing does. Adding `503` to `expected` before you've shown this is hiding it.
- **If `Protocol` is a node gone mid-press:** map Chromium's words for it to `Gone` in `command_error`, and have a press whose node went start again from its find until its step's deadline, as a person pressing again would. That retry lives in `skein-fake-person` or `face.rs`, and every browser tier benefits. It doesn't change the test, which isn't "retried": the test's assertions and timeouts stand. If that would need changing another tree's story, do it and say so.
- **If a render does replace the button under a press,** the product fix comes first (keep the element across the render, at `skein-ui-web`'s tier), and the driver's retry only if the race remains.
- **The tier's `small` profile stays.** Don't raise its limits to hide a failure. If the evidence says the tier's profile is itself wrong, say so in the report and leave it.

## Steps

### Catch

This task departs from `AGENTS.md`, 5, and the lab's "no unchanged rerun": repeating an unchanged test is how an intermittent failure gets caught. Read each failure's log before going on, and keep each failed run's log.

1. First confirm the words: a failed run's report names a URL. If `Log.entryAdded` doesn't carry one for this load, fix that in `skein-browser` and say so.
2. A throwaway loop, untracked, up to 40 runs alone, then up to 8 runs of the whole tier (`notes/common.md`), one heavy command at a time, stopping at each new failure to read it.
3. If a failure hasn't shown: run alone under CPU load on the builder (a throwaway pin of the test to one core, or busy loops beside it), since after-landing's machine was loaded. Change no timeout to do it.
4. If it still hasn't shown, don't wait on luck: provoke the race deterministically (below).

Temporary traces in the product are throwaway: gone before the push.

### Reproduce at the lowest tier, then fix

- A room fault: `jig/crates/jig-protocol-engine-clients-http`'s tests, or `temper/crates/temper-service/src/tests/`, one call past the room.
- A client's resend or a lost sign-out: `jig/crates/jig-client`'s tests, a 503 answered to the sign-out request.
- A driver fault: `skein/tests/browser/src/tests.rs`, the replayed CDP frames, with Chromium's exact error words as you caught them.
- A render replacing a pressed element: a story in `skein/tests/browser/tests/browser_real.rs` or `skein/tests/ui-web`, a page whose render replaces a button between find and press, within the skein tier's budget.

Run the readers' tests where a shared item changes meaning: a change to `command_error` or `Step::Press` runs `-p skein-browser-world -p skein-fake-person-browser -p skein-fake-person`, and once each `tools/tiers/run skein browser` and `tools/tiers/run jig browser` as well as temper's, since their stories press through the same driver.

### Prove and finish

Acceptance 3's runs, on the fixed code, with their log paths. Then `cargo fmt`; each touched tree's `old_forms.py`; `heavy cargo clippy` naming every package changed, `--all-targets -- -D warnings`; the focused tests of those packages and their direct dependents; `python3 -I -B tools/citations/citations.py --base origin/main`.

Commit one cause per commit, each saying why. Push.

## Report

Per the implementer's text. Its first section answers acceptance 5, plainly, for someone who hasn't read this plan. Then: each failure's caught words, with its log path and machine; the cause; the fix and its tier; the test, what you broke and what it said; the runs and how many failed before and after; the tier's CPU and wall against its budget; and which leads above the evidence dropped. dev.fabrig.ai's build isn't known and it isn't probed: say what the user would check there, if your finding gives them something.
