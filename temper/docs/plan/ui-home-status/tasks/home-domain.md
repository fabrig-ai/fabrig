+++
id = "home-domain"
part = "home-domain"
kind = "feature"
needs = ["status-wire"]
files = [
  "temper/crates/temper-client-domain/src/forge/status.rs",
  "temper/crates/temper-client-domain/src/forge.rs",
  "temper/crates/temper-client-domain/src/pages/home.rs",
  "temper/crates/temper-client-domain/src/tests/status.rs",
  "temper/crates/temper-client-domain/src/tests/pages/home.rs",
]
effort = "high"
minutes = 75
+++

# Home watches `status` and looks, once

## Goal

The client's half, to the note: `temper-client-domain`'s Home opens the `status` watch, sends `Looked` once on entering, not while To begin shows, and gives the view `Facts.status`. It is tested against documents it is handed, so it needs no engine. `temper/docs/design/client/README.md`, 3.5, *Home on any project*, its first paragraph.

Acceptance:

- `notes/contracts.md`, *The client*, holds.
- **The watch:** `Home::new` watches the project's `status` beside `settings`; `leave` unwatches both; a watch that can't be made, is refused or ends leaves `Status::Unknown`, and nothing is opened again until the page is.
- **The look:** sent once per entry of the page, the first time the facts are delivered, the dismissals are delivered or ended, and To begin doesn't show; so a skip or a last step sends it. Not resent while out or after an answer. `Looked` goes `Owner::Pages`' key, as "Skip" does.
- **`Status::Shown`** only once the look is answered, whatever the answer, and a document is delivered: so a first paint never says the sitting before's "last looked". Refused, unknown or lost still shows the document.
- `Facts::landed_since` is `landed - seen`, saturating; `Facts::brief` holds where the brief's `landed` is above `seen`.

## Inputs

- `temper/docs/plan/ui-home-status/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5 and 4.6.
- The models: `forge/changes.rs` and `pages/changes.rs` for the watch; `pages/home.rs`'s `skip` and `forge/dismissed.rs`'s `ask` and `answer` for the request.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper-client-domain/src/pages.rs`: `Home::new`'s arguments and `pages.next`'s step past its second handle, at its one call site. `src/route.rs`: the look's call (below).
- `temper-client-domain/src/forge/dismissed.rs`: a `pub fn settled(&self) -> bool` on `Side`, true for `Rows` and `Ended`.
- `temper-client-view/src/tests/fixtures.rs`, `temper/tools/gallery/src/samples.rs`: compile-forced only; a Home with no status document stays `Status::Unknown` and draws as now, so no golden moves.

Options, each pre-approved:

- **Where the look fires.** `facts` needs the whole `Domain`, which `pages::step` hasn't, so this is a function `route.rs` calls at the end of `pages()`, `forge()` and the client's told, the three that move a fact:
  ```rust
  pub(crate) fn look(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>);
  ```
  It returns at once unless the address is `Home` and the page's look isn't sent. If one step function covers them, one call there is enough; say which.
- **`Facts` gains no `PartialEq` trouble:** `w::ProjectStatus` derives it, as the panel's does. If it doesn't, compare by fields in the tests.
- **The serial:** `home.skip` takes `&mut pages.serial`; the look does the same.

## Steps

1. **`forge/status.rs`:** per the contract, `watch` as `changes`', `ask` as `dismissed`', `answer` as `dismissed`' with `w::Outcome::Looked`.
2. **`pages/home.rs`:** `status: Handle`-like state, `look: Look` (`NotSent`, `Out(Key)`, `Answered`), `told`'s arms; `facts` fills `status`; `landed_since`, `brief`.
3. **`look`,** then its call.
4. **Tests,** `tests/status.rs` and `tests/pages/home.rs`, handed documents as `tests/dismissed.rs` is:
   `a_watch_opens_with_the_page_and_closes_with_it`; `a_look_goes_once_when_the_facts_are_delivered`; `no_look_goes_while_to_begin_shows`; `a_skip_sends_the_look`; `a_second_entry_looks_again`; `a_refused_watch_leaves_status_unknown`; `status_waits_for_the_looks_answer`, with the earlier document held back until `Told::Answered`; `a_refused_look_still_shows_the_document`; `landed_since_saturates`; `a_brief_is_shown_only_above_seen`, at equal and one above.
5. **Check,** the implementer's text, with `temper-client-view` and `temper-client-gallery` as dependents, then push.

## Report

As `notes/common.md`, Your report, with where `look` fires.
