+++
id = "repo-link"
part = "link"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/forge/repository.rs",
  "temper/crates/temper-client-view/src/forge_sign_in.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/forge/forges.rs",
  "temper/crates/temper-client-view/src/forge/connect.rs",
  "temper/crates/temper-client-view/src/pages/settings.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/rail.rs",
  "temper/crates/temper-client-view/src/tests/repository_link.rs",
  "temper/crates/temper-client-domain/src/forge/listed.rs",
  "temper/crates/temper-client-domain/src/tests/listed.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-web/tests/suite/switcher.rs",
  "temper/tools/gallery/**",
]
effort = "high"
minutes = 90
+++

# A project's repository is a link to its page on the forge

## Goal

In the browser, a project's repository is a link in a new tab to its page on its forge, wherever it is named with its forge known. The switcher's row becomes two siblings, so that a link never sits in a button: the project's name, a button that still switches project, and the small words, which hold the link. It needs nothing of part 2: on a Forgejo whose page facts offer sign-in the link goes through temper already; elsewhere it is `WEB/OWNER/NAME`, or plain.

Acceptance:

1. The address's building is one function, `forge::repository::address`, and holds each rule of *A repository's link* in its tests: GitHub or another forge with `web`, with and without a trailing `/`; a Forgejo in the page facts, by `redirect_to`; no `web` and no facts is `None`; an unsafe `web` is `None`, never the builder's assert; OWNER and NAME encoded, in the path once and in `redirect_to` twice, such that `a/b`, `a?b`, `a b`, `%2F` and `..` end neither a path segment nor the query.
2. In the browser the switcher's row is a `PROJECTS_ROW` slot holding a `Role::Button` named by the project, bound to `Act::Project`, and a `PROJECTS_REPO` text. With an address the text holds a link named and reading `OWNER/NAME ↗`, its `external` the address, and `+N` after it, outside the link. With none, or for the playground, or "no repository", plain as today. No `Role::Link` is a descendant of a `Role::Button` anywhere in the tree (a test walks the tree of a page with links).
3. Pressing the name still gives `Act::Project`. The terminal's row is byte for byte as today, and the terminal draws no link anywhere in this task's three other places.
4. The same link in: Settings' Repositories (`pages/settings.rs`, `repository_row`), the Forges row's body under "Repositories" (`forge/forges.rs`), the welcome's ticked Forge line (`forge/connect.rs`, `forge_status`: "connected as LOGIN · ", the link, " made"). The Forges summary stays words; the line's `b.name` stays the whole line's words. The playground stays plain in each.
5. The look at 1440 and at the phone's width: same place, size and colour as today, underlined on hover alone, a long name still cut with an ellipsis before `+N`, never the count; the row's hover band and the pressed weight as today; the whole row's left part (the button) still presses. Screenshots of the open menu at 1440 and 420 (`notes/common.md`), and say in the report what 420 draws: at 760 px and under the top bar's switcher has no repository words today, and that stays.
6. The view's node limit prices the row (`limits.rs`, "Switcher row") and its comment says what it now holds; the limit tests pass. Gallery prints and goldens that this makes stale are regenerated.

## Inputs

- `temper/docs/design/client/forge.md`, 5.4, *A repository's link*, and its "Open Forgejo ↗".
- `temper/docs/design/client/README.md`, 5.3, *Projects in the rail*; 3.5, the Forge step's `Forge` region and *Repositories*.
- `temper/docs/design/protocol/sign-in.md`, 9.1, 9.3 (the page facts).
- `temper/docs/plan/ui-repo-link/notes/common.md`.

## Owns and may change

- Owns: this spec's `files`.
- `Held` readers: `frame.rs` and `temper-client-domain/src/tests/listed.rs` alone at the time of the cut.
- Readers of `frame::switcher` and `frame::rail`, `pages::settings::repositories`, `forge::connect::forge_status` and `forge::forges::personal`: `temper-client-view/src/root.rs`, `frame.rs` and their tests may gain the `sign_in: &crate::forge_sign_in::Forges` argument, call sites alone.
- Tests that read the row's old shape: `temper-client-view/src/tests/rail.rs`, `tests/frame.rs`, `tests/forges.rs`, `tests/connect_forge.rs`, `tests/settings*.rs`, `temper/tests/client/**`, `temper/tests/client-browser/**`, `temper-client-shell-web/tests/suite/switcher.rs`: change what the new row changes, nothing else. The `tests` `mod` line in `temper-client-view/src/tests.rs` (or its equivalent) for the new file.
- Options, each pre-approved:
  - `forge::connect::github_repository_address` becomes `repository::web_address`, and `admin_ready`'s link keeps its address through it. Default: do this; it is the one encoder.
  - If a test walks the switcher by `Role::Button` under `PROJECTS_MENU` and now finds none directly, go through `PROJECTS_ROW`; if the banner's rows (`Place::Banner`) are read by the same helper, they keep their direct buttons, for the banner draws no repository.
  - If the row's hover band can't be kept on a slot in the look of today's button with its flex children, keep the band on the row with `:hover` and the name's button filling what the small words leave. Say what you chose.
  - `rel` is skein's mapping's, `noopener noreferrer`, which holds the request's `noopener`: no change in skein.
  - If `first`'s repository (`row.first`) on the Forge line has no address, it reads as today, one run.

## Steps

**Failing tests first.** Sketches, which bind the outer layer:

```rust
// temper-client-domain/src/forge/listed.rs: the forge of the repository named; None for the playground's.
pub enum Held<'a> {
    Unknown,
    None,
    One { forge: Option<&'a Row>, owner: &'a [u8], name: &'a [u8] },
    Several { forge: Option<&'a Row>, owner: &'a [u8], name: &'a [u8], more: u32 },
}

// temper-client-view/src/forge_sign_in.rs
impl Forge {
    /// ROOT/user/oauth2/temper?redirect_to=%2FOWNER%2FNAME (client/forge.md, 5.4), or None past the URL's bound.
    pub fn open_repository(&self, owner: &[u8], name: &[u8]) -> Option<Box<[u8]>>;
}

// temper-client-view/src/forge/repository.rs, new
//! A repository's link to its page on its forge (temper/docs/design/client/forge.md, 5.4).
pub(crate) fn address(row: &Row, sign_in: &Forges, owner: &[u8], name: &[u8]) -> Option<Box<[u8]>>;
/// Opens no node of its own: the link, or the words, in the node the caller holds open.
pub(crate) fn name(b: &mut Builder, w: &Words, row: Option<&Row>, sign_in: &Forges, owner: &[u8], name: &[u8]);

// temper-client-view/src/look.rs: PROJECTS_ROW, a free token, "A switcher row: the name and its small words."
// temper-client-view/src/words.rs: Word::RepositoryLink => (&[b"", b"/", b" \xe2\x86\x97"], &[Name, Name]), a free id near 3100.
```

Rules of `address`: `row.kind == Kind::Forgejo` and `sign_in.number(row.number)` → `open_repository`; otherwise `row.web` → `WEB/OWNER/NAME`; otherwise `None`. Any result failing `skein_ui::markdown::safe_url` is `None`. Nothing reads `row.connection`: a repository named is a repository on the forge.

CSS, under `/* ==== ui: switcher ==== */`, with the old `> button > .temper-projects-repo` rules removed where they stand (`temper.css`, about 1092-1100): the menu's `button` rules that draw a row now draw `.temper-projects-row` and its button; `.temper-projects-repo a { color: inherit; text-decoration: none; }` and `:hover { text-decoration: underline; }`. A rule in `tests/suite/switcher.rs` in the style already there holds `underline` on hover alone, and `flex: none` on the count.

Tests in `temper-client-view/src/tests/repository_link.rs` and `tests/rail.rs`, in the worlds already there (`f::forge_root`, `project_rows`, `crate::forge_sign_in::Forges::read`, `crate::href`):

- `an_address_for_each_kind_of_forge` (table of acceptance 1);
- `a_name_cannot_leave_its_segment_or_the_query`;
- `the_row_is_a_button_and_a_link_beside_it`, then pressing each;
- `several_repositories_keep_the_count_outside_the_link`;
- `no_link_sits_inside_a_button` over Home, Settings' Repositories, Forges and the welcome;
- `the_playground_and_no_repository_stay_plain`;
- `the_terminal_row_is_unchanged`, reading the print as `switcher_rows_keep_repository_words_and_counts_in_both_shells` does.

Then the narrow tests, `cargo fmt`, the gate's old forms for temper, `heavy cargo clippy -p temper-client-view -p temper-client-domain -p temper-client-shell-web --all-targets -- -D warnings`, `heavy cargo nextest run -p temper-client-view -p temper-client-domain -p temper-client-shell-web`, and `heavy cargo check --tests` naming their direct dependents (`temper-client-gallery`, `temper-client-shell-terminal`, `temper-client-tests`, as `cargo tree` names them). Regenerate what `temper/tools/gallery` says is stale. Drop the entries this closes from `temper/docs/design/client/README.status.md` and `forge.status.md` (the first **Not yet** each; keep the **Design departs**). Commit and push.

## Report

As the implementer's text says, with the 420 finding of acceptance 5, what you chose among the options, and the rule each new test broke when you checked that it bites.
