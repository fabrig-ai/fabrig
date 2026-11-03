+++
id = "server-help-link"
part = "repo-links"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/forge/connect.rs",
  "temper/crates/temper-client-view/src/tests/connect_forge.rs",
  "temper/docs/design/client/mockups/pages.js",
]
effort = "medium"
minutes = 20
+++

# The deployment's help link points at the fabrig repository

## Goal

The old temper repository on GitHub is about to be deleted, and two places still link to its
copy of "Run temper on a server": `DEPLOYMENT` in `temper-client-view` (the "Show how" link
of Forgejo on this machine's Details) and one design mockup. Point both at the same page in
the fabrig repository, and pin the new address in a view test.

Acceptance:

- `DEPLOYMENT` (`temper/crates/temper-client-view/src/forge/connect.rs`, ~line 25) is the new
  address of `notes/common.md`.
- `temper/docs/design/client/mockups/pages.js`, ~line 1648: the `href` of "Or connect your
  deployment" is the new address; nothing else on that line or in that file changes.
- A new test in `temper/crates/temper-client-view/src/tests/connect_forge.rs` (below) passes and
  pins the new address on the "Show how" link.
- `git grep -nP 'fabrig-ai/temper(?![\w-])' -- ':!temper/docs/plan/'` prints nothing.
- `temper-client-view`'s lib tests pass.

## Inputs

- `temper/docs/plan/repo-links/notes/common.md`.
- `temper/docs/design/client/forge.md`, 5.4 (*The deployment's page*): Forgejo on this
  machine's Details, where the view draws "Show how".
- `temper/docs/design/client/forge.status.md`, its 5.4 **Not yet** entry naming "Show how"
  (read only: it stays as it is).
- `temper/crates/temper-client-view/src/tests/connect_forge.rs`:
  `a_whole_address_link_keeps_its_external_link_meanwhile` (~line 974), for the shape of an
  assertion on a link's `external`.

## Owns and may change

- **`forge/connect.rs`:** `DEPLOYMENT`'s value, and its doc comment's path
  `(docs/help/server.md)` becomes `(temper/docs/help/server.md)`, the path from the
  repository root. Nothing else in the file changes.
- **`tests/connect_forge.rs`:** one new test, placed after
  `forgejo_here_without_the_box_says_to_start_temper_with_the_box`. Sketch:

  ```rust
  #[test]
  fn forgejo_on_this_machine_shows_how_with_the_server_help_in_fabrig() {
      let root = f::forge_root(Address::ConnectForge { forge: Some(Choice::Owned) }, List::with_capacity(0));
      let view = f::page_view(&root);
      let link = node(view.tree(), Role::Link, b"Show how");
      assert_eq!(
          view.tree().nodes().iter().find(|n| n.id == link).unwrap().external.as_deref(),
          Some(b"https://github.com/fabrig-ai/fabrig/blob/main/temper/docs/help/server.md".as_slice())
      );
  }
  ```

  `connect::step` (`temper-client-domain`'s `forge/connect.rs`) shows `Details(Owned)` for that
  address whatever the rows, so the empty list is enough. If `f::page_view`'s tree holds no
  `external` on the link, use `f::browser_page_view`, as the test named in `## Inputs` does,
  and say so in the report.
- **`mockups/pages.js`:** the one URL on line ~1648. The line is long: replace the address
  alone (for example `sed -i 's#OLD#NEW#'` with the two addresses of `notes/common.md`), then
  check with `git diff --word-diff` that only the address changed.
- **If the search finds another holder** of `fabrig-ai/temper` (not `temper-probe`) outside
  `temper/docs/plan/`: if it is a link to a page of the old repository in code, a test, a golden
  or a fixture of temper, point it at the same page under `https://github.com/fabrig-ai/fabrig/blob/main/temper/`
  and name it in the report; anything else (a draft, a design document, another tree) stays, and
  goes under `## Left`.

## Steps

1. Edit `DEPLOYMENT` and its comment; add the test.
2. Narrow: `heavy cargo nextest run -p temper-client-view --lib -E
   'test(=tests::connect_forge::forgejo_on_this_machine_shows_how_with_the_server_help_in_fabrig)'`
   (check the count: one test). Then the package's lib tests: `heavy cargo nextest run -p
   temper-client-view --lib`. No other package reads `DEPLOYMENT` (it is crate-private), so no
   dependent's tests run: this departs from the implementer's text ("their direct
   dependents"), since no public item changes.
3. The mockup line, checked by `git diff --word-diff`.
4. The search of `## Goal`, and `git grep -n 'fabrig-ai/temper' -- ':!temper/docs/plan/'`, which
   lists only `fabrig-ai/temper-probe` lines (`notes/common.md`).
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py` (no count may rise); `heavy cargo
   clippy -p temper-client-view --all-targets -- -D warnings`; commit, then `python3 -I -B
   tools/citations/citations.py --base origin/main` shows nothing new broken.
6. One commit for the link, its test and the mockup, saying why (the old repository is being
   deleted). Push your branch.

## Report

As the implementer's text says, and: the output of both searches of step 4; the count of each
run of step 2; whether the test needed `browser_page_view`.
