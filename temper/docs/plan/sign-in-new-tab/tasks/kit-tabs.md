+++
id = "kit-tabs"
part = "web-tab"
kind = "feature"
needs = []
files = ["skein/testing/skein-browser/src/**", "skein/testing/skein-fake-person-browser/src/**", "skein/tests/browser/**"]
effort = "high"
minutes = 75
+++

## Goal

The browser kit reads a person's other tabs, and its harness awaits them, so that a story sees a tab a press opened, its address, and whether its opener was cut. Real Chromium is held to the facts this rests on, among them that its pop-up rule refuses an opening with no press.

Acceptance:

- `Request::Tabs` and `Event::Tabs` as `skein/docs/design/browser.md`, sections 5 and 6 have them, in the machine and held by a fake-peer test.
- `skein_fake_person_browser::Step::Tabs` awaits them.
- A real Chromium test in `skein-browser-world` holds: a press's tab listed with its title; `opener` true, then false once the page cuts it; and an opening with no press not listed.
- Remove `Tabs`'s **Not yet** line from `skein/docs/design/browser.status.md`, leaving "No gap." This task may make that edit, against the implementer's text.

## Inputs

- `skein/docs/design/browser.md`, sections 1 (*A test sees what a person sees*), 5, 6 and 9.
- `skein/docs/design/ui-web.md`, 14.2, *Its tabs*.
- `temper/docs/plan/sign-in-new-tab/notes/common.md`.

## Owns and may change

- Owns the files in the header.
- The `match`es on `Event` and `Step` outside the kit take their compiler-forced arms.
- If `Target.getTargets` holds no `canAccessOpener` on the machine's Chromium, or it stays true once `opener` is set to null: stop and ask, with what the reply held. No script runs in the tab without that answer (`browser.md`, section 1).
- If Chromium under the kit's flags lets an opening with no press through: stop and ask, with the version and what you saw. `--block-new-web-contents` is no answer, since it refuses a press's tab too.

## Steps

The sketch, binding in its names and shapes:

```rust
// skein-browser/src/boundary.rs
Request::Tabs { page: Token, op: Token }
Event::Tabs { op: Token, tabs: List<Tab>, more: u32 }
pub struct Tab { pub url: Box<[u8]>, pub title: Box<[u8]>, pub opener: bool }

// skein-fake-person-browser/src/harness/script.rs
Step::Tabs { tabs: Box<[TabSeen]>, within: Duration }
pub struct TabSeen {
    /// The tab's title, whole.
    pub title: Option<Box<[u8]>>,
    /// Bytes its address holds.
    pub address: Option<Box<[u8]>>,
    /// Whether it can reach its opener.
    pub opener: Option<bool>,
}
```

- The machine: as `Request::Address` is, a `Step::Tabs` over a browser-level command (`Target.getTargets`, as `Target.createTarget` goes, no session). Filter by the `Person`'s `browserContextId` and type `page`, and drop every `targetId` a `Page` holds. A `Tab`'s `url` and `title` past `Limits::text` are `Refused(Limit)`, as `Address` is. A closed page refuses, as `Address` does.
- The step: re-ask every `Limits::poll` until the list reads as `tabs` says, in order, one `TabSeen` per tab; past `within` the script fails, naming what was there. A deadline, never a sleep.
- A world fake browser (`skein/tests/browser/src/world.rs`): takes the arm, and a test of its own if the machine's other reads have one.

Tests, failing first:

1. `skein-browser/src/browser/` a `tabs_tests.rs` beside `attribute_tests.rs`, a fake peer: the reply's other person's target, its `service_worker`, and a `Page`'s own target are dropped; `canAccessOpener` absent reads false; a malformed reply is `finish_protocol`.
2. `skein/tests/browser/tests/browser_real.rs`, the shared rig: a fixed page with a button whose listener runs `const t=window.open('','_blank'); t.document.title='Opening'; window.held=t` and a second whose listener runs `window.held.opener=null`, and on load, with no press, a call to `window.open('','_blank')` whose result the page writes in a heading. Fixed pages are served under the rig's policy: put the script where the rig allows.

Run the real test through `heavy cargo nextest run --profile browser-skein -p skein-browser-world --test browser_real -E 'test(=NAME)'`, with `chromium` on the builder's `PATH` or `SKEIN_TEST_CHROMIUM`. If the builder has no Chromium, say so plainly in the report, and don't count it as run.

Finish: `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-browser -p skein-fake-person-browser -p skein-browser-world --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-browser -p skein-fake-person-browser -p skein-browser-world`. Commit and push.

## Report

As the implementer's text asks. Give Chromium's version and the reply's `canAccessOpener` you saw before and after the cut, and say what the rule did with the opening with no press.
