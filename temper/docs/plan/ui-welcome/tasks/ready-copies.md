+++
id = "ready-copies"
part = "ready-copies"
kind = "feature"
needs = ["sheet-choices"]
files = [
  "temper/crates/temper-client-domain/src/forge/making.rs",
  "temper/crates/temper-client-domain/src/forge.rs",
  "temper/crates/temper-client-domain/src/forge/family.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/welcome.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/tests/pages/ready_adoption.rs",
  "temper/crates/temper-client-domain/src/tests/pages/welcome.rs",
  "temper/crates/temper-client-view/src/pages/welcome.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
  "temper/tests/client/tests/suite/first_repository.rs",
  "temper/tools/gallery/**",
  "temper/docs/design/client/journeys/**",
]
effort = "high"
minutes = 120
+++

# Ready copies them in

## Goal

With copies asked at set up, Ready offers the first copy's name, says what the project starts with, and its act makes the
project and asks every copy at once. Request, section 7, *Ready* and *The act*.

Acceptance:

- **The offered name** from the first copy, capitalised: "Widget" for `github.com/acme/widget`.
- **The lead:** `Everything is connected. Create your first project, with acme/widget in it, and start working.`, or `with
  2 repositories in it`. The forge's line: `connected as temper` alone.
- **The act:** `MakeProject`, then one `MakeRepository` per address in order, its org and name the source's,
  `into { project, home: first }`, each under a key of its own, `welcome-copy-N`, all sent at once, not waiting on one
  another; the project opens once made. A taken-name project takes them the same way; if it already has a home, none is
  asked as the home.
- Nothing is adopted beside them: `adopt` finds no first repository, and returns.

## Inputs

- `temper/docs/plan/ui-welcome/notes/common.md`, `notes/contracts.md`, *The copies, on the client*.
- `temper/docs/plan/ui-welcome/inputs/request.md`, section 7.
- `temper/docs/design/client/README.md`, 3.5, *Ready*, *Its copies*; 4.6, `MakeRepository`.
- `temper/docs/design/draft/repository-copies.md`, 5.3 and 7.

## Owns and may change

Owns the files in `files`.

May also change: `temper/docs/design/client/README.status.md`: delete what's left of "3.5, *Forgejo's sheet*, *Its copies*".
The `forges` watch's `welcome_first` reader; a word's hole.

Options, pre-approved:

- A copy's refusal (`MakeRefusal`) or an unknown answer leaves the project made and Home's step offering it: nothing is said
  on Ready, as the adoption's isn't. `pages.rs` keeps no more than it needs.
- `target` that can't read an address kept in the row is left out of what's asked, and said by a test: admission held it,
  so it can't happen short of a store changed by hand.

Not this task's: Home's lines (`home-copies`).

## Steps

1. **`forge/making.rs`:** `Copies`, `copies`, `target`, by `notes/contracts.md`. **`forge.rs`:** `make_repository(&w::MakeRepository, limits) ->
   Option<jig_client::Ask>`, as `ask` wraps `Adopt`; `copy_key(number: u64) -> RequestKey`, `welcome-copy-N`, as
   `first_repository_key` is built.
2. **`pages/welcome.rs`:** `offered(first, copies)`, the first copy's name in capital when there's no first repository;
   `Pages.welcome_copies`, filled in `route.rs`'s `playground_facts` beside `welcome_first`.
3. **`pages.rs`:** where `make_default`'s answer calls `adopt` (`Made`, and the taken name), it also calls
   `copy_all(pages, env, project, out)`: for each of `welcome_copies`, `making::ask` with `Some(address)` and `Some(MakeInto
   { project, home })`, `home` true for the first unless `forge::listed::held` says the project has one, then
   `Request::Client(Event::Send { key, name: Some(copy_key(n)), request })`. A pages key per send: nothing here reads their answers.
4. **The view,** `pages/welcome.rs`: `ready`'s lead; `WelcomeReadyCopiesLead`, `Everything is connected. Create your first
   project, with {number} repositories in it, and start working.`; the name, `full_name` of `target`.
5. **Tests.** `temper-client-domain`'s `tests/pages/ready_adoption.rs`: the act's requests in order, with `into` and keys, one
   address and three; the taken name, with and without a home; `offered`. `temper/tests/client/tests/suite/first_repository.rs`: through a
   front and engine to `Making` rows. `temper-client-view`'s `tests/welcome.rs`: the leads and the forge's line, browser and
   terminal. Gallery: `welcome-ready-copies`, in `names.txt`, `samples.rs`, `tests.rs` and `browser.md`'s 7.2 table.
6. **Check,** `notes/common.md`; render proof beside the mock's `welcome.html?at=ready` with copies; commit and push.

## Report

The requests in order, as the world saw them; the builder and directory.
