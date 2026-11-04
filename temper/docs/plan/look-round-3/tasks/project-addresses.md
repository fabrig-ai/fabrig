+++
id = "project-addresses"
part = "project-addresses"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-client-domain/src/address.rs",
  "temper/crates/temper-client-domain/src/tests/address.rs",
  "temper/crates/temper-client-domain/src/tests/pages/project_pages.rs",
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
]
effort = "high"
minutes = 75
+++

# Home's and Settings' addresses, opening their project

## Goal

Add `Address::Home { project }` at `/projects/<project>` and `Address::Settings { project }` at
`/projects/<project>/settings`: printed, parsed, opening the project they name, each with a plain
page. Nothing links to them yet, so no other page changes. The later tasks build Ready's landing,
the Settings page, the rail and Home's column on this.

Acceptance:

- `print` and `parse` hold both, round trip; `/projects/01`, `/projects/1/other` and
  `/projects/99999999999` are `Place::Missing`; `/projects/1/adopt` is `Adopt` as before.
- Going to either, or reloading at it before `me` arrives, makes the address's project the open one
  once `me` lists it, and leaves the open project alone while `me` doesn't.
- `Act::Project { project }` pressed on either shows that project's same page; elsewhere it behaves
  as today.
- The switcher's rename form survives on them as on the project pages (`Switcher::went`).
- The page: on `Home`, `Role::Main` with `look::MAIN`, its level-1 heading the project's name; with
  a project `me` doesn't list once `me` is delivered, "Project N is not one of yours." and a link
  "Chats"; on `Settings`, the same with "Settings for PROJECT". `build` draws them in the terminal
  too.
- No journey's page changes; `heavy cargo run -p temper-client-journeys` reprints none.
- Every commit carries the request's trailer (`notes/common.md`).

## Inputs

- `temper/docs/plan/look-round-3/notes/common.md` and `notes/contracts.md` (Addresses).
- `temper/docs/design/client/browser.md`, 2.3.
- `temper/docs/design/client/README.md`, 3.5's page table, *Home* and *Settings*.

## Owns and may change

Owns, as `files` lists them, the two new test files with their `mod` lines.

May also change, compiler-forced arms and these alone:

- `temper/crates/temper-client-domain/src/pages.rs`: the page's `went` handling, `choose_project`'s
  callers, `Act::Project`'s arm, `project_changed`.
- `temper/crates/temper-client-domain/src/pages/switcher.rs`: `went`, with the project pages.
- `temper/crates/temper-client-domain/src/{route.rs,departure.rs,acts.rs}`,
  `pages/connect_model.rs`, `pages/chat.rs`: arms.
- `temper/crates/temper-client-view/src/{root.rs,frame.rs,choice.rs,pages.rs,words.rs}`: arms, the
  page's dispatch, its two new words, `pub mod home`.
- `temper/crates/temper-client-shell-terminal/**`, `temper/crates/temper-client-shell-web/**`,
  `temper/tests/**`, `temper/tools/**`: arms.
- `temper/docs/design/client/README.status.md`: its Home and Settings entries stay; this task opens
  neither.

Options, each pre-approved:

- `frame::navigation`'s and `banner`'s arms: neither new address is current on any banner entry
  (`false`). If `crate::choice::banner` must decide, the new addresses draw it as `Inbox` does.
- If a `/projects/<n>` parse arm would shadow `Adopt`'s, order the arms so that it doesn't; `Adopt`
  keeps `project_decimal`.

## Steps

1. **`address.rs`,** as `notes/contracts.md`: `print` as `Adopt`'s; `parse` arms
   `(b"projects", _, b"")` and `(b"projects", _, b"settings")`, each through `project_decimal`. Its
   doc comment lists the new paths.
2. **`pages.rs`:** a small `fn address_project(address: &Address) -> Option<u32>`, `Some` on `Home`
   and `Settings`. Where the page is gone to, and in `project_changed`, after its own rule:

   ```rust
   if let Some(project) = address_project(&pages.address) {
       choose_project(pages, client, project);
   }
   ```

   `Act::Project`'s arm: on `Home` or `Settings`, `choose_project` then `Request::Show` of that
   project's same page.
3. **The view,** `pages/home.rs`:

   ```rust
   /// A project's first page, and its settings (temper/docs/design/client/README.md, 3.5).
   pub(crate) fn main(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32);
   pub(crate) fn settings(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32);
   ```

   `root.rs`'s `build` dispatches them, after the chats' arm, in the same shape. The heading's name
   comes from `crate::pages::project_name`.
4. **Tests.** `temper-client-domain`, `tests/address.rs`: the round trip and the refused paths.
   `tests/pages/project_pages.rs`: the address before `me`, `me` arriving, a project `me` doesn't
   list, `Act::Project` on `Home`, on `Settings` and on `Chats`, and a rename form kept. In
   `temper-client-view`, `tests/home.rs`: the two pages and the not-yours words, in both shells.
5. **Check,** `notes/common.md`, then `heavy cargo nextest run -p temper-client-domain -p
   temper-client-view -p temper-client-shell-terminal -p temper-client-world -p
   temper-client-gallery -p temper-client-journeys`, clippy on the same, then push.

## Report

As `notes/common.md`, Your report.
