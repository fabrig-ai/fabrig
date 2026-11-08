+++
id = "opening-state"
part = "state"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-domain/src/forge/stock_ways.rs", "temper/crates/temper-client-domain/src/tests/forge_stock_ways.rs", "temper/crates/temper-client-domain/src/pages.rs", "temper/crates/temper-client-domain/src/pages/needs.rs"]
effort = "high"
minutes = 110
+++

## Goal

Give Ask temper's opening its page state in the client's domain: the
tab shown and the stop whose card is open, switched in place by two
page acts, and a toast naming the way and the revision when "Use this"
or "Keep it" is answered (`temper/docs/design/client/README.md`, 5.13,
*A way's acts*). The view draws none of it yet but the toast's words.

Acceptance:

- `StockWays::shown`, `open` and `sending`, `pages::Act::WayTab` and
  `WayStop`, and `needs::Toasted::Way` exist exactly as
  `notes/contracts.md`, section 1 writes them, with its rules.
- `act(index)` answers the act while one is out; `sending()` is true
  then; a second act while one is out still sends nothing.
- `temper-client-domain`'s focused tests hold, as rows of the existing
  tables in `src/tests/forge_stock_ways.rs` where one fits:
  - `shown` is the current tab on the first document (a panel with a
    current way, and one with none, "Yours" shown);
  - `WayTab` shows a tab and closes an open card; past the tabs, another
    project: unchanged;
  - `WayStop` opens, moves and closes; a bar's stop, a stop past the
    path, another project: unchanged;
  - a later document keeps the shown way at its new place, falls back to
    the current tab when that way is gone, and keeps the open stop only
    while the shown path's stops are the same;
  - the act answered `Done(SettingsChanged { revision })` says
    `Toasted::Way { name, revision, kept }` for "Use this" (`kept`
    false) and "Keep it" (`kept` true); a refusal says no toast;
  - `act` while sending answers the act, `sending()` true; answered, false.
- The view's toast words: "NAME is this project's way now, as revision
  N." (`ToastWayUsed`, 4300) and "NAME is now this project's own, as
  revision N." (`ToastWayKept`, 4301), drawn where the view draws
  `Toasted`'s other variants, with a row in the view's toast test.
- The client world's ways stories
  (`temper/tests/client/tests/suite/ways.rs`) still pass: they read
  `act`, whose meaning changes.

## Inputs

- `temper/docs/plan/look-ask-ways/notes/common.md`;
  `temper/docs/plan/look-ask-ways/notes/contracts.md`, sections 1 and 4.
- `temper/docs/design/client/README.md`, 5.13, *A way's acts* (this
  cut's revision: the toast and the page's state).
- The look request's message (`notes/common.md`): "A tab and a stop's
  card switch in place, the page's state, in no address"; "a toast names
  the revision".

## Owns and may change

- Owns: `temper-client-domain`'s `src/forge/stock_ways.rs` and its tests
  `src/tests/forge_stock_ways.rs`.
- `src/pages.rs`: the two `Act` variants and their routing to the
  `StockWays` page while `Ask { project }` is open; the toast's `say`
  where the page's `told` answers (as `pages.needs.say(...)` is called for
  a task's toast). If `told`'s `bool` can't carry the toast, give
  `StockWays` a method that hands it over once (default `fn
  toasted(&mut self) -> Option<Toasted>`), and name it in the report.
- `src/pages/needs.rs`: the `Toasted::Way` variant.
- Compiler-forced arms for the new variants in any file of
  `temper-client-domain` (`acts.rs`, `pages/chat.rs`, `pages/task.rs` and
  the like list every `Act`), and in `temper-client-view`: the toast's
  arm in `src/frame.rs` with its two words in `src/words.rs` (numbers
  4300 and 4301, their phrases beside the other toasts'), and a row in
  `src/tests/toast.rs`. Nothing else of `temper-client-view`: its
  `paths` module is `path-look`'s in this wave.
- `temper/tests/client/tests/suite/ways.rs`: only an assertion that read
  `act(..) == None` while an act was out, changed to `sending()`; name it.
- Options: if the panel's `name` for "Yours" is needed in the toast, it
  never is ("Yours" has no act); if a `Done` answer carries an outcome
  other than `SettingsChanged`, say no toast and name the case in the
  report.

## Steps

1. Read section 1 of the contracts and `stock_ways.rs` whole; read how
   `pages.rs` routes `UseWay` and `KeepWay` to it and how a task's toast
   reaches `pages.needs.say`.
2. Add the state (the shown tab's key and the open stop) and the three
   accessors; route `WayTab` and `WayStop`; keep or reset them on each
   new document as section 1 says; track `sending` apart from `act`.
3. Remember the sent way's name and whether it was a keep; on a `Done`
   answer whose outcome is `SettingsChanged`, hand over
   `Toasted::Way`. A `stale` retry keeps both.
4. Add the words and the view's arm; add the view's toast row.
5. Tests as rows, as Acceptance lists. Break a rule once by hand for each
   new row (for example, never reset `open` on `WayTab`) and see its row
   fail.
6. `heavy cargo nextest run -p temper-client-domain --lib -E
   'test(/stock_ways/)'`; then the package's focused tests;
   `heavy cargo nextest run -p temper-client-view --lib -E 'test(/toast/)'`;
   `heavy cargo nextest run -p temper-client-world --test suite -E
   'test(/ways/)'`; `heavy cargo check -p temper-client-tui --tests` and
   `heavy cargo check -p temper-client-shell-terminal --tests` for the
   new `Act` variants.
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-domain -p temper-client-view -p
   temper-client-world --all-targets -- -D warnings`. Commit with the
   trailer `Look-request: ask-temper-ways` and push.

## Report

As the implementer's text says; name each compiler-forced arm by file,
and the rows added with the rule each broke.
